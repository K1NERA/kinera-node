use crate as kine_ranking_list;
use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU32, ConstU64, ConstU128},
	PalletId,
};
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
	testing::Header,
};

type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u128;


// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test 
	{
		System: frame_system,
		Balances: pallet_balances,
		MovieModule: kine_movie,
		TagsModule: kine_tags,
		StatTrackerModule: kine_stat_tracker,
		RankingListModule: kine_ranking_list,
	}
);


// System
impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData =  pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}



// Tags
parameter_types! {
	pub const MaxTags: u32 = 10000;
	pub const TagStringLimit: u32 = 100;
	pub const ContentStringLimit: u32 = 100000;
	pub const CategoryStringLimit: u32 = 100;
	pub const MaxContentWithTag: u32 = 100000;
}

impl kine_tags::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MaxTags = MaxTags;
	type MaxContentWithTag = MaxContentWithTag;
	type ContentStringLimit = ContentStringLimit;
	type TagStringLimit = TagStringLimit;
	type CategoryStringLimit = CategoryStringLimit;
}

// Stat Tracker
parameter_types! {
	pub const DefaultReputation: u32 = 15;
	pub const WalletNameStringLimit: u32 = 50;
	pub const PalletStatTrackerId : PalletId = PalletId(*b"kine/trk");
}

impl kine_stat_tracker::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type DefaultReputation = DefaultReputation;
	type NameStringLimit = WalletNameStringLimit;
	type PalletId = PalletStatTrackerId;
}


// Movie
parameter_types! {
	pub const MovieStringLimit: u32 = 50;
	pub const LinkStringLimit: u32 = 10000;
	pub const MovieCollateral: u32 = 3000;
}

impl kine_movie::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type InternalMovieId = u32;
    type StringLimit = MovieStringLimit;
    type LinkStringLimit = LinkStringLimit;
    type MovieCollateral = MovieCollateral;
}


// Pallet Balances
parameter_types! {
	pub const ExistentialDeposit: u128 = 500;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type MaxHolds = ();
}


// Ranking List
parameter_types!{
    pub const PalletRankingListId : PalletId = PalletId(*b"kine/rnk");
	pub const RankingStringLimit: u32 = 50;
	pub const MaxMoviesInList: u32 = 100000;
	pub const MinimumListDuration: u32 = 3600; // six hours in blocks
	pub const MaxVotersPerList: u32 = 10000000;
	pub const MaxListsPerBlock: u32 = 50;
}

impl kine_ranking_list::Config for Test {
    type RuntimeEvent = RuntimeEvent;
	type MaxListsPerBlock = MaxListsPerBlock;
	type MaxVotersPerList = MaxVotersPerList;
	type MaxMoviesInList = MaxMoviesInList;
	type MinimumListDuration = MinimumListDuration;
    type RankingStringLimit = RankingStringLimit;
    type PalletId = PalletRankingListId;
}





// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}
