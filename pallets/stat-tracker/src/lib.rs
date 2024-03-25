//** About **//
	// Keeps track of a wallet's stats and tokens. Requires registration to be
	// interacted with. If no entries exist when tokens/stats are allocated, one is
	// automatically created for the wallet in question. Most other pallets reference
	// this pallet and use it to keep track of allocated/claimable tokens. It holds
	// tokens alongside the representation of how much each pallet has allocated
	// per wallet. It is also used as an abstraction to transfer funds to/from the
	// treasury, with the amount being designated per feature.
	
	//TODO-0 add comments to the pallet
	//TODO-1 check the use of references in the helper functions that do not need to use .clone()
	//TODO-2 implement the treasury from this pallet and migrate all other treasuries here
	//TODO-3 implement claim all tokens
	//TODO-5 add a static lookup for values like blocks per year when calculating imbalances
	//TODO-6 optimize self::account_id, store the value during genesis because calculating it is expensive

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
					PalletId,
					traits::{
						Currency,
						ReservableCurrency,
						ExistenceRequirement::{
							AllowDeath,
						},
					},
					sp_runtime::{
						traits::{
							CheckedAdd,
							CheckedSub,
							CheckedDiv,
							Saturating,
							AccountIdConversion,
						},
					}
				};
				use frame_system::pallet_prelude::*;
				use scale_info::prelude::string::String;
				use codec::{Decode, Encode, MaxEncodedLen};
	
			//* Config *//
	
				#[pallet::pallet]
				pub struct Pallet<T>(_);
	
				#[pallet::config]
				pub trait Config: frame_system::Config {
					type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
					type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
	
					type DefaultReputation: Get<u32>;
					type NameStringLimit: Get<u32>;
					
					type PalletId: Get<PalletId>;
				}
	
	
				
		//** Types **//	
		
			//* Types *//
	
				type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	
			//* Constants *//
			//* Enums *//
	
				// Allows the desambiguation of feature types.
				// Particularly useful for updating tokens values 
				// related to wallets.	
				#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
				pub enum FeatureType {
					Festival,
					RankingList,
					Moderation,
					Movie,
				}
				
				// Allows the desambiguation of token types.
				// Particularly useful for updating tokens values 
				// related to wallets.		
				#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
				pub enum TokenType {
					Locked,
					Claimable,
				}
	
			//* Structs *//
	
	
				// Stats that are bound to a wallet. This is required by many features, to ensure safety.
				// The "..._public" boolean parameters and the name are both defined by the user upon creation.
				#[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
				pub struct Stats<BoundedName> {
					pub is_name_public: bool,
					pub is_wallet_public: bool,
					pub name: BoundedName,
				}
				
				
				// The "total_..." and "claimable_..." balance parameters are each updated by the corresponding app feature.
				// To get the current locked balance, you must do "total_..." - "claimable_..." = "locked_...". 
				#[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo,)]
				pub struct Tokens<Balance, TokenImbalance> {
					pub reputation_moderation: u32,
					pub locked_tokens_moderation: Balance,
					pub claimable_tokens_moderation: Balance,
					
					pub locked_tokens_festival: Balance,
					pub claimable_tokens_festival: Balance,
					pub total_tokens_won_festival: Balance,

					pub locked_tokens_ranking: Balance,
					pub claimable_tokens_ranking: Balance,
					pub imbalance_tokens_ranking: TokenImbalance,
					pub total_tokens_won_ranking: Balance,
					
					pub locked_tokens_movie: Balance,
					pub claimable_tokens_movie: Balance,
				}
	
	
	
	
	
		//** Storage **//
	
	
			// Contains stats related to the identification of this address.
			// When an entery is created for WalletStats, an entry is automatically
			// created in WalletTokens.
			#[pallet::storage]
			#[pallet::getter(fn get_wallet_stats)]
			pub type WalletStats<T: Config> = 
				StorageMap<
					_, 
					Blake2_128Concat, T::AccountId,
					Stats<
						BoundedVec<u8, T::NameStringLimit>,
					>,
				>;
	
	
			// Keeps track of the amount of tokens (and reputation_moderation) a wallet has.
			// It is independent from the "WalletStats" storage, meaning an entry
			// can exist by itself without being registed in "WalletStats".
			#[pallet::storage]
			#[pallet::getter(fn get_wallet_tokens)]
			pub type WalletTokens<T: Config> = 
				StorageMap<
					_, 
					Blake2_128Concat, T::AccountId,
					Tokens<
						BalanceOf<T>,
						(BalanceOf<T>, BalanceOf<T>),
					>,
				>;
		
		
		//** Events **//
	
			#[pallet::event]
			#[pallet::generate_deposit(pub(super) fn deposit_event)]
			pub enum Event<T: Config> {
				AccountRegisteredAddress(T::AccountId),
				AccountRegisteredName(String),
	
				AccountUnregisteredAddress(T::AccountId),
				AccountUnregisteredName(String),
	
				AccountDataUpdatedAddress(T::AccountId),
				AccountDataUpdatedName(String),
	
				TokensClaimed(T::AccountId),
			}
		
	
	
		//** Errors **//
	
			#[pallet::error]
			pub enum Error<T> {
				WalletAlreadyRegistered,
				WalletNotRegisteredStatTracker,
				WalletStatsNotFound,
				WalletTokensNotFound,
	
				DraftedModeratorNotRegistered,
	
				BadMetadata,
				WalletStatsRegistryRequired,
				
				TokenOverflow,
				ReputationOverflow,
				TokenUnderflow,
				ReputationUnderflow,
				NotEnoughBalance,
			}
	
	
	
		//** Hooks **//
	
			// #[pallet::hooks]
			// impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
		
	
			
		//** Extrinsics **//
			
			#[pallet::call]
			impl<T:Config> Pallet<T> {
	
	
				// Register a new wallet if previously unregistered.
				// This is required by many features in the app.
				#[pallet::call_index(0)]
				#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
				pub fn register_new_wallet(
					origin: OriginFor<T>,
					is_name_public: bool,
					is_wallet_public: bool,
					name_str: String,
				) -> DispatchResultWithPostInfo {
					
					let who = ensure_signed(origin)?;
					ensure!(
						!WalletStats::<T>::contains_key(who.clone()), 
						Error::<T>::WalletAlreadyRegistered
					);
					
					let name: BoundedVec<u8, T::NameStringLimit>
                        	= TryInto::try_into(name_str.as_bytes().to_vec()).map_err(|_|Error::<T>::BadMetadata)?;

					let stats = Stats {
						is_wallet_public: is_wallet_public,
						is_name_public: is_name_public,
						name: name.clone(),
					};
					WalletStats::<T>::insert(who.clone(), stats.clone());
	
					if !WalletTokens::<T>::contains_key(who.clone()) {
						let mut wallet_tokens = Self::do_create_new_wallet_tokens_zero_balance().unwrap();
						WalletTokens::<T>::insert(who.clone(), wallet_tokens.clone());
					};
	
					// check if events should be emitted, depending on the privacy settings
					if is_wallet_public {
						Self::deposit_event(Event::AccountRegisteredAddress(who));   
					}
					else if is_name_public {
						let name_str = String::from_utf8(name.to_vec()).unwrap();
						Self::deposit_event(Event::AccountRegisteredName(name_str));   
					};   
	
					Ok(().into())
				}
	
	
				// Unregister a wallet, automatically claiming any leftover tokens.
				//TODO-2
				//TODO-3
				#[pallet::call_index(1)]
				#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
				pub fn unregister_wallet(
					origin: OriginFor<T>,
					name_str: String,
				) -> DispatchResultWithPostInfo {
					
					let who = ensure_signed(origin)?;
	
					let name: BoundedVec<u8, T::NameStringLimit>
						= TryInto::try_into(name_str.as_bytes().to_vec()).map_err(|_|Error::<T>::BadMetadata)?;

					let stats = WalletStats::<T>::try_get(who.clone()).unwrap();
	
					WalletStats::<T>::remove(who.clone());
	
					// check if events should be emitted, depending on the privacy settings
					if stats.is_wallet_public {
						Self::deposit_event(Event::AccountUnregisteredAddress(who));   
					}
					else if stats.is_name_public {
						Self::deposit_event(Event::AccountUnregisteredName(name_str));   
					}
	
					Ok(().into())
				}
	
	
	
	
				// Update all the privacy settings. This is done all at once, in order to
				// save on otherwise multiple gas fees. 
				#[pallet::call_index(2)]
				#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
				pub fn update_wallet_data(
					origin: OriginFor<T>,
					is_name_public: bool,
					is_wallet_public: bool,
					name_str: String,
				) -> DispatchResultWithPostInfo {
					
					let who = ensure_signed(origin)?;
					ensure!(
						WalletStats::<T>::contains_key(who.clone()), 
						Error::<T>::WalletNotRegisteredStatTracker
					);
	
					WalletStats::<T>::try_mutate(who.clone(), |wallet_stats| -> DispatchResult {
						let stats = wallet_stats.as_mut().ok_or(Error::<T>::WalletStatsNotFound)?;
						
						let name: BoundedVec<u8, T::NameStringLimit>
                        	= TryInto::try_into(name_str.as_bytes().to_vec()).map_err(|_|Error::<T>::BadMetadata)?;

						// update the wallet's data
						stats.is_name_public = is_name_public;
						stats.is_wallet_public = is_wallet_public;
						stats.name = name.clone();
	
						Ok(())
					})?;
	
					// check if events should be emitted, depending on the privacy settings
					if is_wallet_public {
						Self::deposit_event(Event::AccountDataUpdatedAddress(who));   
					}
					else if is_name_public {
						Self::deposit_event(Event::AccountDataUpdatedName(name_str));   
					}
	
					Ok(().into())
				}
	
	
	
				// TODO-3
				#[pallet::call_index(3)]
				#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
				pub fn claim_all_tokens(
					origin: OriginFor<T>,
				) -> DispatchResultWithPostInfo {
					
					let who = ensure_signed(origin)?;
	
					WalletTokens::<T>::try_mutate_exists(who.clone(), |wallet_tokens| -> DispatchResult {
						let tokens = wallet_tokens.as_mut().ok_or(Error::<T>::WalletTokensNotFound)?;
	
						let zero_balance = BalanceOf::<T>::from(0u32);
						
						// add all the claimable tokens into the same var
						let mut total_tokens = 
							zero_balance.clone()
							.checked_add(&tokens.claimable_tokens_moderation)
							.ok_or(Error::<T>::TokenOverflow)?;
						total_tokens = 
							total_tokens
							.checked_add(&tokens.claimable_tokens_festival)
							.ok_or(Error::<T>::TokenOverflow)?;
						total_tokens = 
							total_tokens
							.checked_add(&tokens.claimable_tokens_ranking)
							.ok_or(Error::<T>::TokenOverflow)?;
	
						// ensure the transfer works
						ensure!(
							T::Currency::transfer(
								&Self::account_id(),
								&who.clone(),
								total_tokens.clone(), 
								AllowDeath
							) == Ok(()),
							Error::<T>::NotEnoughBalance
						);
	
						// reset the total claimable tokens 
						tokens.claimable_tokens_moderation = zero_balance.clone();
						tokens.claimable_tokens_festival = zero_balance.clone();
						tokens.claimable_tokens_ranking = zero_balance;
	
						Self::deposit_event(Event::TokensClaimed(who));   
						Ok(().into())
					})?;
	
	
	
					Ok(().into())
				}
	
			}
		
		
		
		//** Helpers **//
		
			impl<T:Config> Pallet<T> {
						
	
				// Balance Changes
				
				// Used to abstract the "update_token" functions.
				// This allows a single function to manage all 
				// the different types of tokens.
				pub fn do_update_wallet_tokens(
					who: T::AccountId,
					feature_type: FeatureType,
					token_type: TokenType,
					token_change: BalanceOf<T>,
					is_slash: bool,
				) -> DispatchResultWithPostInfo {
					
					// this is the first instance where the wallet's tokens are updated
					if !WalletTokens::<T>::contains_key(who.clone()) {
						Self::do_update_wallet_tokens_doesnt_exist(
							who, 
							feature_type, token_type, 
							token_change,
						)?;
					}
	
					// the wallet already contains an entry, retrieve & update
					else {
						Self::do_update_wallet_tokens_exists(
							who, 
							feature_type, token_type, 
							token_change, is_slash,
						)?;
					}
	
					Ok(().into())
				}
	
	
				//
				pub fn do_update_wallet_tokens_doesnt_exist(
					who: T::AccountId,
					feature_type: FeatureType,
					token_type: TokenType,
					token_change: BalanceOf<T>,
				) -> DispatchResultWithPostInfo  {
	
					let mut wallet_tokens = Self::do_create_new_wallet_tokens_zero_balance().unwrap();
	
					match (feature_type, token_type) {
						(FeatureType::Festival, TokenType::Locked) => wallet_tokens.locked_tokens_festival = token_change.clone(),
						(FeatureType::Festival, TokenType::Claimable) => wallet_tokens.claimable_tokens_festival = token_change.clone(),
	
						(FeatureType::RankingList, TokenType::Locked) => wallet_tokens.locked_tokens_ranking = token_change.clone(),
						(FeatureType::RankingList, TokenType::Claimable) => wallet_tokens.claimable_tokens_ranking = token_change.clone(),
						
						(FeatureType::Moderation, TokenType::Locked) => wallet_tokens.locked_tokens_moderation = token_change.clone(),
						(FeatureType::Moderation, TokenType::Claimable) => wallet_tokens.claimable_tokens_moderation = token_change.clone(),
						
						(FeatureType::Movie, TokenType::Locked) => wallet_tokens.locked_tokens_movie = token_change.clone(),
						(FeatureType::Movie, TokenType::Claimable) => wallet_tokens.claimable_tokens_movie = token_change.clone(),
					};
	
					//TODO-4 
					// (FeatureType::Festival, TokenType::Locked) => wallet_tokens.locked_tokens_festival = 
					// zero_balance.clone()
					// .checked_sub(&total_tokens.clone())
					// .ok_or(Error::<T>::TokenUnderflow)?,
	
					WalletTokens::<T>::insert(who.clone(), wallet_tokens.clone());
	
					Ok(().into())
				}
	
	
				//
				pub fn do_update_wallet_tokens_exists(
					who: T::AccountId,
					feature_type: FeatureType,
					token_type: TokenType,
					token_change: BalanceOf<T>,
					is_slash: bool,
				) -> DispatchResultWithPostInfo  {
	
					WalletTokens::<T>::try_mutate(who, |wal_tokens| -> DispatchResult {
						let wallet_tokens = wal_tokens.as_mut().ok_or(Error::<T>::WalletTokensNotFound)?;
						
						// dynamically select which token variable to update
						match (feature_type, token_type) {
							(FeatureType::Festival, TokenType::Locked) => wallet_tokens.locked_tokens_festival = 
								Self::do_calculate_token_change(
									wallet_tokens.locked_tokens_festival.clone(),
									token_change,
									is_slash,
								)?,
							(FeatureType::Festival, TokenType::Claimable) => {
								wallet_tokens.claimable_tokens_festival = 
								Self::do_calculate_token_change(
									wallet_tokens.claimable_tokens_festival.clone(),
									token_change, is_slash,
								)?;
								wallet_tokens.total_tokens_won_festival = 
								Self::do_calculate_token_change(
									wallet_tokens.total_tokens_won_festival.clone(),
									token_change,
									is_slash,
								)?;
							},
		
							(FeatureType::RankingList, TokenType::Locked) => wallet_tokens.locked_tokens_ranking = 
								Self::do_calculate_token_change(
									wallet_tokens.locked_tokens_ranking.clone(),
									token_change, is_slash,
								)?, 
							(FeatureType::RankingList, TokenType::Claimable) => {
								wallet_tokens.claimable_tokens_ranking = 
								Self::do_calculate_token_change(
									wallet_tokens.claimable_tokens_ranking.clone(),
									token_change, is_slash,
								)?;
								wallet_tokens.total_tokens_won_ranking = 
								Self::do_calculate_token_change(
									wallet_tokens.total_tokens_won_ranking.clone(),
									token_change,
									is_slash,
								)?;
							},
							
							(FeatureType::Moderation, TokenType::Locked) => wallet_tokens.locked_tokens_moderation = 
								Self::do_calculate_token_change(
									wallet_tokens.locked_tokens_moderation.clone(),
									token_change, is_slash,
								)?,
							(FeatureType::Moderation, TokenType::Claimable) => wallet_tokens.claimable_tokens_moderation = 
								Self::do_calculate_token_change(
									wallet_tokens.claimable_tokens_moderation.clone(),
									token_change, is_slash,
								)?,
	
							(FeatureType::Movie, TokenType::Locked) => wallet_tokens.locked_tokens_movie = 
								Self::do_calculate_token_change(
									wallet_tokens.locked_tokens_movie.clone(),
									token_change, is_slash,
								)?,
							(FeatureType::Movie, TokenType::Claimable) => wallet_tokens.claimable_tokens_movie = 
								Self::do_calculate_token_change(
									wallet_tokens.claimable_tokens_movie.clone(),
									token_change, is_slash,
								)?,
						};
	
						Ok(().into())
					})?;
	
					Ok(().into())
				}
	
	
	
	
	
				// Imbalance Changes
	
				// Takes an "imbalance" (a fraction of staked/total) and updates 
				// it depending on the current imbalance.
				// if the nominator is higher than the the denominator, the quoeficient
				// as an integer is returned alongside the new imbalance.
				pub fn do_update_wallet_imbalance(
					who: T::AccountId,
					feature_type: FeatureType,
					new_earned: BalanceOf<T>,
					total_earning_needed: BalanceOf<T>,
					is_slash: bool,
				) -> DispatchResultWithPostInfo  {
	
					// this is the first instance where the wallet's tokens are updated
					if !WalletTokens::<T>::contains_key(who.clone()) {
						Self::do_handle_imbalance_wallet_doesnt_exist(
							who, feature_type, 
							new_earned, total_earning_needed,
							is_slash,
						)?;
					}
	
					// the wallet already contains an entry, retrieve & update
					else {
						Self::do_handle_imbalance_wallet_exists(
							who,
							feature_type, 
							new_earned, total_earning_needed,
							is_slash,
						)?;
					}
	
					Ok(().into())
				}
	
	
				pub fn do_handle_imbalance_wallet_doesnt_exist(
					who: T::AccountId,
					feature_type: FeatureType,
					new_earned: BalanceOf<T>,
					total_earning_needed: BalanceOf<T>,
					is_slash: bool,
				) -> DispatchResultWithPostInfo  {
	
					let mut wallet_tokens = Self::do_create_new_wallet_tokens_zero_balance().unwrap();
	
					match feature_type {
						FeatureType::RankingList => {
							let (new_balance, new_imbalance) = 
								Self::do_calculate_imbalance_change(
									BalanceOf::<T>::from(0u32),
									new_earned,
									total_earning_needed,
									is_slash,
								)?;
							
							wallet_tokens.imbalance_tokens_ranking = new_imbalance;
	
							WalletTokens::<T>::insert(who.clone(), wallet_tokens.clone());
	
							if new_balance > BalanceOf::<T>::from(0u32) {
								wallet_tokens.claimable_tokens_ranking = 
									Self::do_calculate_token_change(
										wallet_tokens.claimable_tokens_ranking,
										new_balance, is_slash,
									)?;
								wallet_tokens.total_tokens_won_ranking = 
									Self::do_calculate_token_change(
										wallet_tokens.total_tokens_won_ranking,
										new_balance, is_slash,
									)?;
							}
						},
						
						_ => ()
					};
	
					//TODO-4 
	
					Ok(().into())
				}
	
	
				// it depending on the current imbalance.
				// if the nominator is higher than the the denominator, the quoeficient
				// as an integer is returned alongside the new imbalance.
				pub fn do_handle_imbalance_wallet_exists(
					who: T::AccountId,
					feature_type: FeatureType,
					new_earned: BalanceOf<T>,
					total_earning_needed: BalanceOf<T>,
					is_slash: bool,
				) -> DispatchResultWithPostInfo  {
	
					WalletTokens::<T>::try_mutate(who.clone(), |wal_tokens| -> DispatchResult {
						let wallet_tokens = wal_tokens.as_mut().ok_or(Error::<T>::WalletTokensNotFound)?;
						
						// dynamically select which token variable to update
						match feature_type {
							FeatureType::RankingList => {
								let (current_earned, _) = 
									wallet_tokens.imbalance_tokens_ranking;
								
								let (new_balance, new_imbalance) = 
									Self::do_calculate_imbalance_change(
										current_earned,
										new_earned,
										total_earning_needed,
										is_slash,
									)?;
								
								wallet_tokens.imbalance_tokens_ranking = new_imbalance;
	
								if new_balance > BalanceOf::<T>::from(0u32) {
									wallet_tokens.claimable_tokens_ranking = 
										Self::do_calculate_token_change(
											wallet_tokens.claimable_tokens_ranking,
											new_balance, is_slash,
										)?;
									wallet_tokens.total_tokens_won_ranking = 
										Self::do_calculate_token_change(
											wallet_tokens.total_tokens_won_ranking,
											new_balance, is_slash,
										)?;
								}
							},
	
							_ => ()
	
						};
	
						Ok(().into())
					})?;
	
					Ok(().into())
				}
	
	
	
	
				// Token & Value Calculators
	
				// Calculates a new value for a Balance.
				// If this is a slash, the value is subtracted to a minimum of 0.
				// If not, the value is added to the total.
				// The return is composed of (updated_value, value_change)
				pub fn do_calculate_token_change(
					mut current_tokens: BalanceOf<T>,
					token_change: BalanceOf<T>,
					is_slash: bool,
				) -> Result<BalanceOf::<T>, DispatchError>  {
	
					// reset the locked tokens back to 0
					// if token_change == BalanceOf::<T>::from(0u32) {
					// 	current_tokens = BalanceOf::<T>::from(0u32);
					// }
					if is_slash {
						if current_tokens > token_change {
							current_tokens =
								current_tokens
								.checked_sub(&token_change)
								.ok_or(Error::<T>::ReputationUnderflow)?;
						}
						else {
							current_tokens = BalanceOf::<T>::from(0u32);
						}
					}
					else {
						current_tokens =
							current_tokens.clone()
							.checked_add(&token_change)
							.ok_or(Error::<T>::TokenOverflow)?;
					}
	
					Ok(current_tokens)
				}
	
	
				// Takes an "imbalance" (a fraction of staked/total) and updates 
				// it depending on the current imbalance.
				// if the nominator is higher than the the denominator, the quoeficient
				// as an integer is returned alongside the new imbalance.
				pub fn do_calculate_imbalance_change(
					mut current_earned: BalanceOf<T>,
					new_earned: BalanceOf<T>,
					total_earning_needed: BalanceOf<T>,
					is_slash: bool,
				) -> Result<(BalanceOf::<T>, (BalanceOf::<T>, BalanceOf::<T>)), 
				DispatchError> {
	
					let mut new_balance = BalanceOf::<T>::from(0u32);
	
					// reset the locked tokens back to 0
					// if new_earned == BalanceOf::<T>::from(0u32)
					// && total_earning_needed == BalanceOf::<T>::from(0u32) {
					// 	current_earned = BalanceOf::<T>::from(0u32);
					// }
					//TODO-4
					if is_slash {
						if current_earned > new_earned {
							current_earned =
								current_earned
								.checked_sub(&new_earned)
								.ok_or(Error::<T>::ReputationUnderflow)?;
						}
						else {
							current_earned = BalanceOf::<T>::from(0u32);
						}
					}
					else {
						current_earned =
							current_earned
							.checked_add(&new_earned)
							.ok_or(Error::<T>::TokenOverflow)?;
						
						if current_earned > total_earning_needed {
							new_balance = 
								current_earned
								.checked_div(&total_earning_needed)
								.ok_or(Error::<T>::TokenUnderflow)?;
							
							let imbalance = new_balance.saturating_mul(
								total_earning_needed
							);
							
							current_earned =
								current_earned
								.checked_sub(&imbalance)
								.ok_or(Error::<T>::TokenUnderflow)?;
						}
					}
	
					Ok((new_balance, (current_earned, total_earning_needed)))
				}
	
	
				// Calculates a new value for a wallet's reputation.
				// If this is a slash, the value is subtracted to a minimum of 0.
				// If not, the value is added to the total.
				// The updated values are then returned.
				pub fn do_calculate_reputation_change(
					who: T::AccountId,
					new_reputation: u32,
					is_slash: bool,
				) -> Result<u32, DispatchError> {
					
					let final_reputation: u32 = WalletTokens::<T>::try_mutate(who.clone(), |wal_tokens| -> Result<u32, DispatchError> {
						let wallet_tokens = wal_tokens.as_mut().ok_or(Error::<T>::WalletTokensNotFound)?;
						let mut current_reputation = wallet_tokens.reputation_moderation;
	
						if is_slash {
							if current_reputation > new_reputation {
								current_reputation =
									current_reputation
									.checked_sub(new_reputation)
									.ok_or(Error::<T>::ReputationUnderflow)?;
							}
							else {
								current_reputation = 0u32;
							}
						}
						else {
							current_reputation =
								current_reputation
								.checked_add(new_reputation)
								.ok_or(Error::<T>::ReputationOverflow)?;
						}
	
						Ok(current_reputation)
					})?;
	
					Ok(final_reputation)
				}
					
	
	
				// True if the wallet is registered in the "WalletStats" storage.
				// This always implies that an entry also exists e the 
				// "WalletTokens" storage.
				pub fn do_is_wallet_registered(
					who: T::AccountId,
				) -> Result<bool, DispatchError> {
	
					Ok(WalletStats::<T>::contains_key(who))
				}
	
	
				//TODO-6
				fn account_id() -> T::AccountId {
					<T as Config>::PalletId::get().try_into_account().unwrap()
				}
	
	
	

                fn do_create_new_wallet_tokens_zero_balance(
                ) -> Result<Tokens<BalanceOf<T>,(BalanceOf<T>, BalanceOf<T>)>, DispatchError> {
					
					let zero_balance = BalanceOf::<T>::from(0u32);

					let mut wallet_tokens = Tokens {
						reputation_moderation: T::DefaultReputation::get(),
						locked_tokens_moderation: zero_balance.clone(),
						claimable_tokens_moderation: zero_balance.clone(),
						
						locked_tokens_festival: zero_balance.clone(),
						claimable_tokens_festival: zero_balance.clone(),
						total_tokens_won_festival: zero_balance.clone(),
						
						locked_tokens_ranking: zero_balance.clone(),
						claimable_tokens_ranking: zero_balance.clone(),
						imbalance_tokens_ranking: (zero_balance.clone(), zero_balance.clone()),
						total_tokens_won_ranking: zero_balance.clone(),
						
						locked_tokens_movie: zero_balance.clone(),
						claimable_tokens_movie: zero_balance,
					};

                    Ok(wallet_tokens)
                }



				
			}
	}