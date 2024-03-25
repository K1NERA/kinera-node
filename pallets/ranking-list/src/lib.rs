//** About **//
	// Ranking lists order content based on votes and provide staking
	// based on locked funds.
	// new era > find staking differences > reward top 1000 films
	
	//TODO-0 add dynamic APY to governance
	//TODO-1 add dynamic deadlines, where if a MaxListsPerBlock is exceeded, a new block automatically calculated 
	//TODO-2 compare mint_into vs deposit_into_existing
	//TODO-3 validate the inserted deadline when creating a ranking list
	//TODO-4 improve the blocks_per_year calculation by creating a static variable in stat-tracker
	//TODO-5 sort tied entries by total votes after sorting the winners


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
		// const EXAMPLE_ID: LockIdentifier = *b"example ";
		
		//** Config **//
	
			//* Imports *//
				use super::*;
				use frame_support::{
					dispatch::DispatchResultWithPostInfo,
					pallet_prelude::*,
					traits::{
						Currency,
						ReservableCurrency,
						ExistenceRequirement::AllowDeath,
					},
					PalletId,
					BoundedVec,
					storage::bounded_btree_map::BoundedBTreeMap,
				};
				use frame_system::{
					pallet_prelude::*,
				};
				use sp_runtime::{
					RuntimeDebug, 
					traits::{
						AtLeast32BitUnsigned, 
						CheckedAdd,
						CheckedSub,
						One,
						Zero,
						AccountIdConversion,
						Saturating,
						CheckedDiv,
					},
				};
				use scale_info::{
                    TypeInfo,
                    prelude::{
                        vec::Vec,
                        string::String,
                    },
                };
				use codec::{MaxEncodedLen};
				use core::convert::TryInto;
				
				use kine_tags::{
					CategoryId as CategoryId,
					TagId as TagId,
				};
	
	
			//* Config *//
				#[pallet::pallet]
				pub struct Pallet<T>(_);
	
				#[pallet::config]
				pub trait Config: frame_system::Config + 
				kine_movie::Config + kine_tags::Config + kine_stat_tracker::Config {
				// + pallet_staking::Config + pallet_session::Config + pallet_utility::Config {
					type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
					// type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
					
					// how many ranking lists can be solved per block
					type MaxListsPerBlock: Get<u32>;
					type MaxVotersPerList: Get<u32>;
					type MaxMoviesInList: Get<u32>;
	
					// the minimum amount of blocks between a ranking list's refresh period
					type MinimumListDuration: Get<u32>;
	
					type RankingStringLimit: Get<u32>;
	
					type PalletId : Get<PalletId>;
				}
	
	
	
		//** Types **//	
	
			//* Types *//
				// type BalanceOf<T> =
				// 	<<T as pallet_staking::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
				
				type BalanceOf<T> = <<T as kine_stat_tracker::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	
				pub type RankingListId = u32;	
				
			//* Constants *//
			//* Enums *//
	
				#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug,TypeInfo,MaxEncodedLen)]
				pub enum RankingListStatus {
					Ongoing,
					Finished,
				}
	
				#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug,TypeInfo,MaxEncodedLen)]
				pub enum Conviction {
					/// 0.1x votes, unlocked.
					None,
					/// 1x votes, locked for an enactment period following a successful vote.
					Locked1x,
					/// 2x votes, locked for 2x enactment periods following a successful vote.
					Locked2x,
					/// 3x votes, locked for 4x...
					Locked3x,
					/// 4x votes, locked for 8x...
					Locked4x,
					/// 5x votes, locked for 16x...
					Locked5x,
					/// 6x votes, locked for 32x...
					Locked6x,
				}
	
	
			//* Structs *//
	
				#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug,TypeInfo, MaxEncodedLen)]
				pub struct RankingList<BoundedString, RankingListStatus, BlockNumber, MovieList, VoteMap, Balance, CategoryTagList> {
					pub name: BoundedString,
					pub description: BoundedString,
					pub status: RankingListStatus,
					pub list_duration: BlockNumber,
					pub list_deadline: BlockNumber,
					pub movies_in_list: MovieList, // this becomes a sorted winner list after the "list_deadline" block
					pub votes_by_user: VoteMap,
					pub total_lockup: Balance,
					pub categories_and_tags: CategoryTagList,
				}
	
				#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug,TypeInfo,MaxEncodedLen)]
				pub struct RankingVote<MovieId, BalanceOf, BlockNumber> {
					pub movie_id: MovieId,
					pub locked_amount: BalanceOf,
					pub conviction: Conviction,
					pub unlock_block: BlockNumber,
				}
	
	
				#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug,TypeInfo,MaxEncodedLen)]
				pub struct Deadlines<BoundedRankingLists> {
					pub list_deadlines: BoundedRankingLists,
				}
	
				#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug,TypeInfo,MaxEncodedLen)]
				pub struct UserVoteList<RankingListId> {
					pub user_vote_list: RankingListId,
				}
	
	
		
		//** Genesis **//
			
			#[pallet::genesis_config]
			#[derive(frame_support::DefaultNoBound)]
			pub struct GenesisConfig<T: Config> {
				pub default_ranking_lists: Vec<(
					BoundedVec<u8, T::RankingStringLimit>,
					BoundedVec<u8, T::RankingStringLimit>,
					u64,
					BoundedVec<(CategoryId<T>, TagId<T>), T::MaxTags>,
				)>
			}
	
	
			#[pallet::genesis_build]
			impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
				fn build(&self) {
					
					let category_type: kine_tags::CategoryType<T> =
						TryInto::try_into("Ranking List".as_bytes()
						.to_vec()).map_err(|_|Error::<T>::BadMetadata).unwrap();
	
					for (name, description, duration_u32, categories_and_tags) in &self.default_ranking_lists {
						
						let ranking_list_id =
							NextRankingListId::<T>::try_mutate(|id| -> Result<RankingListId, DispatchError> {
								let current_id = *id;
								*id = id
									.checked_add(One::one())
									.ok_or(Error::<T>::Overflow)?;
								Ok(current_id)
							}).unwrap();
						
						let movies_in_list: BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInList> =
							TryInto::try_into(Vec::new()).map_err(|_| Error::<T>::BadMetadata).unwrap();
						let current_block = <frame_system::Pallet<T>>::block_number();
						
						let duration_blocks: BlockNumberFor<T> =
							TryInto::try_into(T::MinimumListDuration::get()).map_err(|_| Error::<T>::BadMetadata).unwrap();
						let list_deadline_block = current_block.checked_add(&duration_blocks.clone()).ok_or(Error::<T>::Overflow).unwrap();
						Pallet::<T>::create_list_deadline(ranking_list_id, list_deadline_block).unwrap();
					
						let votes_by_user: BoundedBTreeMap<
							T::AccountId,
							BoundedVec<
								RankingVote<BoundedVec<u8, T::LinkStringLimit>, BalanceOf<T>, BlockNumberFor<T>>, T::MaxVotersPerList>,
								T::MaxVotersPerList,
							> 
						= BoundedBTreeMap::new();
						let total_lockup = BalanceOf::<T>::from(0u32);
	
						let ranking_list = RankingList {
							name: name.clone(),
							description: description.clone(),
							status:RankingListStatus::Ongoing,
							list_deadline: list_deadline_block.clone(),
							list_duration: duration_blocks.clone(),
							movies_in_list: movies_in_list.clone(),
							votes_by_user: votes_by_user.clone(),
							total_lockup: total_lockup.clone(),
							categories_and_tags: categories_and_tags.clone(),
						};
						<RankingLists<T>>::insert(ranking_list_id.clone(), ranking_list.clone());
					}
				}
			}
	
	
	
	
	
	
	
		//** Storage **//
	
			// Store the ID of the next Ranking List
			#[pallet::storage]
			#[pallet::getter(fn next_ranking_list_id)]
			pub(super) type NextRankingListId<T: Config> = StorageValue<
				_, 
				RankingListId,
				ValueQuery
			>;
	
			// Matches a RankingListId to that same Ranking List's data.
			// Contains a list of all the MovieIds in the ranking list. These IDs can
			// be used to retrieve the voting information in the ListVotes storage.
			#[pallet::storage]
			#[pallet::getter(fn ranking_list)]
			pub type RankingLists<T: Config> = StorageMap<
				_, 
				Blake2_128Concat, RankingListId, 
				RankingList<
					BoundedVec<u8, T::RankingStringLimit>,
					RankingListStatus,
					BlockNumberFor<T>,
					BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInList>, //Movies in List
					BoundedBTreeMap<
						T::AccountId, 
						BoundedVec<RankingVote<BoundedVec<u8, T::LinkStringLimit>, BalanceOf<T>, BlockNumberFor<T>>, T::MaxVotersPerList>, 
						T::MaxVotersPerList
					>,
					BalanceOf<T>,
					BoundedVec<(CategoryId<T>, TagId<T>), T::MaxTags>,
				>
			>;
	
	
			// Matches a block number to all ranking list's that need to be refreshed.
			// After the set block, the entries are wiped to conserve
			#[pallet::storage]
			#[pallet::getter(fn deadlines)]
			pub type ListDeadlines<T:Config> = StorageMap<
				_, 
				Blake2_128Concat, BlockNumberFor<T>, 
				Deadlines<BoundedVec<RankingListId, T::MaxListsPerBlock>>,
			>;
	
	
	
	
		//** Events **//
	
			#[pallet::event]
			#[pallet::generate_deposit(pub(super) fn deposit_event)]
			pub enum Event<T: Config> {
				RankingListCreated(RankingListId),
				MovieAddedToList(RankingListId, String, T::AccountId),
				VotedInList(T::AccountId, RankingListId),	
				RankingTokensClaimed(T::AccountId, BalanceOf<T>),	
				RankingListPayoff(RankingListId),	
			}
	
	
	
		//** Errors **//
	
			#[pallet::error]
			pub enum Error<T> {
				Overflow,
				Underflow,
				BadMetadata,
				
				NotEnoughBalance,
				MovieIdOverflow,
				MovieAlreadyInList,
				InvalidVote,
				RankingListNotFound,
				MovieNotInRankingList,
				VoteAmountCannotBeZero,
				VoteAmountTooLowForNoConviction,
				VoteValueStillLockedWithConviction,
				ListDurationTooShort,
				WalletStatsRegistryRequired,
				NoVoteInList,
				UnstakeValueTooHigh,
				NoClaimableTokens,
			}
	
	
	
		//** Hooks **//
	
			#[pallet::hooks]
			impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
				// fn on_initialize(_now: BlockNumberFor<T>) -> Weight {
				//     // 0
				// }
	
				fn on_finalize(now: BlockNumberFor<T>) {
					Self::do_resolve_lists_deadline(now);
				}
			}
	
	
	
		//** Extrinsics **//
	
			#[pallet::call]
			impl<T: Config> Pallet<T> {
				
				#[pallet::call_index(0)]#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
				pub fn create_ranking_list(
					origin: OriginFor<T>,
					name: String,
					description: String,
					list_duration: BlockNumberFor<T>,
					category_tag_list: BoundedVec<(CategoryId<T>, TagId<T>), T::MaxTags>,
				) -> DispatchResultWithPostInfo {
	
					let who = ensure_signed(origin)?;
					// ensure!(
					// 	kine_stat_tracker::Pallet::<T>::is_wallet_registered(who)?,
					// 	Error::<T>::WalletStatsRegistryRequired,
					// );
	
					// validate category and tag data
					let category_type: kine_tags::CategoryType<T>
						= TryInto::try_into("Ranking List".as_bytes().to_vec()).map_err(|_|Error::<T>::BadMetadata)?;
					
					kine_tags::Pallet::<T>::do_validate_tag_data(
						category_type.clone(), 
						category_tag_list.clone()
					)?;
	
					// create ranking list data
					let ranking_list_id =
						NextRankingListId::<T>::try_mutate(|id| -> Result<RankingListId, DispatchError> {
							let current_id = *id;
							*id = id
								.checked_add(One::one())
								.ok_or(Error::<T>::Overflow)?;
							Ok(current_id)
						}).unwrap();
					
					let bounded_name: BoundedVec<u8, T::RankingStringLimit> =
						TryInto::try_into(name.as_bytes().to_vec()).map_err(|_| Error::<T>::BadMetadata)?;
					let bounded_description: BoundedVec<u8, T::RankingStringLimit> =
						TryInto::try_into(description.as_bytes().to_vec()).map_err(|_| Error::<T>::BadMetadata)?;
					let movies_in_list: BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInList> =
						TryInto::try_into(Vec::new()).map_err(|_| Error::<T>::BadMetadata)?;
	
					//TODO-3
					ensure!(list_duration >= T::MinimumListDuration::get().into(), Error::<T>::ListDurationTooShort);
					let current_block = <frame_system::Pallet<T>>::block_number();
					let list_deadline_block = current_block.checked_add(&list_duration).ok_or(Error::<T>::Overflow)?;
	
					//setup the deadline
					Self::create_list_deadline(ranking_list_id, list_deadline_block)?;
	
					// create the vote's structure, pairing AccountIds to the Vote info.
					let votes_by_user: BoundedBTreeMap<
						T::AccountId,
						BoundedVec<RankingVote<BoundedVec<u8, T::LinkStringLimit>, BalanceOf<T>, BlockNumberFor<T>>, T::MaxVotersPerList>,
						T::MaxVotersPerList> 
					= BoundedBTreeMap::new();
					let total_lockup = BalanceOf::<T>::from(0u32);
	
					// create ranking list struct & insert into storage
					let ranking_list = RankingList {
						name: bounded_name,
						description: bounded_description,
						status:RankingListStatus::Ongoing,
						list_deadline: list_deadline_block,
						list_duration: list_duration,
						movies_in_list: movies_in_list,
						votes_by_user: votes_by_user,
						total_lockup: total_lockup,
						categories_and_tags: category_tag_list.clone(),
					};
					RankingLists::<T>::insert(ranking_list_id.clone(), ranking_list);
	
					// parse the u32 type into a BoundedVec<u8, T::ContentStringLimit
					let encoded: Vec<u8> = ranking_list_id.encode();
					let bounded_content_id: BoundedVec<u8, T::ContentStringLimit> = 
						TryInto::try_into(encoded).map_err(|_|Error::<T>::BadMetadata)?;
	
					kine_tags::Pallet::<T>::do_update_tag_data(
						category_type, 
						category_tag_list,
						bounded_content_id,
					)?;
	 
	
					// finalize call
					Self::deposit_event(Event::RankingListCreated(ranking_list_id));
					Ok(().into())
				}
	
	
				#[pallet::call_index(1)]#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
				pub fn add_internal_movie_to_ranking_list(
					origin: OriginFor<T>,
					list_id: RankingListId,
					movie_id_str: String,
					amount: BalanceOf<T>,
					conviction: Conviction,
				) -> DispatchResultWithPostInfo {
	
					let who = ensure_signed(origin)?;
					ensure!(amount > BalanceOf::<T>::from(0u32), Error::<T>::VoteAmountCannotBeZero);
					if conviction == Conviction::None {
						ensure!(amount >= BalanceOf::<T>::from(10u32), Error::<T>::VoteAmountTooLowForNoConviction);
					}
					
					let movie_id: BoundedVec<u8, T::LinkStringLimit> =
						TryInto::try_into(movie_id_str.as_bytes().to_vec()).map_err(|_| Error::<T>::BadMetadata)?;

					// ensure movie exists
					kine_movie::Pallet::<T>::do_ensure_internal_movie_exist(movie_id.clone())?;
				
					// insert the movie in the ranking list's movies_list
					RankingLists::<T>::try_mutate_exists(list_id, |ranking_list| -> DispatchResult {
						let list = ranking_list.as_mut().ok_or(Error::<T>::BadMetadata)?;
	
						// ensure no entry for the movie exists in the ranking list
						ensure!(!list.movies_in_list.contains(&movie_id.clone()), Error::<T>::MovieAlreadyInList);
	
						// transfer amount to this pallet's vault
						T::Currency::transfer(
							&who.clone(),
							&Self::account_id(), 
							amount.clone(), 
							AllowDeath
						);
						kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
							who.clone(), 
							kine_stat_tracker::FeatureType::RankingList,
							kine_stat_tracker::TokenType::Locked,
							amount.clone(), false
						).unwrap();
						list.total_lockup =
							list.total_lockup
							.checked_add(&amount.clone())
							.ok_or(Error::<T>::Overflow)?;
	
						// create the Vote
						let unlock_block = Self::do_calculate_unlock_block(conviction).unwrap();
						let vote = RankingVote {
							movie_id: movie_id.clone(),
							locked_amount: amount,
							conviction: conviction.clone(),
							unlock_block: unlock_block,
						};
						
						// create a new vote list, with the user's vote in it and add it
						if !list.votes_by_user.contains_key(&who.clone()) {
							let mut user_votes: BoundedVec<RankingVote<BoundedVec<u8, T::LinkStringLimit>, BalanceOf<T>, BlockNumberFor<T>>, T::MaxVotersPerList> =
								TryInto::try_into(Vec::new()).map_err(|_| Error::<T>::BadMetadata)?;
							user_votes.try_push(vote).unwrap();
							list.votes_by_user.try_insert(who.clone(), user_votes).unwrap();
						}
						// the voter has voted in the list, add a new vote
						else {
							let mut votes = list.votes_by_user.get_mut(&who.clone()).unwrap();
							votes.try_push(vote).unwrap();
						}

						list.movies_in_list.try_push(movie_id.clone()).unwrap();

						Ok(().into())
					})?;
					
					// finalize call
					Self::deposit_event(Event::MovieAddedToList(list_id, movie_id_str, who.clone()));
					Ok(().into())
				}
	
	
				#[pallet::call_index(2)]#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
				pub fn add_external_movie_to_ranking_list(
					origin: OriginFor<T>,
					list_id: RankingListId,
					source: kine_movie::ExternalSource,
					movie_link_str: String,
					category_tag_list: BoundedVec<(CategoryId<T>, TagId<T>), T::MaxTags>,
					amount: BalanceOf<T>,
					conviction: Conviction,
				) -> DispatchResultWithPostInfo {
	
					let who = ensure_signed(origin)?;
					ensure!(amount > BalanceOf::<T>::from(0u32), Error::<T>::VoteAmountCannotBeZero);
					if conviction == Conviction::None {
						ensure!(amount >= BalanceOf::<T>::from(10u32), Error::<T>::VoteAmountTooLowForNoConviction);
					}
					
					let movie_link: BoundedVec<u8, T::LinkStringLimit> =
						TryInto::try_into(movie_link_str.as_bytes().to_vec()).map_err(|_| Error::<T>::BadMetadata)?;

					// ensure movie exists
					let does_movie_exist = kine_movie::Pallet::<T>::do_does_external_movie_exist(movie_link.clone())?;
					if !does_movie_exist {
						kine_movie::Pallet::<T>::do_create_external_movie(
							&who.clone(),
							source,
							movie_link_str.clone(),
							category_tag_list.clone()
						)?;
					}
				
					// insert the movie in the ranking list's movies_list
					RankingLists::<T>::try_mutate_exists(list_id, |ranking_list| -> DispatchResult {
						let list = ranking_list.as_mut().ok_or(Error::<T>::BadMetadata)?;
	
						// ensure no entry for the movie exists in the ranking list
						ensure!(!list.movies_in_list.contains(&movie_link.clone()), Error::<T>::MovieAlreadyInList);
	
						// transfer amount to this pallet's vault
						T::Currency::transfer(
							&who.clone(),
							&Self::account_id(), 
							amount.clone(), 
							AllowDeath
						);
						kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
							who.clone(), 
							kine_stat_tracker::FeatureType::RankingList,
							kine_stat_tracker::TokenType::Locked,
							amount.clone(), false
						).unwrap();
						list.total_lockup =
							list.total_lockup
							.checked_add(&amount.clone())
							.ok_or(Error::<T>::Overflow)?;
	
							
						// create the Vote
						let unlock_block = Self::do_calculate_unlock_block(conviction).unwrap();
						let vote = RankingVote {
							movie_id: movie_link.clone(),
							locked_amount: amount,
							conviction: conviction,
							unlock_block: unlock_block,
						};
		

						// create a new vote list, with the user's vote in it and add it
						if !list.votes_by_user.contains_key(&who.clone()) {
							let mut user_votes: BoundedVec<RankingVote<BoundedVec<u8, T::LinkStringLimit>, BalanceOf<T>, BlockNumberFor<T>>, T::MaxVotersPerList> =
								TryInto::try_into(Vec::new()).map_err(|_| Error::<T>::BadMetadata)?;
							user_votes.try_push(vote).unwrap();
							list.votes_by_user.try_insert(who.clone(), user_votes).unwrap();
						}
						// the voter has voted in the list, add a new vote
						else {
							let mut votes = list.votes_by_user.get_mut(&who.clone()).unwrap();
							votes.try_push(vote).unwrap();
						}

						list.movies_in_list.try_push(movie_link.clone()).unwrap();
					
						Ok(().into())
					})?;
					
					// finalize call
					Self::deposit_event(Event::MovieAddedToList(list_id, movie_link_str, who.clone()));
					Ok(().into())
				}
	
	
	
				#[pallet::call_index(3)]#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
				pub fn vote_for(
					origin: OriginFor<T>,
					list_id: RankingListId,
					movie_id_str: String,
					amount: BalanceOf<T>,
					conviction: Conviction,
					) -> DispatchResultWithPostInfo {
	
					let who = ensure_signed(origin)?;
	
					// ensure ranking list id exists
					ensure!(RankingLists::<T>::contains_key(list_id.clone()), Error::<T>::RankingListNotFound);
					ensure!(amount > BalanceOf::<T>::from(0u32), Error::<T>::VoteAmountCannotBeZero);
					if conviction == Conviction::None {
						ensure!(amount >= BalanceOf::<T>::from(10u32), Error::<T>::VoteAmountTooLowForNoConviction);
					}
					
					//mutate the storage, while creating the Vote & bonding
					RankingLists::<T>::try_mutate_exists(list_id, |ranking_list| -> DispatchResult {
						let list = ranking_list.as_mut().ok_or(Error::<T>::BadMetadata)?;
	
						let movie_id: BoundedVec<u8, T::LinkStringLimit> =
							TryInto::try_into(movie_id_str.as_bytes().to_vec()).map_err(|_| Error::<T>::BadMetadata)?;
						
						// ensure ranking list contains movie
						ensure!(list.movies_in_list.contains(&movie_id), Error::<T>::MovieNotInRankingList);
	
						// transfer amount to this pallet's vault
						T::Currency::transfer(
							&who.clone(),
							&Self::account_id(), 
							amount.clone(), 
							AllowDeath
						);
						kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
							who.clone(), 
							kine_stat_tracker::FeatureType::RankingList,
							kine_stat_tracker::TokenType::Locked,
							amount.clone(), false
						).unwrap();
						list.total_lockup =
							list.total_lockup
							.checked_add(&amount.clone())
							.ok_or(Error::<T>::Overflow)?;
	
						// create the Vote
						let unlock_block = Self::do_calculate_unlock_block(conviction).unwrap();
						let vote = RankingVote {
							movie_id: movie_id.clone(),
							locked_amount: amount,
							conviction: conviction,
							unlock_block: unlock_block,
						};
	
						// retrieve the votes for the ranking list
						
						
						// create a new vote list, with the user's vote in it and add it
						if !list.votes_by_user.contains_key(&who.clone()) {
							let mut user_votes: BoundedVec<RankingVote<BoundedVec<u8, T::LinkStringLimit>, BalanceOf<T>, BlockNumberFor<T>>, T::MaxVotersPerList> =
								TryInto::try_into(Vec::new()).map_err(|_| Error::<T>::BadMetadata)?;
							user_votes.try_push(vote).unwrap();
							list.votes_by_user.try_insert(who.clone(), user_votes).unwrap();
						}
						// the voter has voted in the list, add a new vote
						else {
							let mut votes = list.votes_by_user.get_mut(&who.clone()).unwrap();
							votes.try_push(vote).unwrap();
						}
						
						Ok(().into())
					})?;
					
					Self::deposit_event(Event::VotedInList(who, list_id));
					Ok(().into())
				}
	


				#[pallet::call_index(4)]#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
				pub fn unvote_from(
					origin: OriginFor<T>,
					list_id: RankingListId,
					movie_id_str: String,
					amount: BalanceOf<T>,
					) -> DispatchResultWithPostInfo {
	
					let who = ensure_signed(origin)?;
	
					// ensure ranking list id exists
					ensure!(RankingLists::<T>::contains_key(list_id.clone()), Error::<T>::RankingListNotFound);

					
					//mutate the storage, while creating the Vote & bonding
					RankingLists::<T>::try_mutate_exists(list_id, |ranking_list| -> DispatchResult {
						let list = ranking_list.as_mut().ok_or(Error::<T>::BadMetadata)?;
						let movie_id: BoundedVec<u8, T::LinkStringLimit> =
							TryInto::try_into(movie_id_str.as_bytes().to_vec()).map_err(|_| Error::<T>::BadMetadata)?;
						
						// ensure ranking list contains movie
						ensure!(list.movies_in_list.contains(&movie_id), Error::<T>::MovieNotInRankingList);
						
						// ensure user has voted
						let mut votes = list.votes_by_user.get_mut(&who.clone());
						ensure!(votes != None, Error::<T>::NoVoteInList);

						// unwrap vote list and find the vote for this movie
						let mut vote_to_remove : RankingVote<BoundedVec<u8, T::LinkStringLimit>, BalanceOf<T>, BlockNumberFor<T>>; // used if the vote amount reaches 0
						let unwrapped_votes = votes.unwrap();
						for vote in unwrapped_votes {
							if vote.movie_id == movie_id {

								// ensure enough balance to unstake
								ensure!(amount <= vote.locked_amount, Error::<T>::UnstakeValueTooHigh);

								// ensure enough blocks have passed, due to conviction
								ensure!(
									<frame_system::Pallet<T>>::block_number() >= vote.unlock_block, 
									Error::<T>::VoteValueStillLockedWithConviction
								);
								
								// transfer amount to the wallet from this pallet's vault
								T::Currency::transfer(
									&Self::account_id(), 
									&who.clone(),
									amount.clone(), 
									AllowDeath
								);
								kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
									who.clone(), 
									kine_stat_tracker::FeatureType::RankingList,
									kine_stat_tracker::TokenType::Locked,
									amount.clone(), false
								).unwrap();
		
								// update locked values
								list.total_lockup =
									list.total_lockup
									.checked_sub(&amount.clone())
									.ok_or(Error::<T>::Overflow)?;
								vote.locked_amount =
									vote.locked_amount
									.checked_sub(&amount.clone())
									.ok_or(Error::<T>::Overflow)?;

								if vote.locked_amount == BalanceOf::<T>::from(0u32) {
									vote_to_remove = vote.clone();
								}
							}
						}


						
						Ok(().into())
					})?;
					
					Self::deposit_event(Event::VotedInList(who, list_id));
					Ok(().into())
				}
	



	
				#[pallet::call_index(5)]#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
				pub fn claim_ranking_rewards(
					origin: OriginFor<T>,
				) -> DispatchResultWithPostInfo {
					
					let who = ensure_signed(origin)?;
					
					let claimable_tokens_ranking = 
						kine_stat_tracker::Pallet::<T>
						::get_wallet_tokens(who.clone()).unwrap()
						.claimable_tokens_ranking;
					
					ensure!(
						claimable_tokens_ranking > BalanceOf::<T>::from(0u32), 
						Error::<T>::NoClaimableTokens
					);

					T::Currency::deposit_into_existing(
						&who.clone(),
						claimable_tokens_ranking.clone(), 
					);
					
					kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
						who.clone(), 
						kine_stat_tracker::FeatureType::RankingList,
						kine_stat_tracker::TokenType::Claimable,
						claimable_tokens_ranking.clone(), true
					)?;
				
					Self::deposit_event(Event::RankingTokensClaimed(who, claimable_tokens_ranking));
					Ok(().into())
				}	
	
	
	
	
	
			}
	
		
	
		//** Helpers **//
	
			impl<T: Config> Pallet<T> {
	
				//* Utils *//
	
					// This pallet's vault account ID.
					fn account_id()->T::AccountId{
						<T as Config>::PalletId::get().try_into_account().unwrap()
					}
	
	
					// Creates a deadline entry for a ranking list in ListDeadlines.
					// If no entries exist for the block, a new entry is created and
					// the ranking list's id is added.
					pub fn create_list_deadline(
						ranking_list_id: RankingListId,
						list_deadline_block: BlockNumberFor<T>,
					)
					-> DispatchResultWithPostInfo {
						
						// create deadlines for the ranking lists end, if none exist in that block
						if !ListDeadlines::<T>::contains_key(list_deadline_block) == true {
							let mut bounded_list_deadlines: BoundedVec<RankingListId, T::MaxListsPerBlock> =
								TryInto::try_into(Vec::new()).map_err(|_| Error::<T>::BadMetadata)?;
							bounded_list_deadlines.try_push(ranking_list_id.clone()).unwrap();
							let deadlines = Deadlines {
								list_deadlines: bounded_list_deadlines,
							};
							ListDeadlines::<T>::insert(list_deadline_block, deadlines);
						}
						// fetch existing deadlines for the ranking lists end and add it to the list
						else {
							ListDeadlines::<T>::try_mutate(list_deadline_block, |deadlines_list| -> DispatchResult {
								let deadlines = deadlines_list.as_mut().ok_or(Error::<T>::BadMetadata)?;
								deadlines.list_deadlines.try_push(ranking_list_id.clone()).unwrap();
								Ok(())
							})?;
						}
	
						Ok(().into())
					}
	
	
					// Concludes the ranking list and determines the winners. This is triggered by hooks.
					// This takes into account the total voting power (tokens * conviction) of each movie.
					// Users who vote in the top ranked movies will receive rewards based on their total tokens locked.
					pub fn do_resolve_lists_deadline(
						block_deadline: BlockNumberFor<T>
					) -> DispatchResultWithPostInfo {
					
						// check if any entries exist for the block
						ListDeadlines::<T>::try_mutate_exists(block_deadline, |deadlines_list| -> DispatchResult {
							let deadlines = deadlines_list.as_mut().ok_or(Error::<T>::BadMetadata)?;
							
							// check if any entries exist and if so refresh them
							for list_id in deadlines.list_deadlines.iter() {
								let sorted_ranking_list = Self::resolve_ranking_list(list_id.clone())?;
	
								// update the ranking list's sorted movies & determine the new deadline
								RankingLists::<T>::try_mutate(list_id, |ranking_list| -> DispatchResult {
									let list = ranking_list.as_mut().ok_or(Error::<T>::BadMetadata)?;
									
									list.movies_in_list = sorted_ranking_list;
									list.list_deadline =
										<frame_system::Pallet<T>>::block_number().checked_add(&list.list_duration).ok_or(Error::<T>::Overflow)?;
									Self::create_list_deadline(list_id.clone(), list.list_deadline).unwrap();
									Ok(().into())
								})?;
							}
							
							Ok(())
						})?;
	
						//remove the entries from storage to free up space
						ListDeadlines::<T>::remove(block_deadline);
	
						Ok(().into())
					}
	
	
	
					// Resolves a single Ranking List. 
					// This means determining the winner(s) and distributing the rewards accordingly.
					pub fn resolve_ranking_list(
						list_id: RankingListId
					) -> Result<BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInList>, DispatchError> {
					
						// get the ranking list
						let ranking_list = RankingLists::<T>::try_get(list_id.clone()).unwrap();
	
						// create a Btree that pairs movie ids to their total voting power
						let mut movies_by_power: BoundedBTreeMap<
							BoundedVec<u8, T::LinkStringLimit>,
							BalanceOf::<T>,
							T::MaxVotersPerList
						> = BoundedBTreeMap::new();
						for movie_id in ranking_list.movies_in_list {
							movies_by_power.try_insert(movie_id.clone(), BalanceOf::<T>::from(0u32)).unwrap();
						}
	
						// iterate each user's votes for the ranking list and calculate the return
						let blocks_in_year: u32 = 5256000; //TODO-4
						for (account_id, vote_list) in ranking_list.votes_by_user.iter() {
							
							// get each vote's total power, adding it to movies_by_power and tallying the total tokens
							let mut total_return = BalanceOf::<T>::from(0u32);
							for vote in vote_list {
								// increase total voting power for the user's choice
								let voting_power = Self::do_calculate_voting_power(vote.locked_amount, vote.conviction)?;
								movies_by_power.get_mut(&vote.movie_id).unwrap().checked_add(&voting_power).ok_or(Error::<T>::Overflow)?;
								// tally the tokens
								total_return = total_return.checked_add(&vote.locked_amount).ok_or(Error::<T>::Overflow)?;
							}
	
							// (total_stake / Blocks_Per_Year) * (APY / 100) = (total_stake * APY) / (Blocks_Per_Year * 100)
							let new_earning = 
								total_return
								.saturating_mul(18u32.into());
								
							let total_earning_needed =
								blocks_in_year
								.saturating_mul(100u32.into());
	
							// return the 18% APY
							kine_stat_tracker::Pallet::<T>::do_update_wallet_imbalance(
								account_id.clone(), 
								kine_stat_tracker::FeatureType::RankingList,
								new_earning, BalanceOf::<T>::from(total_earning_needed),
								false,
							).unwrap();
	
							// kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
							// 	account_id.clone(), 
							// 	kine_stat_tracker::FeatureType::RankingList,
							// 	kine_stat_tracker::TokenType::Claimable,
							// 	BalanceOf::<T>::from(total_earning_needed), false
							// )?;
						}
	
						// swap the id for the weight so the elements can be conveniently sorted
						let mut movies_aux: Vec<(BalanceOf::<T>, BoundedVec<u8, T::LinkStringLimit>)> =
							movies_by_power.into_iter()
							.map(|(movie_id, voting_power)| (voting_power, movie_id))
							.collect();
	
						// sort the list to get the rankings in order
						movies_aux.sort_by(|a, b| b.cmp(a));
						let ordered_movies_power: Vec<BoundedVec<u8, T::LinkStringLimit>> = movies_aux.into_iter().map(|(_, movie_id)| movie_id).collect(); 
						let ordered_movies: BoundedVec<BoundedVec<u8, T::LinkStringLimit>, T::MaxMoviesInList>
							= TryInto::try_into(ordered_movies_power).map_err(|_|Error::<T>::BadMetadata)?;
						
						Self::deposit_event(Event::RankingListPayoff(list_id));
						Ok(ordered_movies)
					}
	
	
					// Takes the total tokens locked in a vote and multiplies their value
					// based on the chosen conviction multiplier. In other words, this calculates
					// a vote's voting power.
					pub fn do_calculate_voting_power(
						vote: BalanceOf::<T>,
						conviction: Conviction,
					)
					-> Result<BalanceOf::<T>, DispatchError> {
						
						// match a conviction to its respective power multiplier and return the result
						match conviction {
							Conviction::None => return Ok(vote.checked_div(&BalanceOf::<T>::from(10u32)).ok_or(Error::<T>::Overflow)?),
	
							Conviction::Locked1x => return Ok(vote),
	
							Conviction::Locked2x => return Ok(vote.saturating_mul(2u32.into())),
	
							Conviction::Locked3x => return Ok(vote.saturating_mul(3u32.into())),
	
							Conviction::Locked4x => return Ok(vote.saturating_mul(4u32.into())),
	
							Conviction::Locked5x => return Ok(vote.saturating_mul(5u32.into())),
	
							Conviction::Locked6x => return Ok(vote.saturating_mul(6u32.into())),
						};
					}
	
	
					// Takes the chosen conviction multiplier and calculates the unlock_block
					// for the vote. The higher the conviction, the longer the period.
					pub fn do_calculate_unlock_block(
						conviction: Conviction,
					)
					-> Result<BlockNumberFor<T>, DispatchError> {
						
						let mut unlock_block = <frame_system::Pallet<T>>::block_number();

						let blocked_period: u32 = match conviction {
							Conviction::None => 0u32,
	
							Conviction::Locked1x => T::MinimumListDuration::get(),
	
							Conviction::Locked2x => T::MinimumListDuration::get().saturating_mul(2u32),
	
							Conviction::Locked3x => T::MinimumListDuration::get().saturating_mul(3u32),
	
							Conviction::Locked4x => T::MinimumListDuration::get().saturating_mul(4u32),
	
							Conviction::Locked5x => T::MinimumListDuration::get().saturating_mul(5u32),
	
							Conviction::Locked6x => T::MinimumListDuration::get().saturating_mul(6u32),
						};

						unlock_block = 
							unlock_block
							.checked_add(&blocked_period.into())
							.ok_or(Error::<T>::Overflow)?;

						Ok(unlock_block)
					}
	
	
	
					
					// // Lets now calculate how this is split to the nominators.
					// // Reward only the clipped exposures. Note this is not necessarily sorted.
					// for nominator in exposure.others.iter() {
					// 	let nominator_exposure_part = Perbill::from_rational(nominator.value, exposure.total);
	
					// 	let nominator_reward: BalanceOf<T> =
					// 		nominator_exposure_part * validator_leftover_payout;
					// 	// We can now make nominator payout:
					// 	if let Some(imbalance) = Self::make_payout(&nominator.who, nominator_reward) {
					// 		// Note: this logic does not count payouts for `RewardDestination::None`.
					// 		nominator_payout_count += 1;
					// 		let e =
					// 			Event::<T>::Rewarded { stash: nominator.who.clone(), amount: imbalance.peek() };
					// 		Self::deposit_event(e);
					// 		total_imbalance.subsume(imbalance);
					// 	}
					// }
	
	
	
	
	
	
	
	
	
	
	
			}
	
	}