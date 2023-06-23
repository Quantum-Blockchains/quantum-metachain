use frame_support::{
    assert_err, assert_ok, parameter_types,
    traits::{ConstU32, ConstU64},
};
use sp_core::{sr25519::Signature, H256};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup, Verify},
    BuildStorage,
};

use crate as randao;
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

impl Config for Test {
    type Event = Event;
    type Call = Call;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = GenesisConfig {
        system: Default::default(),
    }
    .build_storage()
    .unwrap();
    t.into()
}

#[test]
fn should_create_a_new_campaign_if_the_data_transmitted_meets_the_requirements() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Randao::create_new_campaign(
            11,
            COMMIT_BALKLINE,
            COMMIT_DEADLINE
        ));
        assert!(Campaigns::<Test>::contains_key(11));
        System::set_block_number(345);
        assert_ok!(Randao::create_new_campaign(
            355,
            COMMIT_BALKLINE,
            COMMIT_DEADLINE
        ));
        assert!(Campaigns::<Test>::contains_key(355));
        System::set_block_number(87);
        assert_ok!(Randao::create_new_campaign(
            97,
            COMMIT_BALKLINE,
            COMMIT_DEADLINE
        ));
        assert!(Campaigns::<Test>::contains_key(97));
    });
}

#[test]
fn should_return_an_error_if_a_compony_for_a_certain_block_exists() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Randao::create_new_campaign(
            11,
            COMMIT_BALKLINE,
            COMMIT_DEADLINE
        ));
        assert_err!(
            Randao::create_new_campaign(11, COMMIT_BALKLINE, COMMIT_DEADLINE),
            Error::<Test>::CampaignIsAlreadyThere
        );
        System::set_block_number(345);
        assert_ok!(Randao::create_new_campaign(
            355,
            COMMIT_BALKLINE,
            COMMIT_DEADLINE
        ));
        assert_err!(
            Randao::create_new_campaign(355, COMMIT_BALKLINE, COMMIT_DEADLINE),
            Error::<Test>::CampaignIsAlreadyThere
        );
        System::set_block_number(87);
        assert_ok!(Randao::create_new_campaign(
            97,
            COMMIT_BALKLINE,
            COMMIT_DEADLINE
        ));
        assert_err!(
            Randao::create_new_campaign(97, COMMIT_BALKLINE, COMMIT_DEADLINE),
            Error::<Test>::CampaignIsAlreadyThere
        );
    });
}

#[test]
fn should_return_an_error_if_a_bad_timeline_was_specified_when_creating_a_new_company() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        assert_err!(
            Randao::create_new_campaign(11, 2, 10),
            Error::<Test>::TimeLineCheck
        );
        System::set_block_number(345);
        assert_err!(
            Randao::create_new_campaign(355, 4, 5),
            Error::<Test>::TimeLineCheck
        );
        System::set_block_number(87);
        assert_err!(
            Randao::create_new_campaign(97, 0, 0),
            Error::<Test>::TimeLineCheck
        );
    });
}

#[test]
fn should_commit_the_hash_sent_if_there_is_a_company_and_the_time_frame_is_met() {
    new_test_ext().execute_with(|| {
        let block_num: u64 = 11;
        let from: [u8; 52] = *b"12D3KooWD3eckifWpRn9wQpMG9R9hX3sD158z7EqHWmweQAJU5SA";
        let hash = Randao::hash_num(1298474330019282);

        System::set_block_number(1);
        assert_ok!(Randao::create_new_campaign(
            block_num,
            COMMIT_BALKLINE,
            COMMIT_DEADLINE
        ));
        System::set_block_number(3);
        assert_ok!(Randao::commit_hash(from, block_num, hash));
        assert!(ParticipantsOfCampaigns::<Test>::contains_key(
            block_num, from
        ));
    });
}

#[test]
fn should_return_an_error_if_the_participant_has_already_sent_a_hash_for_campaign() {
    new_test_ext().execute_with(|| {
        let block_num: u64 = 11;
        let from: [u8; 52] = *b"12D3KooWD3eckifWpRn9wQpMG9R9hX3sD158z7EqHWmweQAJU5SA";
        let hash = Randao::hash_num(1298474330019282);

        System::set_block_number(1);
        assert_ok!(Randao::create_new_campaign(
            block_num,
            COMMIT_BALKLINE,
            COMMIT_DEADLINE
        ));
        System::set_block_number(3);
        assert_ok!(Randao::commit_hash(from, block_num, hash));
        System::set_block_number(4);
        assert_err!(
            Randao::commit_hash(from, block_num, hash),
            Error::<Test>::ParticipantIsAlreadyThere
        );
    });
}

#[test]
fn should_return_an_error_if_the_participant_sends_a_hash_for_a_campaign_that_does_not_exist() {
    new_test_ext().execute_with(|| {
        let block_num: u64 = 11;
        let from: [u8; 52] = *b"12D3KooWD3eckifWpRn9wQpMG9R9hX3sD158z7EqHWmweQAJU5SA";
        let hash = Randao::hash_num(1298474330019282);

        System::set_block_number(1);
        assert_ok!(Randao::create_new_campaign(
            block_num,
            COMMIT_BALKLINE,
            COMMIT_DEADLINE
        ));
        System::set_block_number(2);
        assert_err!(
            Randao::commit_hash(from, 12, hash),
            Error::<Test>::IncorrectId
        );
        System::set_block_number(10);
        assert_err!(
            Randao::commit_hash(from, 345, hash),
            Error::<Test>::IncorrectId
        );
    });
}

#[test]
fn should_return_an_error_if_the_participant_sent_the_hash_within_the_time_frame_not_allowed() {
    new_test_ext().execute_with(|| {
        let block_num: u64 = 11;
        let from: [u8; 52] = *b"12D3KooWD3eckifWpRn9wQpMG9R9hX3sD158z7EqHWmweQAJU5SA";
        let hash = Randao::hash_num(1298474330019282);

        System::set_block_number(1);
        assert_ok!(Randao::create_new_campaign(
            block_num,
            COMMIT_BALKLINE,
            COMMIT_DEADLINE
        ));
        System::set_block_number(2);
        assert_err!(
            Randao::commit_hash(from, block_num, hash),
            Error::<Test>::TimeLineCommitPhase
        );
        System::set_block_number(10);
        assert_err!(
            Randao::commit_hash(from, block_num, hash),
            Error::<Test>::TimeLineCommitPhase
        );
    });
}

#[test]
fn should_return_error_if_the_participant_resends_a_hash_for_a_campaign() {
    new_test_ext().execute_with(|| {
        let block_num: u64 = 11;
        let from: [u8; 52] = *b"12D3KooWD3eckifWpRn9wQpMG9R9hX3sD158z7EqHWmweQAJU5SA";
        let hash = Randao::hash_num(1298474330019282);

        System::set_block_number(1);
        assert_ok!(Randao::create_new_campaign(
            block_num,
            COMMIT_BALKLINE,
            COMMIT_DEADLINE
        ));
        System::set_block_number(3);
        assert_ok!(Randao::commit_hash(from, block_num, hash));
        System::set_block_number(4);
        assert_err!(
            Randao::commit_hash(from, block_num, hash),
            Error::<Test>::ParticipantIsAlreadyThere
        );
    });
}

#[test]
fn should_return_reveal_a_secret_if_the_participant_followe_all_the_rules() {
    new_test_ext().execute_with(|| {
        let block_num: u64 = 11;
        let from: [u8; 52] = *b"12D3KooWD3eckifWpRn9wQpMG9R9hX3sD158z7EqHWmweQAJU5SA";
        let secret: u64 = 1298474330019282;
        let hash = Randao::hash_num(secret);

        System::set_block_number(1);
        assert_ok!(Randao::create_new_campaign(
            block_num,
            COMMIT_BALKLINE,
            COMMIT_DEADLINE
        ));
        System::set_block_number(3);
        assert_ok!(Randao::commit_hash(from, block_num, hash));
        System::set_block_number(9);
        assert_ok!(Randao::reveal_secret(from, block_num, secret));
    });
}

#[test]
fn should_return_an_error_if_the_participant_did_not_send_the_hash_earlier() {
    new_test_ext().execute_with(|| {
        let block_num: u64 = 11;
        let from: [u8; 52] = *b"12D3KooWD3eckifWpRn9wQpMG9R9hX3sD158z7EqHWmweQAJU5SA";
        let secret: u64 = 1298474330019282;

        System::set_block_number(1);
        assert_ok!(Randao::create_new_campaign(
            block_num,
            COMMIT_BALKLINE,
            COMMIT_DEADLINE
        ));
        System::set_block_number(9);
        assert_err!(
            Randao::reveal_secret(from, block_num, secret),
            Error::<Test>::IsNotAParticipant
        );
    });
}

#[test]
fn should_return_an_error_if_the_participant_sent_the_secret_within_the_time_frame_not_allowed() {
    new_test_ext().execute_with(|| {
        let block_num: u64 = 11;
        let from: [u8; 52] = *b"12D3KooWD3eckifWpRn9wQpMG9R9hX3sD158z7EqHWmweQAJU5SA";
        let secret: u64 = 1298474330019282;
        let hash = Randao::hash_num(secret);

        System::set_block_number(1);
        assert_ok!(Randao::create_new_campaign(
            block_num,
            COMMIT_BALKLINE,
            COMMIT_DEADLINE
        ));
        System::set_block_number(3);
        assert_ok!(Randao::commit_hash(from, block_num, hash));
        System::set_block_number(4);
        assert_err!(
            Randao::reveal_secret(from, block_num, secret),
            Error::<Test>::TimeLineRevealPhase
        );
        System::set_block_number(11);
        assert_err!(
            Randao::reveal_secret(from, block_num, secret),
            Error::<Test>::TimeLineRevealPhase
        );
    });
}

#[test]
fn should_return_an_error_if_the_participant_sent_a_secret_that_does_not_match_the_hash() {
    new_test_ext().execute_with(|| {
        let block_num: u64 = 11;
        let from: [u8; 52] = *b"12D3KooWD3eckifWpRn9wQpMG9R9hX3sD158z7EqHWmweQAJU5SA";
        let secret: u64 = 1298474330019282;
        let hash = Randao::hash_num(secret);

        System::set_block_number(1);
        assert_ok!(Randao::create_new_campaign(
            block_num,
            COMMIT_BALKLINE,
            COMMIT_DEADLINE
        ));
        System::set_block_number(3);
        assert_ok!(Randao::commit_hash(from, block_num, hash));
        System::set_block_number(8);
        assert_err!(
            Randao::reveal_secret(from, block_num, 5462),
            Error::<Test>::SecretDoesNotMatchTheHash
        );
    });
}

#[test]
fn should_return_secret() {
    new_test_ext().execute_with(|| {
        let block_num: u64 = 11;
        let from_1: [u8; 52] = *b"12D3KooWD3eckifWpRn9wQpMG9R9hX3sD158z7EqHWmweQAJU5SA";
        let secret_1: u64 = 1298474330019282;
        let hash_1 = Randao::hash_num(secret_1);
        let from_2: [u8; 52] = *b"12D3KooWD3eckifWpRn9wQpMG9R9hX3sD158z7EqHWmweQAJU5QA";
        let secret_2: u64 = 9876543;
        let hash_2 = Randao::hash_num(secret_2);
        let secret: u64 = 0 ^ secret_1 ^ secret_2;

        System::set_block_number(1);
        assert_ok!(Randao::create_new_campaign(
            block_num,
            COMMIT_BALKLINE,
            COMMIT_DEADLINE
        ));
        System::set_block_number(3);
        assert_ok!(Randao::commit_hash(from_1, block_num, hash_1));
        assert_ok!(Randao::commit_hash(from_2, block_num, hash_2));
        System::set_block_number(8);
        assert_ok!(Randao::reveal_secret(from_1, block_num, secret_1));
        assert_ok!(Randao::reveal_secret(from_2, block_num, secret_2));

        System::set_block_number(11);
        assert_eq!(Randao::get_secret(11), Ok(secret));
    });
}
