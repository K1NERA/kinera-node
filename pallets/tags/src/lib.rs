//** About **//
	// This pallet handles information regarding Categories/Tags. 
	// These classifications act as a way to classify existing content,
	// providing a framework to feed other systems with information.
	
	//TODO-0 add resctrictions and limitations to add new categories/tags
	//TODO-1 check if ContentStringLimit is needed as is, or parse boundedvecs into this specific format
	//TODO-2 validate description when creating a new tag
	//TODO-3 implement genesis defaukt values

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
					pallet_prelude::*,
					sp_runtime,
				};
				use frame_system::pallet_prelude::*;
				use scale_info::prelude::vec::Vec;
	
				
	
			//* Config *//
			
				#[pallet::pallet]
				pub struct Pallet<T>(_);
	
				#[pallet::config]
				pub trait Config: frame_system::Config {
					type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
					
					type MaxTags: Get<u32>;
					type MaxContentWithTag: Get<u32>; 
					type ContentStringLimit: Get<u32>; //TODO-1
	
					type CategoryStringLimit: Get<u32>;
					type TagStringLimit: Get<u32>;
				}
	
	
	
		//** Types **//	
		
			//* Types *//
	
				pub type CategoryType<T> = BoundedVec<u8, <T as Config>::CategoryStringLimit>;
				pub type CategoryId<T> = BoundedVec<u8, <T as Config>::CategoryStringLimit>;
				pub type TagId<T> = BoundedVec<u8, <T as Config>::TagStringLimit>;
			
			//* Constants *//
			//* Enums *//
			//* Structs *//
	
				#[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
				#[scale_info(skip_type_params(T))]
				pub struct TagIdList<BoundedTagList> {
					pub tag_list: BoundedTagList,
				}
	
				#[derive(Clone, Encode, Copy, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
				#[scale_info(skip_type_params(T))]
				pub struct TagData<BoundedTagList> {
					pub content_with_tag: BoundedTagList,
				}
	
	
	
		//** Genesis **//
			
	
	
			// #[pallet::genesis_config]
			// #[derive(frame_support::DefaultNoBound)]
			// pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
			// 	_config: sp_std::marker::PhantomData<(T,I)>,
			// 	_myfield: u32,
			// }
			
	
			#[pallet::genesis_config]
			#[derive(frame_support::DefaultNoBound)]
			pub struct GenesisConfig<T: Config> {
				pub category_to_tag_map: Vec<(
					(CategoryType<T>, CategoryId<T>),
					BoundedVec<TagId<T>, T::MaxTags>
				)>,
			}
	
	
			//TODO-3
			// #[cfg(feature = "std")]
			// impl<T: Config> Default for GenesisConfig<T> {
			// 	fn default() -> Self {
			// 		Self { 
			// 			category_to_tag_map: Default::default() 
			// 		}
			// 	}
			// }
	
	
			// #[pallet::genesis_build]
			// impl<T: Config<I>, I: 'static> BuildGenesisConfig for GenesisConfig<T, I> {
			// 	fn build(&self) {}
			// }
	
	
			#[pallet::genesis_build]
			impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
				fn build(&self) {
					for ((category_type, category_id), bounded_tag_id_list) in &self.category_to_tag_map {
						
						// initialize the tag id list
						let tag_id_list = TagIdList {
							tag_list: bounded_tag_id_list.clone(),
						};
						
						// insert into the categories storage
						<Categories<T>>::insert(
							(category_type.clone(), category_id.clone()),
							tag_id_list
						);
	
						// create an entry for each of the type/category's tags
						// to contain the relevant TagData 
						let bounded_content_with_tag: BoundedVec<BoundedVec<u8, T::ContentStringLimit>, T::MaxContentWithTag>
							= TryInto::try_into(Vec::new()).unwrap();
	
						for tag_id in bounded_tag_id_list {
							let tag = TagData {
								content_with_tag: bounded_content_with_tag.clone(),
							};
	
							<Tags<T>>::insert(
								(category_type, category_id),
								tag_id,
								tag
							);
						}
					}
				}
			}
	
	
	
		//** Storage **//
	
			//* Category *// 
	
				// Matches a tuple of CategoryType (ex: Moderation) 
				// and a CategoryId (ex: Offensive)
				// to the list of all the respective TagIds.
				// ex: (Moderation, Offensive) -> [Racism, Hate Speech, ...]. 
				#[pallet::storage]
				#[pallet::getter(fn get_category)]
				pub type Categories <T: Config> =
					StorageMap<
						_, 
						Blake2_128Concat, (CategoryType<T>, CategoryId<T>), 
						TagIdList<
							BoundedVec<TagId<T>, T::MaxTags>, 
						>,
						OptionQuery
					>;
	
					
				// Matches a tuple of CategoryType (ex: Moderation) 
				// and a CategoryId (ex: Offensive) with a secondary TagId
				// (ex: Hate Speech) to the respective tag data.
				// ex: (Moderation, Offensive), Hate Speech 
				// -> Moderation / Offensive / Hate Speech Tag Data. 
				#[pallet::storage]
				#[pallet::getter(fn get_tag)]
				pub type Tags <T: Config> =
					StorageDoubleMap<
						_, 
						Blake2_128Concat, (CategoryType<T>, CategoryId<T>),
						Blake2_128Concat, TagId<T>,
						TagData<
							BoundedVec<
								BoundedVec<u8, T::ContentStringLimit>,
								T::MaxContentWithTag,
							>
							
						>,
						OptionQuery
					>;
	
	
	
					
	
	
		//** Events **//
	
			#[pallet::event]
			#[pallet::generate_deposit(pub(super) fn deposit_event)]
			pub enum Event<T: Config> {
				CategoryCreated(T::AccountId, CategoryId<T>),
				TagCreated(T::AccountId, TagId<T>, CategoryId<T>),
			}
	
	
	
		//** Errors **//
			
			#[pallet::error]
			pub enum Error<T> {
				NoneValue,
				StorageOverflow,
				BadMetadata,
	
				CategoryAlreadyExists,
				NonexistentCategory,
				
				TagAlreadyExists,
				NonexistentTag,
			}
	
			
	
		//** Extrinsics **//
	
			#[pallet::call]
			impl<T: Config> Pallet<T> {
			
			// 	#[pallet::weight(10_000)]
			//     pub fn create_category(
			//         origin: OriginFor<T>,
			//         bounded_name: CategoryId<T>,
			//     ) -> DispatchResult {
					
			//         let who = ensure_signed(origin)?;
			//         Self::do_validate_category(bounded_name.clone())?;
	
			//         Self::do_create_category(bounded_name.clone())?;
	
			//         Self::deposit_event(Event::CategoryCreated(who.clone(), bounded_name));
			//         Ok(())
			//     }
	
				
				// #[pallet::weight(10_000)]
				// pub fn create_tag(
				//     origin: OriginFor<T>,
				//     bounded_tag: TagId<T>,
				//     bounded_category: CategoryId<T>,
				//     bounded_description: BoundedVec<u8, T::DescStringLimit>,
				// ) -> DispatchResult {
					
				//     let who = ensure_signed(origin)?;
				//     Self::do_validate_tag(bounded_tag.clone(), bounded_category.clone())?;
				// 	//TODO-2
	
				//     Self::do_create_tag(bounded_tag.clone(), bounded_category.clone(), bounded_description.clone())?;
					
				//     Self::deposit_event(Event::TagCreated(who.clone(), bounded_tag, bounded_category));
				//     Ok(())
				// }
			
			
			}
	
	
			
		//** Helpers **//
	
			impl<T: Config> Pallet<T> {
	
	
				//* Category *//
	
					// pub fn do_validate_category (
					// 	bounded_name:  CategoryId<T>,
					// )-> Result<(), DispatchError> {
	
					// 	ensure!(!Categories::<T>::contains_key(bounded_name), Error::<T>::CategoryAlreadyExists);
					// 	Ok(())
					// }
	
	
					// pub fn do_create_category (
					// 	bounded_name:  CategoryId<T>,
					// )-> Result<(), DispatchError> {
							
					// 	let bounded_tag_list: BoundedVec<TagId<T>, T::MaxTags>
					// 		= TryInto::try_into(Vec::new()).map_err(|_|Error::<T>::BadMetadata)?;
	
					// 	let category = Category {
					// 		tag_list: bounded_tag_list,
					// 	};
	
					// 	Categories::<T>::insert(bounded_name, category);
					// 	Ok(())
					// }
	
	
	
				//* Tag *//
	
					// pub fn do_validate_tag (
					// 	bounded_tag:  TagId<T>,
					// 	bounded_category:  CategoryId<T>,
					// )-> Result<(), DispatchError> {
	
					// 	ensure!(Categories::<T>::contains_key(bounded_category), Error::<T>::NonexistentCategory);
					// 	ensure!(!Tags::<T>::contains_key(bounded_tag), Error::<T>::TagAlreadyExists);
					// 	Ok(())
					// }
	
	
					// pub fn do_create_tag (
					// 	bounded_tag:  TagId<T>,
					// 	bounded_category:  CategoryId<T>,
					// 	bounded_description: BoundedVec<u8, T::DescStringLimit>,
					// )-> Result<(), DispatchError> {
							
					// 	let tag = Tag {
					// 		parent_category: bounded_category.clone(),
					// 		description: bounded_description,
					// 	};
	
					// 	Categories::<T>::try_mutate_exists(bounded_category, |category| -> DispatchResult {
					// 		let cat = category.as_mut().ok_or(Error::<T>::BadMetadata)?;
					// 		ensure!(!cat.tag_list.contains(&bounded_tag), Error::<T>::TagAlreadyExists);
							
					// 		cat.tag_list.try_push(bounded_tag.clone()).unwrap();
					// 		Ok(())
					// 	})?;
	
					// 	Tags::<T>::insert(bounded_tag, tag);
	
					// 	Ok(())
					// }
	
	
					pub fn do_validate_tag_data (
						category_type: CategoryType<T>,
						category_tag_list: BoundedVec<(CategoryId<T>, TagId<T>), T::MaxTags>,
					)-> Result<(), DispatchError> {
							
						for (category_id, tag_id) in category_tag_list {
							let tag_list = Categories::<T>::try_get((category_type.clone(), category_id.clone()));
							ensure!(tag_list.is_ok(), Error::<T>::NonexistentCategory);
							
							let tag_data = Tags::<T>::try_get((category_type.clone(), category_id), tag_id);
							ensure!(tag_data.is_ok(), Error::<T>::NonexistentTag);
						}
	
						Ok(())
					}
	
					pub fn do_update_tag_data (
						category_type: CategoryType<T>,
						category_tag_list: BoundedVec<(CategoryId<T>, TagId<T>), T::MaxTags>,
						content_id: BoundedVec<u8, T::ContentStringLimit>,
					)-> Result<(), DispatchError> {
							
						for (category_id, tag_id) in category_tag_list {
	
							Tags::<T>::try_mutate_exists(
							(category_type.clone(), category_id), 
							tag_id, |content_with_tag| -> DispatchResult {
								
								let tag_content_data = content_with_tag.as_mut().ok_or(Error::<T>::BadMetadata)?;
								tag_content_data.content_with_tag.try_push(content_id.clone()).unwrap();
	
								Ok(())
							})?;
	
	
	
						}
	
						Ok(())
					}
	
	
	
	
					
				
			}
	
	
	}