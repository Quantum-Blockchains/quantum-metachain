#![cfg_attr(not(feature = "std"), no_std)]

use serde::Deserialize;
use sp_core::{Decode, Encode, Hasher};
use sp_io::offchain::timestamp;
use sp_runtime::offchain::{Duration, http::Request};
use sp_std::{str, vec::Vec};
use scale_info::prelude::string::String;
use codec::alloc::string::ToString;



pub use pallet::*;

use crate::Error::{DeserializeError, HttpFetchError};

#[derive(Deserialize, Encode, Decode, Default)]
struct QRNGResponse {
    result: Vec<u8>,
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

            let storage_qrng_api_url = StorageValueRef::persistent(b"qrng-api-url");

            let qrng_api_url = match storage_qrng_api_url.get::<Vec<u8>>() {
                Ok(Some(bytes)) => {
                    match String::from_utf8(bytes) {
                        Ok(url) => url,
                        Err(err) => {
                            log::error!("Failed to convert bytes to string: {:?}", err);
                            return;
                        }
                    }
                }
                Ok(None) => {
                    // Use default URL if not found in storage
                    "http://172.16.0.202:8085/qrng/hex?size=8".to_string()
                }
                Err(err) => {
                    log::error!(
            "Error occurred while fetching QRNG API url. {:?}",
            err
        );
                    return;
                }
            };


            let qrng_data = match Self::fetch_qrng_data(&qrng_api_url) {
                Ok(qrng_data) => qrng_data,
                Err(err) => {
                    log::error!("Failed to fetch qrng data. {:?}", err);
                    return;
                }
            };
            let random_num_vec = match Self::parse_qrng_data(&qrng_data) {
                Ok(qrng_data) => qrng_data.result,
                Err(err) => {
                    log::error!("Failed to parse qrng response. {:?}", err);
                    return;
                }
            };

            let current_block_number = block_number.encode();
            let storage_random_num = StorageValueRef::persistent(current_block_number.as_slice());

            let bytes: [u8; 8] = random_num_vec[0..8].try_into().unwrap();
            let random_num = u64::from_le_bytes(bytes);
            // let random_num = match Self::hex_string_to_u64(&random_num_vec) {
            //     Ok(random_num) => random_num,
            //     Err(err) => {
            //         log::error!("Failed to convert hex response to num. {:?}", err);
            //         return;
            //     }
            // };
            let hashed_random_num = Self::hash_random_num(random_num);
            storage_random_num.set(&random_num);

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
    fn fetch_qrng_data(qrng_api_url: &str) -> Result<Vec<u8>, Error<T>> {
        let request = Request::get(qrng_api_url);
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

    fn hex_string_to_u64(hex_string: &str) -> Result<u64, Error<T>> {
        // Convert the hex string to bytes
        let mut bytes = Vec::new();
        for i in (0..hex_string.len()).step_by(2) {
            let byte = u8::from_str_radix(&hex_string[i..i + 2], 16).map_err(|_| DeserializeError)?;
            bytes.push(byte);
        }

        // Convert the bytes to u64
        let mut array = [0u8; 8];
        array.copy_from_slice(&bytes[0..8]);
        let value = u64::from_be_bytes(array); // Use from_le_bytes for little-endian

        Ok(value)
    }
}
