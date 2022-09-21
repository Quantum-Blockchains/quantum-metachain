use frame_support::{
    parameter_types,
    traits::{ConstU32, ConstU64},
};
use frame_support_test::TestRandomness;
use sp_core::{sr25519::Signature, H256};
use sp_std::collections::btree_map::BTreeMap;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup, Verify},
};

use crate as ocw_qkd;
use crate::*;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        OcwQkd: ocw_qkd::{Pallet, Call, Event<T>},
    }
);

parameter_types! {
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(1024);
}

impl frame_system::Config for Test {
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
    type AccountId = sp_core::sr25519::Public;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

impl frame_system::offchain::SigningTypes for Test {
    type Public = <Signature as Verify>::Signer;
    type Signature = Signature;
}

impl Config for Test {
    type Event = Event;
    type Call = Call;
    type Randomness = TestRandomness<Self>;
    type TargetKeysAmount = ConstU32<5>;
}

#[test]
fn should_generate_required_num_of_keys() {
    sp_io::TestExternalities::default().execute_with(|| {
        let storage = &mut <BTreeMap<u8, [u8; 32]>>::default();
        let keys_len_before = OcwQkd::get_node_keys_len(storage);

        OcwQkd::generate_keys(storage, 5).unwrap();

        let keys_len_after = OcwQkd::get_node_keys_len(storage);
        assert_eq!(keys_len_before, 0);
        assert_eq!(keys_len_after, 5);
    });
}

#[test]
fn should_properly_calculate_amount_of_keysto_generate() {
    sp_io::TestExternalities::default().execute_with(|| {
        let storage = &mut <BTreeMap<u8, [u8; 32]>>::default();
        let amount_to_generate_before = OcwQkd::calculate_amount_to_generate(storage);

        OcwQkd::generate_keys(storage, 1).unwrap();

        let amount_to_generate_after = OcwQkd::calculate_amount_to_generate(storage);
        assert_eq!(amount_to_generate_before, 5);
        assert_eq!(amount_to_generate_after, 4);
    });
}
