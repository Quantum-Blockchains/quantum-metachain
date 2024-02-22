#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod tests;

use frame_support::{
    dispatch::DispatchResult,
    ensure,
    pallet_prelude::{DispatchError, MaxEncodedLen, RuntimeDebug, TypeInfo},
};
use frame_system::offchain::{SendTransactionTypes, SubmitTransaction};

use sp_core::{Decode, Encode, Hasher};
use sp_runtime::traits::SaturatedConversion;
use sp_std::{str, vec::Vec};

const NUM_BLOCK_FOR_CAMPAIGN: u64 = 10;
const COMMIT_BALKLINE: u64 = 8;
const COMMIT_DEADLINE: u64 = 4;
const UNSIGNED_TXS_PRIORITY: u64 = 100;

/// This is a structure that is equivalent to a random number generation participant.
/// secret - secret number
/// commitment - hash of secret number
#[derive(Clone, Encode, Decode, PartialEq, Eq, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Participant {
    pub secret: u64,
    pub commitment: [u8; 32],
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Campaign {
    pub secret: u64,
    pub commit_balkline: u64,
    pub commit_deadline: u64,
    pub commit_num: u64,
    pub reveals_num: u64,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: SendTransactionTypes<Call<Self>> + frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type RuntimeCall: From<Call<Self>>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        LogCampaignAdded {
            block_num: u64,
            commit_balkline: u64,
            commit_deadline: u64,
        },
        LogCommit {
            block_num: u64,
            from: [u8; 52],
            commitment: [u8; 32],
        },
        LogReveal {
            block_num: u64,
            from: [u8; 52],
            secret: u64,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        TimeLineCheck,
        TimeLineCommitPhase,
        TimeLineRevealPhase,
        IncorrectId,
        CampaignIsAlreadyThere,
        ParticipantIsAlreadyThere,
        IsNotAParticipant,
        SecretDoesNotMatchTheHash,
        OffchainUnsignedTxError,
        CampaignIsNotOver,
        FailedCompany,
    }

    #[pallet::storage]
    pub(super) type Campaigns<T: Config> = StorageMap<_, Twox64Concat, u64, Campaign>;

    /// The first key is the campaign id - block number (type u64)
    /// The second key is the peer id (type [u8; 52]).
    /// The length of this array is so that we get the encoded peer id.
    /// Example:
    /// 12D3KooWD3eckifWpRn9wQpMG9R9hX3sD158z7EqHWmweQAJU5SA -- Peer ID (ed25519, using the "identity" multihash) encoded as a raw base58btc multihash.
    #[pallet::storage]
    pub(super) type ParticipantsOfCampaigns<T: Config> =
        StorageDoubleMap<_, Twox64Concat, u64, Twox64Concat, [u8; 52], Participant>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn create(
            origin: OriginFor<T>,
            block_num: u64,
            commit_balkline: u64,
            commit_deadline: u64,
        ) -> DispatchResult {
            ensure_none(origin)?;
            Self::create_new_campaign(block_num, commit_balkline, commit_deadline)
        }

        #[pallet::weight(0)]
        pub fn commit(
            origin: OriginFor<T>,
            from: [u8; 52],
            block_num: u64,
            commitment: [u8; 32],
        ) -> DispatchResult {
            ensure_none(origin)?;
            Self::commit_hash(from, block_num, commitment)
        }

        #[pallet::weight(0)]
        pub fn reveal(
            origin: OriginFor<T>,
            from: [u8; 52],
            block_num: u64,
            secret: u64,
        ) -> DispatchResult {
            ensure_none(origin)?;
            Self::reveal_secret(from, block_num, secret)
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
    where
        u64: From<<<<T as frame_system::Config>::Block as sp_runtime::traits::Block>::Header as sp_runtime::traits::Header>::Number>
    {
        fn offchain_worker(block_number: BlockNumberFor<T>) {
            let current_block_number: u64 = block_number.into();
            let block_num = current_block_number + NUM_BLOCK_FOR_CAMPAIGN;
            let commit_balkline = COMMIT_BALKLINE;
            let commit_deadline = COMMIT_DEADLINE;

            match Self::create_and_raw_unsigned(block_num, commit_balkline, commit_deadline) {
                Ok(()) => {
                    log::info!(
                        "[RANDAO] Successful created a campaign for the block {:?}",
                        block_num
                    )
                }
                Err(err) => log::info!(
                    "[RANDAO] Failed to create a campaign for the block {:?} : {:?}",
                    block_num,
                    err
                ),
            };
        }
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            let valid_tx = |provide| {
                ValidTransaction::with_tag_prefix("randao")
                    .priority(UNSIGNED_TXS_PRIORITY)
                    .and_provides([&provide])
                    .longevity(3)
                    .propagate(true)
                    .build()
            };

            match call {
                Call::create {
                    block_num: _,
                    commit_balkline: _,
                    commit_deadline: _,
                } => valid_tx(b"create".to_vec()),
                Call::commit {
                    from: _,
                    block_num: _,
                    commitment: _,
                } => valid_tx(b"commit".to_vec()),
                Call::reveal {
                    from: _,
                    block_num: _,
                    secret: _,
                } => valid_tx(b"reveal".to_vec()),
                _ => InvalidTransaction::Call.into(),
            }
        }
    }
}

impl<T: Config> Pallet<T> {
    fn create_and_raw_unsigned(
        block_num: u64,
        commit_balkline: u64,
        commit_deadline: u64,
    ) -> Result<(), &'static str> {
        let call = Call::create {
            block_num,
            commit_balkline,
            commit_deadline,
        };
        SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
            .map_err(|_| "Unable to submit unsigned transaction.")?;
        Ok(())
    }

    pub fn commit_and_raw_unsigned(
        from: [u8; 52],
        block_num: u64,
        commitment: [u8; 32],
    ) -> Result<(), &'static str> {
        let call = Call::commit {
            from,
            block_num,
            commitment,
        };
        SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
            .map_err(|_| "Unable to submit unsigned transaction.")?;
        Ok(())
    }

    pub fn reveal_and_raw_unsigned(
        from: [u8; 52],
        block_num: u64,
        secret: u64,
    ) -> Result<(), &'static str> {
        let call = Call::reveal {
            from,
            block_num,
            secret,
        };
        SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
            .map_err(|_| "Unable to submit unsigned transaction.")?;
        Ok(())
    }

    pub fn get_secret(block_num: u64) -> Result<u64, DispatchError> {
        let campaigns = Campaigns::<T>::get(block_num).ok_or(Error::<T>::IncorrectId)?;
        let block = frame_system::Pallet::<T>::block_number();
        let current_block_num: u64 = block.saturated_into::<u64>();
        ensure!(
            current_block_num >= block_num,
            Error::<T>::CampaignIsNotOver
        );
        ensure!(campaigns.commit_num > 0, Error::<T>::FailedCompany);
        ensure!(
            campaigns.reveals_num >= campaigns.commit_num / 2,
            Error::<T>::FailedCompany
        );
        Ok(campaigns.secret)
    }

    fn create_new_campaign(
        block_num: u64,
        commit_balkline: u64,
        commit_deadline: u64,
    ) -> DispatchResult {
        let block = frame_system::Pallet::<T>::block_number();
        let current_block_num: u64 = block.saturated_into::<u64>();
        // ensure!(
        //     block_num - NUM_BLOCK_FOR_CAMPAIGN == current_block_num,
        //     Error::<T>::TimeLineCheck
        // );
        ensure!(
            !Campaigns::<T>::contains_key(block_num),
            Error::<T>::CampaignIsAlreadyThere
        );
        ensure!(current_block_num < block_num, Error::<T>::TimeLineCheck);
        ensure!(commit_deadline < commit_balkline, Error::<T>::TimeLineCheck);
        ensure!(
            current_block_num < block_num - commit_balkline,
            Error::<T>::TimeLineCheck
        );
        let new_campaign = Campaign {
            secret: 0,
            commit_balkline,
            commit_deadline,
            commit_num: 0,
            reveals_num: 0,
        };
        Campaigns::<T>::insert(block_num, new_campaign);
        Self::deposit_event(Event::LogCampaignAdded {
            block_num,
            commit_balkline,
            commit_deadline,
        });
        Ok(())
    }

    fn commit_hash(from: [u8; 52], block_num: u64, commitment: [u8; 32]) -> DispatchResult {
        let block = frame_system::Pallet::<T>::block_number();
        let current_block_num: u64 = block.saturated_into::<u64>();
        ensure!(
            !ParticipantsOfCampaigns::<T>::contains_key(block_num, &from),
            Error::<T>::ParticipantIsAlreadyThere
        );
        let mut campaign = Campaigns::<T>::get(block_num).ok_or(Error::<T>::IncorrectId)?;
        ensure!(
            current_block_num >= block_num - campaign.commit_balkline,
            Error::<T>::TimeLineCommitPhase
        );
        ensure!(
            current_block_num <= block_num - campaign.commit_deadline,
            Error::<T>::TimeLineCommitPhase
        );
        let new_participant = Participant {
            secret: 0,
            commitment,
        };

        ParticipantsOfCampaigns::<T>::insert(block_num, &from, new_participant);
        campaign.commit_num += 1;
        Campaigns::<T>::insert(block_num, campaign);
        Self::deposit_event(Event::LogCommit {
            block_num,
            from,
            commitment,
        });
        log::info!(
            "[RANDAO] The account with the ID {:?} did commit for campaign {:?}",
            from,
            block_num
        );

        Ok(())
    }

    fn reveal_secret(from: [u8; 52], block_num: u64, secret: u64) -> DispatchResult {
        ensure!(
            ParticipantsOfCampaigns::<T>::contains_key(block_num, &from),
            Error::<T>::IsNotAParticipant
        );

        let block = frame_system::Pallet::<T>::block_number();
        let current_block_num: u64 = block.saturated_into::<u64>();
        let mut campaign = Campaigns::<T>::get(block_num).ok_or(Error::<T>::IncorrectId)?;
        let mut participant = ParticipantsOfCampaigns::<T>::get(block_num, &from).unwrap();

        ensure!(
            current_block_num > block_num - campaign.commit_deadline,
            Error::<T>::TimeLineRevealPhase
        );

        ensure!(
            current_block_num < block_num,
            Error::<T>::TimeLineRevealPhase
        );

        ensure!(
            Self::check_secret(participant.commitment, secret),
            Error::<T>::SecretDoesNotMatchTheHash
        );

        campaign.secret ^= secret;

        participant.secret = secret;
        ParticipantsOfCampaigns::<T>::insert(block_num, &from, participant);
        campaign.reveals_num += 1;
        Campaigns::<T>::insert(block_num, campaign);
        Self::deposit_event(Event::LogReveal {
            block_num,
            from,
            secret,
        });
        log::info!(
            "[RANDAO] The account with the ID {:?} did reveal for campaign {:?}",
            from,
            block_num
        );

        Ok(())
    }

    fn check_secret(hash: [u8; 32], secret: u64) -> bool {
        let _hash: [u8; 32] = Self::hash_num(secret);
        hash == _hash
    }

    fn hash_num(num: u64) -> [u8; 32] {
        let data: [u8; 8] = num.to_le_bytes();
        let hashed_random_num = <T>::Hashing::hash(&data);
        Self::vec_to_bytes_array(hashed_random_num.encode())
    }

    fn vec_to_bytes_array(vec: Vec<u8>) -> [u8; 32] {
        vec.try_into()
            .expect("[RANDAO] Vector length doesn't match the target array")
    }
}
