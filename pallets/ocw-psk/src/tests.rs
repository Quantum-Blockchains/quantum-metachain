use frame_support::{
    parameter_types,
    traits::{ConstU32, ConstU64},
};
use frame_support_test::TestRandomness;
use sp_core::{sr25519::Signature, H256};
use sp_runtime::{
    generic::BlockId::Hash,
    testing::Header,
    traits::{BlakeTwo256, ConstU128, IdentityLookup, Verify},
};

use crate as ocw_psk;
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
        Randao: randao::{Pallet, Call, Event<T>},
        OcwPsk: ocw_psk::{Pallet, Call, Event<T>},
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

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Test
where
    Call: From<LocalCall>,
{
    type Extrinsic = UncheckedExtrinsic;
    type OverarchingCall = Call;
}

impl randao::Config for Test {
    type Event = Event;
    type Call = Call;
}

impl Config for Test {
    type Event = Event;
    type Call = Call;
    type Randomness = TestRandomness<Self>;
    type PskDifficulty1 = ConstU128<1000000u128>;
    type PskDifficulty2 = ConstU128<{ u128::MAX }>;
}

#[test]
fn psk_creator_is_chosen_when_one_peer_pass_the_difficulty() {
    sp_io::TestExternalities::default().execute_with(|| {
        let entropy = H256::from_low_u64_be(1298474330019282);
        let peers = vec![
            "12D3KooWQijTyPBAQcqZeSD1fh3Ep8iW6ZAogEwUwcAKgSouyusV".to_string(),
            "12D3KooWHg3Xq65A8MpywPGsTgLhHQqfo9kBhibXouSzgJzCmhic".to_string(),
        ];

        let result = OcwPsk::choose_psk_creator(entropy, peers);

        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            "12D3KooWHg3Xq65A8MpywPGsTgLhHQqfo9kBhibXouSzgJzCmhic".to_string()
        );
    });
}

#[test]
fn creator_is_not_chosen_when_2_peers_pass_the_difficulty() {
    sp_io::TestExternalities::default().execute_with(|| {
        let entropy = H256::from_low_u64_be(1298474330019282);
        let peers = vec![
            "12D3KooWEh8KPSuGWdSNivtffFQEy1WziYdrtQXpktjPfHqzr5rp".to_string(),
            "12D3KooWQijTyPBAQcqZeSD1fh3Ep8iW6ZAogEwUwcAKgSouyusV".to_string(),
            "12D3KooWHg3Xq65A8MpywPGsTgLhHQqfo9kBhibXouSzgJzCmhic".to_string(),
        ];

        let result = OcwPsk::choose_psk_creator(entropy, peers);

        // In this case 2 of the peers are qualified to be psk creator,
        // therefore result is ignored and None is returned
        assert!(result.is_none());
    });
}

#[test]
fn creator_is_not_chosen_when_because_none_of_them_pass_the_difficulty() {
    sp_io::TestExternalities::default().execute_with(|| {
        let entropy = H256::from_low_u64_be(1298474330019282);
        let peers = vec!["12D3KooWQijTyPBAQcqZeSD1fh3Ep8iW6ZAogEwUwcAKgSouyusV".to_string()];

        let result = OcwPsk::choose_psk_creator(entropy, peers);

        // In this case peer didn't pass the difficulty,
        // therefore result is ignored and None is returned
        assert!(result.is_none());
    });
}
