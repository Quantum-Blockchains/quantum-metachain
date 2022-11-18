#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

// use frame_support::traits::Randomness;
pub use pallet::*;
use sp_io::offchain::timestamp;
use sp_runtime::offchain::{http::Request, Duration};
use sp_std::vec::Vec;
use alloc::string::String;
use crate::Error::HttpFetchingError;
use serde::{Deserialize, Deserializer};
// use sp_runtime::offchain::http::Response;

#[macro_use]
extern crate alloc;

#[derive(Deserialize)]
pub struct PeerInfoResponse {
    pub result: Vec<Vec<PeerInfoResult>>
}

#[derive(Deserialize, Debug)]
pub struct PeerInfoResult {
    pub peerId: String,
}
// use crate::Error::CannotGenerateKeyFromEntropy;

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
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// QKD offchain worker entry point.
        fn offchain_worker(_block_number: T::BlockNumber) {
            let storage_psk = StorageValueRef::persistent(b"pre-shared-key");
            let storage_rpc_port = StorageValueRef::persistent(b"rpc-port");
            let rpc_port = match storage_rpc_port.get::<u16>() {
                Ok(p) => match p {
                    Some(port) => port,
                    None => {
                        log::error!("The RPC port is not passed to the offchain worker.");
                        return;
                    }
                },
                Err(_err) => {
                    log::error!("The RPC port is not passed to the offchain worker.");
                    return;
                }
            };
            let new_psk = Self::generate_new_pre_shared_key();

            storage_psk.set(&new_psk);

            match Self::send_reqwest_save_new_psk(rpc_port) {
                Ok(_) => {
                    log::info!("The new pre-shared key is saved.");
                }
                Err(err) => {
                    log::error!("Error: {:?}", err);
                }
            }

            match Self::fetch_peers(rpc_port) {
                Ok(resp_body) => {
                    log::info!("Peers are fetched: {:?}", resp_body);
                }
                Err(err) => {
                    log::error!("Error: {:?}", err);
                }
            }

            match Self::fetch_n_parse(rpc_port) {
                Ok(res) => {
                    log::info!("Peers are parsed: {:?}", res);
                }
                Err(err) => {
                    log::error!("Error: {:?}", err);
                }
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    #[pallet::event]
    pub enum Event<T: Config> {}

    #[pallet::error]
    pub enum Error<T> {
        CannotGenerateKeyFromEntropy,
        HttpFetchingError,
    }
}

impl<T: Config> Pallet<T> {
    // TODO generate ne pre-shared key
    /// This function generates a new key.
    fn generate_new_pre_shared_key() -> &'static [u8; 64] {
        log::info!("A new pre-shared key is generated.");
        b"28617dff4efef20450dd5eafc060fd85faacca13d95ace3bda0be32e4694fcd7"
    }

    /// This function calls tje "psk_saveKey" RPC method, which writes a new key to the file.
    fn send_reqwest_save_new_psk(rpc_port: u16) -> Result<(), Error<T>> {
        let url = format!("http://localhost:{}", rpc_port);

        let mut vec_body: Vec<&[u8]> = Vec::new();
        let data = b"{\"id\": 1, \"jsonrpc\": \"2.0\", \"method\": \"psk_saveKey\"}";
        vec_body.push(data);

        let request = Request::post(&url, vec_body);
        let timeout = timestamp().add(Duration::from_millis(3000));

        let pending = request
            .add_header("Content-Type", "application/json")
            .deadline(timeout)
            .send()
            .map_err(|_| <Error<T>>::HttpFetchingError)?;

        let response = pending
            .try_wait(timeout)
            .map_err(|_| <Error<T>>::HttpFetchingError)?
            .map_err(|_| <Error<T>>::HttpFetchingError)?;

        if response.code != 200 {
            log::error!("Unexpected http request status code: {}", response.code);
            return Err(<Error<T>>::HttpFetchingError);
        }
        Ok(())
    }

    // Fetching peers test
    fn fetch_peers(rpc_port: u16) -> Result<Vec<u8>, Error<T>> {
        let url = format!("http://localhost:{}", rpc_port);

        let mut vec_body: Vec<&[u8]> = Vec::new();
        // https://polkadot.js.org/docs/substrate/rpc/#peers-vecpeerinfo
        let data = b"{\"id\": 1, \"jsonrpc\": \"2.0\", \"method\": \"system_peers\"}";
        vec_body.push(data);

        // Request::get takes only url arg
        let request = Request::post(&url, vec_body);
        let timeout = timestamp().add(Duration::from_millis(3000));

        let pending = request
            .add_header("Content-Type", "application/json")
            .deadline(timeout)
            .send()
            .map_err(|_| <Error<T>>::HttpFetchingError)?;

        let response = pending
            .try_wait(timeout)
            .map_err(|_| <Error<T>>::HttpFetchingError)?
            .map_err(|_| <Error<T>>::HttpFetchingError)?;

        if response.code != 200 {
            log::error!("Unexpected http request status code: {}", response.code);
            return Err(<Error<T>>::HttpFetchingError);
        }

        Ok(response.body().collect::<Vec<u8>>())
    }

    fn fetch_n_parse(rpc_port: u16) -> Result<Vec<Vec<PeerInfoResult>>, Error<T>> {
        let resp_bytes = Self::fetch_peers(rpc_port)
            .map_err(|e| {
                log::error!("fetch_from_remote error: {:?}", e);
                <Error<T>>::HttpFetchingError
            })?;
    

        let json_res: PeerInfoResponse = serde_json::from_slice(&resp_bytes).map_err(|_| <Error<T>>::HttpFetchingError)?;
        log::info!("Peer info vector has length {}", json_res.result.len());
        log::info!("Peer info vector: {:?}", json_res.result);
        
        Ok(json_res.result)
    }
}
