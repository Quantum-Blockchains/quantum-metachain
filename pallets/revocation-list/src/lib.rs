#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::ensure;
pub use pallet::*;
use sp_std::vec::Vec;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
    use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};
    use sp_io::hashing::blake2_256;
    use sp_runtime::traits::Zero;
    use sp_std::vec;

    const STATUSLIST_PREFIX: &[u8] = b"did:qsb:statuslist:";
    const STATUSLIST_PREFIX_ALT: &[u8] = b"did:qmc:statuslist:";
    const STATUSLIST_MATERIAL_PREFIX: &[u8] = b"QSB_STATUSLIST";
    const MIN_LIST_NONCE_BYTES: usize = 16;

    #[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct StatusList {
        pub version: u64,
        pub issuer_did: Vec<u8>,
        pub list_nonce: Vec<u8>,
        pub bitmap: Vec<u8>,
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::storage]
    pub(super) type StatusLists<T: Config> =
        StorageMap<_, Twox64Concat, [u8; 32], StatusList, OptionQuery>;

    #[pallet::error]
    pub enum Error<T> {
        StatusListAlreadyExists,
        StatusListNotFound,
        InvalidStatusListId,
        InvalidListNonce,
        IssuerMismatch,
        StatusIndexOutOfBounds,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        StatusListCreated { status_list_id: Vec<u8>, issuer_did: Vec<u8> },
        StatusUpdated { status_list_id: Vec<u8>, status_index: u32, revoked: bool },
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight({0})]
        pub fn create_status_list(
            origin: OriginFor<T>,
            issuer_did: Vec<u8>,
            list_nonce: Vec<u8>,
            list_length: u32,
            _did_signature: Vec<u8>,
        ) -> DispatchResult {
            let _ = frame_system::ensure_signed(origin)?;
            ensure!(
                list_nonce.len() >= MIN_LIST_NONCE_BYTES,
                Error::<T>::InvalidListNonce
            );

            let status_list_id = Self::status_list_id_from_parts(&issuer_did, &list_nonce);
            ensure!(
                !StatusLists::<T>::contains_key(status_list_id),
                Error::<T>::StatusListAlreadyExists
            );

            let bitmap_len = list_length
                .checked_add(7)
                .ok_or(Error::<T>::StatusIndexOutOfBounds)?
                / 8;
            let bitmap = vec![0u8; bitmap_len as usize];
            let record = StatusList {
                version: 0,
                issuer_did: issuer_did.clone(),
                list_nonce,
                bitmap,
            };

            StatusLists::<T>::insert(status_list_id, record);
            let status_list_id_full = Self::status_list_string_from_id(&status_list_id);
            Self::deposit_event(Event::StatusListCreated {
                status_list_id: status_list_id_full,
                issuer_did,
            });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight({0})]
        pub fn set_status(
            origin: OriginFor<T>,
            status_list_id: Vec<u8>,
            issuer_did: Vec<u8>,
            status_index: u32,
            revoked: bool,
            _did_signature: Vec<u8>,
        ) -> DispatchResult {
            let _ = frame_system::ensure_signed(origin)?;
            let status_list_id = Self::decode_status_list_id(&status_list_id)?;
            let status_list_id_full = Self::status_list_string_from_id(&status_list_id);

            StatusLists::<T>::try_mutate(status_list_id, |maybe_record| -> DispatchResult {
                let record = maybe_record
                    .as_mut()
                    .ok_or(Error::<T>::StatusListNotFound)?;
                ensure!(record.issuer_did == issuer_did, Error::<T>::IssuerMismatch);

                let bit_count = record
                    .bitmap
                    .len()
                    .checked_mul(8)
                    .ok_or(Error::<T>::StatusIndexOutOfBounds)?;
                let status_index_usize = status_index as usize;
                ensure!(status_index_usize < bit_count, Error::<T>::StatusIndexOutOfBounds);

                let byte_index = status_index_usize / 8;
                let bit_index = (status_index_usize % 8) as u8;
                let mask = 1u8 << bit_index;
                if revoked {
                    record.bitmap[byte_index] |= mask;
                } else {
                    record.bitmap[byte_index] &= !mask;
                }
                record.version = record.version.saturating_add(1);
                Ok(())
            })?;

            Self::deposit_event(Event::StatusUpdated {
                status_list_id: status_list_id_full,
                status_index,
                revoked,
            });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn status_list_id_from_parts(issuer_did: &[u8], list_nonce: &[u8]) -> [u8; 32] {
            let genesis = frame_system::Pallet::<T>::block_hash(BlockNumberFor::<T>::zero());
            let mut material = Vec::with_capacity(
                STATUSLIST_MATERIAL_PREFIX.len()
                    + genesis.as_ref().len()
                    + issuer_did.len()
                    + list_nonce.len(),
            );
            material.extend_from_slice(STATUSLIST_MATERIAL_PREFIX);
            material.extend_from_slice(genesis.as_ref());
            material.extend_from_slice(issuer_did);
            material.extend_from_slice(list_nonce);
            blake2_256(&material)
        }

        fn status_list_string_from_id(status_list_id: &[u8; 32]) -> Vec<u8> {
            let status_list_id_b58 = bs58::encode(status_list_id).into_string();
            let mut status_list_id_full =
                Vec::with_capacity(STATUSLIST_PREFIX.len() + status_list_id_b58.len());
            status_list_id_full.extend_from_slice(STATUSLIST_PREFIX);
            status_list_id_full.extend_from_slice(status_list_id_b58.as_bytes());
            status_list_id_full
        }

        fn decode_status_list_id(input: &[u8]) -> Result<[u8; 32], Error<T>> {
            let status_list_id_bytes = if input.starts_with(STATUSLIST_PREFIX) {
                &input[STATUSLIST_PREFIX.len()..]
            } else if input.starts_with(STATUSLIST_PREFIX_ALT) {
                &input[STATUSLIST_PREFIX_ALT.len()..]
            } else {
                input
            };

            let decoded = bs58::decode(status_list_id_bytes)
                .into_vec()
                .map_err(|_| Error::<T>::InvalidStatusListId)?;
            let status_list_id: [u8; 32] = decoded
                .try_into()
                .map_err(|_| Error::<T>::InvalidStatusListId)?;
            Ok(status_list_id)
        }

        pub fn get_status_list(status_list_id: Vec<u8>) -> Result<StatusList, Error<T>> {
            let status_list_id = Self::decode_status_list_id(&status_list_id)?;
            StatusLists::<T>::get(status_list_id).ok_or(Error::<T>::StatusListNotFound)
        }
    }
}
