#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use serde::Deserialize;
use sp_core::{Decode, Encode, Hasher};
use sp_io::offchain::timestamp;
use sp_runtime::offchain::{http::Request, Duration};
use sp_std::{str, vec::Vec};

use crate::Error::{DeserializeError, HttpFetchError};

#[derive(Deserialize, Encode, Decode, Default)]
struct QRNGResponseData {
    result: [u64; 1],
}

#[derive(Deserialize, Encode, Decode, Default)]
struct QRNGResponse {
    data: QRNGResponseData,
}

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::BlockNumberFor;
    use sp_runtime::offchain::storage::StorageValueRef;

    use super::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Call: From<Call<Self>>;
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
    where
        u64: From<<T as frame_system::Config>::BlockNumber>,
    {
        /// RANDAO offchain worker entry point.
        fn offchain_worker(block_number: T::BlockNumber) {
            log::info!(
                "Running RANDAO offchain worker in block: {:?}",
                block_number
            );
            let qrng_data = match Self::fetch_qrng_data() {
                Ok(qrng_data) => qrng_data,
                Err(err) => {
                    log::error!("Failed to fetch qrng data. {:?}", err);
                    return;
                }
            };
            let random_num = match Self::parse_qrng_data(&qrng_data) {
                Ok(random_num) => random_num.data.result[0],
                Err(err) => {
                    log::error!("Failed to parse qrng response. {:?}", err);
                    return;
                }
            };

            let current_block_number = block_number.encode();
            let storage_random_num = StorageValueRef::persistent(current_block_number.as_slice());
            storage_rundom_num.set(&random_num);

            let hashed_random_num = Self::hash_random_num(random_num);
            log::debug!("Random num: {:?}", random_num);
            log::info!("Hashed random num {:?}", &hashed_random_num);
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    #[pallet::event]
    pub enum Event<T: Config> {}

    #[pallet::error]
    pub enum Error<T> {
        HttpFetchError,
        DeserializeError,
    }
}

impl<T: Config> Pallet<T> {
    fn fetch_qrng_data() -> Result<Vec<u8>, Error<T>> {
        // TODO pass api key from config (JEQB-254)
        let request = Request::get("https://qrng.qbck.io/<api_key>/qbck/block/long?size=1");
        let timeout = timestamp().add(Duration::from_millis(5000));

        let pending = request
            .deadline(timeout)
            .send()
            .map_err(|_| HttpFetchError)?;

        let response = pending
            .try_wait(timeout)
            .map_err(|_| HttpFetchError)?
            .map_err(|_| HttpFetchError)?;

        if response.code != 200 {
            return Err(HttpFetchError);
        }

        let response_body_bytes = response.body().collect::<Vec<u8>>();
        Ok(response_body_bytes)
    }

    fn parse_qrng_data(qrng_data: &[u8]) -> Result<QRNGResponse, Error<T>> {
        let resp_str = str::from_utf8(qrng_data).map_err(|err| {
            log::error!(
                "Failed to deserialize qrng data: {:?} to string, err: {:?}",
                qrng_data,
                err
            );
            DeserializeError
        })?;
        let qrng_response: QRNGResponse = serde_json::from_str(resp_str).map_err(|err| {
            log::error!(
                "Failed to deserialize qrng data: {:?} to object, err: {:?}",
                resp_str,
                err
            );
            DeserializeError
        })?;
        Ok(qrng_response)
    }

    fn hash_random_num(num: u64) -> Vec<u8> {
        let data: [u8; 8] = num.to_le_bytes();
        let hashed_random_num = <T>::Hashing::hash(&data);
        hashed_random_num.encode()
    }
}
