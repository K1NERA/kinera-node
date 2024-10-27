#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::{GenesisBuild, Get}};
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
    pub struct Community<BoundedNameString, BoundedDescString, CommunityType, Status, AccountId, BlockNumber, BoundedIconString, MaxSocialUserNameLength, MaxReasonLength> {
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
        pub voting_period_start: BlockNumber,
        pub voting_period_end: BlockNumber,
        pub vote_result: VoteResult,
        pub icon: BoundedIconString,
        pub social_user_name: MaxSocialUserNameLength,
        pub proposal_reason: MaxReasonLength,
    }

    #[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
    pub struct CommunityMemberDetails<AccountId, BoundedNameString, MaxSocialUserNameLength> {
        pub community_id: u32,
        pub user: AccountId,
        pub community_name: BoundedNameString,
        pub social_name: MaxSocialUserNameLength,
    }

    #[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
    pub struct PendingEntry<AccountId, BoundedNameString, MaxSocialUserNameLength> {
        pub user: AccountId,
        pub community_name: BoundedNameString,
        pub social_name: MaxSocialUserNameLength,
    }

    #[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
    pub struct CommunityRemove<AccountId, BoundedNameString, MaxSocialUserNameLength> {
        pub user: AccountId,
        pub community_name: BoundedNameString,
        pub social_name: MaxSocialUserNameLength,
    }

    #[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
    pub struct CommunityCreateVoter<AccountId, BoundedNameString, MaxSocialUserNameLength> {
        pub user: AccountId,
        pub community_name: BoundedNameString,
        pub social_name: MaxSocialUserNameLength,
        pub vote_result: VoteResult,
    }

    #[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
    pub struct RemoveUserCommunityVoter<AccountId, BoundedNameString, MaxSocialUserNameLength> {
        pub user: AccountId,
        pub community_name: BoundedNameString,
        pub social_name: MaxSocialUserNameLength,
    }

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub initial_community_name: Vec<u8>,
        pub initial_community_description: Vec<u8>,
        pub initial_community_owner: Option<T::AccountId>,
        pub initial_community_icon: Vec<u8>,
        pub initial_reason: Vec<u8>,
        pub initial_social_owner: Vec<u8>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            if let Some(owner) = &self.initial_community_owner {
                let current_block = <frame_system::Pallet<T>>::block_number();
                let voting_period_start = current_block;
                let voting_period_end = current_block;

                let community_id = Pallet::<T>::next_community_id().expect("Failed to get community ID");

                let bounded_name: BoundedVec<u8, T::MaxNameLength> = self.initial_community_name
                    .clone()
                    .try_into()
                    .expect("Community name too long");

                let bounded_desc: BoundedVec<u8, T::MaxDescLength> = self.initial_community_description
                    .clone()
                    .try_into()
                    .expect("Community description too long");

                let bounded_icon: BoundedVec<u8, T::MaxIconLength> = self.initial_community_icon
                    .clone()
                    .try_into()
                    .expect("Icon too long");

                let initial_reason: BoundedVec<u8, T::MaxReasonLength> = self.initial_reason
                    .clone()
                    .try_into()
                    .expect("Reason too long");

                let initial_social_owner: BoundedVec<u8, T::MaxSocialUserNameLength> = self.initial_social_owner
                    .clone()
                    .try_into()
                    .expect("Social name too long");
                let initial_social_owner_clone = initial_social_owner.clone();
                let community = Community {
                    id: community_id,
                    name: bounded_name.clone(),
                    description: bounded_desc,
                    community_type: CommunityType::Private,
                    status: Status::Ended,
                    votes_approve: 1,
                    votes_reject: 0,
                    members: 1,
                    created_by: owner.clone(),
                    created_at: current_block,
                    voting_period_start,
                    voting_period_end,
                    vote_result: VoteResult::Approve,
                    icon: bounded_icon,
                    social_user_name: initial_social_owner,
                    proposal_reason: initial_reason,
                };

                Communities::<T>::insert(community_id, community.clone());
                
                Pallet::<T>::add_community_to_user(&owner, community.clone()).expect("Erro ao adicionar a comunidade ao criador no UserCommunities");
                CommunitiesByStatus::<T>::mutate(Status::Ended, |communities| {
                    communities.try_push(community.clone()).expect("Erro ao adicionar comunidade em CommunitiesByStatus");
                });
                CommunitiesByVoteResult::<T>::mutate(VoteResult::Approve, |communities| {
                    communities.try_push(community).expect("Erro ao adicionar comunidade em CommunitiesByVoteResult");
                });

                let mut members = BoundedVec::<CommunityMemberDetails<T::AccountId, BoundedVec<u8, T::MaxNameLength>, BoundedVec<u8, T::MaxSocialUserNameLength>>, T::MaxDescLength>::default();
                members.try_push(CommunityMemberDetails {
                    community_id,
                    user: owner.clone(),
                    community_name: bounded_name.clone(),
                    social_name: initial_social_owner_clone,
                }).expect("Erro ao adicionar criador como membro da comunidade");

                CommunityMembers::<T>::insert(community_id, members);
              
                Pallet::<T>::deposit_event(Event::CommunityCreated(community_id, owner.clone()));
                Pallet::<T>::deposit_event(Event::VotingConcluded(community_id, VoteResult::Approve));
            }
        }
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type VotingDuration: Get<u32>;
        type MaxNameLength: Get<u32>;
        type MaxDescLength: Get<u32>;
        type MaxReasonLength: Get<u32>;
        type MaxIconLength: Get<u32>;
        type MaxSocialUserNameLength: Get<u32>;
        type MaxCommunitiesPerUser: Get<u32>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        CommunityCreated(u32, T::AccountId),
        VoteSubmitted(u32, bool, T::AccountId),
        VotingConcluded(u32, VoteResult),
        VotingStarted(u32),
        MemberAdded(T::AccountId, u32),
        MemberPendingApproval(T::AccountId, u32),
        MemberRemoved(T::AccountId, u32),
        CommunityDeleted(u32),
        CommunityAddedToUser(T::AccountId, u32),
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
            BlockNumberFor<T>,
            BoundedVec<u8, T::MaxIconLength>,
            BoundedVec<u8, T::MaxSocialUserNameLength>,
            BoundedVec<u8, T::MaxReasonLength>
        >,
        OptionQuery
    >;

    #[pallet::storage]
    #[pallet::getter(fn user_communities)]
    pub type UserCommunities<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId, // A chave será o AccountId (endereço do usuário)
        BoundedVec<
            Community<
                BoundedVec<u8, T::MaxNameLength>,
                BoundedVec<u8, T::MaxDescLength>,
                CommunityType,
                Status,
                T::AccountId,
                BlockNumberFor<T>,
                BoundedVec<u8, T::MaxIconLength>,
                BoundedVec<u8, T::MaxSocialUserNameLength>,
                BoundedVec<u8, T::MaxReasonLength>
            >,
            T::MaxCommunitiesPerUser
        >, // O valor será uma lista de comunidades completas
        ValueQuery
    >;

    #[pallet::storage]
    #[pallet::getter(fn community_members)]
    pub type CommunityMembers<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32,
        BoundedVec<CommunityMemberDetails<T::AccountId, BoundedVec<u8, T::MaxNameLength>, BoundedVec<u8, T::MaxSocialUserNameLength>>, T::MaxDescLength>,
        ValueQuery
    >;

    #[pallet::storage]
    #[pallet::getter(fn pending_entry)]
    pub type PendingEntryRequests<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32,
        BoundedVec<PendingEntry<T::AccountId, BoundedVec<u8, T::MaxNameLength>, BoundedVec<u8, T::MaxSocialUserNameLength>>, T::MaxDescLength>,
        ValueQuery
    >;

    #[pallet::storage]
    #[pallet::getter(fn community_removes)]
    pub type CommunityRemoves<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32,  // community_id
        BoundedVec<(
            T::AccountId,  // user_remove (quem está sendo removido)
            BoundedVec<(T::AccountId, VoteResult), T::MaxDescLength>,  // Lista de votantes e seus votos
            BlockNumberFor<T>,  // block_start
            BlockNumberFor<T>   // block_end
        ), T::MaxDescLength>,
        ValueQuery
    >;

    #[pallet::storage]
    #[pallet::getter(fn community_create_voters)]
    pub type CommunityCreateVoters<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32,
        BoundedVec<CommunityCreateVoter<T::AccountId, BoundedVec<u8, T::MaxNameLength>, BoundedVec<u8, T::MaxSocialUserNameLength>>, T::MaxDescLength>,
        ValueQuery
    >;

    #[pallet::storage]
    #[pallet::getter(fn remove_user_community_voters)]
    pub type RemoveUserCommunityVoters<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32,  // community_id
        BoundedVec<(
            T::AccountId,  // voter (quem votou)
            T::AccountId,  // user_to_remove (quem está sendo removido)
            BlockNumberFor<T>, // block_start
            BlockNumberFor<T>  // block_end
        ), T::MaxDescLength>,
        ValueQuery
    >;

    #[pallet::storage]
    #[pallet::getter(fn communities_by_status)]
    pub type CommunitiesByStatus<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        Status,
        BoundedVec<
            Community<
                BoundedVec<u8, T::MaxNameLength>,
                BoundedVec<u8, T::MaxDescLength>,
                CommunityType,
                Status,
                T::AccountId,
                BlockNumberFor<T>,
                BoundedVec<u8, T::MaxIconLength>,
                BoundedVec<u8, T::MaxSocialUserNameLength>,
                BoundedVec<u8, T::MaxReasonLength>
            >,
            T::MaxDescLength
        >,
        ValueQuery
    >;

    #[pallet::storage]
    #[pallet::getter(fn communities_by_vote_result)]
    pub type CommunitiesByVoteResult<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        VoteResult,
        BoundedVec<
            Community<
                BoundedVec<u8, T::MaxNameLength>,
                BoundedVec<u8, T::MaxDescLength>,
                CommunityType,
                Status,
                T::AccountId,
                BlockNumberFor<T>,
                BoundedVec<u8, T::MaxIconLength>,
                BoundedVec<u8, T::MaxSocialUserNameLength>,
                BoundedVec<u8, T::MaxReasonLength>
            >,
            T::MaxDescLength
        >,
        ValueQuery
    >;

    #[pallet::error]
    pub enum Error<T> {
        NameTooLong,
        DescriptionTooLong,
        CommunityAlreadyExists,
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
        AlreadyVoted,
        MaxCommunitiesReached,
    }

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
            icon: Vec<u8>,
            social_user_name: Vec<u8>,
            proposal_reason: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let current_block = <frame_system::Pallet<T>>::block_number();
            let voting_period_start = current_block;
            let voting_period_end = current_block.saturating_add(50u32.into());

            let community_id = Self::next_community_id()?;
            let bounded_name: BoundedVec<u8, T::MaxNameLength> = name.try_into().map_err(|_| Error::<T>::NameTooLong)?;

            for (_, community) in Communities::<T>::iter() {
                if community.name == bounded_name {
                    return Err(Error::<T>::CommunityAlreadyExists.into());
                }
            }

            let bounded_desc: BoundedVec<u8, T::MaxDescLength> = description.try_into().map_err(|_| Error::<T>::DescriptionTooLong)?;
            let bounded_icon: BoundedVec<u8, T::MaxIconLength> = icon.try_into().map_err(|_| Error::<T>::DescriptionTooLong)?;
            let bounded_social_user_name: BoundedVec<u8, T::MaxSocialUserNameLength> = social_user_name.try_into().map_err(|_| Error::<T>::DescriptionTooLong)?;
            let bounded_proposal_reason: BoundedVec<u8, T::MaxReasonLength> = proposal_reason.try_into().map_err(|_| Error::<T>::DescriptionTooLong)?;

            let community = Community {
                id: community_id,
                name: bounded_name,
                description: bounded_desc,
                community_type,
                status: Status::Voting,
                votes_approve: 1,
                votes_reject: 0,
                members: 0,
                created_by: who.clone(),
                created_at: current_block,
                voting_period_start,
                voting_period_end,
                vote_result: VoteResult::Undecided,
                icon: bounded_icon,
                social_user_name: bounded_social_user_name,
                proposal_reason: bounded_proposal_reason,
            };

            let mut create_voters = CommunityCreateVoters::<T>::get(community_id);
            create_voters.try_push(CommunityCreateVoter {
                user: who.clone(),
                community_name: community.name.clone(),
                social_name: community.social_user_name.clone(),
                vote_result: VoteResult::Approve,
            }).map_err(|_| Error::<T>::DescriptionTooLong)?;

            CommunityCreateVoters::<T>::insert(community_id, create_voters);

            CommunitiesByStatus::<T>::mutate(Status::Voting, |communities| {
                communities.try_push(community.clone()).expect("Erro ao adicionar comunidade");
            });
            CommunitiesByVoteResult::<T>::mutate(VoteResult::Undecided, |communities| {
                communities.try_push(community.clone()).expect("Erro ao adicionar comunidade");
            });

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
            social_user_name: Vec<u8>,
        ) -> DispatchResult {
          let who = ensure_signed(origin)?;

          let create_voters = CommunityCreateVoters::<T>::get(community_id);
          ensure!(!create_voters.iter().any(|v| v.user == who), Error::<T>::AlreadyVoted);

          Communities::<T>::try_mutate(community_id, |community| {
            let community = community.as_mut().ok_or(Error::<T>::CommunityNotFound)?;

            let current_block = <frame_system::Pallet<T>>::block_number();
            ensure!(current_block >= community.voting_period_start, Error::<T>::VotingNotStarted);
            ensure!(current_block <= community.voting_period_end, Error::<T>::VotingClosed);

            if approve {
                community.votes_approve = community.votes_approve.saturating_add(1);
            } else {
                community.votes_reject = community.votes_reject.saturating_add(1);
            }

            Communities::<T>::insert(community_id, community.clone());

            let mut create_voters = CommunityCreateVoters::<T>::get(community_id);
            create_voters.try_push(CommunityCreateVoter {
                user: who.clone(),
                community_name: community.name.clone(),
                social_name: social_user_name.clone().try_into().map_err(|_| Error::<T>::DescriptionTooLong)?,
                vote_result: if approve { VoteResult::Approve } else { VoteResult::Reject },
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
            community_id: u32,
            social_user_name: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
        
            Communities::<T>::try_mutate(community_id, |community| {
                let community = community.as_mut().ok_or(Error::<T>::CommunityNotFound)?;
        
                if community.community_type == CommunityType::Private {
                    // Para comunidades privadas, o membro vai para PendingEntryRequests
                    let mut pending_requests = PendingEntryRequests::<T>::get(community_id);
                    ensure!(!pending_requests.iter().any(|entry| entry.user == who), Error::<T>::AlreadyMember);
        
                    pending_requests.try_push(PendingEntry {
                        user: who.clone(),
                        community_name: community.name.clone(),
                        social_name: social_user_name.clone().try_into().map_err(|_| Error::<T>::DescriptionTooLong)?,
                    }).map_err(|_| Error::<T>::DescriptionTooLong)?;
        
                    PendingEntryRequests::<T>::insert(community_id, pending_requests);
        
                    Self::deposit_event(Event::MemberPendingApproval(who.clone(), community_id));
                } else {
                    // Para comunidades públicas, adiciona diretamente
                    let mut members = CommunityMembers::<T>::get(community_id);
                    ensure!(!members.iter().any(|m| m.user == who), Error::<T>::AlreadyMember);
        
                    let member_details = CommunityMemberDetails {
                        community_id,
                        user: who.clone(),
                        community_name: community.name.clone(),
                        social_name: social_user_name.clone().try_into().map_err(|_| Error::<T>::DescriptionTooLong)?,
                    };
        
                    members.try_push(member_details).map_err(|_| Error::<T>::DescriptionTooLong)?;
                    CommunityMembers::<T>::insert(community_id, members);
        
                    // Atualiza a contagem de membros
                    community.members = community.members.saturating_add(1);
                    Communities::<T>::insert(community_id, community.clone());
        
                    // Adicionar a comunidade ao UserCommunities do usuário
                    UserCommunities::<T>::mutate(&who, |communities| {
                        communities.try_push(community.clone()).expect("Erro ao adicionar comunidade ao UserCommunities");
                    });
        
                    Self::deposit_event(Event::MemberAdded(who, community_id));
                }
                Ok(())
            })
        }
        

        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1, 1))]
        pub fn approve_member(
            origin: OriginFor<T>,
            community_id: u32,
            user: T::AccountId,
            social_user_name: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
        
            Communities::<T>::try_mutate(community_id, |community| {
                let community = community.as_mut().ok_or(Error::<T>::CommunityNotFound)?;
                ensure!(community.created_by == who, Error::<T>::NotCommunityOwner);
        
                // Verifica se o usuário está na lista de entrada pendente
                let mut pending_requests = PendingEntryRequests::<T>::get(community_id);
                ensure!(pending_requests.iter().any(|entry| entry.user == user), Error::<T>::NotMember);
        
                // Remove o usuário da lista de pendentes
                pending_requests.retain(|entry| entry.user != user);
                PendingEntryRequests::<T>::insert(community_id, pending_requests);
        
                // Adiciona o usuário aprovado à lista de membros
                let mut members = CommunityMembers::<T>::get(community_id);
                let member_details = CommunityMemberDetails {
                    community_id,
                    user: user.clone(),
                    community_name: community.name.clone(),
                    social_name: social_user_name.clone().try_into().map_err(|_| Error::<T>::DescriptionTooLong)?,
                };
        
                members.try_push(member_details).map_err(|_| Error::<T>::DescriptionTooLong)?;
                CommunityMembers::<T>::insert(community_id, members);
        
                // Atualiza a contagem de membros
                community.members = community.members.saturating_add(1);
                Communities::<T>::insert(community_id, community.clone());
        
                CommunitiesByVoteResult::<T>::mutate(community.vote_result, |communities| {
                    if let Some(pos) = communities.iter().position(|c| c.id == community_id) {
                        communities[pos].members = community.members;
                    }
                });
        
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
            social_user_name: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
        
            Communities::<T>::try_mutate(community_id, |community| {
                let community = community.as_mut().ok_or(Error::<T>::CommunityNotFound)?;
        
                ensure!(community.created_by != user, Error::<T>::CommunityOwnerCannotBeRemoved);
        
                let mut members = CommunityMembers::<T>::get(community_id);
                ensure!(members.iter().any(|m| m.user == user), Error::<T>::NotMember);
        
                // Remove o membro da lista de membros
                members.retain(|m| m.user != user);
                CommunityMembers::<T>::insert(community_id, members);
        
                // Atualiza a contagem de membros
                community.members = community.members.saturating_sub(1);
                Communities::<T>::insert(community_id, community.clone());
        
                CommunitiesByVoteResult::<T>::mutate(community.vote_result, |communities| {
                    if let Some(pos) = communities.iter().position(|c| c.id == community_id) {
                        communities[pos].members = community.members;
                    }
                });
        
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
          approve: bool,
          social_user_name: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
        
            let current_block = <frame_system::Pallet<T>>::block_number();
        
            let remove_voters = RemoveUserCommunityVoters::<T>::get(community_id);
            ensure!(!remove_voters.iter().any(|(voter, _, _, _)| voter == &who), Error::<T>::AlreadyVoted);
        
            Communities::<T>::try_mutate(community_id, |community| {
                let community = community.as_mut().ok_or(Error::<T>::CommunityNotFound)?;
        
                ensure!(current_block >= community.voting_period_start, Error::<T>::VotingNotStarted);
                ensure!(current_block <= community.voting_period_end, Error::<T>::VotingClosed);
        
                let voting_end_block = community.voting_period_end;
                let voting_period_end = current_block.saturating_add(50u32.into());
                let mut remove_voters = RemoveUserCommunityVoters::<T>::get(community_id);
                remove_voters.try_push((
                    who.clone(),         // quem votou
                    user_to_remove.clone(), // quem está sendo removido
                    current_block,       // bloco de início
                    voting_period_end     // bloco de fim
                )).map_err(|_| Error::<T>::DescriptionTooLong)?;
        
                RemoveUserCommunityVoters::<T>::insert(community_id, remove_voters);
        
                if approve {
                    community.votes_approve = community.votes_approve.saturating_add(1);
                } else {
                    community.votes_reject = community.votes_reject.saturating_add(1);
                }
        
                Communities::<T>::insert(community_id, community);
        
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

                ensure!(community.created_by == who, Error::<T>::NotCommunityOwner);
                ensure!(community.status == Status::Ended, Error::<T>::CannotDeleteCommunityInVoting);

                CommunitiesByStatus::<T>::mutate(community.status, |communities| {
                    communities.retain(|c| c.id != community_id);
                });

                CommunitiesByVoteResult::<T>::mutate(community.vote_result, |communities| {
                    communities.retain(|c| c.id != community_id);
                });

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

    impl<T: Config> Pallet<T> {
        pub fn next_community_id() -> Result<u32, DispatchError> {
          let next_id = Communities::<T>::iter().count() as u32 + 1;
          Ok(next_id)
        }

        pub fn add_community_to_user(user: &T::AccountId, community: Community<
            BoundedVec<u8, T::MaxNameLength>,
            BoundedVec<u8, T::MaxDescLength>,
            CommunityType,
            Status,
            T::AccountId,
            BlockNumberFor<T>,
            BoundedVec<u8, T::MaxIconLength>,
            BoundedVec<u8, T::MaxSocialUserNameLength>,
            BoundedVec<u8, T::MaxReasonLength>
        >) -> DispatchResult {
          let community_clone = community.clone(); // Clone a comunidade para que possa ser usada depois da closure.

          UserCommunities::<T>::try_mutate(user, |communities| {
              communities.try_push(community).map_err(|_| Error::<T>::MaxCommunitiesReached)
          })?;
      
          Self::deposit_event(Event::CommunityAddedToUser(user.clone(), community_clone.id)); // Agora use a cópia clonada da comunidade.
          Ok(())
        }

        pub fn conclude_voting(current_block: BlockNumberFor<T>) {
            for (community_id, mut community) in Communities::<T>::iter() {
                if community.voting_period_end < current_block && community.status == Status::Voting {
                    let vote_result = if community.votes_approve > community.votes_reject {
                        VoteResult::Approve
                    } else {
                        VoteResult::Reject
                    };
                    community.status = Status::Ended;
                    community.vote_result = vote_result;
    
                    CommunitiesByStatus::<T>::mutate(Status::Voting, |communities| {
                        communities.retain(|c| c.id != community_id);
                    });
                    CommunitiesByStatus::<T>::mutate(Status::Ended, |communities| {
                        communities.try_push(community.clone()).expect("Erro ao atualizar comunidades por status");
                    });
    
                    // Atualizar o CommunitiesByVoteResult
                    CommunitiesByVoteResult::<T>::mutate(VoteResult::Undecided, |communities| {
                        communities.retain(|c| c.id != community_id);
                    });
                    CommunitiesByVoteResult::<T>::mutate(vote_result, |communities| {
                        communities.try_push(community.clone()).expect("Erro ao atualizar comunidades por resultado de voto");
                    });
    
                    Communities::<T>::insert(community_id, community.clone());

                    CommunitiesByVoteResult::<T>::mutate(vote_result, |communities| {
                      if let Some(pos) = communities.iter().position(|c| c.id == community_id) {
                          communities[pos].members = community.members; // Atualiza o número de membros.
                      } else {
                          communities.try_push(community.clone()).expect("Erro ao atualizar comunidades por resultado de voto");
                      }
                  });
    
                    Self::deposit_event(Event::VotingConcluded(community_id, vote_result));
    
                    // Adicionar o dono como membro automaticamente ao final da votação, se aprovado
                    if vote_result == VoteResult::Approve {
                        let created_by = community.created_by.clone();
                        
                        // Atualize a contagem de membros ao adicionar o dono
                        let member_details = CommunityMemberDetails {
                            community_id,
                            user: created_by.clone(),
                            community_name: community.name.clone(),
                            social_name: community.social_user_name.clone(),
                        };
    
                        CommunityMembers::<T>::mutate(community_id, |members| {
                            if !members.iter().any(|m| m.user == created_by) {
                                members.try_push(member_details).expect("Erro ao adicionar criador à lista de membros");
                                community.members = community.members.saturating_add(1); // Atualize o número de membros
                            }
                        });
    
                        // Atualizar a comunidade para refletir o novo número de membros
                        Communities::<T>::insert(community_id, community.clone());
    
                        // Adicionar a comunidade ao UserCommunities, se ainda não estiver presente
                        UserCommunities::<T>::mutate(&created_by, |communities| {
                            if !communities.iter().any(|c| c.id == community_id) {
                                communities.try_push(community.clone()).expect("Erro ao adicionar comunidade ao UserCommunities");
                            }
                        });
    
                        // Emitir o evento de que o criador foi adicionado como membro
                        Self::deposit_event(Event::MemberAdded(created_by, community_id));
                    }
    
                    // Gerenciar remoções de membros, se aplicável
                    let removals = CommunityRemoves::<T>::get(community_id);
                    for removal in removals.iter() {
                        if vote_result == VoteResult::Approve {
                            CommunityMembers::<T>::mutate(community_id, |members| {
                                members.retain(|member| member.user != removal.0); // Remover o membro se aprovado
                            });
                            Self::deposit_event(Event::MemberRemoved(removal.0.clone(), community_id));
                        }
                    }
                }
            }
        }
        
    }
}
