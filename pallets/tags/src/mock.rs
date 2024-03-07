use crate as kine_tags;
use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU64},
};
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
	testing::Header,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test 
	{
		System: frame_system,
		TagsModule: kine_tags,
	}
);

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
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}


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

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}