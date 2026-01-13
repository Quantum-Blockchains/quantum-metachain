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

    const DID_PREFIX: &[u8] = b"did:qsb:";
    const DID_MATERIAL_PREFIX: &[u8] = b"QSB_DID";
    const DID_CREATE_PREFIX: &[u8] = b"QSB_DID_CREATE";

    #[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub enum KeyRole {
        Authentication,
        AssertionMethod,
        KeyAgreement,
        CapabilityInvocation,
        CapabilityDelegation,
    }

    #[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct DidKey {
        pub public_key: Vec<u8>,
        pub roles: Vec<KeyRole>,
        pub revoked: bool,
    }

    #[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct ServiceEndpoint {
        pub id: Vec<u8>,
        pub service_type: Vec<u8>,
        pub endpoint: Vec<u8>,
    }

    #[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct MetadataEntry {
        pub key: Vec<u8>,
        pub value: Vec<u8>,
    }

    #[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct DidDetails {
        pub version: u64,
        pub deactivated: bool,
        pub keys: Vec<DidKey>,
        pub services: Vec<ServiceEndpoint>,
        pub metadata: Vec<MetadataEntry>,
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::storage]
    pub(super) type DidRecords<T: Config> = StorageMap<_, Twox64Concat, [u8; 32], DidDetails, OptionQuery>;

    #[pallet::error]
    pub enum Error<T> {
        DidAlreadyExists,
        DidNotFound,
        DidDeactivated,
        KeyAlreadyExists,
        KeyNotFound,
        KeyAlreadyRevoked,
        InvalidDidId,
        ServiceAlreadyExists,
        ServiceNotFound,
        MetadataNotFound,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        DidCreated { did: Vec<u8> },
        KeyAdded { did: Vec<u8>, public_key: Vec<u8> },
        KeyRevoked { did: Vec<u8>, public_key: Vec<u8> },
        DidDeactivated { did: Vec<u8> },
        KeyRotated { did: Vec<u8>, old_public_key: Vec<u8>, new_public_key: Vec<u8> },
        RolesUpdated { did: Vec<u8>, public_key: Vec<u8> },
        ServiceAdded { did: Vec<u8>, service_id: Vec<u8> },
        ServiceRemoved { did: Vec<u8>, service_id: Vec<u8> },
        MetadataSet { did: Vec<u8>, key: Vec<u8> },
        MetadataRemoved { did: Vec<u8>, key: Vec<u8> },
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight({0})]
        pub fn create_did(
            origin: OriginFor<T>,
            public_key: Vec<u8>,
            _did_signature: Vec<u8>,
        ) -> DispatchResult {
            let _ = frame_system::ensure_signed(origin)?;
            let did_id = Self::did_id_from_public_key(&public_key);
            ensure!(!DidRecords::<T>::contains_key(did_id), Error::<T>::DidAlreadyExists);

            let details = DidDetails {
                version: 0,
                deactivated: false,
                keys: vec![DidKey {
                    public_key,
                    roles: vec![KeyRole::Authentication],
                    revoked: false,
                }],
                services: Vec::new(),
                metadata: Vec::new(),
            };

            DidRecords::<T>::insert(did_id, details);
            let did = Self::did_string_from_did_id(&did_id);
            Self::deposit_event(Event::DidCreated { did });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight({0})]
        pub fn add_key(
            origin: OriginFor<T>,
            did_id: Vec<u8>,
            public_key: Vec<u8>,
            roles: Vec<KeyRole>,
        ) -> DispatchResult {
            let _ = frame_system::ensure_signed(origin)?;
            let did_id = Self::decode_did_id(&did_id)?;
            let did = Self::did_string_from_did_id(&did_id);

            DidRecords::<T>::try_mutate(did_id, |maybe_details| -> DispatchResult {
                let details = maybe_details.as_mut().ok_or(Error::<T>::DidNotFound)?;
                ensure!(!details.deactivated, Error::<T>::DidDeactivated);
                ensure!(
                    !details.keys.iter().any(|key| key.public_key == public_key),
                    Error::<T>::KeyAlreadyExists
                );

                details.keys.push(DidKey {
                    public_key: public_key.clone(),
                    roles,
                    revoked: false,
                });
                details.version = details.version.saturating_add(1);
                Ok(())
            })?;

            Self::deposit_event(Event::KeyAdded { did, public_key });
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight({0})]
        pub fn revoke_key(
            origin: OriginFor<T>,
            did_id: Vec<u8>,
            public_key: Vec<u8>,
        ) -> DispatchResult {
            let _ = frame_system::ensure_signed(origin)?;
            let did_id = Self::decode_did_id(&did_id)?;
            let did = Self::did_string_from_did_id(&did_id);

            DidRecords::<T>::try_mutate(did_id, |maybe_details| -> DispatchResult {
                let details = maybe_details.as_mut().ok_or(Error::<T>::DidNotFound)?;
                ensure!(!details.deactivated, Error::<T>::DidDeactivated);

                let key = details
                    .keys
                    .iter_mut()
                    .find(|key| key.public_key == public_key)
                    .ok_or(Error::<T>::KeyNotFound)?;

                ensure!(!key.revoked, Error::<T>::KeyAlreadyRevoked);
                key.revoked = true;
                details.version = details.version.saturating_add(1);
                Ok(())
            })?;

            Self::deposit_event(Event::KeyRevoked { did, public_key });
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight({0})]
        pub fn deactivate_did(origin: OriginFor<T>, did_id: Vec<u8>) -> DispatchResult {
            let _ = frame_system::ensure_signed(origin)?;
            let did_id = Self::decode_did_id(&did_id)?;
            let did = Self::did_string_from_did_id(&did_id);

            DidRecords::<T>::try_mutate(did_id, |maybe_details| -> DispatchResult {
                let details = maybe_details.as_mut().ok_or(Error::<T>::DidNotFound)?;
                ensure!(!details.deactivated, Error::<T>::DidDeactivated);
                details.deactivated = true;
                details.version = details.version.saturating_add(1);
                Ok(())
            })?;

            Self::deposit_event(Event::DidDeactivated { did });
            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight({0})]
        pub fn add_service(
            origin: OriginFor<T>,
            did_id: Vec<u8>,
            service: ServiceEndpoint,
        ) -> DispatchResult {
            let _ = frame_system::ensure_signed(origin)?;
            let did_id = Self::decode_did_id(&did_id)?;
            let did = Self::did_string_from_did_id(&did_id);
            let service_id = service.id.clone();

            DidRecords::<T>::try_mutate(did_id, |maybe_details| -> DispatchResult {
                let details = maybe_details.as_mut().ok_or(Error::<T>::DidNotFound)?;
                ensure!(!details.deactivated, Error::<T>::DidDeactivated);
                ensure!(
                    !details.services.iter().any(|entry| entry.id == service.id),
                    Error::<T>::ServiceAlreadyExists
                );
                details.services.push(service);
                details.version = details.version.saturating_add(1);
                Ok(())
            })?;

            Self::deposit_event(Event::ServiceAdded { did, service_id });
            Ok(())
        }

        #[pallet::call_index(5)]
        #[pallet::weight({0})]
        pub fn remove_service(
            origin: OriginFor<T>,
            did_id: Vec<u8>,
            service_id: Vec<u8>,
        ) -> DispatchResult {
            let _ = frame_system::ensure_signed(origin)?;
            let did_id = Self::decode_did_id(&did_id)?;
            let did = Self::did_string_from_did_id(&did_id);

            DidRecords::<T>::try_mutate(did_id, |maybe_details| -> DispatchResult {
                let details = maybe_details.as_mut().ok_or(Error::<T>::DidNotFound)?;
                ensure!(!details.deactivated, Error::<T>::DidDeactivated);
                let index = details
                    .services
                    .iter()
                    .position(|entry| entry.id == service_id)
                    .ok_or(Error::<T>::ServiceNotFound)?;
                details.services.swap_remove(index);
                details.version = details.version.saturating_add(1);
                Ok(())
            })?;

            Self::deposit_event(Event::ServiceRemoved { did, service_id });
            Ok(())
        }

        #[pallet::call_index(6)]
        #[pallet::weight({0})]
        pub fn set_metadata(
            origin: OriginFor<T>,
            did_id: Vec<u8>,
            entry: MetadataEntry,
        ) -> DispatchResult {
            let _ = frame_system::ensure_signed(origin)?;
            let did_id = Self::decode_did_id(&did_id)?;
            let did = Self::did_string_from_did_id(&did_id);
            let key = entry.key.clone();

            DidRecords::<T>::try_mutate(did_id, |maybe_details| -> DispatchResult {
                let details = maybe_details.as_mut().ok_or(Error::<T>::DidNotFound)?;
                ensure!(!details.deactivated, Error::<T>::DidDeactivated);
                if let Some(existing) = details.metadata.iter_mut().find(|item| item.key == entry.key) {
                    existing.value = entry.value;
                } else {
                    details.metadata.push(entry);
                }
                details.version = details.version.saturating_add(1);
                Ok(())
            })?;

            Self::deposit_event(Event::MetadataSet { did, key });
            Ok(())
        }

        #[pallet::call_index(7)]
        #[pallet::weight({0})]
        pub fn remove_metadata(
            origin: OriginFor<T>,
            did_id: Vec<u8>,
            key: Vec<u8>,
        ) -> DispatchResult {
            let _ = frame_system::ensure_signed(origin)?;
            let did_id = Self::decode_did_id(&did_id)?;
            let did = Self::did_string_from_did_id(&did_id);

            DidRecords::<T>::try_mutate(did_id, |maybe_details| -> DispatchResult {
                let details = maybe_details.as_mut().ok_or(Error::<T>::DidNotFound)?;
                ensure!(!details.deactivated, Error::<T>::DidDeactivated);
                let index = details
                    .metadata
                    .iter()
                    .position(|item| item.key == key)
                    .ok_or(Error::<T>::MetadataNotFound)?;
                details.metadata.swap_remove(index);
                details.version = details.version.saturating_add(1);
                Ok(())
            })?;

            Self::deposit_event(Event::MetadataRemoved { did, key });
            Ok(())
        }

        #[pallet::call_index(8)]
        #[pallet::weight({0})]
        pub fn rotate_key(
            origin: OriginFor<T>,
            did_id: Vec<u8>,
            old_public_key: Vec<u8>,
            new_public_key: Vec<u8>,
            roles: Vec<KeyRole>,
        ) -> DispatchResult {
            let _ = frame_system::ensure_signed(origin)?;
            let did_id = Self::decode_did_id(&did_id)?;
            let did = Self::did_string_from_did_id(&did_id);

            DidRecords::<T>::try_mutate(did_id, |maybe_details| -> DispatchResult {
                let details = maybe_details.as_mut().ok_or(Error::<T>::DidNotFound)?;
                ensure!(!details.deactivated, Error::<T>::DidDeactivated);
                ensure!(
                    !details.keys.iter().any(|key| key.public_key == new_public_key),
                    Error::<T>::KeyAlreadyExists
                );

                let key = details
                    .keys
                    .iter_mut()
                    .find(|key| key.public_key == old_public_key)
                    .ok_or(Error::<T>::KeyNotFound)?;
                ensure!(!key.revoked, Error::<T>::KeyAlreadyRevoked);
                key.revoked = true;

                details.keys.push(DidKey {
                    public_key: new_public_key.clone(),
                    roles,
                    revoked: false,
                });
                details.version = details.version.saturating_add(1);
                Ok(())
            })?;

            Self::deposit_event(Event::KeyRotated {
                did,
                old_public_key,
                new_public_key,
            });
            Ok(())
        }

        #[pallet::call_index(9)]
        #[pallet::weight({0})]
        pub fn update_roles(
            origin: OriginFor<T>,
            did_id: Vec<u8>,
            public_key: Vec<u8>,
            roles: Vec<KeyRole>,
        ) -> DispatchResult {
            let _ = frame_system::ensure_signed(origin)?;
            let did_id = Self::decode_did_id(&did_id)?;
            let did = Self::did_string_from_did_id(&did_id);

            DidRecords::<T>::try_mutate(did_id, |maybe_details| -> DispatchResult {
                let details = maybe_details.as_mut().ok_or(Error::<T>::DidNotFound)?;
                ensure!(!details.deactivated, Error::<T>::DidDeactivated);
                let key = details
                    .keys
                    .iter_mut()
                    .find(|key| key.public_key == public_key)
                    .ok_or(Error::<T>::KeyNotFound)?;
                ensure!(!key.revoked, Error::<T>::KeyAlreadyRevoked);
                key.roles = roles;
                details.version = details.version.saturating_add(1);
                Ok(())
            })?;

            Self::deposit_event(Event::RolesUpdated { did, public_key });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn did_id_from_public_key(public_key: &[u8]) -> [u8; 32] {
            let genesis = frame_system::Pallet::<T>::block_hash(BlockNumberFor::<T>::zero());
            let mut material = Vec::with_capacity(
                DID_MATERIAL_PREFIX.len() + genesis.as_ref().len() + public_key.len(),
            );
            material.extend_from_slice(DID_MATERIAL_PREFIX);
            material.extend_from_slice(genesis.as_ref());
            material.extend_from_slice(public_key);
            blake2_256(&material)
        }

        fn did_string_from_did_id(did_id: &[u8; 32]) -> Vec<u8> {
            let did_id_b58 = bs58::encode(did_id).into_string();
            let mut did = Vec::with_capacity(DID_PREFIX.len() + did_id_b58.len());
            did.extend_from_slice(DID_PREFIX);
            did.extend_from_slice(did_id_b58.as_bytes());
            did
        }

        fn decode_did_id(input: &[u8]) -> Result<[u8; 32], Error<T>> {
            let did_id_bytes = if input.starts_with(DID_PREFIX) {
                &input[DID_PREFIX.len()..]
            } else {
                input
            };

            let decoded = bs58::decode(did_id_bytes)
                .into_vec()
                .map_err(|_| Error::<T>::InvalidDidId)?;
            let did_id: [u8; 32] = decoded.try_into().map_err(|_| Error::<T>::InvalidDidId)?;
            Ok(did_id)
        }

        #[allow(dead_code)]
        fn create_payload(public_key: &[u8]) -> Vec<u8> {
            let mut payload = Vec::with_capacity(DID_CREATE_PREFIX.len() + public_key.len());
            payload.extend_from_slice(DID_CREATE_PREFIX);
            payload.extend_from_slice(public_key);
            payload
        }

        pub fn get_did(did_id: Vec<u8>) -> Result<DidDetails, Error<T>> {
            let did_id = Self::decode_did_id(&did_id)?;
            DidRecords::<T>::get(did_id).ok_or(Error::<T>::DidNotFound)
        }
    }
}
