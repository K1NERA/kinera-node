#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
  use frame_support::{
      dispatch::DispatchResultWithPostInfo,
      pallet_prelude::*,
      traits::{
          Currency,
          ReservableCurrency,
          ExistenceRequirement::{
              AllowDeath,
              KeepAlive,
          },
      },
      storage::bounded_btree_map::BoundedBTreeMap,
      PalletId
  };
  use frame_system::pallet_prelude::*;
  use codec::{Decode, Encode, MaxEncodedLen};
  use sp_runtime::{
      RuntimeDebug, 
      traits::{
          AccountIdConversion, 
          AtLeast32BitUnsigned, 
          CheckedAdd, CheckedSub, CheckedDiv, 
          Saturating, One
      },
      Perbill,
  };
  use scale_info::{
      TypeInfo,
      prelude::{
          vec::Vec,
          string::String,
      },
  };
  use core::convert::TryInto;
  use frame_support::BoundedVec;
  use sp_std::{collections::btree_map::BTreeMap,vec};
  use kine_movie;
  use kine_tags;

  // why does this need to be a crate?
  use crate::pallet::kine_tags::{
      CategoryId as CategoryId,
      TagId as TagId
  };


//* Config *//

  #[pallet::pallet]
  pub struct Pallet<T>(_);

  #[pallet::config]
  pub trait Config: frame_system::Config 
  + kine_movie::Config + kine_tags::Config + kine_stat_tracker::Config {
          type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
          // type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
          type FestivalId: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
          type MaxMoviesInFest: Get<u32>;
          type MaxOwnedFestivals: Get<u32>;
          type MinFesBlockDuration: Get<u32>;
          type MaxFestivalsPerBlock: Get<u32>;
          type MaxVotes: Get<u32>;
          type DescStringLimit: Get<u32>;
          type FestBlockSafetyMargin: Get<u32>;
          type PalletId: Get<PalletId>;
      }

  type BalanceOf<T> = <<T as kine_stat_tracker::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

  #[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
  pub enum FestivalStatus {
    AwaitingActivation,
    AwaitingStartBlock,
    Active,
    Finished,
    FinishedNotEnoughVotes,
  }

//* Structs *//

  #[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
  pub struct Festival<
  FestivalId,
  AccountId,
  BoundedOwnerNameString,
  BoundedNameString,
  BoundedDescString,
  FestivalStatus,
  BalanceOf,
  VoteMap,
  CategoryTagList,
  MoviesInFest,
  BlockStartEnd,
  BlockNumber,
> {
  pub id: FestivalId,
  pub owner: AccountId,
  pub owner_id: BoundedOwnerNameString, // Novo campo adicionado
  pub name: BoundedNameString,
  pub description: BoundedDescString,
  pub status: FestivalStatus,
  pub max_entry: BalanceOf,
  pub total_lockup: BalanceOf,
  pub vote_map: VoteMap,
  pub categories_and_tags: CategoryTagList,
  pub internal_movies: MoviesInFest,
  pub external_movies: MoviesInFest,
  pub winners: MoviesInFest,
  pub block_start_end: BlockStartEnd,
  pub vote_power_decrease_block: BlockNumber,
}

  #[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
  pub struct BlockAssignment<BoundedFestivals> {
    pub to_start: BoundedFestivals,
    pub to_end: BoundedFestivals,
  }

  #[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
  pub struct Vote<AccountId, Balance, BoundedNameString> {
    pub voter: AccountId,
    pub amount: Balance,
    pub amount_after_weight: Balance,
    pub user: BoundedNameString,
  }

  #[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
  pub struct WalletData<BoundedFestivals> {
      pub all_owned_festivals: BoundedFestivals,
      pub awaiting_activation_festivals: BoundedFestivals,
      pub awaiting_start_festivals: BoundedFestivals,
      pub active_festivals: BoundedFestivals,
      pub finished_festivals: BoundedFestivals,
      pub won_festivals: BoundedFestivals,

      pub active_voted_festivals: BoundedFestivals,
      pub finished_voted_festivals: BoundedFestivals, // only festivals that were not won
      pub finished_won_festivals: BoundedFestivals, // only festivals that were not won
    }
    #[pallet::storage]
    #[pallet::getter(fn next_festival_id)]
    pub(super) type NextFestivalId<T: Config> = 
      StorageValue<
        _, 
        T::FestivalId, 
        ValueQuery
      >;

    #[pallet::storage]
    #[pallet::getter(fn get_festival)]
    pub type Festivals<T: Config> = 
    StorageMap<
      _, 
      Blake2_128Concat, T::FestivalId, 
      Festival<
          T::FestivalId, 
          T::AccountId,
          BoundedVec<u8, T::NameStringLimit>,
          BoundedVec<u8, T::NameStringLimit>, //BoundedNameString
          BoundedVec<u8, T::DescStringLimit>, //TODO-5
          FestivalStatus,
          BalanceOf<T>, //BalanceOf
          BoundedBTreeMap<
              BoundedVec<u8, T::LinkStringLimit>, 
              BoundedVec<Vote<T::AccountId, BalanceOf<T>, BoundedVec<u8, T::NameStringLimit>>, T::MaxVotes>,
              T::MaxVotes,
          >, //VoteMap
          BoundedVec<(CategoryId<T>, TagId<T>), T::MaxTags>, //CategoryTagList
          BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInFest>, //MoviesInFest
          (BlockNumberFor<T>, BlockNumberFor<T>),
          BlockNumberFor<T>,
      >,
      OptionQuery
    >;


    #[pallet::storage]
    #[pallet::getter(fn get_block_assignments)]
    pub(super) type BlockAssignments<T: Config> = 
    StorageMap<
        _,
        Blake2_128Concat, BlockNumberFor<T>,
        BlockAssignment<BoundedVec<T::FestivalId, T::MaxFestivalsPerBlock>>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn get_wallet_festival_data)]
    pub(super) type WalletFestivalData<T: Config> = 
      StorageMap<
        _,
        Blake2_128Concat, T::AccountId,
        WalletData<BoundedVec<T::FestivalId, T::MaxOwnedFestivals>>,
    >;

  #[pallet::event]
  #[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
      FestivalCreated(T::AccountId, T::FestivalId),
      MovieAddedToFestival(T::FestivalId, String, T::AccountId),
      MoviesAddedToFestival(T::FestivalId, T::AccountId),
      VotedForMovieInFestival(T::FestivalId, String, T::AccountId, String),
      FestivalHasBegun(T::FestivalId),
      // FestivalHasEnded(T::FestivalId), //TODO-6
      FestivalHasEnded(T::FestivalId, Vec<T::AccountId>, BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInFest>), 
      FestivalHasEndedUnsuccessfully(T::FestivalId),
      FestivalActivated(T::FestivalId, T::AccountId),
      FestivalTokensClaimed(T::AccountId, BalanceOf<T>),
      FestivalDetailsUpdated(T::FestivalId, T::AccountId, Vec<u8>, Vec<u8>),
      FestivalRemoved(T::FestivalId, T::AccountId),
  }

  #[pallet::error]
  pub enum Error<T> {
    Overflow,
    Underflow,
    BadMetadata,
    InsufficientBalance,
    WalletStatsRegistryRequired,
    NotEnoughBalance,
    PastStartDate,
    FestivalPeriodTooShort,
    NoFestivalAdminAccess,
    NotEnoughMoviesInFestival,
    NotAwaitingActivation,
    NotPermissionToRemove,
    FestivalHaveVote,
    MovieAlreadyInFestival,
    MovieNotInFestival,
    InvalidFestival,
    NonexistentFestival,
    NonexistentMovie,
    FestivalNotActive,
    FestivalNotAcceptingNewMovies,
    CannotVoteInOwnFestival,
    CannotRemoveFestivalWithVotes,
    VoteMaxAmountCannotBeZero,
    VoteValueTooHigh,
    VoteValueCannotBeZero,
    InvalidBlockPeriod,
    NoClaimableTokens,
  }

//** Hooks **//

  #[pallet::hooks]
  impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    fn on_finalize(now: BlockNumberFor<T>){
      Self::hook_deactivate_festival(now);
      Self::hook_activate_festival(now);
    }
  }

  #[pallet::call]
  impl<T: Config> Pallet<T> {
    #[pallet::call_index(0)]#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
    pub fn create_festival(
        origin: OriginFor<T>,
        owner: String,
        bounded_name: String,
        bounded_description: String, 
        max_entry: BalanceOf<T>,
        internal_movie_ids: BoundedVec<String, T::MaxMoviesInFest>,
        external_movie_ids: BoundedVec<String, T::MaxMoviesInFest>,
        category_tag_list: BoundedVec<(CategoryId<T>, TagId<T>), T::MaxTags>,
    ) -> DispatchResultWithPostInfo {
        
      let who = ensure_signed(origin)?;
      let (bounded_internal_movie_ids, bounded_external_movie_ids) 
          = Self::do_validate_add_movies_to_new_festival(internal_movie_ids.clone(), external_movie_ids.clone())?;

      let category_type: kine_tags::CategoryType<T>
          = TryInto::try_into("Festival".as_bytes().to_vec())
          .map_err(|_|Error::<T>::BadMetadata)?;
      kine_tags::Pallet::<T>::do_validate_tag_data(
          category_type.clone(), 
          category_tag_list.clone()
      )?;

      // create the festival & bind the owner to it
      let festival_id = Self::do_create_festival(
        who.clone(),
        owner,
        bounded_name, bounded_description, max_entry,
        bounded_internal_movie_ids, bounded_external_movie_ids,
        category_tag_list.clone(), FestivalStatus::AwaitingActivation
      )?;
      Self::do_bind_owners_to_festival(who.clone(), festival_id)?;
      
      // parse the festival_id into a BoundedVec<u8, T::ContentStringLimit>
      let encoded: Vec<u8> = festival_id.encode();
      let bounded_content_id: BoundedVec<u8, T::ContentStringLimit> = 
        TryInto::try_into(encoded).map_err(|_|Error::<T>::BadMetadata)?;

      // update tags with the encoded bounded_content_id
      kine_tags::Pallet::<T>::do_update_tag_data(
        category_type, 
        category_tag_list,
        bounded_content_id,
      )?;

      Self::deposit_event(Event::FestivalCreated(who.clone(), festival_id));
      Ok(().into())
    }

    #[pallet::call_index(1)]#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
    pub fn activate_festival(
        origin: OriginFor<T>,
        festival_id: T::FestivalId,
        start_block: BlockNumberFor<T>,
        end_block: BlockNumberFor<T>,
    )-> DispatchResultWithPostInfo{
        
      let who = ensure_signed(origin)?;

      // mutate the festival from storage
      Festivals::<T>::try_mutate_exists( festival_id.clone(),|fes| -> DispatchResult{
        let festival = fes.as_mut().ok_or(Error::<T>::BadMetadata)?;

        // ensure the owner owns the festival 
        ensure!(
            festival.owner == who.clone(),
            Error::<T>::NoFestivalAdminAccess
        );
        // ensure the festival has at least 2 movies
        ensure!(
            festival.internal_movies.len() + festival.external_movies.len() > 1,
            Error::<T>::NotEnoughMoviesInFestival
        );
        // ensure the status is AwaitingActivation
        ensure!(
            festival.status == FestivalStatus::AwaitingActivation,
            Error::<T>::NotAwaitingActivation
        );

        // ensure the block periods are valid
        let safe_start_time = start_block
            .checked_sub(&BlockNumberFor::<T>::from(T::FestBlockSafetyMargin::get()))
            .ok_or(Error::<T>::InvalidBlockPeriod)?;
        ensure!(
            frame_system::Pallet::<T>::block_number() < safe_start_time, 
            Error::<T>::PastStartDate
        );
        ensure!(
            end_block-safe_start_time >= BlockNumberFor::<T>::from(T::MinFesBlockDuration::get()), 
            Error::<T>::FestivalPeriodTooShort
        );

        // update the festival ownership status
        WalletFestivalData::<T>::try_mutate_exists( who.clone(), |wal_data| -> DispatchResult{
          let wallet_data = wal_data.as_mut().ok_or(Error::<T>::NonexistentFestival)?;
          
          //filter the movie from the awaiting activation list
          wallet_data.awaiting_activation_festivals.retain(
              |fes_id| 
              fes_id != &festival_id.clone()
          );
          wallet_data.awaiting_start_festivals.try_push(festival_id).unwrap();
          
          Ok(())
        })?;

        //bind the duration to the festival
        Self::do_bind_start_block_to_festival(festival_id, start_block)?;
        Self::do_bind_end_block_to_festival(festival_id, end_block)?;
        festival.block_start_end = (start_block, end_block);
        festival.status = FestivalStatus::AwaitingStartBlock;

        Self::deposit_event(Event::FestivalActivated(festival_id, who));
        Ok(().into())
      })?;

      Ok(().into())
    }

    #[pallet::call_index(2)]#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
    pub fn activate_festival_asap(
        origin: OriginFor<T>,
        festival_id: T::FestivalId,
        end_block: BlockNumberFor<T>,
    )-> DispatchResultWithPostInfo{
        
      let who = ensure_signed(origin)?;
      
      // mutate the festival from storage
      Festivals::<T>::try_mutate_exists( festival_id,|fes| -> DispatchResult{
        let festival = fes.as_mut().ok_or(Error::<T>::BadMetadata)?;

        // ensure the owner owns the festival 
        ensure!(
            festival.owner == who.clone(),
            Error::<T>::NoFestivalAdminAccess
        );
        // ensure the festival has at least 2 movies
        ensure!(
            festival.internal_movies.len() + festival.external_movies.len() > 1,
            Error::<T>::NotEnoughMoviesInFestival
        );
        // ensure the status is AwaitingActivation
        ensure!(
            festival.status == FestivalStatus::AwaitingActivation,
            Error::<T>::NotAwaitingActivation
        );

        // ensure the block periods are valid
        let now = frame_system::Pallet::<T>::block_number();
        ensure!(
            end_block - now >= BlockNumberFor::<T>::from(T::MinFesBlockDuration::get()), 
            Error::<T>::FestivalPeriodTooShort
        );

        // update the festival ownership status
        WalletFestivalData::<T>::try_mutate_exists( who.clone(), |wal_data| -> DispatchResult{
          let wallet_data = wal_data.as_mut().ok_or(Error::<T>::NonexistentFestival)?;
          
          //filter the movie from the awaiting activation list
          wallet_data.awaiting_activation_festivals.retain(
              |fes_id| 
              fes_id != &festival_id.clone()
                  
          );
          wallet_data.active_festivals.try_push(festival_id).unwrap();
          
          Ok(().into())
        })?;

        //bind the duration to the festival
        Self::do_bind_end_block_to_festival(festival_id, end_block)?;
        let start_block = <frame_system::Pallet<T>>::block_number();
        let mut festival_duration = 
            end_block
            .checked_sub(&start_block)
            .ok_or(Error::<T>::Underflow)?;
        let vote_decrease_margin =
            festival_duration
            .checked_div(&BlockNumberFor::<T>::from(10u32))
            .ok_or(Error::<T>::Underflow)?;
        let power_decrease_block = 
            end_block
            .checked_sub(&vote_decrease_margin)
            .ok_or(Error::<T>::Underflow)?;

        festival.block_start_end = (start_block, end_block);
        festival.status = FestivalStatus::Active;
        festival.vote_power_decrease_block = power_decrease_block;


        Self::deposit_event(Event::FestivalActivated(festival_id, who));
        Ok(().into())
      })?;

      Ok(().into())
    }
              
    #[pallet::call_index(3)]#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
    pub fn add_movies_to_fest(
        origin: OriginFor<T>,
        festival_id: T::FestivalId,
        mut internal_movie_ids: BoundedVec<String, T::MaxMoviesInFest>,
        mut external_movie_ids: BoundedVec<String, T::MaxMoviesInFest>,
    )-> DispatchResultWithPostInfo{
    
      let who = ensure_signed(origin)?;

      let (bounded_internal_movie_ids, bounded_external_movie_ids) 
          = Self::do_validate_add_movies_to_existing_festival(festival_id.clone(), internal_movie_ids.clone(), external_movie_ids.clone())?;

      Festivals::<T>::try_mutate_exists(festival_id, |festival| -> DispatchResult {
        let fes = festival.as_mut().ok_or(Error::<T>::NonexistentFestival)?;
        ensure!(
            fes.status == FestivalStatus::AwaitingActivation,
            Error::<T>::FestivalNotAcceptingNewMovies
        );


        // add the movies to the festival
        for internal_movie in bounded_internal_movie_ids {
            fes.internal_movies.try_push(internal_movie);
        }

        // add the movies to the festival

        for external_movie in bounded_external_movie_ids {
            fes.external_movies.try_push(external_movie);
        }
        
        Ok(().into())
      })?;

      Self::deposit_event(Event::MoviesAddedToFestival(festival_id, who.clone()));
      Ok(().into())
    }
    
    #[pallet::call_index(4)]#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
    pub fn remove_movies_from_fest(
        origin: OriginFor<T>,
        festival_id: T::FestivalId,
        internal_movie_ids: BoundedVec<String, T::MaxMoviesInFest>,
        external_movie_ids: BoundedVec<String, T::MaxMoviesInFest>,
    )-> DispatchResultWithPostInfo{
    
      let who = ensure_signed(origin)?;
      // ensure!(
      // 	kine_stat_tracker::Pallet::<T>::is_wallet_registered(who.clone())?,
      // 	Error::<T>::WalletStatsRegistryRequired,
      // );
      
      Festivals::<T>::try_mutate_exists(festival_id, |festival| -> DispatchResult {
        let fes = festival.as_mut().ok_or(Error::<T>::BadMetadata)?;

        // Validate the names
        let mut temp_name: BoundedVec<u8, T::LinkStringLimit>;
        
        let mut validated_internal_movie_ids: BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInFest>
            = TryInto::try_into(Vec::new()).unwrap();
        for internal_movie in internal_movie_ids {
            temp_name = TryInto::try_into(internal_movie.as_bytes().to_vec()).map_err(|_|Error::<T>::BadMetadata)?;
            ensure!(!fes.vote_map.contains_key(&temp_name), Error::<T>::CannotRemoveFestivalWithVotes);
            validated_internal_movie_ids.try_push(temp_name);
        }

        let mut validated_external_movie_ids: BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInFest>
            = TryInto::try_into(Vec::new()).unwrap();
        for external_movie in external_movie_ids {
            temp_name = TryInto::try_into(external_movie.as_bytes().to_vec()).map_err(|_|Error::<T>::BadMetadata)?;
            ensure!(!fes.vote_map.contains_key(&temp_name), Error::<T>::CannotRemoveFestivalWithVotes);
            validated_external_movie_ids.try_push(temp_name);
        }

        //filter only the movies not in internal_movie_ids
        fes.internal_movies.retain(
            |movie_id| 
            !validated_internal_movie_ids.contains(movie_id)
        );

        //filter only the movies not in external_movie_ids
        fes.external_movies.retain(
            |movie_id| 
            !validated_external_movie_ids.contains(movie_id)
        );
        
        Ok(().into())
      })?;

      Self::deposit_event(Event::MoviesAddedToFestival(festival_id, who.clone()));
      Ok(().into())
    }

    // Cast a vote for a movie included in the festival.
    #[pallet::call_index(5)]#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
    pub fn vote_for_movie_in_festival(
        origin: OriginFor<T>,
        festival_id: T::FestivalId,
        movie_id: String,
        vote_amount: BalanceOf<T>,
        user: String,
    )-> DispatchResultWithPostInfo{
        
      let who = ensure_signed(origin)?;

      Self::do_vote_for_movie_in_festival(&who,festival_id, movie_id.clone(), vote_amount, user.clone())?;

      Self::deposit_event(Event::VotedForMovieInFestival(festival_id, movie_id, who.clone(), user));
      Ok(().into())
    }

    #[pallet::call_index(6)]#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
    pub fn claim_festival_rewards(
        origin: OriginFor<T>,
    ) -> DispatchResultWithPostInfo {
        
      let who = ensure_signed(origin)?;
      
      let mut reward = BalanceOf::<T>::from(0u32);
      
      let claimable_tokens_festival = 
        kine_stat_tracker::Pallet::<T>
        ::get_wallet_tokens(who.clone()).unwrap()
        .claimable_tokens_festival;
      
      ensure!(
        claimable_tokens_festival > BalanceOf::<T>::from(0u32),
        Error::<T>::NoClaimableTokens
      );

      <T as kine_stat_tracker::Config>::Currency::transfer(
        &Self::account_id(),  &who.clone(),
        claimable_tokens_festival.clone(), AllowDeath, 
      );
          
      kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
        who.clone(), 
        kine_stat_tracker::FeatureType::Festival,
        kine_stat_tracker::TokenType::Claimable,
        claimable_tokens_festival.clone(), true
      )?;
  
      Self::deposit_event(Event::FestivalTokensClaimed(who, claimable_tokens_festival));
      Ok(().into())
    }	

    #[pallet::call_index(7)]#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
    pub fn edit_festival_details(
        origin: OriginFor<T>,
        festival_id: T::FestivalId,
        new_name_str: String,
        new_description_str: String,
    ) -> DispatchResultWithPostInfo {
      let who = ensure_signed(origin)?;
      Festivals::<T>::try_mutate_exists(festival_id, |festival_option| -> DispatchResult {
          let festival = festival_option.as_mut().ok_or(Error::<T>::NonexistentFestival)?;
          ensure!(
              festival.owner == who,
              Error::<T>::NoFestivalAdminAccess
          );
          let new_name: BoundedVec<u8, T::NameStringLimit> =
              TryInto::try_into(new_name_str.as_bytes().to_vec()).map_err(|_| Error::<T>::BadMetadata)?;
          let new_description: BoundedVec<u8, T::DescStringLimit> =
              TryInto::try_into(new_description_str.as_bytes().to_vec()).map_err(|_| Error::<T>::BadMetadata)?;
          festival.name = new_name.clone();
          festival.description = new_description.clone();
  
          Self::deposit_event(Event::FestivalDetailsUpdated(festival_id, who, new_name.to_vec(), new_description.to_vec()));
  
          Ok(().into())
        })?;
  
      Ok(().into())
    }

    #[pallet::call_index(8)] #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
    pub fn remove_festival(
      origin: OriginFor<T>,
      festival_id: T::FestivalId,
    ) -> DispatchResultWithPostInfo {
    let who = ensure_signed(origin)?;

    Festivals::<T>::try_mutate_exists(festival_id, |festival_option| -> DispatchResult {
      let festival = festival_option.as_mut().ok_or(Error::<T>::NonexistentFestival)?;

      ensure!(
          festival.owner == who,
          Error::<T>::NoFestivalAdminAccess
      );

      ensure!(
        festival.status != FestivalStatus::Active ,
        Error::<T>::NotPermissionToRemove
      );

      ensure!(
        festival.vote_map.is_empty(),
        Error::<T>::FestivalHaveVote
      );

      *festival_option = None;

      Self::deposit_event(Event::FestivalRemoved(festival_id, who));

      Ok(().into())
    })?;

    Ok(().into())
  }
  }

  impl<T: Config> Pallet<T> {
    pub fn do_create_festival(
      who: T::AccountId,
      name_str: String,
      owner_id: String, // Novo parâmetro adicionado
      description_str: String,
      min_ticket_price: BalanceOf<T>,
      internal_movie_ids: BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInFest>,
      external_movie_ids: BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInFest>,
      category_tag_list: BoundedVec<(CategoryId<T>, TagId<T>), T::MaxTags>,
      status: FestivalStatus,
  ) -> Result<T::FestivalId, DispatchError> {
      let festival_id = NextFestivalId::<T>::try_mutate(|id| -> Result<T::FestivalId, DispatchError> {
          let current_id = *id;
          *id = id.checked_add(&One::one()).ok_or(Error::<T>::Overflow)?;
          Ok(current_id)
      })?;
  
      let name: BoundedVec<u8, T::NameStringLimit> =
          TryInto::try_into(name_str.as_bytes().to_vec()).map_err(|_| Error::<T>::BadMetadata)?;
  
      let owner_id_bounded: BoundedVec<u8, T::NameStringLimit> =
          TryInto::try_into(owner_id.as_bytes().to_vec()).map_err(|_| Error::<T>::BadMetadata)?;
  
      let description: BoundedVec<u8, T::DescStringLimit> =
          TryInto::try_into(description_str.as_bytes().to_vec()).map_err(|_| Error::<T>::BadMetadata)?;
  
      let bounded_film_list: BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInFest> =
          TryInto::try_into(Vec::new()).map_err(|_| Error::<T>::BadMetadata)?;
  
      let bounded_vote_map: BoundedBTreeMap<
          BoundedVec<u8, T::LinkStringLimit>,
          BoundedVec<Vote<T::AccountId, BalanceOf<T>, BoundedVec<u8, T::NameStringLimit>>, T::MaxVotes>,
          T::MaxVotes,
      > = BoundedBTreeMap::new();
  
      let zero_lockup = BalanceOf::<T>::from(0u32);
  
      let mut festival = Festival {
          id: festival_id.clone(),
          owner: who,
          owner_id: owner_id_bounded, // Inicializando o novo campo
          name,
          description,
          internal_movies: bounded_film_list.clone(),
          external_movies: bounded_film_list.clone(),
          winners: bounded_film_list,
          status,
          max_entry: min_ticket_price,
          total_lockup: zero_lockup,
          vote_map: bounded_vote_map,
          categories_and_tags: category_tag_list,
          block_start_end: (BlockNumberFor::<T>::from(0u32), BlockNumberFor::<T>::from(0u32)),
          vote_power_decrease_block: BlockNumberFor::<T>::from(0u32),
      };
  
      for movie_id in internal_movie_ids {
          festival.internal_movies.try_push(movie_id);
      }
  
      for movie_id in external_movie_ids {
          festival.external_movies.try_push(movie_id);
      }
  
      Festivals::<T>::insert(festival_id, festival);
  
      Ok(festival_id)
  }

    pub fn do_bind_owners_to_festival(
        who : T::AccountId,
        festival_id : T::FestivalId,
    ) -> Result<(), DispatchError> {

      if !WalletFestivalData::<T>::contains_key(who.clone()) {
        let mut new_data = Self::do_create_new_wallet_data().unwrap();
        new_data.all_owned_festivals.try_push(festival_id).unwrap();
        new_data.awaiting_activation_festivals.try_push(festival_id).unwrap();
        WalletFestivalData::<T>::insert(who.clone(), new_data);
      }
      else {
        WalletFestivalData::<T>::try_mutate(who.clone(), |festival_data| -> DispatchResult{
          let fes_data = festival_data.as_mut().ok_or(Error::<T>::NonexistentFestival)?;
          fes_data.all_owned_festivals.try_push(festival_id).unwrap();
          fes_data.awaiting_activation_festivals.try_push(festival_id).unwrap();

          Ok(())
        })?;
      }
      Ok(())
    }

    pub fn do_bind_start_block_to_festival(
        festival_id : T::FestivalId,
        start_block: BlockNumberFor<T>
    ) -> Result<(), DispatchError> {
        
      // check if any entries exist for the start block and push the movie if true
      if BlockAssignments::<T>::contains_key(start_block) {
        BlockAssignments::<T>::mutate_exists(start_block, |assignments| -> DispatchResult {
            let start_assignments = assignments.as_mut().ok_or(Error::<T>::BadMetadata)?;

            start_assignments.to_start.try_push(festival_id).unwrap();
            Ok(())
        })?;
      }
      // create a new entry for the start block if none exist and then push the movie
      else {
        let mut bounded_start_list: BoundedVec<T::FestivalId, T::MaxFestivalsPerBlock>
            = TryInto::try_into(Vec::new()).map_err(|_|Error::<T>::BadMetadata)?;
        bounded_start_list.try_push(festival_id).unwrap();
        let mut bounded_end_list: BoundedVec<T::FestivalId, T::MaxFestivalsPerBlock>
            = TryInto::try_into(Vec::new()).map_err(|_|Error::<T>::BadMetadata)?;

        let assignment = BlockAssignment {
            to_start: bounded_start_list.clone(),
            to_end: bounded_end_list.clone(),
        };
        BlockAssignments::<T>::insert(start_block.clone(), assignment);
      }

      Ok(())
    }

    pub fn do_bind_end_block_to_festival(
        festival_id : T::FestivalId,
        end_block: BlockNumberFor<T>
    ) -> Result<(), DispatchError> {

      // check if any entries exist for the end block and push the movie if true
      if BlockAssignments::<T>::contains_key(end_block) {
        BlockAssignments::<T>::mutate_exists(end_block, |assignments| -> DispatchResult {
            let end_block_assignments = assignments.as_mut().ok_or(Error::<T>::BadMetadata)?;
            
            end_block_assignments.to_end.try_push(festival_id).unwrap();
            Ok(())
        })?;
      }

      else {
        let mut bounded_start_list: BoundedVec<T::FestivalId, T::MaxFestivalsPerBlock>
            = TryInto::try_into(Vec::new()).map_err(|_|Error::<T>::BadMetadata)?;
        let mut bounded_end_list: BoundedVec<T::FestivalId, T::MaxFestivalsPerBlock>
            = TryInto::try_into(Vec::new()).map_err(|_|Error::<T>::BadMetadata)?;
        bounded_end_list.try_push(festival_id).unwrap();
        
        let assignment = BlockAssignment {
            to_start: bounded_start_list.clone(),
            to_end: bounded_end_list.clone(),
        };
        BlockAssignments::<T>::insert(end_block.clone(), assignment);
      }

      Ok(())
    }

    pub fn do_create_empty_block_assignments(
        festival_id : T::FestivalId,
    ) -> Result<(), DispatchError> {

      let mut bounded_start_list: BoundedVec<T::FestivalId, T::MaxOwnedFestivals>
          = TryInto::try_into(Vec::new()).map_err(|_|Error::<T>::BadMetadata)?;
      bounded_start_list.try_push(festival_id).unwrap();
      
      let mut bounded_end_list: BoundedVec<T::FestivalId, T::MaxOwnedFestivals>
          = TryInto::try_into(Vec::new()).map_err(|_|Error::<T>::BadMetadata)?;
      bounded_end_list.try_push(festival_id).unwrap();
      
      
      let assignment = BlockAssignment {
          to_start: bounded_start_list.clone(),
          to_end: bounded_end_list.clone(),
      };
      
      Ok(())
    }

    fn hook_activate_festival(
        now : BlockNumberFor<T>,
    ) -> DispatchResult {
        
      let fests = BlockAssignments::<T>::try_get(now);
      ensure!(fests.is_ok(), Error::<T>::NonexistentFestival);

      let festivals = fests.unwrap();
      for festival_id in festivals.to_start.iter() {
        Festivals::<T>::try_mutate_exists( festival_id,|festival| -> DispatchResult {
          let fest = festival.as_mut().ok_or(Error::<T>::NonexistentFestival)?;

          let is_fest_new = fest.status == FestivalStatus::AwaitingStartBlock;
          // let is_creator_registered = (kine_stat_tracker::Pallet::<T>::is_wallet_registered(fest.owner.clone())?); //TODO-8
          let is_creator_registered = true;
          if is_fest_new && is_creator_registered {
              // update the festival ownership status
              WalletFestivalData::<T>::try_mutate_exists( fest.owner.clone(), |wal_data| -> DispatchResult{
                let wallet_data = wal_data.as_mut().ok_or(Error::<T>::NonexistentFestival)?;
                
                //filter the festival from the awaiting activation list
                wallet_data.awaiting_start_festivals.retain(
                    |fes_id| 
                    fes_id != &festival_id.clone()
                );
                wallet_data.active_festivals.try_push(festival_id.clone()).unwrap();
                
                Ok(())
              })?;
              
              fest.status = FestivalStatus::Active;
              Self::deposit_event(Event::FestivalHasBegun(festival_id.clone()));
          } //TODO-9
            
            Ok(())
        })?;
      }
      Ok(())
    }

    fn hook_deactivate_festival(
        now : BlockNumberFor<T>,
    ) -> DispatchResult {
        
      let fests = BlockAssignments::<T>::try_get(now);
      ensure!(fests.is_ok(), Error::<T>::NonexistentFestival);
      
      let festivals = fests.unwrap();
      for festival_id in festivals.to_end.iter() {
        Festivals::<T>::try_mutate_exists( festival_id,|festival| -> DispatchResult {
          let fest = festival.as_mut().ok_or(Error::<T>::NonexistentFestival)?;
          
          if fest.status == FestivalStatus::Active {
              
            // update the festival ownership status
            Self::do_active_to_finished_fest_ownership(fest.owner.clone(), festival_id.clone());
            
            if fest.vote_map.values().len() > 0 {
              fest.status = FestivalStatus::Finished;
              let (winning_voters, winning_movies) = Self::do_resolve_market(festival_id.clone())?;
              for movie_id in winning_movies.clone() {
                  fest.winners.try_push(movie_id.clone()).unwrap();
              }
              Self::deposit_event(Event::FestivalHasEnded(festival_id.clone(), winning_voters, winning_movies));
            }
            else {
              fest.status = FestivalStatus::FinishedNotEnoughVotes;
              Self::deposit_event(Event::FestivalHasEndedUnsuccessfully(festival_id.clone()));
            }

          }

          Ok(())
        })?;
      }
      
      Ok(())
    }

    // This function is isolated so that if it fails, the rest of the festivals
    // in the hook are not compromised.
    fn do_active_to_finished_fest_ownership(
        owner: T::AccountId,
        festival_id : T::FestivalId
    ) -> DispatchResult {
        
      // update the festival ownership status
      WalletFestivalData::<T>::try_mutate_exists(owner, |wal_data| -> DispatchResult{
        let wallet_data = wal_data.as_mut().ok_or(Error::<T>::NonexistentFestival)?;
        
        //filter the movie from the awaiting activation list
        wallet_data.active_festivals.retain(
            |fes_id| 
            fes_id != &festival_id.clone()
        );
        wallet_data.finished_festivals.try_push(festival_id.clone()).unwrap();
        
        Ok(())
      })?;

      Ok(())
    }

    pub fn do_vote_for_movie_in_festival(
        who: &T::AccountId,
        festival_id: T::FestivalId,
        movie_id_str: String,
        vote_amount : BalanceOf<T>,
        user_str: String,
    )-> Result<(), DispatchError> {
        
      Festivals::<T>::try_mutate_exists(festival_id, |festival| -> DispatchResult {
        let mut fest = festival.as_mut().ok_or(Error::<T>::NonexistentFestival)?;   
        let user: BoundedVec<u8, T::NameStringLimit> = TryInto::try_into(user_str.into_bytes()).map_err(|_| Error::<T>::BadMetadata)?;
        let movie_id: BoundedVec<u8, T::LinkStringLimit>
            = TryInto::try_into(movie_id_str.as_bytes().to_vec()).map_err(|_|Error::<T>::BadMetadata)?;

        ensure!(
            (fest.internal_movies.contains(&movie_id.clone())
            || fest.external_movies.contains(&movie_id.clone())),
            Error::<T>::MovieNotInFestival
        );
        // ensure!(fest.owner != who.clone(), Error::<T>::CannotVoteInOwnFestival);
        ensure!(fest.status == FestivalStatus::Active, Error::<T>::FestivalNotActive);
        ensure!(vote_amount >  BalanceOf::<T>::from(0u32), Error::<T>::VoteValueCannotBeZero);
        if fest.max_entry > BalanceOf::<T>::from(0u32) {
            ensure!(vote_amount <= fest.max_entry, Error::<T>::VoteValueTooHigh);
        }

        let mut vote_weight = vote_amount.clone();
        let current_block = <frame_system::Pallet<T>>::block_number();
        if current_block > fest.vote_power_decrease_block {
          let (_, end_block) = fest.block_start_end;
          
          let vote_moment_aux =
              end_block
              .checked_sub(&current_block)
              .ok_or(Error::<T>::Underflow)?;
          ensure!(vote_moment_aux > BlockNumberFor::<T>::from(1u32), Error::<T>::FestivalNotActive);
          
          let vote_decrease_aux = 
              end_block
              .checked_sub(&fest.vote_power_decrease_block.into())
              .ok_or(Error::<T>::Underflow)?;

          // let vote_moment_margin: u32 = vote_moment_aux.into();

          let vote_moment_margin: u32
              = TryInto::try_into(vote_moment_aux).map_err(|_|Error::<T>::BadMetadata)?;

          let vote_decrease_margin: u32
              = TryInto::try_into(vote_decrease_aux).map_err(|_|Error::<T>::BadMetadata)?;

          vote_weight =
              vote_weight
              .saturating_mul(BalanceOf::<T>::from(vote_moment_margin));

          vote_weight =
              vote_weight
              .checked_div(&BalanceOf::<T>::from(vote_decrease_margin))
              .ok_or(Error::<T>::Underflow)?;
        }

        <T as kine_stat_tracker::Config>::Currency::transfer(
            who, &Self::account_id(),
            vote_amount, AllowDeath,
        );
        kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
            who.clone(), 
            kine_stat_tracker::FeatureType::Festival,
            kine_stat_tracker::TokenType::Locked,
            vote_amount.clone(), false
        ).unwrap();
        
        let vote = Vote {
            voter: who.clone(),
            amount: vote_amount,
            amount_after_weight: vote_weight,
            user: user
        };

        fest.total_lockup = fest.total_lockup.checked_add(&vote_amount).ok_or(Error::<T>::Overflow)?;
        
        if fest.vote_map.contains_key(&movie_id.clone()) {
            fest.vote_map.get_mut(&movie_id.clone()).unwrap().try_push(vote).unwrap();
        }
        else {
            let mut bounded_vote_list : BoundedVec<Vote<T::AccountId, BalanceOf<T>,  BoundedVec<u8, T::NameStringLimit>>, T::MaxVotes>
                = TryInto::try_into(Vec::new()).map_err(|_|Error::<T>::BadMetadata)?;
            bounded_vote_list.try_push(vote).unwrap();
            fest.vote_map.try_insert(movie_id.clone(), bounded_vote_list).unwrap();
        }

        if !WalletFestivalData::<T>::contains_key(who.clone()) {
            let mut new_data = Self::do_create_new_wallet_data().unwrap();
            new_data.active_voted_festivals.try_push(festival_id).unwrap();
            WalletFestivalData::<T>::insert(who.clone(), new_data);
        }
        else {
            WalletFestivalData::<T>::try_mutate( who.clone(), |festival_data| -> DispatchResult{
                let fes_data = festival_data.as_mut().ok_or(Error::<T>::NonexistentFestival)?;
                fes_data.active_voted_festivals.try_push(festival_id).unwrap();
                
                Ok(())
            })?;
        }
        Ok(())
      })       
    }

    fn account_id() -> T::AccountId {
        <T as Config>::PalletId::get().try_into_account().unwrap()
    }

    fn do_validate_add_movies_to_new_festival(
        internal_movie_ids: BoundedVec<String, T::MaxMoviesInFest>,
        external_movie_ids: BoundedVec<String, T::MaxMoviesInFest>,
    ) -> Result<(BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInFest>, BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInFest>), DispatchError> {
        
      let mut bounded_movie_name: BoundedVec<u8, T::LinkStringLimit>;

      // validate internal movies
      let mut validated_internal_movie_ids: BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInFest>
          = TryInto::try_into(Vec::new()).unwrap();
      
      for internal_movie in internal_movie_ids {
          bounded_movie_name = TryInto::try_into(internal_movie.as_bytes().to_vec()).map_err(|_|Error::<T>::BadMetadata)?;
          kine_movie::Pallet::<T>::do_ensure_internal_movie_exist(bounded_movie_name.clone())?;
          validated_internal_movie_ids.try_push(bounded_movie_name);
      }

      // validate external movies
      let mut validated_external_movie_ids: BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInFest>
          = TryInto::try_into(Vec::new()).unwrap();
      
      for external_movie in external_movie_ids {
          bounded_movie_name = TryInto::try_into(external_movie.as_bytes().to_vec()).map_err(|_|Error::<T>::BadMetadata)?;
          kine_movie::Pallet::<T>::do_ensure_external_movie_exists(bounded_movie_name.clone())?;
          validated_external_movie_ids.try_push(bounded_movie_name);
      }
      
      Ok((validated_internal_movie_ids, validated_external_movie_ids))
    }

    fn do_validate_add_movies_to_existing_festival(
        festival_id: T::FestivalId,
        internal_movie_ids: BoundedVec<String, T::MaxMoviesInFest>,
        external_movie_ids: BoundedVec<String, T::MaxMoviesInFest>,
    ) -> Result<(BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInFest>, BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInFest>), DispatchError> {
        
      let (validated_internal_movie_ids, validated_external_movie_ids) = Festivals::<T>::try_mutate_exists(festival_id, |fest| -> 
      Result<(BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInFest>, BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInFest>), DispatchError> {
        let festival = fest.as_mut().ok_or(Error::<T>::NonexistentFestival)?; 

        let mut bounded_movie_name: BoundedVec<u8, T::LinkStringLimit>;

        // create aux lists
        let mut validated_internal_movie_ids: BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInFest>
            = TryInto::try_into(Vec::new()).unwrap();
        let mut validated_external_movie_ids: BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInFest>
            = TryInto::try_into(Vec::new()).unwrap();

        // validate internal movies
        for internal_movie in internal_movie_ids {
            bounded_movie_name = TryInto::try_into(internal_movie.as_bytes().to_vec()).map_err(|_|Error::<T>::BadMetadata)?;
            // kine_movie::Pallet::<T>::do_ensure_internal_movie_exist(bounded_movie_name.clone())?;
            ensure!(!festival.internal_movies.contains(&bounded_movie_name), Error::<T>::NoFestivalAdminAccess);
            validated_internal_movie_ids.try_push(bounded_movie_name);
        }

        // validate external movies
        for external_movie in external_movie_ids {
            bounded_movie_name = TryInto::try_into(external_movie.as_bytes().to_vec()).map_err(|_|Error::<T>::BadMetadata)?;
            // kine_movie::Pallet::<T>::do_ensure_external_movie_exists(bounded_movie_name.clone())?;
            validated_external_movie_ids.try_push(bounded_movie_name);
        }
        Ok((validated_internal_movie_ids, validated_external_movie_ids))

      })?;

      
      Ok((validated_internal_movie_ids, validated_external_movie_ids))
    }

    fn do_resolve_market(
        festival_id: T::FestivalId
    ) -> Result<(Vec<T::AccountId>, BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInFest>), DispatchError> {
        
      // Pay the owner's share and calculate the remaining pool
      //TODO-14
      let (winning_voters, winning_movies) = Festivals::<T>::try_mutate(festival_id, |fest| 
      -> Result<(Vec<T::AccountId>, BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInFest>), DispatchError> {
          let mut festival = fest.as_mut().ok_or(Error::<T>::NonexistentFestival)?;  
              
          // Get the winning movie_ids by vote power
          let festival_winners = Self::do_get_winning_options(festival_id).unwrap();
          // Self::do_assign_wins_to_uploaders(festival_id, festival_winners).unwrap();
          
          let (owner_reward, total_lockup_after_owner) = Self::do_calculate_owner_reward(festival.owner.clone(), festival.total_lockup).unwrap();
          
          // get the winning voter's lockup and each of their respective winning vote lockup and the total winning votes
          let (winning_voters_lockup, winning_vote_map) = Self::do_get_winners_total_lockup(festival_id, festival_winners.clone()).unwrap();
          
          let remaining_lockup = Self::do_calculate_voters_reward(total_lockup_after_owner, winning_voters_lockup, winning_vote_map.clone()).unwrap();

          Self::do_handle_owner_reward( festival.owner.clone(), owner_reward, remaining_lockup).unwrap();

          Ok((winning_vote_map.into_keys().collect(), festival_winners.clone()))
      })?;

      Ok((winning_voters, winning_movies))
    }

    fn do_get_winning_options(
        festival_id : T::FestivalId
    ) -> Result<BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInFest>, DispatchError> {
    
      let mut vote_power_map = BTreeMap::new(); // map of movie_id: total_voting_power

      // iterate the festival's votes and calculate the total voting power of each
      let fes_votes = Festivals::<T>::try_get(festival_id).unwrap().vote_map;
      for (movie_id, vote_list) in fes_votes {
        for vote in vote_list {
            let movie_power =  vote_power_map.entry(movie_id.clone()).or_insert(BalanceOf::<T>::from(0u32));
            *movie_power += vote.amount_after_weight;
        }
      }

      // determine the first winner and add it to a list
      let first_winner = vote_power_map
          .iter()
          // .clone()
          .max_by_key(|p| p.1)
          .unwrap();
      let mut winners = vec![first_winner.0.clone()];
      
      // untie by adding all entries with the same lockup to the winner list
      for (movie, lockup) in &vote_power_map {
        if lockup == first_winner.1 && movie != first_winner.0 {
            winners.push(movie.clone());
        }
      }

      let bounded_winners: BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInFest>
        = TryInto::try_into(winners).map_err(|_|Error::<T>::BadMetadata)?;

      Ok(bounded_winners)
    }

    // Calculates both the winner's total lockup and each of the voter's individual lockup and total winning votes. 
    fn do_get_winners_total_lockup(
        festival_id: T::FestivalId, 
        winning_movies: BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInFest>
    ) -> Result<(BalanceOf<T>, BTreeMap<T::AccountId, (BalanceOf<T>, u32)>), DispatchError> {
        
      let mut winners_total_lockup = BalanceOf::<T>::from(0u32);
      let mut winning_vote_map = BTreeMap::new(); // map of voter_id: total_correct_tokens_voted

      let fes_votes = Festivals::<T>::try_get(festival_id).unwrap().vote_map; // movie_id: vote_list
      for movie_id in winning_movies {
          
        let winning_votes = fes_votes.get(&movie_id.clone()).unwrap();
        for vote in winning_votes {

          winners_total_lockup = 
              winners_total_lockup
              .checked_add(&vote.amount.clone())
              .ok_or(Error::<T>::Overflow)?;

          let (token_earning_total, winning_vote_total) =
              winning_vote_map
              .entry(vote.voter.clone())
              .or_insert((
                  BalanceOf::<T>::from(0u32),
                  0u32
              ));

          *token_earning_total += vote.amount;
          *winning_vote_total += 1u32;
        }
      }
  
      Ok((winners_total_lockup, winning_vote_map))
    }

    fn do_assign_wins_to_uploaders(
        festival_id: T::FestivalId,
        winners: Vec<BoundedVec<u8, T::LinkStringLimit>>,
    ) -> DispatchResultWithPostInfo {
        
      // verify if movies still exist, and assign the win to the uploader
      for movie_id in winners.clone() {
          let uploader = kine_movie::Pallet::<T>
              ::get_movie_uploader(movie_id)?;
          // assign wins to the uploaders of the winning movies
          if !WalletFestivalData::<T>::contains_key(uploader.clone()) {
            let mut new_data = Self::do_create_new_wallet_data().unwrap();
            new_data.won_festivals.try_push(festival_id).unwrap();
            WalletFestivalData::<T>::insert(uploader.clone(), new_data);
          }
          else {
            WalletFestivalData::<T>::try_mutate_exists( uploader.clone(), |festival_data| -> DispatchResult{
              let fes_data = festival_data.as_mut().ok_or(Error::<T>::NonexistentFestival)?;
              fes_data.won_festivals.try_push(festival_id).unwrap();
              
              Ok(())
            })?;
          }
      }

      Ok(().into())
    }

    fn do_calculate_voters_reward(
        total_lockup: BalanceOf<T>,
        winners_lockup: BalanceOf<T>,
        winning_vote_map: BTreeMap<T::AccountId, (BalanceOf<T>, u32)>,
    ) -> Result<BalanceOf<T>, DispatchError> {

      let mut user_share: Perbill;
      let mut user_reward: BalanceOf<T>;
      let mut remaining_lockup = total_lockup;

      for (voter, (user_winning_votes_lockup, _)) in winning_vote_map {

        user_share = Perbill::from_rational(user_winning_votes_lockup, winners_lockup);
        user_reward = 
            user_share
            .mul_floor(total_lockup);
        remaining_lockup =
            remaining_lockup
            .checked_sub(&user_reward)
            .ok_or(Error::<T>::Underflow)?;

        // update the claimable tokens
        kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
            voter.clone(), 
            kine_stat_tracker::FeatureType::Festival,
            kine_stat_tracker::TokenType::Claimable,
            user_reward, false,
        ).unwrap();

      }
      Ok(remaining_lockup)
    }

    // Returns the owner's share and the remaining prize pool.
    fn do_calculate_owner_reward(
        owner_id: T::AccountId,
        total_lockup: BalanceOf<T>,
    ) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
      //TODO-16
      let owner_reward = 
          total_lockup
          .checked_div(&50u32.into()) // 2%
          .ok_or(Error::<T>::Underflow)?;

      let mut remaining_lockup =
          total_lockup
          .checked_sub(&owner_reward)
          .ok_or(Error::<T>::Underflow)?;
      Ok((owner_reward, remaining_lockup))
    }

    fn do_handle_owner_reward(
        owner_id: T::AccountId,
        owner_share: BalanceOf<T>,
        remaining_lockup: BalanceOf<T>,
    ) -> DispatchResultWithPostInfo {
            
      let total_share = 
          owner_share
          .checked_add(&remaining_lockup) // 2%
          .ok_or(Error::<T>::Overflow)?;

      kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
          owner_id, 
          kine_stat_tracker::FeatureType::Festival,
          kine_stat_tracker::TokenType::Claimable,
          total_share, false,
      ).unwrap();

      Ok(().into())
    }
    
    fn do_create_new_wallet_data(
    ) -> Result<WalletData<BoundedVec<T::FestivalId, T::MaxOwnedFestivals>>, DispatchError> {

      let bounded_festival_list : BoundedVec<T::FestivalId, T::MaxOwnedFestivals>
          = TryInto::try_into(Vec::new()).map_err(|_|Error::<T>::BadMetadata)?;

      let new_data = WalletData {
          all_owned_festivals: bounded_festival_list.clone(),
          awaiting_activation_festivals: bounded_festival_list.clone(),
          awaiting_start_festivals: bounded_festival_list.clone(),
          active_festivals: bounded_festival_list.clone(),
          finished_festivals: bounded_festival_list.clone(),
          won_festivals: bounded_festival_list.clone(),

          active_voted_festivals: bounded_festival_list.clone(),
          finished_voted_festivals: bounded_festival_list.clone(),
          finished_won_festivals: bounded_festival_list,
      };

      Ok(new_data)
    }
  }

}
