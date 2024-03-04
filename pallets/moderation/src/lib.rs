//** About **//
	// This pallet handles the moderation process. It manages all interactions related to assigning moderators, 
	// drafting them for reports, handling voting consensus and reward splitting.
	// A reported content's id is matched to unique report structures that return its specific information,
	// or information regarding each moderation "tier" or court.
	//TODO-0 remove the claimable amount from this pallet (already in stat_tracker)
	//TODO-1 check T::Currency::unreserve(&reportee, collateral);
	//TODO-2 implement MaxReportsByTier dynamically with ModeratorLimitByTier, through a switch case
	//TODO-3 make a new storage for unallocated_moderators, or find another solution
	//TODO-4 update veredicts after time limit if no appeal or vote is made
	//TODO-5 call stat tracker and check if enough reputation when apllying for moderator
	//TODO-6 automatically retrieve the reportee_id when creating a report
	//TODO-7 check if report is already ongoing for that content when creating a new report
	//TODO-8 suspend content after a successful report
	//TODO-9 optimize moderator drafting
	//TODO-10 check if the justification is not empty when creating a report
	//TODO-11 optimize do_get_current_report_tier_data
	//TODO-12 check iter_key_prefix
	//TODO-13 add ok_or to do_create_vote
	//TODO-14 use arithmetic in do_calculate_vote_consensus
	//TODO-15 use drain_filter (currently unstable) instead of retain
	//TODO-16 to report content you need at least the minimum reputation to become a moderator
	//TODO-17 add a direct reference to the content being reported in a report (ie. a movie_id of an innapropriate movie title in a festival)




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
	
	//** Config **//

		//* Imports *//
			
			use frame_support::{
				dispatch::DispatchResultWithPostInfo,
				pallet_prelude::*,
				traits::{
					Currency,
					ReservableCurrency,
					ExistenceRequirement::{
						AllowDeath, 
					},
				},
				BoundedVec,
				PalletId,
			};
			use frame_system::pallet_prelude::*;
			use codec::{Decode, Encode, MaxEncodedLen};
			use sp_runtime::{
				RuntimeDebug, 
				traits::{
					AtLeast32BitUnsigned, 
					CheckedAdd, 
					CheckedSub, 
					One,
					AccountIdConversion,
					CheckedDiv, 
					Saturating, 
				},
			};
			use scale_info::{
				TypeInfo,
				prelude::vec::Vec
			};
			use core::convert::TryInto;
			use sp_std::{
				collections::btree_map::BTreeMap,
				vec,
			};

			use kine_tags::{
				CategoryId as CategoryId,
				TagId as TagId,
			};


		//* Config *//

			#[pallet::pallet]
			pub struct Pallet<T>(_);

			#[pallet::config]
			pub trait Config: frame_system::Config + kine_stat_tracker::Config + kine_tags::Config {
				type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
			
				#[pallet::constant]
				type JustificationLimit: Get<u32>;

				type ContentId: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
				type MaxReportsByModerator: Get<u32>;
				type TotalTierOneModerators: Get<u32>;
				type MaxReportsByTier: Get<u32>;
			
				type MinimumReputationForModeration: Get<u32>; 
				type MinimumReputationForSeniorship: Get<u32>; 
				type MinimumTokensForModeration: Get<BalanceOf<Self>>; 
				type MovieCollateral: Get<BalanceOf<Self>>; 

				// type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
				type PalletId: Get<PalletId>;
			}

	//** Types **//	
	
		//* Types *//

			type BalanceOf<T> = <<T as kine_stat_tracker::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
			
		//* Constants *//
		//* Enums *//

			#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
			pub enum InfringimentType {
				Violence,
				Discrimination,
				LackOfConsent,
				Impersonation,
				Terrorism,
				Copyright,
				FakeNews,
				Pornography,
				Extreme,
				Naming,
				Categorization,
			}

			#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
			pub enum ContentType {
				Festival,
				Movie,
				Tag,
				RankingList,
				SocialSpace,
			}

			#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
			pub enum ReportStatus {
				InResolution,
				MajorityVotedFor,
				MajorityVotedAgainst,
				AppealedByReporter,
				AppealedByReportee,
				Accepted,
				Refused,
			}

			#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
			pub enum Tiers {
				TierOne,
				TierTwo,
				TierThree,
			}

			#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
			pub enum VoteChoice {
				For,
				Against,
				Abstinence,
			}

			#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
			pub enum ModeratorRank {
				Junior,
				Senior,
			}

		//* Structs *//

			#[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
			pub struct Report <AccountId, ReportStatus, BoundedString, CategoryTagList> {
				pub	reporter_id: AccountId,
				pub reportee_id: AccountId,
				pub justification: BoundedString,
				pub status: ReportStatus,
				pub categories_and_tags: CategoryTagList,
			}

			#[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
			pub struct ReportOutcome<VoteList, BalanceOf> {
				pub staked_tokens: BalanceOf,
				pub required_votes: u32,
				pub votes_for: u32,
				pub votes: VoteList
			}

			#[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
			pub struct Moderator<ReportList, ModeratorRank> {
				pub assigned_reports: ReportList,
				pub rank: ModeratorRank,
			}

			#[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
			pub struct Vote <AccountId> {
				pub voter: AccountId,
				pub is_for: VoteChoice,
			}



	//** Storage **//

		//* Reports *//

			// matches a report id to the report data
			#[pallet::storage]
			#[pallet::getter(fn report)]
			pub type Reports<T: Config> =
				StorageMap<
					_, 
					Blake2_128Concat, (T::ContentId, ContentType), 
					Report<
						T::AccountId, 
						ReportStatus, 
						BoundedVec<u8, T::JustificationLimit>,
						BoundedVec<(CategoryId<T>, TagId<T>), T::MaxTags>,
					>,
				>;

			// matches a report id to its current status
			#[pallet::storage]
			#[pallet::getter(fn report_tier_verdict)]
			pub type ReportVerdicts<T: Config> =
				StorageDoubleMap<
					_,
					Blake2_128Concat, (T::ContentId, ContentType), 
					Blake2_128Concat, Tiers,
					ReportOutcome<BoundedVec<Vote<T::AccountId>, T::MaxReportsByTier>, BalanceOf<T>>,
					OptionQuery,
				>;

		
		//* Moderators *//

			// Match a moderator's id to its moderator information.
			// Also keeps track of the total number of moderators,
			// to ensure there are enough moderators.
			#[pallet::storage]
			pub type Moderators<T: Config> =
				CountedStorageMap<
					_, 
					Blake2_128Concat, T::AccountId, 
					Moderator<
						BoundedVec<T::ContentId, T::MaxReportsByModerator>, 
						ModeratorRank,
					>,
					OptionQuery,
				>;

	
	
	//** Events **//

		#[pallet::event]
		#[pallet::generate_deposit(pub(super) fn deposit_event)]
		pub enum Event<T: Config> {
			ModeratorRegistered(T::AccountId),
			ModerationActivitySuspended(T::AccountId),
			ModerationRewardsClaimed(T::AccountId),

			VoteSubmitted(),
			
			ReportCreated(T::ContentId),
			ReportClosed(T::ContentId, ReportStatus),
			ReportAppealed(T::ContentId),
			ReportAppealAccepted(T::ContentId),
			ReportAppealRefused(T::ContentId),
			FestivalReported,
			MovieReported,
			CategoryReported,
			CommentReported,

			JuryDraftedTierOne,
			ModerationStartedTierOne,
			TierOneFinalized,

			EscalateToTierTwo,
			JuryDraftedTierTwo,
			ModerationStartedTierTwo,
			TierTwoFinalized,

			EscalateToTierThree,
			JuryDraftedTierThree,
			ModerationStartedTierThree,
			TierThreeFinalized,
		}
	


	//** Errors **//

		#[pallet::error]
		pub enum Error<T> {
			Overflow,
			VoteOverflow,
			Underflow,
			BadMetadata,
			
			AlreadyRegisteredAsModerator,
			NonexistentModerator,
			InvalidModeratorData,
			ModeratorNotDraftedForReport,
			NotEnoughModeratorsAvailable,
			
			UserCannotAcceptVerdict,
			
			NonexistentReport,
			ReportsAwaitingVote,
			ReportAlreadyOngoing,
			ReportAppealLimitReached,

			InsuficientBalance,
			WalletStatsRegistryRequired,
			NotEnoughBalance,
			NotEnoughReputation,
		}



	//** Hooks **//

		#[pallet::hooks]
		impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
			
		}
	

		
	//** Extrinsics **//
		
		#[pallet::call]
		impl<T: Config> Pallet<T> {
			

			#[pallet::call_index(0)]#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
			pub fn apply_for_moderation(
				origin: OriginFor<T>,
			) -> DispatchResultWithPostInfo {
				
				let who = ensure_signed(origin)?;
				Self::do_check_if_already_moderator(who.clone())?;
				ensure!(
					kine_stat_tracker::Pallet::<T>::do_is_wallet_registered(who.clone())?,
					Error::<T>::WalletStatsRegistryRequired,
				);
				ensure!(
					kine_stat_tracker::WalletTokens::<T>::try_get(who.clone()).unwrap().reputation_moderation 
					>= T::MinimumReputationForModeration::get(),
					Error::<T>::NotEnoughReputation,
				);
				
				Self::do_transfer_funds_to_treasury(who.clone(), T::MinimumTokensForModeration::get())?;
				kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
					who.clone(), 
					kine_stat_tracker::FeatureType::Moderation,
					kine_stat_tracker::TokenType::Locked,
					T::MinimumTokensForModeration::get(), false
				)?;
				Self::do_create_moderator(who.clone())?;
				
				Self::deposit_event(Event::ModeratorRegistered(who));
				Ok(().into())
			}	

			

			#[pallet::call_index(1)]#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
			pub fn update_reputation_senior(
				origin: OriginFor<T>,
			) -> DispatchResultWithPostInfo {
				
				let who = ensure_signed(origin)?;
				Self::do_check_if_already_moderator(who.clone())?;
				ensure!(
					kine_stat_tracker::Pallet::<T>::do_is_wallet_registered(who.clone())?,
					Error::<T>::WalletStatsRegistryRequired,
				);
				ensure!(
					kine_stat_tracker::WalletTokens::<T>::try_get(who.clone()).unwrap().reputation_moderation 
					>= T::MinimumReputationForModeration::get(),
					Error::<T>::NotEnoughReputation,
				);
				
				Self::do_transfer_funds_to_treasury(who.clone(), T::MinimumTokensForModeration::get())?;
				kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
					who.clone(), 
					kine_stat_tracker::FeatureType::Moderation,
					kine_stat_tracker::TokenType::Locked,
					T::MinimumTokensForModeration::get(), false
				)?;
				Self::do_create_moderator(who.clone())?;
				
				Self::deposit_event(Event::ModeratorRegistered(who));
				Ok(().into())
			}	


			#[pallet::call_index(2)]#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
			pub fn suspend_moderation_activity(
				origin: OriginFor<T>,
			) -> DispatchResultWithPostInfo {
				
				let who = ensure_signed(origin)?;
				ensure!(
					Self::do_can_moderator_suspend(who.clone())?,
					Error::<T>::ReportsAwaitingVote
				);
				
				Self::do_suspend_moderation(who.clone())?;

				Self::deposit_event(Event::ModerationActivitySuspended(who));
				Ok(().into())
			}	


			#[pallet::call_index(3)]#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
			pub fn claim_moderation_rewards(
				origin: OriginFor<T>,
			) -> DispatchResultWithPostInfo {
				
				let who = ensure_signed(origin)?;
				Self::do_claim_moderation_reward(who.clone())?;

				Self::deposit_event(Event::ModerationRewardsClaimed(who));
				Ok(().into())
			}	


			
			#[pallet::call_index(4)]#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
			pub fn create_report(
				origin: OriginFor<T>,
				content_id: T::ContentId,
				content_type: ContentType,
				reportee_id: T::AccountId,
				justification: BoundedVec<u8, T::JustificationLimit>,
				category_tag_list: BoundedVec<(CategoryId<T>, TagId<T>), T::MaxTags>,
			) -> DispatchResultWithPostInfo {
				
				let who = ensure_signed(origin)?;
				ensure!(
					kine_stat_tracker::Pallet::<T>::do_is_wallet_registered(who.clone())?,
					Error::<T>::WalletStatsRegistryRequired,
				);
				ensure!(
					kine_stat_tracker::WalletTokens::<T>::try_get(who.clone()).unwrap().reputation_moderation 
					>= T::MinimumReputationForModeration::get(),
					Error::<T>::NotEnoughReputation,
				);

				// validate category and tag
				let category_type: kine_tags::CategoryType<T>
					= TryInto::try_into("Moderation".as_bytes().to_vec()).map_err(|_|Error::<T>::BadMetadata)?;

				Self::do_validate_report_data (
					content_id, content_type, justification.clone(), 
					category_type.clone(), category_tag_list.clone()
				)?; 

				let reward_pool = Self::do_calculate_report_pool(T::TotalTierOneModerators::get())?;
				Self::do_transfer_funds_to_treasury(who.clone(), reward_pool.0)?;
				kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
					who.clone(), 
					kine_stat_tracker::FeatureType::Moderation,
					kine_stat_tracker::TokenType::Locked,
					reward_pool.0, false,
				)?;

				let required_moderators = Self::do_calculate_moderators_in_tier(Tiers::TierOne)?;
				let drafted_moderators = Self::do_draft_moderators(who.clone(), reportee_id.clone(), required_moderators)?;

				Self::do_create_report(who, content_id, content_type, reportee_id, justification, category_tag_list.clone())?;
				Self::do_create_report_verdict(content_id, content_type, Tiers::TierOne, reward_pool.1)?;
				Self::do_assign_report_to_moderators(content_id, drafted_moderators)?;

				// parse the u32 type into a BoundedVec<u8, T::ContentStringLimit
				let encoded: Vec<u8> = content_id.encode();
				let bounded_content_id: BoundedVec<u8, T::ContentStringLimit> = 
					TryInto::try_into(encoded).map_err(|_|Error::<T>::BadMetadata)?;

				kine_tags::Pallet::<T>::do_update_tag_data(
					category_type, 
					category_tag_list,
					bounded_content_id,
				)?;

				Self::deposit_event(Event::ReportCreated(content_id));
				Ok(().into())
			}


			#[pallet::call_index(5)]#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
			pub fn submit_vote(
				origin: OriginFor<T>,
				content_id: T::ContentId,
				content_type: ContentType,
				vote: VoteChoice,
			) -> DispatchResultWithPostInfo {
				
				let who = ensure_signed(origin)?;
				Self::do_can_moderator_vote(who.clone(), content_id)?;

				let tier_data = Self::do_get_current_report_tier_data(content_id, content_type)?;
				Self::do_create_vote(content_id, content_type, tier_data.0, who.clone(), vote)?;
				Self::do_deallocate_moderator_from_report(who, content_id)?;
				
				if Self::do_are_all_votes_submitted(content_id, content_type, tier_data.0)? == true {
					let consensus = Self::do_calculate_vote_consensus(content_id, content_type, tier_data.0)?;
					Self::do_update_report_status(content_id, content_type, consensus)?;
				}

				Self::deposit_event(Event::VoteSubmitted());
				Ok(().into())
			}


			#[pallet::call_index(6)]#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
			pub fn submit_report_consensus_decision(
				origin: OriginFor<T>,
				content_id: T::ContentId,
				content_type: ContentType,
				decision: bool,
			) -> DispatchResultWithPostInfo {
				
				let who = ensure_signed(origin)?;
				let consensus = Self::do_check_if_report_verdict_is_acceptable(content_id, content_type)?;
				let is_reporter = Self::do_check_if_reporter_or_reportee(who.clone(), content_id, content_type)?;
				Self::do_can_user_accept_verdict(is_reporter, content_id, content_type)?;
				let tier_data = Self::do_get_current_report_tier_data(content_id, content_type)?;
				let report_status : ReportStatus;
				
				if decision == false { // appeal verdict
					ensure!(tier_data.2 == true, Error::<T>::ReportAppealLimitReached); // tier_data.2 == is_appealable
					
					let reward_pool = Self::do_calculate_report_pool(T::TotalTierOneModerators::get())?;
					Self::do_transfer_funds_to_treasury(who.clone(), reward_pool.0)?;
					kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
						who, 
						kine_stat_tracker::FeatureType::Moderation,
						kine_stat_tracker::TokenType::Locked,
						reward_pool.0, false,
					)?;
					report_status = Self::do_get_report_status_on_appeal(is_reporter)?;
					Self::deposit_event(Event::ReportAppealed(content_id));
				}
					
				else { // accept verdict
					let reporter_id = Self::do_get_reporter(content_id, content_type)?;
					let reportee_id = Self::do_get_reportee(content_id, content_type)?;
					report_status = Self::do_get_report_status_on_accept(is_reporter)?;
					
					if !is_reporter {
						let reportee_slash = Self::do_convert_collateral_to_balance()?;
						Self::do_grab_reportee_collateral(content_id, content_type, reportee_slash)?;
					}

					for tier in tier_data.3 { // iterate all existing tiers
						let report_voters = Self::do_get_report_voters_by_vote(content_id, content_type, tier, consensus)?;
						
						let reward_pool = Self::do_get_total_moderation_pool(content_id, content_type, tier)?;
						let majority_voter_reward = Self::do_calculate_majority_voter_reward(reward_pool, report_voters.0.len() as u32)?;
						
						Self::do_distribute_rewards_to_majority_voters(report_voters.0, majority_voter_reward)?;
						Self::do_slash_minority_voters(report_voters.1)?;
						
						if is_reporter {
							let reportee_reward = Self::do_calculate_reportee_reward(reward_pool)?;
							kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
								reportee_id.clone(), 
								kine_stat_tracker::FeatureType::Moderation,
								kine_stat_tracker::TokenType::Claimable,
								reportee_reward, false,
							)?;
							
						}
						else {
							let reporter_reward = Self::do_calculate_reporter_reward(reward_pool)?;
							kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
								reporter_id.clone(), 
								kine_stat_tracker::FeatureType::Moderation,
								kine_stat_tracker::TokenType::Claimable,
								reporter_reward, false,
							)?;
							
						}
					} 
					Self::deposit_event(Event::ReportClosed(content_id, report_status));
				}
				Self::do_update_report_status(content_id, content_type, report_status)?;
				
				Ok(().into())
			}


			#[pallet::call_index(7)]#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
			pub fn submit_report_appeal_decision(
				origin: OriginFor<T>,
				content_id: T::ContentId,
				content_type: ContentType,
				decision: bool,
			) -> DispatchResultWithPostInfo {
			
				let who = ensure_signed(origin)?;
				let consensus = Self::do_check_if_report_appeal_is_acceptable(content_id, content_type)?;
				let is_reporter = Self::do_check_if_reporter_or_reportee(who, content_id, content_type)?;
				Self::do_can_user_accept_appeal(is_reporter, content_id, content_type)?;
				let tier_data = Self::do_get_current_report_tier_data(content_id, content_type)?;

				if decision == false {
					let reporter_id = Self::do_get_reporter(content_id, content_type)?;
					let reportee_id = Self::do_get_reportee(content_id, content_type)?;

					if !is_reporter {
						let reportee_slash = Self::do_convert_collateral_to_balance()?;
						Self::do_grab_reportee_collateral(content_id, content_type, reportee_slash)?;
					}
					
					for tier in tier_data.3 {
						let report_voters = Self::do_get_report_voters_by_vote(content_id, content_type, tier, consensus)?;
						let reward_pool = Self::do_get_total_moderation_pool(content_id, content_type, tier)?;
						let majority_voter_reward = Self::do_calculate_majority_voter_reward(reward_pool, report_voters.0.len() as u32)?;
						
						Self::do_distribute_rewards_to_majority_voters(report_voters.0, majority_voter_reward)?;
						Self::do_slash_minority_voters(report_voters.1)?;
						
						if is_reporter {
							let reportee_reward = Self::do_calculate_reportee_reward(reward_pool)?;
							kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
								reportee_id.clone(), 
								kine_stat_tracker::FeatureType::Moderation,
								kine_stat_tracker::TokenType::Claimable,
								reportee_reward, false,
							)?;
							Self::do_update_report_status(content_id, content_type, ReportStatus::Refused)?;
						}
						else {
							let reporter_reward = Self::do_calculate_reporter_reward(reward_pool)?;
							kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
								reporter_id.clone(), 
								kine_stat_tracker::FeatureType::Moderation,
								kine_stat_tracker::TokenType::Claimable,
								reporter_reward, false,
							)?;
							Self::do_update_report_status(content_id, content_type, ReportStatus::Accepted)?;
						}
					}
					
					let appeal_fee = BalanceOf::<T>::from(Self::do_calculate_report_pool(T::TotalTierOneModerators::get())?.0);
					if is_reporter {
						kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
							reportee_id, 
							kine_stat_tracker::FeatureType::Moderation,
							kine_stat_tracker::TokenType::Claimable,
							appeal_fee, false,
						)?;
					}
					else {
						kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
							reporter_id, 
							kine_stat_tracker::FeatureType::Moderation,
							kine_stat_tracker::TokenType::Claimable,
							appeal_fee, false,
						)?;
					}
					Self::deposit_event(Event::ReportAppealRefused(content_id));

				}

				else {
					let appeal_fee = Self::do_calculate_report_pool(T::TotalTierOneModerators::get())?.0;
					let reporter_id = Self::do_get_reporter(content_id, content_type)?;
					let reportee_id = Self::do_get_reportee(content_id, content_type)?;
					if is_reporter {
						Self::do_transfer_funds_to_treasury(reporter_id.clone(), appeal_fee)?;
						kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
							reporter_id.clone(), 
							kine_stat_tracker::FeatureType::Moderation,
							kine_stat_tracker::TokenType::Locked,
							appeal_fee, false,
						)?;
					}
					else {
						Self::do_transfer_funds_to_treasury(reportee_id.clone(), appeal_fee)?;
						kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
							reportee_id.clone(), 
							kine_stat_tracker::FeatureType::Moderation,
							kine_stat_tracker::TokenType::Locked,
							appeal_fee, false,
						)?;
					}

					let required_moderators = Self::do_calculate_moderators_in_tier(tier_data.1)?;
					let drafted_moderators = Self::do_draft_moderators(reporter_id, reportee_id, required_moderators)?;
					let reward_pool = Self::do_calculate_report_pool(T::TotalTierOneModerators::get())?;

					Self::do_create_report_verdict(content_id, content_type, tier_data.1, reward_pool.1)?;
					Self::do_assign_report_to_moderators(content_id, drafted_moderators)?;
					Self::do_update_report_status(content_id, content_type, ReportStatus::InResolution)?;
					Self::deposit_event(Event::ReportAppealAccepted(content_id));
				}
				
				Ok(().into())
			}


		}
	
	
	
	//** Helpers **//
	
		impl<T: Config> Pallet<T> {

			//* Moderator *//


				pub fn do_check_if_already_moderator(
					who: T::AccountId,
				) -> Result<(), DispatchError> {
					
					frame_support::ensure!(!Moderators::<T>::contains_key(who), Error::<T>::AlreadyRegisteredAsModerator);
					Ok(())
				} 


				pub fn do_create_moderator(
					who: T::AccountId,
				) -> Result<(), DispatchError> {
					
					let empty_bounded_reports: BoundedVec<T::ContentId, T::MaxReportsByModerator>
						= TryInto::try_into(Vec::new()).map_err(|_|Error::<T>::BadMetadata)?;
					
					let moderator = Moderator {	
						assigned_reports: empty_bounded_reports,
						rank: ModeratorRank::Junior,
					};
					
					Moderators::<T>::insert(who, moderator.clone());
					
					Ok(())
				} 


				pub fn do_draft_moderators(
					reporter_id: T::AccountId,
					reportee_id: T::AccountId,
					required_moderators: u32,
				) -> Result<Vec<T::AccountId>, DispatchError> { 
					
					let mut moderator_data: Vec<T::AccountId> = 
						Moderators::<T>::iter().map(|(x, _)| x).collect(); 
					// add iter keys, this version does not support them (https://github.com/paritytech/substrate/pull/9238))
					
					moderator_data.retain(|x| x != &reporter_id && x != &reportee_id);
					//(x != reporter_id && x != reportee_id)
					// majority_votes.retain(|vote| vote.is_for == VoteChoice::For); 
					
					ensure!(moderator_data.len() as u32 >= required_moderators, Error::<T>::NotEnoughModeratorsAvailable);

					let drafted_moderators = moderator_data.drain(0..required_moderators as usize).collect();
					// ensure!(moderator_data.len() >= T::TotalTierOneModerators::get() as usize, Error::<T>::NotEnoughModeratorsAvailable);
					// let btree = kine_stat_tracker::Pallet::<T>::create_moderator_btree(drafted_moderators).unwrap();

					Ok(drafted_moderators)
				} 	
				
				

				pub fn do_assign_report_to_moderators(
					content_id: T::ContentId,
					moderators: Vec<T::AccountId>,
				) -> Result<(), DispatchError> {
					
					for moderator_id in moderators.iter() { // assign the report to each selected moderator
						Moderators::<T>::try_mutate_exists(moderator_id, |moderator_data| -> DispatchResult {
							let moderator  = moderator_data.as_mut().ok_or(Error::<T>::NonexistentModerator)?;
							moderator.assigned_reports.try_push(content_id).unwrap();
							
							Ok(())
						})?;
					}
					
					Ok(())
				} 


				pub fn do_deallocate_moderator_from_report(
					moderator_id: T::AccountId,
					content_id: T::ContentId,
				) -> Result<(), DispatchError> {
					
					Moderators::<T>::try_mutate_exists(moderator_id, |moderator_data| -> DispatchResult {
						let mod_data = moderator_data.as_mut().ok_or(Error::<T>::NonexistentModerator)?;
						mod_data.assigned_reports.retain(|assigned_id| *assigned_id != content_id);
						Ok(())
					})?;
					
					Ok(())
				} 


				pub fn do_can_moderator_suspend(
					who: T::AccountId,
				) -> Result<bool, DispatchError> {
					
					let moderator = Moderators::<T>::try_get(who).unwrap();
					let mut can_suspend = true;
					if moderator.assigned_reports.len() == 0 {
						can_suspend = false;
					}

					Ok(can_suspend)
				} 
				

				pub fn do_suspend_moderation(
					who: T::AccountId,
				) -> Result<(), DispatchError> {
					
					let moderator = Moderators::<T>::try_get(who.clone()).unwrap();
					
					let tokens_moderation = 
						kine_stat_tracker::Pallet::<T>::
						get_wallet_tokens(who.clone()).unwrap();
					
					Self::do_transfer_funds_from_treasury(who.clone(), tokens_moderation.locked_tokens_moderation)?;
					
					kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
						who.clone(), 
						kine_stat_tracker::FeatureType::Moderation,
						kine_stat_tracker::TokenType::Locked,
						tokens_moderation.locked_tokens_moderation, true,
					).unwrap();


					Self::do_transfer_funds_from_treasury(who.clone(), tokens_moderation.claimable_tokens_moderation)?;
					
					kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
						who.clone(), 
						kine_stat_tracker::FeatureType::Moderation,
						kine_stat_tracker::TokenType::Claimable,
						tokens_moderation.claimable_tokens_moderation, true,
					).unwrap();
					
					
					
					Moderators::<T>::remove(who);
					Ok(())
				} 


				pub fn do_can_moderator_vote(
					who: T::AccountId,
					content_id: T::ContentId,
				) -> Result<(), DispatchError> {
					
					let assigned_reports = Moderators::<T>::try_get(who).unwrap().assigned_reports;
					ensure!(assigned_reports.contains(&content_id), Error::<T>::ModeratorNotDraftedForReport);

					Ok(())
				} 



			//* Report *//

				pub fn do_validate_report_data(
					content_id: T::ContentId,
					content_type: ContentType,
					_justification: BoundedVec<u8, T::JustificationLimit>,
					category_type: kine_tags::CategoryType<T>,
					category_tag_list: BoundedVec<(CategoryId<T>, TagId<T>), T::MaxTags>,
				) -> Result<(), DispatchError> {
					
                    kine_tags::Pallet::<T>::do_validate_tag_data(
                        category_type, 
                        category_tag_list.clone()
                    )?;

					ensure!(!Reports::<T>::contains_key((content_id, content_type)), Error::<T>::ReportAlreadyOngoing);
					
					Ok(())
				} 
				

				pub fn do_check_if_report_verdict_is_acceptable(
					content_id: T::ContentId,
					content_type: ContentType,
				) -> Result<ReportStatus, DispatchError> {
					
					let report = Reports::<T>::try_get((content_id.clone(), content_type)).unwrap();
					ensure!(
						(report.status == ReportStatus::MajorityVotedFor || report.status == ReportStatus::MajorityVotedAgainst), 
						Error::<T>::NonexistentReport
					);
					Ok(report.status)
				} 
				
				
				pub fn do_check_if_report_appeal_is_acceptable(
					content_id: T::ContentId,
					content_type: ContentType,
				) -> Result<ReportStatus, DispatchError> {
					
					let report = Reports::<T>::try_get((content_id.clone(), content_type)).unwrap();
					ensure!(
						(report.status == ReportStatus::AppealedByReporter || report.status == ReportStatus::AppealedByReportee), 
						Error::<T>::NonexistentReport
					);
					Ok(report.status)
				} 
				
					
				pub fn do_create_report(
					who: T::AccountId,
					content_id: T::ContentId,
					content_type: ContentType,
					reportee_id: T::AccountId,
					justification:BoundedVec<u8, T::JustificationLimit>,
					category_tag_list: BoundedVec<(CategoryId<T>, TagId<T>), T::MaxTags>,
				) -> Result<(), DispatchError> {

					let report = Report {
							reporter_id: who.clone(),
							reportee_id: reportee_id.clone(),
							justification: justification.clone(),
							status: ReportStatus::InResolution,
							categories_and_tags: category_tag_list,
						};
					Reports::<T>::insert((content_id.clone(), content_type), report.clone());
			
					Ok(())
				} 
				
					
				pub fn do_create_report_verdict(
					content_id: T::ContentId,
					content_type: ContentType,
					tier: Tiers,
					reward_pool: BalanceOf<T>,
				) -> Result<(), DispatchError> {
					
					let empty_bounded_votes: BoundedVec<Vote<T::AccountId>, T::MaxReportsByTier>
						= TryInto::try_into(Vec::new()).map_err(|_|Error::<T>::BadMetadata)?; // new empty BoundedVec
					let report_outcome = ReportOutcome {
						staked_tokens: reward_pool,
						required_votes: T::TotalTierOneModerators::get(),
						votes_for: 0,
						votes: empty_bounded_votes,
					};
					ReportVerdicts::<T>::insert((content_id.clone(), content_type), tier, report_outcome);
			
					Ok(())
				} 
			

				pub fn do_get_current_report_tier_data(
					content_id: T::ContentId,
					content_type: ContentType,
				) -> Result<(Tiers, Tiers, bool, Vec<Tiers>), DispatchError> {
					
					ensure!(ReportVerdicts::<T>::contains_key((content_id.clone(), content_type), Tiers::TierOne), Error::<T>::NonexistentReport);
					let mut current_tier = Tiers::TierOne;
					let mut next_tier = Tiers::TierTwo;
					let mut is_appealable = true;
					let mut all_tiers : Vec<Tiers> = vec![Tiers::TierOne];

					if ReportVerdicts::<T>::contains_key((content_id.clone(), content_type), Tiers::TierThree) {
						current_tier = Tiers::TierThree; 
						next_tier = Tiers::TierThree;
						all_tiers.append(&mut vec![Tiers::TierTwo, Tiers::TierThree]);
						is_appealable = false
					}
					else if ReportVerdicts::<T>::contains_key((content_id.clone(), content_type), Tiers::TierTwo) {
						current_tier = Tiers::TierTwo; 
						next_tier = Tiers::TierThree;
						all_tiers.append(&mut vec![Tiers::TierTwo]);
					}
					//TODO-12

					Ok((current_tier, next_tier, is_appealable, all_tiers))
				} 
			
				
				pub fn do_update_report_status(
					content_id: T::ContentId,
					content_type: ContentType,
					report_status: ReportStatus,
				) -> Result<(), DispatchError> {
					
					Reports::<T>::try_mutate_exists((content_id.clone(), content_type), |report| -> DispatchResult {
						let mut rep = report.as_mut().ok_or(Error::<T>::NonexistentReport)?;
						rep.status = report_status;
						Ok(())
					})
				} 
				
			
				pub fn do_check_if_reporter_or_reportee(
					who: T::AccountId,
					content_id: T::ContentId,
					content_type: ContentType,
				) -> Result<bool, DispatchError> {
					
					let mut is_reporter = true;

					let report = Reports::<T>::try_get((content_id.clone(), content_type)).unwrap();
					if report.reportee_id == who {
						is_reporter = false;
					}
					else { ensure!(report.reporter_id == who, Error::<T>::UserCannotAcceptVerdict) }

					Ok(is_reporter)
				} 		
			
			
				pub fn do_can_user_accept_verdict(
					is_reporter: bool,
					content_id: T::ContentId,
					content_type: ContentType,
				) -> Result<(), DispatchError> {
					
					let report = Reports::<T>::try_get((content_id.clone(), content_type)).unwrap();
					if report.status == ReportStatus::MajorityVotedFor {
						ensure!(!is_reporter, Error::<T>::UserCannotAcceptVerdict);
					} else if report.status == ReportStatus::MajorityVotedAgainst {
						ensure!(is_reporter, Error::<T>::UserCannotAcceptVerdict);
					}

					Ok(())
				} 		
				
			
				pub fn do_can_user_accept_appeal(
					is_reporter: bool,
					content_id: T::ContentId,
					content_type: ContentType,
				) -> Result<(), DispatchError> {
					
					let report = Reports::<T>::try_get((content_id.clone(), content_type)).unwrap();
					if report.status == ReportStatus::AppealedByReporter {
						ensure!(!is_reporter, Error::<T>::UserCannotAcceptVerdict);
					} else if report.status == ReportStatus::AppealedByReportee {
						ensure!(is_reporter, Error::<T>::UserCannotAcceptVerdict);
					}

					Ok(())
				} 		
				

				pub fn do_get_reporter(
					content_id: T::ContentId,
					content_type: ContentType,
				) -> Result<T::AccountId, DispatchError> {

					let report = Reports::<T>::get((content_id.clone(), content_type)).ok_or(Error::<T>::NonexistentReport)?;
					Ok(report.reporter_id) // return (reporter/reportee's reward, majority voter rewards per capita)
				}

				
				pub fn do_get_reportee(
					content_id: T::ContentId,
					content_type: ContentType,
				) -> Result<T::AccountId, DispatchError> {

					let report = Reports::<T>::get((content_id.clone(), content_type)).ok_or(Error::<T>::NonexistentReport)?;
					Ok(report.reportee_id) // return (reporter/reportee's reward, majority voter rewards per capita)
				}

				
				pub fn do_get_report_status_on_accept(
					is_reporter: bool,
				) -> Result<ReportStatus, DispatchError> {

					let report_status : ReportStatus;
					if is_reporter {
						report_status = ReportStatus::Refused;
					}
					else {
						report_status = ReportStatus::Accepted;
					}

					Ok(report_status)
				}

				
				pub fn do_get_report_status_on_appeal(
					is_reporter: bool,
				) -> Result<ReportStatus, DispatchError> {

					let report_status : ReportStatus;
					if is_reporter {
						report_status = ReportStatus::AppealedByReporter;
					}
					else {
						report_status = ReportStatus::AppealedByReportee;
					}

					Ok(report_status)
				}



			//* Vote *//
			
				pub fn do_create_vote(
					content_id: T::ContentId,
					content_type: ContentType,
					tier: Tiers,
					who: T::AccountId,
					is_for: VoteChoice,
				) -> Result<(), DispatchError> {

					let vote = Vote {
						voter: who.clone(),
						is_for: is_for,
					};

					ReportVerdicts::<T>::try_mutate_exists((content_id.clone(), content_type), tier, |report_outcome| -> DispatchResult {
						let outcome = report_outcome.as_mut().ok_or(Error::<T>::NonexistentReport)?;
						outcome.votes.try_push(vote).unwrap();
						if is_for == VoteChoice::For { 
							outcome.votes_for = 
								outcome.votes_for
								.checked_add(One::one())
								.ok_or(Error::<T>::VoteOverflow)?;
						};
						Ok(())
					})//TODO-13
				} 
				
				
				pub fn do_are_all_votes_submitted(
					content_id: T::ContentId,
					content_type: ContentType,
					tier: Tiers,
				) -> Result<bool, DispatchError> {

					let mut are_all_votes_submitted = false;
					let verdict = ReportVerdicts::<T>::get((content_id.clone(), content_type), tier).ok_or(Error::<T>::NonexistentReport)?;
					if verdict.votes.len() == (verdict.required_votes as usize) {are_all_votes_submitted = true;}

					Ok(are_all_votes_submitted)
				} 
				

				pub fn do_calculate_vote_consensus(
					content_id: T::ContentId,
					content_type: ContentType,
					tier: Tiers,
				) -> Result<ReportStatus, DispatchError> {
					
					let mut result = ReportStatus::MajorityVotedAgainst;
					let vote_consensus = ReportVerdicts::<T>::get((content_id.clone(), content_type), tier).ok_or(Error::<T>::NonexistentReport)?;
					if (vote_consensus.votes_for * 1000) > ((vote_consensus.required_votes * 1000) / 2) as u32  { 
						result = ReportStatus::MajorityVotedFor;
					}
					
					Ok(result)
				}


				pub fn do_get_report_voters_by_vote(
					content_id: T::ContentId,
					content_type: ContentType,
					tier: Tiers,
					consensus: ReportStatus,
				) -> Result<(Vec<T::AccountId>, Vec<T::AccountId>), DispatchError> {
					
					let mut majority_votes = ReportVerdicts::<T>::try_get((content_id.clone(), content_type), tier).unwrap().votes.clone();
					let mut minority_votes = majority_votes.clone();

					if consensus == ReportStatus::MajorityVotedFor || consensus == ReportStatus::AppealedByReportee {
						majority_votes.retain(|vote| vote.is_for == VoteChoice::For); 
						minority_votes.retain(|vote| vote.is_for == VoteChoice::Against); 
						
					} 
					else if consensus == ReportStatus::MajorityVotedAgainst || consensus == ReportStatus::AppealedByReporter {
						majority_votes.retain(|vote| vote.is_for == VoteChoice::Against); 
						minority_votes.retain(|vote| vote.is_for == VoteChoice::For); 
					}
					
					let majority_voters: Vec<T::AccountId> = majority_votes.into_iter().map(|vote| vote.voter ).collect();
					let minority_voters: Vec<T::AccountId> = minority_votes.into_iter().map(|vote| vote.voter ).collect();
					
					Ok((majority_voters, minority_voters))
				}



			//* Treasury *//

				pub fn do_transfer_funds_to_treasury(
					who: T::AccountId,
					amount: BalanceOf<T>,
				) -> Result<(), DispatchError> {

					<T as kine_stat_tracker::Config>::Currency::transfer(
						&who, &Self::account_id(),
						amount, AllowDeath, 
					);

					Ok(()) 
				}


				pub fn do_transfer_funds_from_treasury(
					who: T::AccountId,
					amount: BalanceOf<T>,
				) -> Result<(), DispatchError> {

					<T as kine_stat_tracker::Config>::Currency::transfer(
						&Self::account_id(),  &who,
						amount, AllowDeath, 
					);

					Ok(()) 
				}
			

				pub fn do_grab_reportee_collateral(
					content_id: T::ContentId,
					content_type: ContentType,
					collateral: BalanceOf<T>,
				) -> Result<(), DispatchError> {

					let report = Reports::<T>::get((content_id.clone(), content_type)).ok_or(Error::<T>::NonexistentReport)?;
					let reportee = report.reportee_id;

					let treasury = &Self::account_id();
					<T as kine_stat_tracker::Config>::Currency::transfer(
						&reportee.clone(), treasury,
						collateral.clone(), AllowDeath,
					);

					kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
						reportee, 
						kine_stat_tracker::FeatureType::Moderation,
						kine_stat_tracker::TokenType::Locked,
						collateral, true,
					).unwrap();

					Ok(())
				}
			

				pub fn do_claim_moderation_reward(
					moderator_id: T::AccountId,
				) -> DispatchResult {

					let claimable_tokens = 
							kine_stat_tracker::Pallet::<T>
							::get_wallet_tokens(moderator_id.clone()).unwrap().claimable_tokens_moderation;
						
						Self::do_transfer_funds_from_treasury(moderator_id.clone(), claimable_tokens)?;

						kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
							moderator_id, 
							kine_stat_tracker::FeatureType::Moderation,
							kine_stat_tracker::TokenType::Claimable,
							BalanceOf::<T>::from(0u32), true,
						).unwrap();

					Ok(())
				}



			//* Reward Distribution *//

				pub fn do_get_total_moderation_pool(
					content_id: T::ContentId,
					content_type: ContentType,
					tier: Tiers,
				) -> Result<BalanceOf<T>, DispatchError> {

					let report_outcome = ReportVerdicts::<T>::get((content_id.clone(), content_type), tier).ok_or(Error::<T>::NonexistentReport)?;
					Ok(report_outcome.staked_tokens) // return (reporter/reportee's reward, majority voter rewards per capita)
				}
			

				pub fn do_distribute_rewards_to_majority_voters(
					majority_voters: Vec<T::AccountId>,
					reward: BalanceOf<T>,
				) -> Result<(), DispatchError> {	
					
					for moderator_id in majority_voters.iter() {
						
						kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
							moderator_id.clone(), 
							kine_stat_tracker::FeatureType::Moderation,
							kine_stat_tracker::TokenType::Claimable,
							reward, false,
						).unwrap();
						
						let new_reputation = kine_stat_tracker::Pallet::<T>::do_calculate_reputation_change(
							moderator_id.clone(), 
							3u32, false,
						)?;
						
						if new_reputation >= T::MinimumReputationForSeniorship::get()  {
							Moderators::<T>::try_mutate_exists(moderator_id, |mod_data| -> DispatchResult {
								let moderator_data = mod_data.as_mut().ok_or(Error::<T>::NonexistentModerator)?;
								moderator_data.rank = ModeratorRank::Senior;
	
								Ok(())
							})?;
						}
						
					}

					Ok(())
				}


				pub fn do_slash_minority_voters(
					minority_voters: Vec<T::AccountId>,
				) -> Result<(), DispatchError> {	
					let moderator_fee: BalanceOf<T> = Self::do_calculate_moderator_fee()?;

					for moderator_id in minority_voters.iter() {
						
						let suspend_data = Moderators::<T>::try_mutate_exists(moderator_id, |mod_data| 
						-> Result<(bool, BoundedVec<T::ContentId, T::MaxReportsByModerator>), DispatchError> {
							let moderator_data = mod_data.as_mut().ok_or(Error::<T>::NonexistentModerator)?;
							let mut should_remove = false;

							kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
								moderator_id.clone(), 
								kine_stat_tracker::FeatureType::Moderation,
								kine_stat_tracker::TokenType::Locked,
								moderator_fee, true
							).unwrap();
							
							let new_reputation = kine_stat_tracker::Pallet::<T>::do_calculate_reputation_change(
								moderator_id.clone(), 
								3u32, true,
							)?;
							
							// suspend moderation
							if new_reputation < T::MinimumReputationForModeration::get() {
								should_remove = true;
							}
							// demote to junior if senior
							else if new_reputation < T::MinimumReputationForSeniorship::get() 
							&& moderator_data.rank == ModeratorRank::Senior {
								moderator_data.rank = ModeratorRank::Junior;
							}

							Ok((should_remove, moderator_data.assigned_reports.clone()))
						})?;

						if suspend_data.0 { // should_suspend
							for report_id in suspend_data.1 {
								//TODO reallocate all the pending reports, 
							}
							
							Self::do_suspend_moderation(moderator_id.clone())?;
						}


					}
					Ok(())
				}

			//* Reward Calculation *//

				pub fn do_calculate_report_pool(
					total_moderators: u32,
				) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
					// one part reportee, one part reporter, one part split among moderators
					let moderator_fee = Self::do_calculate_moderator_fee()?;
					let report_pool_third = moderator_fee.saturating_mul(total_moderators.into()); // in tier one this is the same as MovieCollateral
					let report_pool_total = moderator_fee.saturating_mul(3u32.into());
					Ok((report_pool_third, report_pool_total))
				}


				pub fn do_calculate_reportee_reward(
					reward_pool: BalanceOf<T>,
				) -> Result<BalanceOf<T>, DispatchError> {

					let half_pool = 
						reward_pool
						.checked_div(&BalanceOf::<T>::from(2u32))
						.ok_or(Error::<T>::Overflow)?;
					let one_third_pool = 
						reward_pool
						.checked_div(&BalanceOf::<T>::from(3u32))
						.ok_or(Error::<T>::Overflow)?;
					let reportee_reward = 
						half_pool
						.checked_sub(&one_third_pool)
						.ok_or(Error::<T>::Overflow)?;
					Ok(reportee_reward)
				}


				pub fn do_calculate_reporter_reward(
					reward_pool: BalanceOf<T>,
				) -> Result<BalanceOf<T>, DispatchError> {

					let half_pool = 
						reward_pool
						.checked_div(&BalanceOf::<T>::from(2u32))
						.ok_or(Error::<T>::Overflow)?;
					Ok(half_pool)
				}


				pub fn do_calculate_majority_voter_reward(
					reward_pool: BalanceOf<T>,
					total_majority_voters: u32,
				) -> Result<BalanceOf<T>, DispatchError> {

					let tokens_per_voter = 
						reward_pool
						.checked_div(&BalanceOf::<T>::from(total_majority_voters))
						.ok_or(Error::<T>::Overflow)?;
					let moderator_fee = BalanceOf::<T>::from(Self::do_calculate_moderator_fee()?);
					
					let majority_voter_reward = 
						tokens_per_voter
						.checked_sub(&moderator_fee)
						.ok_or(Error::<T>::Overflow)?;
					Ok(majority_voter_reward)
				}


				pub fn do_calculate_moderator_fee(
				) -> Result<BalanceOf<T>, DispatchError> {
					// let opaque = value.saturating_mul(_1000_balance).checked_div(total_value)?;
					
					let movie_collateral: BalanceOf<T> = T::MovieCollateral::get();
					let moderator_fee = 
						movie_collateral
						.checked_div(&BalanceOf::<T>::from(T::TotalTierOneModerators::get()))
						.ok_or(Error::<T>::Overflow)?;
					
					Ok(moderator_fee)
				}

				pub fn do_convert_collateral_to_balance(
				) -> Result<BalanceOf<T>, DispatchError> {

					let collateral_balance = BalanceOf::<T>::from(T::MovieCollateral::get());
					Ok(collateral_balance)
				}


				pub fn do_calculate_moderators_in_tier(
					tier: Tiers,
				) -> Result<u32, DispatchError> {
					
					 

					let mut tier_index = match tier {
						Tiers::TierOne => 1u32,
						Tiers::TierTwo => 2u32,
						Tiers::TierThree => 3u32,
					};



					let mut total_moderators = T::TotalTierOneModerators::get();
					
					while tier_index > 0u32 {
						total_moderators = 
							total_moderators
							.saturating_mul(2u32);
						
						total_moderators = 
							total_moderators
							.checked_add(1u32)
							.ok_or(Error::<T>::Overflow)?;
							
						tier_index =
							tier_index
							.checked_sub(1u32)
							.ok_or(Error::<T>::Underflow)?;
					}
					
					Ok(total_moderators)
				}






			//* Utils *//

				// The account ID of the vault
				fn account_id() -> T::AccountId {
					<T as Config>::PalletId::get().try_into_account().unwrap()
				}
		
		}
	
}