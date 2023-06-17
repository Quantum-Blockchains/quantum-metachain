#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    dispatch::DispatchResult,
    ensure,
    pallet_prelude::{DispatchError, MaxEncodedLen, RuntimeDebug, StorageMap, TypeInfo},
    traits::Currency,
};
pub use pallet::*;
use sp_core::{Decode, Encode, Hasher};
use sp_std::{str, vec::Vec};

/// This is a structure that is equivalent to a random number generation participant.
/// secret - secret number
/// commitment - hash of secret number
#[derive(Clone, Encode, Decode, PartialEq, Eq, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Participant {
    pub secret: u64,
    pub commitment: [u8; 32],
}

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    use super::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub(super) type Participants<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, Participant>;

    #[pallet::error]
    pub enum Error<T> {
        ParticipantIsAlreadyThere,
        IsNotAParticipant,
        SecretDoesNotMatchTheHash,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        LogCommit {
            from: T::AccountId,
            commitment: [u8; 32],
        },
        LogReval {
            from: T::AccountId,
            secret: u64,
        },
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
        pub fn commit(origin: OriginFor<T>, hash: [u8; 32]) -> DispatchResult {
            // TODO Check if the hash is sent within the allowed time
            let sender = ensure_signed(origin)?;
            Self::commit_hash(&sender, hash)
        }

        #[pallet::weight(0)]
        pub fn reval(origin: OriginFor<T>, secret: u64) -> DispatchResult {
            // TODO Check if the hash is sent within the allowed time
            let sender = ensure_signed(origin)?;
            Self::reval_secret(&sender, secret)
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn commit_hash(from: &T::AccountId, commitment: [u8; 32]) -> Result<(), DispatchError> {
        ensure!(
            !Participants::<T>::contains_key(&from),
            Error::<T>::ParticipantIsAlreadyThere
        );

        let new_participant = Participant {
            secret: 0,
            commitment,
        };

        Participants::<T>::insert(&from, new_participant);
        Self::deposit_event(Event::LogCommit {
            from: from.clone(),
            commitment,
        });
        Ok(())
    }

    pub fn reval_secret(from: &T::AccountId, secret: u64) -> Result<(), DispatchError> {
        ensure!(
            Participants::<T>::contains_key(&from),
            Error::<T>::IsNotAParticipant
        );

        let mut participant = Participants::<T>::get(&from).unwrap();

        ensure!(
            Self::check_secret(participant.commitment, secret),
            Error::<T>::SecretDoesNotMatchTheHash
        );

        participant.secret = secret;
        Participants::<T>::insert(&from, participant);
        Self::deposit_event(Event::LogReval {
            from: from.clone(),
            secret,
        });
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
            .expect("Vector length doesn't match the target array")
    }
}
