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
struct QRNGResponse {
    data: Vec<u32>,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::dispatch::DispatchResult;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {}

    #[pallet::error]
    pub enum Error<T> {
        HttpFetchError,
        DeserializeError,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn test(origin: OriginFor<T>) -> DispatchResult {
            let (_result, _res) = Self::random(&b"mycontext"[..]);
            Ok(())
        }
    }
}

impl<T: Config> Randomness<T::Hash, T::BlockNumber> for Pallet<T> {
    fn random(subject: &[u8]) -> (T::Hash, T::BlockNumber) {
        debug!("Starting generating random number");
        let block_number = <frame_system::Pallet<T>>::block_number();

        let qrng_data = match fetch_qrng_data() {
            Ok(t) => t,
            Err(_err) => return (T::Hash::default(), block_number)
        };
        
        (T::Hash::default(), block_number)
    }
}

fn fetch_qrng_data() -> Result<QRNGResponse, Error<T>> {
    let request = offchain::http::Request::get("https://qrng.qbck.io/4148e4a4-ff17-4c77-a8a1-80d8bef3ea3b/qbck/block/bigint?size=1");
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
        return Err(<Error<T>>::HttpFetchError);
    }

    let response_body_bytes = response.body().collect::<Vec<u8>>();
    let response_body_string = str::from_utf8(&response_body_bytes)?;

    let qrng_result = serde_json::from_str(&response_body_string).map_err(|_| DeserializeError)?;

    return qrng_result;
}
