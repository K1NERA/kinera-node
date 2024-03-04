//** About **//
	// This pallet tracks all information regarding movie entries.
    // The movies it contains can be either internal (uploaded to a storage platform associated with the network)
    // and external (content that externally sourced).
    
    //TODO-0 merge internal and external storages into a single storage. for example through a doublemap
    //TODO-1 add checks to see if the movies still exist.


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
                    BoundedVec,
                    traits::{
                        Currency, 
                        ReservableCurrency,
                    }
                };
                use frame_system::pallet_prelude::*;
                use codec::{Decode, Encode, MaxEncodedLen};
                use sp_runtime::{RuntimeDebug, traits::{AtLeast32BitUnsigned, CheckedAdd, One}};
                use scale_info::{TypeInfo};
                use scale_info::prelude::vec::Vec;
                use core::convert::TryInto;
    
                use kine_tags::{
                    CategoryId as CategoryId,
                    TagId as TagId,
                };
                // use kine_stat_tracker::*;
    
    
        
            //* Config *//
            
                #[pallet::pallet]
                pub struct Pallet<T>(_);
    
                #[pallet::config]
                pub trait Config: frame_system::Config + kine_tags::Config + kine_stat_tracker::Config {
                // pub trait Config: frame_system::Config + pallet_tags::Config + pallet_stat_tracker::Config {
                    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
                    type InternalMovieId: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
                    
                    #[pallet::constant]
                    type StringLimit: Get<u32>;
                    type LinkStringLimit: Get<u32>;
    
                    type MovieCollateral: Get<u32>;
                }
    
        
    
        //** Types **//	
        
            //* Types *//
                type BalanceOf<T> = <<T as kine_stat_tracker::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    
            //* Constants *//
            //* Enums *//
    
                #[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
                pub enum ExternalSource {
                    Youtube,
                    Other,
                }
    
            //* Structs *//
    
                #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen,TypeInfo)]
                pub struct Movie<AccountId,BoundedString, BoundedLinkString, CategoryTagList> {
                    pub	uploader: AccountId,
                    pub name:BoundedString,
                    pub synopsis:BoundedString,
                    pub movie_description:BoundedString,
                    pub classification:u32,
                    pub release:BoundedString,
                    pub director:BoundedString,
                    pub lang:BoundedString,
                    pub country:BoundedString,
                    pub rating:u32,
                    pub aspect_ratio:BoundedString,
                    pub trailer:BoundedString,
                    pub imdb:BoundedString,
                    pub social:BoundedString,
                    pub ipfs:BoundedLinkString,
                    pub link:BoundedString,
                    pub categories_and_tags: CategoryTagList,
                }
                
                #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen,TypeInfo)]
                pub struct ExternalMovie<AccountId, ExternalSource, CategoryTagList> {
                    pub uploader: AccountId,
                    pub source: ExternalSource,
                    pub categories_and_tags: CategoryTagList,
                }
    
    
    
        //** Storage **//
    
            //* Internal Movies *//
            
            #[pallet::storage]
            #[pallet::getter(fn next_internal_movie_id)]
            pub(super) type NextInternalMovieId<T: Config> = StorageValue<
                _, 
                T::InternalMovieId, ValueQuery
            >;
    
            #[pallet::storage]
            #[pallet::getter(fn internal_movies)]
            pub type InternalMovies<T: Config> = StorageMap<
                _, 
                Blake2_128Concat, BoundedVec<u8, T::LinkStringLimit>, 
                Movie<
                    T::AccountId,
                    BoundedVec<u8, T::StringLimit>,
                    BoundedVec<u8, T::LinkStringLimit>,
                    BoundedVec<(CategoryId<T>, TagId<T>), T::MaxTags>,
                >
            >;
    
    
            //* External Movies *//
    
            #[pallet::storage]
            #[pallet::getter(fn external_movies)]
            pub type ExternalMovies<T: Config> = StorageMap<
                _, 
                Blake2_128Concat, BoundedVec<u8, T::LinkStringLimit>,
                ExternalMovie<
                    T::AccountId,
                    ExternalSource,
                    BoundedVec<(CategoryId<T>, TagId<T>), T::MaxTags>,
                >
            >;
    
    
    
    
    
    
        //** Events **//
    
            #[pallet::event]
            #[pallet::generate_deposit(pub(super) fn deposit_event)]
            pub enum Event<T: Config> {
                InternalMovieCreated(BoundedVec<u8, T::LinkStringLimit>, T::AccountId),
                ExternalMovieCreated(BoundedVec<u8, T::LinkStringLimit>, T::AccountId),
            }
       
    
    
        //** Errors **//
    
            #[pallet::error]
            pub enum Error<T> {
                NoAvailableMovieId,
                Overflow,
                Underflow,
                BadMetadata,
                WalletStatsRegistryRequired,
            }
    
    
    
        //** Hooks **//
    
            #[pallet::hooks]
            impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
    
    
        //** Extrinsics **//
    
            #[pallet::call]
            impl<T: Config> Pallet<T> {
                
                #[pallet::call_index(0)]#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
                pub fn create_internal_movie(
                    origin: OriginFor<T>,
                    name:Vec<u8>,
                    synopsis:Vec<u8>,
                    movie_description:Vec<u8>,
                    classification:u32,
                    release:Vec<u8>,
                    director:Vec<u8>,
                    lang:Vec<u8>,
                    country:Vec<u8>,
                    rating:u32,
                    aspect_ratio:Vec<u8>,
                    trailer:Vec<u8>,
                    imdb:Vec<u8>,
                    social:Vec<u8>,
                    ipfs:Vec<u8>,
                    link:Vec<u8>,
                    category_tag_list: BoundedVec<(CategoryId<T>, TagId<T>), T::MaxTags>,
                ) -> DispatchResultWithPostInfo {
                    let who = ensure_signed(origin)?;
                    // ensure!(
                    // 	pallet_stat_tracker::Pallet::<T>::is_wallet_registered(who.clone())?,
                    // 	Error::<T>::WalletStatsRegistryRequired,
                    // );
    
                    T::Currency::reserve(
                        &who, 
                        BalanceOf::<T>::from(T::MovieCollateral::get())
                    );
                    kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
                        who.clone(), 
                        kine_stat_tracker::FeatureType::Movie,
                        kine_stat_tracker::TokenType::Locked,
                        BalanceOf::<T>::from(T::MovieCollateral::get()), false
                    )?;
                    
                    Self::do_create_internal_movie(
                        &who, name,synopsis, movie_description,
                        classification, release, director, lang,
                        country, rating, aspect_ratio, trailer,
                        imdb, social, ipfs, link, category_tag_list,
                    )?;
    
                    Ok(().into())
                }
                
                #[pallet::call_index(1)]#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
                pub fn create_external_movie(
                    origin: OriginFor<T>,
                    source: ExternalSource,
                    link:BoundedVec<u8, T::LinkStringLimit>,
                    category_tag_list: BoundedVec<(CategoryId<T>, TagId<T>), T::MaxTags>,
                ) -> DispatchResultWithPostInfo {
                    
                    let who = ensure_signed(origin)?;
                    // ensure!(
                    // 	pallet_stat_tracker::Pallet::<T>::is_wallet_registered(who.clone())?,
                    // 	Error::<T>::WalletStatsRegistryRequired,
                    // );
    
                    T::Currency::reserve(
                        &who, 
                        BalanceOf::<T>::from(T::MovieCollateral::get())
                    );
                    kine_stat_tracker::Pallet::<T>::do_update_wallet_tokens(
                        who.clone(), 
                        kine_stat_tracker::FeatureType::Movie,
                        kine_stat_tracker::TokenType::Locked,
                        BalanceOf::<T>::from(T::MovieCollateral::get()), false
                    )?;
    
                    Self::do_create_external_movie(&who, source, link, category_tag_list)?;
    
                    Ok(().into())
                }
    
            }
    
    
        //** Helpers **//
    
            impl<T: Config> Pallet<T> {
    
                pub fn do_create_internal_movie(
                    who: &T::AccountId,
                    name:Vec<u8>,
                    synopsis:Vec<u8>,
                    movie_description:Vec<u8>,
                    classification:u32,
                    release:Vec<u8>,
                    director:Vec<u8>,
                    lang:Vec<u8>,
                    country:Vec<u8>,
                    rating:u32,
                    aspect_ratio:Vec<u8>,
                    trailer:Vec<u8>,
                    imdb:Vec<u8>,
                    social:Vec<u8>,
                    ipfs:Vec<u8>,
                    link:Vec<u8>,
                    category_tag_list: BoundedVec<(CategoryId<T>, TagId<T>), T::MaxTags>,
                ) -> Result<T::InternalMovieId, DispatchError> {
     
                    let movie_id =
                        NextInternalMovieId::<T>::try_mutate(|id| -> Result<T::InternalMovieId, DispatchError> {
                            let current_id = *id;
                            *id = id
                                .checked_add(&One::one())
                                .ok_or(Error::<T>::Overflow)?;
                            // let bounded_id: BoundedVec<u8, T::LinkStringLimit> = TryInto::try_into(current_id).map_err(|_| Error::<T>::BadMetadata)?;
                            Ok(current_id)
                        })?;
            
                    let category_type: kine_tags::CategoryType<T>
                        = TryInto::try_into("Movie".as_bytes().to_vec()).map_err(|_|Error::<T>::BadMetadata)?;
                    
                    kine_tags::Pallet::<T>::do_validate_tag_data(
                        category_type.clone(), 
                        category_tag_list.clone()
                    )?;
                    
                    let bounded_name: BoundedVec<u8, T::StringLimit> = TryInto::try_into(name).map_err(|_| Error::<T>::BadMetadata)?;
                    
                    let bounded_synopsis: BoundedVec<u8, T::StringLimit> =
                        TryInto::try_into(synopsis).map_err(|_|Error::<T>::BadMetadata)?;
                    
                    let bounded_movie_description: BoundedVec<u8, T::StringLimit> =
                        TryInto::try_into(movie_description).map_err(|_| Error::<T>::BadMetadata)?;
                    
                    let bounded_release: BoundedVec<u8, T::StringLimit> =
                        TryInto::try_into(release).map_err(|_| Error::<T>::BadMetadata)?;
    
                    let bounded_director: BoundedVec<u8, T::StringLimit> =
                        TryInto::try_into(director).map_err(|_| Error::<T>::BadMetadata)?;
                    
                    let bounded_lang: BoundedVec<u8, T::StringLimit> =
                        TryInto::try_into(lang).map_err(|_| Error::<T>::BadMetadata)?;
    
                    let bounded_country: BoundedVec<u8, T::StringLimit> =
                        TryInto::try_into(country).map_err(|_| Error::<T>::BadMetadata)?;
                    let bounded_aspect_ratio: BoundedVec<u8, T::StringLimit> =
                        TryInto::try_into(aspect_ratio).map_err(|_| Error::<T>::BadMetadata)?;
    
                    let bounded_trailer: BoundedVec<u8, T::StringLimit> =
                        TryInto::try_into(trailer).map_err(|_|Error::<T>::BadMetadata)?;
                    
                    let bounded_imdb: BoundedVec<u8, T::StringLimit> =
                        TryInto::try_into(imdb).map_err(|_|Error::<T>::BadMetadata)?;
    
                    let bounded_social: BoundedVec<u8, T::StringLimit> =
                        TryInto::try_into(social).map_err(|_|Error::<T>::BadMetadata)?;
    
                    let bounded_link: BoundedVec<u8, T::StringLimit> =
                        TryInto::try_into(link).map_err(|_|Error::<T>::BadMetadata)?;
    
                    let bounded_ipfs: BoundedVec<u8, T::LinkStringLimit> =
                    TryInto::try_into(ipfs).map_err(|_|Error::<T>::BadMetadata)?;
                    
                    let movie = Movie {
                        uploader:who.clone(),
                        name:bounded_name,
                        synopsis:bounded_synopsis,
                        movie_description:bounded_movie_description,
                        classification:classification,
                        release:bounded_release,
                        director:bounded_director,
                        lang:bounded_lang,
                        country:bounded_country,
                        rating:rating,
                        aspect_ratio:bounded_aspect_ratio,
                        trailer:bounded_trailer,
                        imdb:bounded_imdb,
                        social:bounded_social,
                        ipfs:bounded_ipfs,
                        link:bounded_link,
                        categories_and_tags: category_tag_list.clone(),
                    };
            
                    // parse the u32 type into a BoundedVec<u8, T::StringLimit>
                    let encoded: Vec<u8> = movie_id.encode();
                    let bounded_movie_id: BoundedVec<u8, T::LinkStringLimit> = 
                        TryInto::try_into(encoded.clone()).map_err(|_|Error::<T>::BadMetadata)?;
    
                    InternalMovies::<T>::insert(bounded_movie_id.clone(), movie.clone());
    
                    // parse the u32 type into a BoundedVec<u8, T::ContentStringLimit
                    let bounded_content_id: BoundedVec<u8, T::ContentStringLimit> = 
                        TryInto::try_into(encoded).map_err(|_|Error::<T>::BadMetadata)?;
    
                    kine_tags::Pallet::<T>::do_update_tag_data(
                        category_type, 
                        category_tag_list,
                        bounded_content_id,
                    )?;
            
                    Self::deposit_event(Event::InternalMovieCreated(bounded_movie_id, who.clone()));
                    Ok(movie_id.clone())
                } 
            
    
                pub fn do_create_external_movie(
                    who: &T::AccountId,
                    source: ExternalSource,
                    link: BoundedVec<u8, T::LinkStringLimit>,
                    category_tag_list: BoundedVec<(CategoryId<T>, TagId<T>), T::MaxTags>,
                ) -> Result<BoundedVec<u8, T::LinkStringLimit>, DispatchError> {
            
                    Self::do_ensure_external_movie_doesnt_exist(link.clone()).unwrap();
    
                    let category_type: kine_tags::CategoryType<T>
                        = TryInto::try_into("Movie".as_bytes().to_vec()).map_err(|_|Error::<T>::BadMetadata)?;
                
                    kine_tags::Pallet::<T>::do_validate_tag_data(
                        category_type.clone(), 
                        category_tag_list.clone()
                    )?;
    
                    let movie = ExternalMovie {
                        uploader:who.clone(),
                        source: source,
                        categories_and_tags: category_tag_list,
                    };
                
                    ExternalMovies::<T>::insert(link.clone(), movie.clone());
            
                    Self::deposit_event(Event::ExternalMovieCreated(link.clone(), who.clone()));
                    Ok(link)
                } 
            
    
                                        
    
                pub fn do_ensure_internal_movie_exist(
                    movie_id : BoundedVec<u8, T::LinkStringLimit>,
                ) -> DispatchResultWithPostInfo {
        
                    ensure!(InternalMovies::<T>::contains_key(movie_id), Error::<T>::NoAvailableMovieId); 
                    Ok(().into())
                }
    
                pub fn do_does_internal_movie_exist(
                    movie_id : BoundedVec<u8, T::LinkStringLimit>,
                ) -> Result<bool, DispatchError> {
        
                    Ok(InternalMovies::<T>::contains_key(movie_id))
                }
    
    
    
    
                pub fn do_does_external_movie_exist(
                    movie_id : BoundedVec<u8, T::LinkStringLimit>,
                ) -> Result<bool, DispatchError> {
        
                    Ok(ExternalMovies::<T>::contains_key(movie_id))
                }
            
                pub fn do_ensure_external_movie_doesnt_exist(
                    movie_id : BoundedVec<u8, T::LinkStringLimit>,
                ) -> DispatchResultWithPostInfo {
        
                    ensure!(!ExternalMovies::<T>::contains_key(movie_id), Error::<T>::NoAvailableMovieId); 
                    Ok(().into())
                }
            
                pub fn do_ensure_external_movie_exists(
                    movie_id : BoundedVec<u8, T::LinkStringLimit>,
                ) -> DispatchResultWithPostInfo {
        
                    ensure!(ExternalMovies::<T>::contains_key(movie_id), Error::<T>::NoAvailableMovieId); 
                    Ok(().into())
                }
    
                pub fn get_movie_uploader(
                    movie_id : BoundedVec<u8, T::LinkStringLimit>,
                ) -> Result<T::AccountId, DispatchError> {
                    
                    let mut uploader;
                    if InternalMovies::<T>::contains_key(movie_id.clone()) {
                        uploader = InternalMovies::<T>::get(movie_id).unwrap().uploader;
                    }
                    else {
                        uploader = ExternalMovies::<T>::try_get(movie_id).unwrap().uploader;
                    }
                    
                    Ok(uploader)
                }
    
    
            }
    }