#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::Get};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Saturating;
    use sp_std::prelude::*;
    use scale_info::{TypeInfo, prelude::vec::Vec};

    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum CommunityType {
        Public,
        Private,
    }

    #[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum Status {
        Voting,
        Ended,
    }

    #[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum VoteResult {
        Approve,
        Reject,
        Undecided,
    }

    #[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
    pub struct Community<BoundedNameString, BoundedDescString, CommunityType, Status, AccountId, BlockNumber> {
        pub id: u32,
        pub name: BoundedNameString,
        pub description: BoundedDescString,
        pub community_type: CommunityType,
        pub status: Status,
        pub votes_approve: u32,
        pub votes_reject: u32,
        pub members: u32,
        pub created_by: AccountId,
        pub created_at: BlockNumber,
        pub voting_period_start: BlockNumber, // Início do período de votação
        pub voting_period_end: BlockNumber,   // Fim do período de votação
        pub vote_result: VoteResult,          // Resultado da votação
    }

    #[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
    pub struct PendingEntry<AccountId, BoundedNameString> {
        pub user: AccountId,
        pub community_name: BoundedNameString,
    }

    #[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
    pub struct CommunityRemove<AccountId, BoundedNameString> {
        pub user: AccountId,
        pub community_name: BoundedNameString,
    }

    #[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
    pub struct CommunityCreateVoter<AccountId, BoundedNameString> {
        pub user: AccountId,
        pub community_name: BoundedNameString,
    }

    #[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
    pub struct RemoveUserCommunityVoter<AccountId, BoundedNameString> {
        pub user: AccountId,
        pub community_name: BoundedNameString,
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type VotingDuration: Get<u32>; // Duração da votação em blocos
        type MaxNameLength: Get<u32>;
        type MaxDescLength: Get<u32>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        CommunityCreated(u32, T::AccountId),
        VoteSubmitted(u32, bool, T::AccountId), // id da comunidade, voto (true para aprovar, false para rejeitar), votante
        VotingConcluded(u32, VoteResult),       // id da comunidade e resultado da votação
        VotingStarted(u32),                     // id da comunidade
        MemberAdded(T::AccountId, u32),         // Membro adicionado à comunidade
        MemberPendingApproval(T::AccountId, u32), // Membro aguardando aprovação em uma comunidade privada
        MemberRemoved(T::AccountId, u32),       // Membro removido de uma comunidade
        CommunityDeleted(u32),                  // Comunidade deletada
    }

    #[pallet::storage]
    #[pallet::getter(fn communities)]
    pub type Communities<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        u32, 
        Community<
            BoundedVec<u8, T::MaxNameLength>, 
            BoundedVec<u8, T::MaxDescLength>, 
            CommunityType, 
            Status, 
            T::AccountId, 
            BlockNumberFor<T>
        >, 
        OptionQuery
    >;

    #[pallet::storage]
    #[pallet::getter(fn community_members)]
    pub type CommunityMembers<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        u32, // id da comunidade
        BoundedVec<T::AccountId, T::MaxDescLength>, // Lista de membros
        ValueQuery
    >;

    #[pallet::storage]
    #[pallet::getter(fn pending_entry)]
    pub type PendingEntryRequests<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        u32, // id da comunidade
        BoundedVec<PendingEntry<T::AccountId, BoundedVec<u8, T::MaxNameLength>>, T::MaxDescLength>, // Lista de pendências
        ValueQuery
    >;

    #[pallet::storage]
    #[pallet::getter(fn community_removes)]
    pub type CommunityRemoves<T: Config> = StorageMap<
        _, 
        Blake2_128Concat, 
        u32, // id da comunidade
        BoundedVec<CommunityRemove<T::AccountId, BoundedVec<u8, T::MaxNameLength>>, T::MaxDescLength>, // Lista de removidos
        ValueQuery
    >;

    #[pallet::storage]
    #[pallet::getter(fn community_create_voters)]
    pub type CommunityCreateVoters<T: Config> = StorageMap<
      _, 
      Blake2_128Concat, 
      u32, // id da comunidade
      BoundedVec<CommunityCreateVoter<T::AccountId, BoundedVec<u8, T::MaxNameLength>>, T::MaxDescLength>, // Lista de criadores
      ValueQuery
    >;

    #[pallet::storage]
    #[pallet::getter(fn remove_user_community_voters)]
    pub type RemoveUserCommunityVoters<T: Config> = StorageMap<
      _, 
      Blake2_128Concat, 
      u32, // id da comunidade
      BoundedVec<RemoveUserCommunityVoter<T::AccountId, BoundedVec<u8, T::MaxNameLength>>, T::MaxDescLength>, // Lista de votantes
      ValueQuery
    >;

    #[pallet::error]
    pub enum Error<T> {
        NameTooLong,
        DescriptionTooLong,
        CommunityNotFound,
        VotingClosed,
        VotingStillOpen,
        VotingNotStarted,
        InvalidVotingPeriod,
        NotCommunityOwner,
        AlreadyMember,
        NotMember,
        VotingInProgress,
        CommunityOwnerCannotBeRemoved,
        CannotDeleteCommunityInVoting,
        AlreadyVoted
    }

    // Hooks para finalizar a votação automaticamente quando o bloco de fim for atingido
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(now: BlockNumberFor<T>) {
            Self::conclude_voting(now);
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1, 1))]
        pub fn create_community(
          origin: OriginFor<T>,
          name: Vec<u8>,
          description: Vec<u8>,
          community_type: CommunityType,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let current_block = <frame_system::Pallet<T>>::block_number();
            let voting_period_start = current_block;
            let voting_period_end = current_block.saturating_add(14_400u32.into());  // 1 dia em blocos
    
            let community_id = Self::next_community_id()?;
            let bounded_name: BoundedVec<u8, T::MaxNameLength> = name.try_into().map_err(|_| Error::<T>::NameTooLong)?;
            let bounded_desc: BoundedVec<u8, T::MaxDescLength> = description.try_into().map_err(|_| Error::<T>::DescriptionTooLong)?;
    
            let community = Community {
                id: community_id,
                name: bounded_name,
                description: bounded_desc,
                community_type,
                status: Status::Voting,
                votes_approve: 0,
                votes_reject: 0,
                members: 0,
                created_by: who.clone(),
                created_at: current_block,
                voting_period_start,
                voting_period_end,
                vote_result: VoteResult::Undecided,
            };
    
            // Armazena o usuário que criou a comunidade
            let mut create_voters = CommunityCreateVoters::<T>::get(community_id);
            create_voters.try_push(CommunityCreateVoter {
                user: who.clone(),
                community_name: community.name.clone(),
            }).map_err(|_| Error::<T>::DescriptionTooLong)?;
    
            CommunityCreateVoters::<T>::insert(community_id, create_voters);
    
            Communities::<T>::insert(community_id, community);
            Self::deposit_event(Event::CommunityCreated(community_id, who));
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1, 1))]
        pub fn submit_vote(
          origin: OriginFor<T>,
          community_id: u32,
          approve: bool, 
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
        
            // Verificar se o usuário já votou nesta comunidade
            let create_voters = CommunityCreateVoters::<T>::get(community_id);
            ensure!(!create_voters.iter().any(|v| v.user == who), Error::<T>::AlreadyVoted);
        
            Communities::<T>::try_mutate(community_id, |community| {
                let community = community.as_mut().ok_or(Error::<T>::CommunityNotFound)?;
        
                let current_block = <frame_system::Pallet<T>>::block_number();
                ensure!(current_block >= community.voting_period_start, Error::<T>::VotingNotStarted);
                ensure!(current_block <= community.voting_period_end, Error::<T>::VotingClosed);
        
                // Registra o voto
                if approve {
                    community.votes_approve = community.votes_approve.saturating_add(1);
                } else {
                    community.votes_reject = community.votes_reject.saturating_add(1);
                }
        
                // Armazena o usuário que votou para evitar votos duplicados
                let mut create_voters = CommunityCreateVoters::<T>::get(community_id);
                create_voters.try_push(CommunityCreateVoter {
                    user: who.clone(),
                    community_name: community.name.clone(),
                }).map_err(|_| Error::<T>::DescriptionTooLong)?;
        
                CommunityCreateVoters::<T>::insert(community_id, create_voters);
        
                Self::deposit_event(Event::VoteSubmitted(community_id, approve, who));
                Ok(())
            })
        }

        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1, 1))]
        pub fn add_member(
            origin: OriginFor<T>,
            community_id: u32
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Communities::<T>::try_mutate(community_id, |community| {
                let community = community.as_mut().ok_or(Error::<T>::CommunityNotFound)?;

                // Não pode adicionar membros enquanto a votação ainda não acabou
                ensure!(community.status == Status::Ended && community.vote_result == VoteResult::Approve, Error::<T>::VotingInProgress);

                let mut members = CommunityMembers::<T>::get(community_id);
                ensure!(!members.contains(&who), Error::<T>::AlreadyMember);

                members.try_push(who.clone()).map_err(|_| Error::<T>::DescriptionTooLong)?;
                CommunityMembers::<T>::insert(community_id, members);

                // Atualiza a contagem de membros da comunidade
                community.members = community.members.saturating_add(1);
                Communities::<T>::insert(community_id, community);

                Self::deposit_event(Event::MemberAdded(who, community_id));
                Ok(())
            })
        }

        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1, 1))]
        pub fn approve_member(
            origin: OriginFor<T>,
            community_id: u32,
            user: T::AccountId
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Communities::<T>::try_mutate(community_id, |community| {
                let community = community.as_mut().ok_or(Error::<T>::CommunityNotFound)?;

                ensure!(community.created_by == who, Error::<T>::NotCommunityOwner);

                let mut pending_requests = PendingEntryRequests::<T>::get(community_id);
                pending_requests.retain(|entry| entry.user != user);
                PendingEntryRequests::<T>::insert(community_id, pending_requests);

                let mut members = CommunityMembers::<T>::get(community_id);
                members.try_push(user.clone()).map_err(|_| Error::<T>::DescriptionTooLong)?;
                CommunityMembers::<T>::insert(community_id, members);

                // Atualiza a contagem de membros da comunidade
                community.members = community.members.saturating_add(1);
                Communities::<T>::insert(community_id, community);

                Self::deposit_event(Event::MemberAdded(user, community_id));
                Ok(())
            })
        }

        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1, 1))]
        pub fn remove_member(
          origin: OriginFor<T>,
          community_id: u32,
          user: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
    
            let current_block = <frame_system::Pallet<T>>::block_number();
            let voting_period_start = current_block;
            let voting_period_end = current_block.saturating_add(14_400u32.into());  // 1 dia em blocos
    
            Communities::<T>::try_mutate(community_id, |community| {
                let community = community.as_mut().ok_or(Error::<T>::CommunityNotFound)?;
    
                ensure!(community.created_by != user, Error::<T>::CommunityOwnerCannotBeRemoved);
    
                let mut members = CommunityMembers::<T>::get(community_id);
                ensure!(members.contains(&user), Error::<T>::NotMember);
    
                community.voting_period_start = voting_period_start;
                community.voting_period_end = voting_period_end;
                community.vote_result = VoteResult::Undecided;
    
                CommunityRemoves::<T>::mutate(community_id, |removes| {
                    removes.try_push(CommunityRemove {
                        user: user.clone(),
                        community_name: community.name.clone(),
                    }).map_err(|_| Error::<T>::DescriptionTooLong)
                })?;
    
                Self::deposit_event(Event::MemberRemoved(user.clone(), community_id));
                Ok(())
            })
        }

        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1, 1))]
        pub fn vote_member_removal(
          origin: OriginFor<T>,
          community_id: u32,
          user_to_remove: T::AccountId,
          approve: bool // `true` para aprovar a remoção, `false` para rejeitar
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
        
            // Verifica se o usuário já votou para remover o membro
            let remove_voters = RemoveUserCommunityVoters::<T>::get(community_id);
            ensure!(!remove_voters.iter().any(|v| v.user == who), Error::<T>::AlreadyVoted);
        
            Communities::<T>::try_mutate(community_id, |community| {
                let community = community.as_mut().ok_or(Error::<T>::CommunityNotFound)?;
        
                let current_block = <frame_system::Pallet<T>>::block_number();
                ensure!(current_block >= community.voting_period_start, Error::<T>::VotingNotStarted);
                ensure!(current_block <= community.voting_period_end, Error::<T>::VotingClosed);
        
                // O dono da comunidade não pode ser removido
                ensure!(community.created_by != user_to_remove, Error::<T>::CommunityOwnerCannotBeRemoved);
        
                // Armazenar o voto
                let mut remove_voters = RemoveUserCommunityVoters::<T>::get(community_id);
                remove_voters.try_push(RemoveUserCommunityVoter {
                    user: who.clone(),
                    community_name: community.name.clone(),
                }).map_err(|_| Error::<T>::DescriptionTooLong)?;
        
                RemoveUserCommunityVoters::<T>::insert(community_id, remove_voters);
        
                // Registrar o voto
                if approve {
                    community.votes_approve = community.votes_approve.saturating_add(1);
                } else {
                    community.votes_reject = community.votes_reject.saturating_add(1);
                }
        
                Self::deposit_event(Event::VoteSubmitted(community_id, approve, who));
                Ok(())
            })
        }

        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1, 1))]
        pub fn delete_community(
            origin: OriginFor<T>,
            community_id: u32
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Communities::<T>::try_mutate_exists(community_id, |maybe_community| {
                let community = maybe_community.take().ok_or(Error::<T>::CommunityNotFound)?;

                // Apenas o dono da comunidade pode apagar a comunidade
                ensure!(community.created_by == who, Error::<T>::NotCommunityOwner);

                // Não pode deletar enquanto a votação está em andamento
                ensure!(community.status == Status::Ended, Error::<T>::CannotDeleteCommunityInVoting);

                // Remove todas as entradas relacionadas à comunidade
                CommunityMembers::<T>::remove(community_id);
                PendingEntryRequests::<T>::remove(community_id);
                CommunityRemoves::<T>::remove(community_id);
                CommunityCreateVoters::<T>::remove(community_id);
                RemoveUserCommunityVoters::<T>::remove(community_id);

                Self::deposit_event(Event::CommunityDeleted(community_id));

                Ok(())
            })
        }
    }

    // Implementação da função `conclude_voting`
  impl<T: Config> Pallet<T> {
    // Função para obter o próximo ID de comunidade
    pub fn next_community_id() -> Result<u32, DispatchError> {
        let next_id = Communities::<T>::iter().count() as u32 + 1;
        Ok(next_id)
    }

    pub fn conclude_voting(current_block: BlockNumberFor<T>) {
      for (community_id, mut community) in Communities::<T>::iter() {
        if community.voting_period_end < current_block && community.status == Status::Voting {
            let vote_result = if community.votes_approve > community.votes_reject {
                VoteResult::Approve
            } else {
                VoteResult::Reject
            };

            // Atualizando o status e resultado da votação
            community.status = Status::Ended;
            community.vote_result = vote_result;

            // Armazenando a comunidade após a atualização
            Communities::<T>::insert(community_id, community.clone());

            Self::deposit_event(Event::VotingConcluded(community_id, vote_result));

            // Se a votação foi aprovada, adiciona o criador da comunidade aos membros
            if vote_result == VoteResult::Approve {
                let created_by = community.created_by.clone(); // Fazendo uma cópia de `created_by`
                let mut members = CommunityMembers::<T>::get(community_id);

                if !members.contains(&created_by) {
                    members.try_push(created_by).unwrap();
                    CommunityMembers::<T>::insert(community_id, members);
                }
            }

            // Verifica votações de remoção de membros
            let removals = CommunityRemoves::<T>::get(community_id);
            for removal in removals.iter() {
                if vote_result == VoteResult::Approve {
                    CommunityMembers::<T>::mutate(community_id, |members| {
                        members.retain(|member| member != &removal.user);
                    });
                    Self::deposit_event(Event::MemberRemoved(removal.user.clone(), community_id));
                }
            }
        }
      }
    }
  }
}
