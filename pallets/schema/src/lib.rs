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
    const SCHEMA_PREFIX: &[u8] = b"did:qsb:schema:";
    const SCHEMA_MATERIAL_PREFIX: &[u8] = b"QSB_SCHEMA";

    #[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct SchemaRecord {
        pub version: u64,
        pub deprecated: bool,
        pub issuer_did: Vec<u8>,
        pub schema_hash: [u8; 32],
        pub schema_uri: Vec<u8>,
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::storage]
    pub(super) type Schemas<T: Config> =
        StorageMap<_, Twox64Concat, [u8; 32], SchemaRecord, OptionQuery>;

    #[pallet::error]
    pub enum Error<T> {
        SchemaAlreadyExists,
        SchemaNotFound,
        SchemaDeprecated,
        InvalidSchemaId,
        IssuerMismatch,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        SchemaRegistered { schema_id: Vec<u8>, issuer_did: Vec<u8> },
        SchemaDeprecated { schema_id: Vec<u8>, issuer_did: Vec<u8> },
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight({0})]
        pub fn register_schema(
            origin: OriginFor<T>,
            schema_json: Vec<u8>,
            schema_uri: Vec<u8>,
            issuer_did: Vec<u8>,
            _did_signature: Vec<u8>,
        ) -> DispatchResult {
            let _ = frame_system::ensure_signed(origin)?;
            let schema_id = Self::schema_id_from_schema(&schema_json);
            ensure!(
                !Schemas::<T>::contains_key(schema_id),
                Error::<T>::SchemaAlreadyExists
            );

            let schema_hash = blake2_256(&schema_json);
            let record = SchemaRecord {
                version: 0,
                deprecated: false,
                issuer_did: issuer_did.clone(),
                schema_hash,
                schema_uri,
            };

            Schemas::<T>::insert(schema_id, record);
            let schema_id_full = Self::schema_string_from_schema_id(&schema_id);
            Self::deposit_event(Event::SchemaRegistered {
                schema_id: schema_id_full,
                issuer_did,
            });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight({0})]
        pub fn deprecate_schema(
            origin: OriginFor<T>,
            schema_id: Vec<u8>,
            issuer_did: Vec<u8>,
            _did_signature: Vec<u8>,
        ) -> DispatchResult {
            let _ = frame_system::ensure_signed(origin)?;
            let schema_id = Self::decode_schema_id(&schema_id)?;
            let schema_id_full = Self::schema_string_from_schema_id(&schema_id);

            Schemas::<T>::try_mutate(schema_id, |maybe_record| -> DispatchResult {
                let record = maybe_record.as_mut().ok_or(Error::<T>::SchemaNotFound)?;
                ensure!(!record.deprecated, Error::<T>::SchemaDeprecated);
                ensure!(record.issuer_did == issuer_did, Error::<T>::IssuerMismatch);
                record.deprecated = true;
                record.version = record.version.saturating_add(1);
                Ok(())
            })?;

            Self::deposit_event(Event::SchemaDeprecated {
                schema_id: schema_id_full,
                issuer_did,
            });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn schema_id_from_schema(schema_json: &[u8]) -> [u8; 32] {
            let genesis = frame_system::Pallet::<T>::block_hash(BlockNumberFor::<T>::zero());
            let mut material = Vec::with_capacity(
                SCHEMA_MATERIAL_PREFIX.len() + genesis.as_ref().len() + schema_json.len(),
            );
            material.extend_from_slice(SCHEMA_MATERIAL_PREFIX);
            material.extend_from_slice(genesis.as_ref());
            material.extend_from_slice(schema_json);
            blake2_256(&material)
        }

        fn schema_string_from_schema_id(schema_id: &[u8; 32]) -> Vec<u8> {
            let schema_id_b58 = bs58::encode(schema_id).into_string();
            let mut schema_id_full = Vec::with_capacity(SCHEMA_PREFIX.len() + schema_id_b58.len());
            schema_id_full.extend_from_slice(SCHEMA_PREFIX);
            schema_id_full.extend_from_slice(schema_id_b58.as_bytes());
            schema_id_full
        }

        fn decode_schema_id(input: &[u8]) -> Result<[u8; 32], Error<T>> {
            let schema_id_bytes = if input.starts_with(SCHEMA_PREFIX) {
                &input[SCHEMA_PREFIX.len()..]
            } else {
                input
            };

            let decoded = bs58::decode(schema_id_bytes)
                .into_vec()
                .map_err(|_| Error::<T>::InvalidSchemaId)?;
            let schema_id: [u8; 32] = decoded
                .try_into()
                .map_err(|_| Error::<T>::InvalidSchemaId)?;
            Ok(schema_id)
        }

        pub fn get_schema(schema_id: Vec<u8>) -> Result<SchemaRecord, Error<T>> {
            let schema_id = Self::decode_schema_id(&schema_id)?;
            Schemas::<T>::get(schema_id).ok_or(Error::<T>::SchemaNotFound)
        }
    }
}
