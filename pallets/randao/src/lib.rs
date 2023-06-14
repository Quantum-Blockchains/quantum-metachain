#![cfg_attr(not(feature = "std"), no_std)]


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
use frame_system::offchain::{ SendUnsignedTransaction, SubmitTransaction, SendTransactionTypes};
use sp_runtime::traits::SaturatedConversion;


const NUM_BLOCK_FOR_CAMPAIGN: u64 = 10;
const COMMIT_BALKLINE: u64 = 8;
const COMMIT_DEADLINE: u64 = 4;
const UNSIGNED_TXS_PRIORITY: u64 = 100;

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
    use frame_system::offchain::SubmitTransaction;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: SendTransactionTypes<Call<Self>> + frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Call: From<Call<Self>>;
        type Currency: Currency<Self::AccountId>;
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
        LogCommit { from: [u8; 52], commitment: [u8; 32] },
        LogReval { from: [u8; 52], secret: u64 },
    }

    #[pallet::error]
    pub enum Error<T> {
        TimeLineCheck,
        TimeLineCommitPhase,
        TimeLineRevalPhase,
        IncorrectId,
        ParticipantIsAlreadyThere,
        IsNotAParticipant,
        SecretDoesNotMatchTheHash,
        OffchainUnsignedTxError,
    }

    #[pallet::storage]
    pub(super) type Campaigns<T: Config> = StorageMap<_, Twox64Concat, u64, Campaign>;

    #[pallet::storage]
    pub(super) type ParticipantsOfCampaigns<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        u64,
        Twox64Concat,
        [u8; 52],
        Participant,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
	    pub fn create(
            origin: OriginFor<T>,
            id: u64,
            block_num: u64,
            commit_balkline: u64,
            commit_deadline: u64,
        ) -> DispatchResult
        {
            ensure_none(origin)?;
            Self::create_new_campaign(id, block_num, commit_balkline, commit_deadline);
			Ok(())
		}

        #[pallet::weight(0)]
	    pub fn commit(
            origin: OriginFor<T>,
            from: [u8; 52],
            capaign_id: u64,
            commitment: [u8; 32],
        ) -> DispatchResult
        {
            ensure_none(origin)?;
            Self::commit_hash(from, capaign_id, commitment);
			Ok(())
		}

        #[pallet::weight(0)]
	    pub fn reval(
            origin: OriginFor<T>,
            from: [u8; 52],
            capaign_id: u64,
            secret: u64,
        ) -> DispatchResult
        {
            ensure_none(origin)?;
            Self::reval_secret(from, capaign_id, secret);
			Ok(())
		}

    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
    where
        u64: From<<T as frame_system::Config>::BlockNumber>,
    {
        fn offchain_worker(block_number: T::BlockNumber) {
            log::info!("[RANDAO] Running offchain worker...");
            let current_block_number: u64 = block_number.into();

            let id = current_block_number + NUM_BLOCK_FOR_CAMPAIGN;
            let block_num = current_block_number + NUM_BLOCK_FOR_CAMPAIGN;
            let commit_balkline = COMMIT_BALKLINE;
            let commit_deadline = COMMIT_DEADLINE;

            match Self::create_and_raw_unsigned(
                id,
                block_num,
                commit_balkline,
                commit_deadline
            ) {
                Ok(()) => log::info!("[RANDAO] Successful created a campaign for the block {:?}", id),
                Err(err) => log::info!("[RANDAO] Failed to create a campaign for the block {:?} : {:?}", id, err),
            };

        }
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
	    type Call = Call<T>;

		fn validate_unsigned(source: TransactionSource, call: &Self::Call) -> TransactionValidity {
		    let valid_tx = |provide|
                ValidTransaction::with_tag_prefix("randao")
			    .priority(UNSIGNED_TXS_PRIORITY)
			    .and_provides([&provide])
			    .longevity(3)
			    .propagate(true)
			    .build();

            match call {
		        Call::create { id, block_num, commit_balkline, commit_deadline } => valid_tx(b"create".to_vec()),
                Call::commit { from, capaign_id, commitment } => valid_tx(b"commit".to_vec()),
                Call::reval { from, capaign_id, secret } => valid_tx(b"reval".to_vec()),
		        _ => InvalidTransaction::Call.into(),
	        }
		}
    }
}

impl<T: Config> Pallet<T> {

    fn create_and_raw_unsigned(
        id: u64,
        block_num: u64,
        commit_balkline: u64,
        commit_deadline: u64,
    ) -> Result<(), &'static str>
    {
        let call = Call::create { id, block_num, commit_balkline, commit_deadline };
        SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
			.map_err(|err| "Unable to submit unsigned transaction.")?;
		Ok(())
    }

    pub fn commit_and_raw_unsigned(
        from: [u8; 52],
        capaign_id: u64,
        commitment: [u8; 32],
    ) -> Result<(), &'static str>
    {
        let call = Call::commit { from, capaign_id, commitment };
        SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
			.map_err(|err| "Unable to submit unsigned transaction.")?;
		Ok(())
    }

    pub fn reval_and_raw_unsigned(
        from: [u8; 52],
        capaign_id: u64,
        secret: u64,
    ) -> Result<(), &'static str>
    {
        let call = Call::reval { from, capaign_id, secret };
        SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
			.map_err(|err| "Unable to submit unsigned transaction.")?;
		Ok(())
    }

    pub fn get_secret(campaign_id: u64) -> Result<u64, DispatchError> {
        let campaigns = Campaigns::<T>::get(campaign_id).ok_or(Error::<T>::IncorrectId)?;
        Ok(campaigns.secret)
    }

    fn create_new_campaign(
        id: u64,
        block_num: u64,
        commit_balkline: u64,
        commit_deadline: u64,
    ) -> DispatchResult
    {
        let block: T::BlockNumber = frame_system::pallet::Pallet::<T>::block_number();
        let current_block_num: u64 = block.saturated_into::<u64>();

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

        let new_campaign = Campaign {
            secret: 0,
            block_num: block_num,
            commit_balkline: commit_balkline,
            commit_deadline: commit_deadline,
        };

        Campaigns::<T>::insert(id, new_campaign);

        Self::deposit_event(Event::LogCampaignAdded {
            capaign_id: id,
            block_num: block_num,
            commit_balkline: commit_balkline,
            commit_deadline: commit_deadline,
        });

        Ok(())
    }

    fn commit_hash(
        from: [u8; 52],
        campaign_id: u64,
        commitment: [u8; 32],
    ) -> DispatchResult
    {
        let block: T::BlockNumber = frame_system::pallet::Pallet::<T>::block_number();
        let current_block_num: u64 = block.saturated_into::<u64>();

        ensure!(
            !ParticipantsOfCampaigns::<T>::contains_key(campaign_id, &from),
            Error::<T>::ParticipantIsAlreadyThere
        );

        let campaign = Campaigns::<T>::get(campaign_id).ok_or(Error::<T>::IncorrectId)?;

        ensure!(
            current_block_num >= campaign.block_num - campaign.commit_balkline,
            Error::<T>::TimeLineCommitPhase
        );
        ensure!(
            current_block_num <= campaign.block_num - campaign.commit_deadline,
            Error::<T>::TimeLineCommitPhase
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

    fn reval_secret(
        from: [u8; 52],
        campaign_id: u64,
        secret: u64,
    ) -> DispatchResult
    {
        ensure!(
            ParticipantsOfCampaigns::<T>::contains_key(campaign_id, &from),
            Error::<T>::IsNotAParticipant
        );

        let block: T::BlockNumber = frame_system::pallet::Pallet::<T>::block_number();
        let current_block_num: u64 = block.saturated_into::<u64>();
        let mut campaign = Campaigns::<T>::get(campaign_id).ok_or(Error::<T>::IncorrectId)?;
        let mut participant = ParticipantsOfCampaigns::<T>::get(campaign_id, &from).unwrap();

        ensure!(
            current_block_num > campaign.block_num - campaign.commit_deadline,
            Error::<T>::TimeLineRevalPhase
        );

        ensure!(
            current_block_num < campaign.block_num,
            Error::<T>::TimeLineRevalPhase
        );

        ensure!(
            Self::check_secret(participant.commitment, secret),
            Error::<T>::SecretDoesNotMatchTheHash
        );

        campaign.secret ^= secret;

        participant.secret = secret;
        ParticipantsOfCampaigns::<T>::insert (campaign_id, &from, participant);
        Campaigns::<T>::insert(campaign_id, campaign);
        Self::deposit_event(Event::LogReval {
            from: from.clone(),
            secret: secret,
        });
        log::info!("[RANDAO] The account with the ID {:?} did reval for campaign {:?}", from, campaign_id);
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
        return vec.try_into().expect("Vector length doesn't match the target array")
    }
}
