#![cfg_attr(not(feature = "std"), no_std)]

use std::ops::Add;
use frame_support::dispatch::DispatchResult;
use frame_support::pallet_prelude::DispatchError;
use frame_support::ensure;
pub use pallet::*;
use frame_support::traits::Currency;
use frame_support::pallet_prelude::{
    MaxEncodedLen, RuntimeDebug, StorageMap, TypeInfo, StorageDoubleMap
};
use sp_core::{Decode, Encode, Hasher};
use sp_std::{str, vec::Vec};


const NUM_BLOCK_FOR_CAMPAIGN: u64 = 10;
const COMMIT_BALKLINE: u64 = 2;
const COMMIT_DEADLINE: u64 = 6;

/// This is a structure that is equivalent to a random number generation participant.
/// secret - secret number
/// commitment - hash of secret number
#[derive(Clone, Encode, Decode, PartialEq, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Participant {
    pub secret: u64,
    pub commitment: [u8; 32],
}

#[derive(Clone, Encode, Decode, PartialEq, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Campaign {
    pub id: u64,
    pub secret: u64,
    pub block_num: u64,
    pub commit_balkline: u64,
    pub commit_deadline: u64,
}

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet;
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
	pub(super) type CampaignsCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
	pub(super) type CurrentBlock<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    pub(super) type Campaigns<T: Config> = StorageMap<_, Twox64Concat, u64, Campaign>;

    #[pallet::storage]
    pub(super) type ParticipantsOfCampaigns<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        u64,
        Twox64Concat,
        T::AccountId,
        Participant,
    >;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
    where
        u64: From<<T as frame_system::Config>::BlockNumber>,
    {
        fn offchain_worker(block_number: T::BlockNumber) {
            let current_block_number: u64 = block_number.into();
            CurrentBlock::<T>::put(current_block_number);
            Self::create_new_campaign(current_block_number + NUM_BLOCK_FOR_CAMPAIGN, COMMIT_BALKLINE, COMMIT_DEADLINE).expect("TODO: panic message");
        }
    }

    #[pallet::error]
    pub enum Error<T> {
        TimeLineCheck,
        IncorrectId,
        ParticipantIsAlreadyThere,
        IsNotAParticipant,
        SecretDoesNotMatchTheHash,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        LogCampaignAdded {
            capaign_id: u64,
            block_num: u64,
            commit_balkline: u64,
            commit_deadline: u64
        },
        LogCommit { from: T::AccountId, commitment: [u8; 32] },
        LogReval { from: T::AccountId, secret: u64 },
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Call: From<Call<Self>>;
        type Currency: Currency<Self::AccountId>;

    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {

        #[pallet::weight(0)]
        pub fn commit(origin: OriginFor<T>, capaign_id: u64, hash: [u8; 32]) -> DispatchResult {
            // TODO Check if the hash is sent within the allowed time
			let sender = ensure_signed(origin)?;
			Self::commit_hash(&sender, capaign_id, hash)
		}

        #[pallet::weight(0)]
        pub fn reval(origin: OriginFor<T>, capaign_id: u64, secret: u64) -> DispatchResult {
            // TODO Check if the hash is sent within the allowed time
            let sender = ensure_signed(origin)?;
            Self::reval_secret(&sender, capaign_id, secret)
        }
    }
}

impl<T: Config> Pallet<T> {

    pub fn create_new_campaign(
        block_num: u64,
        commit_balkline: u64,
        commit_deadline: u64,
    ) -> Result<u64, DispatchError> {
        let current_block_num: u64 = CurrentBlock::<T>::get();

        ensure!(
            current_block_num < block_num,
            Error::<T>::TimeLineCheck
        );
        ensure!(
            commit_deadline < commit_balkline,
            Error::<T>::TimeLineCheck
        );
        ensure!(
            current_block_num < block_num - commit_balkline,
            Error::<T>::TimeLineCheck
        );

        let count = CampaignsCount::<T>::get();
        let new_count = count + 1;

        let new_campaign = Campaign {
            id: count,
            secret: 0,
            block_num: block_num,
            commit_balkline: commit_balkline,
            commit_deadline: commit_deadline,
        };

        Campaigns::<T>::insert(count, new_campaign);

        CampaignsCount::<T>::put(new_count);

        Self::deposit_event(Event::LogCampaignAdded {
            capaign_id: count,
            block_num: block_num,
            commit_balkline: commit_balkline,
            commit_deadline: commit_deadline,
        });
        log::info!("[RANDAO] Created new campaign {:?} for block number {:?}", count, block_num);

        Ok(count)
    }

    pub fn commit_hash(
        from: &T::AccountId,
        campaign_id: u64,
        commitment: [u8; 32],
    ) -> Result<(), DispatchError> {
        ensure!(
            !Campaigns::<T>::contains_key(campaign_id),
            Error::<T>::IncorrectId
        );

        ensure!(
            !ParticipantsOfCampaigns::<T>::contains_key(campaign_id, &from),
            Error::<T>::ParticipantIsAlreadyThere
        );

        let new_participant = Participant{
            secret: 0,
            commitment: commitment
        };

        ParticipantsOfCampaigns::<T>::insert(campaign_id, &from, new_participant);
        Self::deposit_event(Event::LogCommit {
            from: from.clone(),
            commitment: commitment,
        });
        log::info!("[RANDAO] The account with the ID {:?} did commit for campaign {:?}", from, campaign_id);
        Ok(())
    }

    pub fn reval_secret(
        from: &T::AccountId,
        capaign_id: u64,
        secret: u64,
    ) -> Result<(), DispatchError> {

        ensure!(
            ParticipantsOfCampaigns::<T>::contains_key(capaign_id, &from),
            Error::<T>::IsNotAParticipant
        );

        let mut participant = ParticipantsOfCampaigns::<T>::get(capaign_id, &from).unwrap();

        ensure!(
            Self::check_secret(participant.commitment, secret),
            Error::<T>::SecretDoesNotMatchTheHash
        );

        let mut campaings = Campaigns::<T>::get(capaign_id).unwrap();

        campaings.secret ^= secret;

        participant.secret = secret;
        ParticipantsOfCampaigns::<T>::insert (capaign_id, &from, participant);
        Campaigns::<T>::insert(capaign_id, campaings);
        Self::deposit_event(Event::LogReval {
            from: from.clone(),
            secret: secret,
        });
        log::info!("[RANDAO] The account with the ID {:?} did reval for campaign {:?}", from, capaign_id);
        Ok(())
    }

    fn check_secret(hash: [u8; 32], secret: u64) -> bool {
        let _hash: [u8; 32] = Self::hash_num(secret);
        return hash == _hash
    }

    fn hash_num(num: u64) -> [u8; 32] {
        let data: [u8; 8] = num.to_le_bytes();
        let hashed_random_num = <T>::Hashing::hash(&data);
        Self::vec_to_bytes_array(hashed_random_num.encode())
    }

    fn vec_to_bytes_array(vec: Vec<u8>) -> [u8; 32] {
        // let mut tmp: [u8; 32] = [0; 32];
        // for i in 0..32 {
        //     tmp[i] = vec[i];
        // }
        // return tmp
        return vec.try_into().expect("Vector length doesn't match the target array")
    }
}
