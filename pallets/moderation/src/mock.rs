use crate as kine_moderation;
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
		ModerationModule: kine_moderation,
		TagsModule: kine_tags,
		StatTrackerModule: kine_stat_tracker,
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



// Moderation
parameter_types! {
	pub const ReportJustificationLimit: u32 = 250;
	pub const MaxReportsByModerator: u32 = 3;
	pub const TotalTierOneModerators: u32 = 3;
	pub const MaxReportsByTier: u32 = 23;
	pub const MinimumReputationForSeniorship: u32 = 30;
	pub const MinimumReputationForModeration: u32 = 10;
	pub const MinimumTokensForModeration: u32 = 10000;
	pub const PalletModerationId: PalletId = PalletId(*b"ModStash");
	pub const MovieCollateral: u32 = 3000;
}

impl kine_moderation::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    
	type JustificationLimit = ReportJustificationLimit;
    
	type ContentId = u32;
	type MaxReportsByModerator = MaxReportsByModerator;
	type TotalTierOneModerators = TotalTierOneModerators;
	type MaxReportsByTier = MaxReportsByTier;
	
	type MinimumReputationForSeniorship = MinimumReputationForSeniorship;
	type MinimumReputationForModeration = MinimumReputationForModeration;
	type MinimumTokensForModeration = MinimumTokensForModeration;
	type MovieCollateral = MovieCollateral;

	type PalletId = PalletModerationId;
}




// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}
