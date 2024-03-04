use crate as pallet_moderation;
use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU32, ConstU64},
	PalletId,
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		ModerationModule: pallet_moderation::{Pallet, Call, Storage, Event<T>},
		TagsModule: pallet_tags::{Pallet, Call, Storage, Event<T>},
		StatTrackerModule: pallet_stat_tracker::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Config<T>, Storage, Event<T>},
	}
);

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}


parameter_types! {
	// moderation
	pub const ReportJustificationLimit: u32 = 250;
	pub const MaxReportsByModerator: u32 = 3;
	pub const TotalTierOneModerators: u32 = 3;
	pub const MaxReportsByTier: u32 = 23;
	pub const MinimumTokensForModeration: u32 = 100;
	pub const MovieCollateral: u32 = 30;
	pub const PalletModerationId: PalletId = PalletId(*b"ModStash");

	// tags
	pub const MaxTags: u32 = 10000;
	pub const TagStringLimit: u32 = 100;
	pub const ContentStringLimit: u32 = 100000;
	pub const CategoryStringLimit: u32 = 100;
	pub const MaxContentWithTag: u32 = 100000;

	// stat tracker
	pub const DefaultReputation: u32 = 100;
	
	//balances
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_moderation::Config for Test {
    type Event = Event;
    
	type JustificationLimit = ReportJustificationLimit;
    
	type ContentId = u32;
	type MaxReportsByModerator = MaxReportsByModerator;
	type TotalTierOneModerators = TotalTierOneModerators;
	type MaxReportsByTier = MaxReportsByTier;
	
	type MinimumTokensForModeration = MinimumTokensForModeration;
	type MovieCollateral = MovieCollateral;

	type Currency = Balances;
	type PalletId = PalletModerationId;
}

impl pallet_tags::Config for Test {
    type Event = Event;

	type MaxTags = MaxTags;
	type MaxContentWithTag = MaxContentWithTag;

	type ContentStringLimit = ContentStringLimit;
	type TagStringLimit = TagStringLimit;
	type CategoryStringLimit = CategoryStringLimit;
}


impl pallet_stat_tracker::Config for Test {
    type Event = Event;
	type DefaultReputation = DefaultReputation;
}


impl pallet_balances::Config for Test {
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = u64;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}


// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
