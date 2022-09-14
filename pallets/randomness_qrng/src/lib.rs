#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::Randomness;
use frame_support::log::debug;
use sp_runtime::offchain;
use sp_io;
use sp_std::vec::Vec;
use sp_std::str;
use sp_core::{Decode, Encode};
pub use pallet::*;
use serde::Deserialize;
use crate::Error::{DeserializeError, HttpFetchError};

#[derive(Deserialize, Encode, Decode, Default)]
struct QRNGResponseData {
    result: [[u8;16];2],
}

#[derive(Deserialize, Encode, Decode, Default)]
struct QRNGResponse {
    data: QRNGResponseData,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config {}

    #[pallet::error]
    pub enum Error<T> {
        HttpFetchError,
        DeserializeError,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {}
}

impl<T: Config> Randomness<T::Hash, T::BlockNumber> for Pallet<T> {
    fn random(subject: &[u8]) -> (T::Hash, T::BlockNumber) {
        debug!("Starting generating random number");
        let block_number = <frame_system::Pallet<T>>::block_number();

        let qrng_data = match Self::fetch_qrng_data() {
            Ok(t) => t,
            Err(_err) => return (T::Hash::default(), block_number)
        };

        (T::Hash::default(), block_number)
    }
}

impl<T: Config> Pallet<T> {
    fn fetch_qrng_data() -> Result<Vec<u8>, Error<T>> {
        let request = offchain::http::Request::get("https://qrng.qbck.io/4148e4a4-ff17-4c77-a8a1-80d8bef3ea3b/qbck/block/hex?size=2");
        let timeout = sp_io::offchain::timestamp()
            .add(offchain::Duration::from_millis(5000));

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

    fn parse_qrng_data() -> Result<QRNGResponse, Error<T>> {
        let resp_bytes = Self::fetch_qrng_data().map_err(|e| {
            log::error!("fetch_from_remote error: {:?}", e);
            DeserializeError
        })?;
        let resp_str = str::from_utf8(&resp_bytes).map_err(|_| DeserializeError)?;

        let qrng_response: QRNGResponse =
            serde_json::from_str(resp_str).map_err(|_| DeserializeError)?;
        Ok(qrng_response)
    }

}
