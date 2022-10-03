#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

use frame_support::traits::Randomness;
pub use pallet::*;
use sp_runtime::traits::Get;
use sp_std::{
    str,
    collections::btree_map::BTreeMap,
    vec::Vec,
};
use codec::{Encode, Decode};
use sp_runtime::offchain::http::{Request, Response};
use sp_runtime::{
    offchain,
    RuntimeDebug,
};
use serde::{Deserialize, Deserializer};
use serde_json;
use serde_json::Value;
use serde_with::json;

use crate::Error::{FetchHttpRequestError, GenerateKeyFromEntropyError, ResponseDeserializeError};

#[derive(Deserialize, Encode, Decode, Default, RuntimeDebug, scale_info::TypeInfo)]
pub struct QkdKeyResponse {
    keys: Vec<QkdKey>,
}

#[derive(Deserialize, Encode, Decode, Default, RuntimeDebug, scale_info::TypeInfo)]
struct QkdKey {
    #[serde(deserialize_with = "de_string_to_bytes")]
    key_ID: Vec<u8>,
    #[serde(deserialize_with = "de_string_to_bytes")]
    key: Vec<u8>,
}

pub fn de_string_to_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
where
D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(de)?;
    Ok(s.as_bytes().to_vec())
}

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::Randomness};
    use frame_system::pallet_prelude::*;
    use sp_runtime::offchain::storage::StorageValueRef;

    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Call: From<Call<Self>>;
        type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

        #[pallet::constant]
        type TargetKeysAmount: Get<u32>;

        // #[pallet::constant]
        // type BaseQKDURL: Get<String>;
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// QKD offchain worker entry point.
        fn offchain_worker(block_number: T::BlockNumber) {
            let key  = match Self::generate_qkd_key() {
                Ok(t) => {
                    log::info!("{}", t['keys']);
                    t
                },
                Err(err) => {
                    log::error!("Couldn't generate QKD keys: {:?}", err);
                    return;
                }
            };
            let storage_persistent = StorageValueRef::persistent(b"ocw-qkd-storage");
            let temp_storage = &mut match storage_persistent.get::<BTreeMap<u8, [u8; 32]>>() {
                Ok(v) => match v {
                    Some(t) => t,
                    None => <BTreeMap<u8, [u8; 32]>>::default(),
                },
                Err(err) => {
                    log::error!("Couldn't get keys from local storage, {:?}", err);

                    return;
                }
            };
            let amount_to_generate = Self::calculate_amount_to_generate(temp_storage);

            if amount_to_generate > 0 {
                log::debug!(
                    "Block number: {:?} - generating {:?} single-use keys",
                    &block_number,
                    &amount_to_generate
                );
                let result = Self::generate_keys(temp_storage, amount_to_generate);
                if let Err(e) = result {
                    log::error!("Key generation failed: {:?}", e);
                }
            } else {
                log::debug!(
                    "Block number: {:?} - skipping keys generation, target keys amount fulfilled",
                    block_number
                );
            }

            storage_persistent.set(temp_storage);
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    #[pallet::event]
    pub enum Event<T: Config> {}

    #[pallet::error]
    pub enum Error<T> {
        GenerateKeyFromEntropyError,
        FetchHttpRequestError,
        ResponseDeserializeError,
    }
}

impl<T: Config> Pallet<T> {
    fn calculate_amount_to_generate(storage: &mut BTreeMap<u8, [u8; 32]>) -> u32 {
        let keys_len = Self::get_node_keys_len(storage);
        T::TargetKeysAmount::get() - keys_len
    }

    fn get_node_keys_len(storage: &mut BTreeMap<u8, [u8; 32]>) -> u32 {
        <u32>::try_from(storage.len()).unwrap()
    }

    fn generate_keys(storage: &mut BTreeMap<u8, [u8; 32]>, amount: u32) -> Result<(), Error<T>> {
        for n in 0..amount {
            let (seed, _) = T::Randomness::random_seed();
            let key: [u8; 32] =
                <[u8; 32]>::try_from(seed.as_ref()).map_err(|_| GenerateKeyFromEntropyError)?;
            log::debug!("Random Key generated: {:?}", &key);

            storage.insert(<u8>::try_from(n).unwrap(), key);
        }
        Ok(())
    }

    fn generate_qkd_key() -> Result<(Value), Error<T>> {
        let request = Request::get("http://193.28.230.244:8888/alice/enc_keys?size=256");

        let timeout = sp_io::offchain::timestamp()
            .add(offchain::Duration::from_millis(5_000));

        let pending = request
            .deadline(timeout) // Setting the timeout time
            .send() // Sending the request out by the host
            .map_err(|_| Error::<T>::FetchHttpRequestError)?;

        let response = pending
            .try_wait(timeout)
            .map_err(|_| Error::<T>::FetchHttpRequestError)?
            .map_err(|_| Error::<T>::FetchHttpRequestError)?;

        if response.code != 200 {
            log::error!("Unexpected http request status code: {}", response.code);
            return Err(FetchHttpRequestError);
        }

        let resp_bytes = response.body().collect::<Vec<u8>>();

        let resp_str = str::from_utf8(&resp_bytes).map_err(|_| GenerateKeyFromEntropyError)?;

        let key: Value = serde_json::from_str(&resp_str).map_err(|err| {
            log::error!("error from serde: {}", err);
            ResponseDeserializeError
        })?;

        Ok(key)
    }
}
