#![cfg_attr(not(feature = "std"), no_std)]

// use ed25519_zebra::*;
// use bs58;
use sp_std::vec::Vec;
pub use pallet::*;
use frame_support::ensure;
mod ssi_types;
pub use crate::ssi_types::{
    Request, Dest, NymRoles, Nym, Schema, CredentialDefinition, RevocationRegistryDefinition, RevocationList
};

#[frame_support::pallet]
pub mod pallet {

    use super::*;
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
    use frame_system::pallet_prelude::OriginFor;

    #[pallet::pallet]
    #[pallet::without_storage_info] //TODO zobaczyc co to powaduje
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::storage]
    pub(super) type Nyms<T:Config> = StorageMap<_, Twox64Concat, Dest, Nym, OptionQuery>;

    #[pallet::storage]
    pub(super) type Schemas<T:Config> = StorageMap<_, Twox64Concat, Dest, Schema, OptionQuery>;

    #[pallet::storage]
    pub(super) type CredentialDefinitions<T:Config> = StorageMap<_, Twox64Concat, Dest, CredentialDefinition, OptionQuery>;

    #[pallet::storage]
    pub(super) type RevocationRegistryDefinitions<T:Config> = StorageMap<_, Twox64Concat, Dest, RevocationRegistryDefinition, OptionQuery>;

    #[pallet::storage]
    pub(super) type RevocationLists<T:Config> = StorageMap<_, Twox64Concat, Dest, RevocationList, OptionQuery>;

    #[pallet::error]
    pub enum Error<T> {
        NymDontExist,
        OnlyTrustee,
        NymAlreadyExists,
        InvalidSignature,
        SchemaAlreadyExists,
        NymDontHaveSchemas,
        CredentialDefinitionAlreadyExists,
        RevocationRegistryDefinitionAlreadyExists,
        RevocationListAlreadyExists,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
	    /// Set a value.
	    RegisteredNym { nym_id: Dest },
        RegisteredSchema {schema_id: Dest, issuer_id: Dest},
        RegisteredCredentialDefinition {cred_def_id: Dest, schema_id: Dest},
        RegisteredRevocationRegistryDefinition {rev_reg_def_id: Dest, cred_def_id: Dest},
        RegisteredRevocationList {rev_reg_def_id: Dest, issuer_id: Dest},
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {

        #[pallet::call_index(0)]
        #[pallet::weight({0})]
        pub fn register_nym(
            _origin: OriginFor<T>,
            alias: Vec<u8>,
            did: Vec<u8>,
            ver_key: Vec<u8>,
            role: NymRoles
        ) -> DispatchResult {
            ensure!(!Nyms::<T>::contains_key(did.clone()), Error::<T>::NymAlreadyExists);
            let nym = Nym {
                alias,
                did: did.clone(),
                role,
                ver_key,
                ver: "1.0".into()
            };
            Nyms::<T>::insert(did.clone(), nym);
            Self::deposit_event(Event::RegisteredNym{nym_id: did});
            Ok(())
        }
        
        #[pallet::call_index(1)]
        #[pallet::weight({0})]
        pub fn create_schema(
            _origin: OriginFor<T>,
            schema: Schema
        ) -> DispatchResult {
            ensure!(Nyms::<T>::contains_key(schema.issuer_id.clone()), Error::<T>::NymDontExist);
            ensure!(!Schemas::<T>::contains_key(schema.schema_id.clone()), Error::<T>::SchemaAlreadyExists);
            Schemas::<T>::insert(schema.schema_id.clone(), schema.clone());
            Self::deposit_event(Event::RegisteredSchema { schema_id: schema.schema_id, issuer_id: schema.issuer_id });
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight({0})]
        pub fn create_credential_definition(
            _origin: OriginFor<T>,
            cred_def: CredentialDefinition
        ) -> DispatchResult {
            ensure!(!CredentialDefinitions::<T>::contains_key(cred_def.cred_def_id.clone()), Error::<T>::CredentialDefinitionAlreadyExists);
            // TODO prowieric jesc li schema
            CredentialDefinitions::<T>::insert(cred_def.cred_def_id.clone(), cred_def.clone());
            Self::deposit_event(Event::RegisteredCredentialDefinition { cred_def_id: cred_def.cred_def_id, schema_id: cred_def.schema_id });
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight({0})]
        pub fn create_revocation_registry_definition(
            _origin: OriginFor<T>,
            rev_reg_def: RevocationRegistryDefinition
        ) -> DispatchResult {
            ensure!(!RevocationRegistryDefinitions::<T>::contains_key(rev_reg_def.rev_reg_def_id.clone()), Error::<T>::RevocationRegistryDefinitionAlreadyExists);
            // TODO prowieric jesc li schema
            RevocationRegistryDefinitions::<T>::insert(rev_reg_def.rev_reg_def_id.clone(), rev_reg_def.clone());
            Self::deposit_event(Event::RegisteredRevocationRegistryDefinition { rev_reg_def_id: rev_reg_def.rev_reg_def_id, cred_def_id: rev_reg_def.cred_def_id });
            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight({0})]
        pub fn create_revocation_list(
            _origin: OriginFor<T>,
            rev_list: RevocationList
        ) -> DispatchResult {
            ensure!(!RevocationLists::<T>::contains_key(rev_list.rev_reg_def_id.clone()), Error::<T>::RevocationListAlreadyExists);
            // TODO prowieric jesc li schema
            RevocationLists::<T>::insert(rev_list.rev_reg_def_id.clone(), rev_list.clone());
            Self::deposit_event(Event::RegisteredRevocationList { rev_reg_def_id: rev_list.rev_reg_def_id, issuer_id: rev_list.issuer_id });
            Ok(())
        }

        #[pallet::call_index(5)]
        #[pallet::weight({0})]
        pub fn write(_origin: OriginFor<T>, _args: Vec<u8>) -> DispatchResult {
//             let (head, body, _tail) = unsafe { args.align_to::<Request>() };
//             if head.is_empty() {
//                 //TODO
//                 ()
//             }
//             let req: &Request = &body[0];
//
//             let identifier = req.identifier.clone();
//
// //             let caller: Nym =  Nyms::<T>::get(request.identifier).ok_or(Error::<T>::NYMdontExist)?;
// //             let valid = Self::verifyRequest(nym.clone(), reqMsg, request.signature);
//
// //             let caller: Nym = match Self::authenticate(req.clone(), args){
// //                 Ok(c) => c,
// //                 Err(err) => return pallet::DispatchError::Module()
// //                 };
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {

//         fn _authenticate(request: Request, req_msg: Vec<u8>) -> Result<Nym, Error<T>> {
//             let nym: Nym =  Nyms::<T>::get(request.identifier).ok_or(Error::<T>::NymDontExist)?;
//             let valid = Self::_verify_request(&nym.ver_key, req_msg, request.signature);
//             ensure!(valid, Error::InvalidSignature);
// //             if !valid {
// //                 return Err(Error::InvalidSignature)
// //             }
//             Ok(nym)
//         }

//         fn _verify_request(verkey: &Vec<u8>, message: Vec<u8>, signature: Vec<u8>) -> bool {
//             let v: Vec<u8> = bs58::decode(verkey).into_vec().unwrap();
//             let public_key: [u8; 32] = v.try_into().unwrap();
//             let sig: Vec<u8> =  bs58::decode(signature).into_vec().unwrap();
//             let sig: [u8; 64] = sig.try_into().unwrap();
//             VerificationKey::try_from(public_key).and_then(|vk| vk.verify(&sig.into(), &message)).is_ok()
//         }

    }
}

