#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

use alloc::string::{String, ToString};
use frame_support::traits::Randomness;
use sp_core::Hasher;

pub use pallet::*;
use serde::{Deserialize, Serialize};
use sp_io::offchain::timestamp;
use sp_runtime::offchain::{http::Request, Duration};
use sp_runtime::traits::Get;
use sp_std::vec::Vec;

#[macro_use]
extern crate alloc;

#[derive(Deserialize)]
pub struct PeerInfoResponse {
    pub result: Vec<PeerInfoResult>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PeerInfoResult {
    pub peer_id: String,
}

#[derive(Deserialize)]
pub struct LocalPeeridResponse {
    pub result: String,
}

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::{Randomness, EnsureOrigin}, dispatch::{DispatchResult, Output}};
    use frame_system::pallet_prelude::*;
    use sp_runtime::offchain::storage::StorageValueRef;

    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Call: From<Call<Self>>;
        type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

        // Max const value is u128 16 bytes, but entropy is u256 by default, hence we need to
        // concat two 16 bytes long slices to get proper difficulty
        #[pallet::constant]
        type PskDifficulty1: Get<u128>;
        #[pallet::constant]
        type PskDifficulty2: Get<u128>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// PSK offchain worker entry point.
        fn offchain_worker(block_number: T::BlockNumber) {
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

            let (entropy, _) = T::Randomness::random(&b"PSK creator chosing"[..]);
            log::info!("Entropy in block {:?}: {:?}", block_number, entropy);

            let mut peer_ids = match Self::fetch_n_parse_peers(rpc_port) {
                Ok(peers) => peers,
                Err(_err) => {
                    log::error!("Failed to retrieve peers");
                    return;
                }
            };

            let local_peer_id = match Self::fetch_n_parse_local_peerid(rpc_port) {
                Ok(id) => id,
                Err(_err) => {
                    log::error!("Failed to retrieve local peer id");
                    return;
                }
            };

            peer_ids.push(local_peer_id);
            Self::choose_psk_creator(entropy, peer_ids);
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    #[pallet::event]
    pub enum Event<T: Config> {}

    #[pallet::error]
    pub enum Error<T> {
        HttpFetchingError,
    }
}

impl<T: Config> Pallet<T> {
    fn fetch_peers(rpc_port: u16) -> Result<Vec<u8>, Error<T>> {
        let url = format!("http://localhost:{}", rpc_port);

        let mut vec_body: Vec<&[u8]> = Vec::new();
        let data = b"{\"id\": 1, \"jsonrpc\": \"2.0\", \"method\": \"system_peers\"}";
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

        Ok(response.body().collect::<Vec<u8>>())
    }

    fn fetch_n_parse_peers(rpc_port: u16) -> Result<Vec<String>, Error<T>> {
        let resp_bytes = Self::fetch_peers(rpc_port).map_err(|e| {
            log::error!("fetch_peers error: {:?}", e);
            <Error<T>>::HttpFetchingError
        })?;

        let json_res: PeerInfoResponse =
            serde_json::from_slice(&resp_bytes).map_err(|e: serde_json::Error| {
                log::error!("Parse peers error: {:?}", e);
                <Error<T>>::HttpFetchingError
            })?;

        Ok(json_res.result.iter()
            .map(|peer| peer.peer_id.to_string())
            .collect())
    }

    fn fetch_local_peerid(rpc_port: u16) -> Result<Vec<u8>, Error<T>> {
        let url = format!("http://localhost:{}", rpc_port);

        let mut vec_body: Vec<&[u8]> = Vec::new();
        let data = b"{\"id\": 1, \"jsonrpc\": \"2.0\", \"method\": \"system_localPeerId\"}";
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

        Ok(response.body().collect::<Vec<u8>>())
    }

    fn fetch_n_parse_local_peerid(rpc_port: u16) -> Result<String, Error<T>> {
        let resp_bytes = Self::fetch_local_peerid(rpc_port).map_err(|e| {
            log::error!("fetch_local_peerid error: {:?}", e);
            <Error<T>>::HttpFetchingError
        })?;

        let json_res: LocalPeeridResponse =
            serde_json::from_slice(&resp_bytes).map_err(|e: serde_json::Error| {
                log::error!("Parse local peerid error: {:?}", e);
                <Error<T>>::HttpFetchingError
            })?;

        Ok(json_res.result)
    }

    fn choose_psk_creator(
        entropy: T::Hash,
        mut peer_ids: Vec<String>,
    ) -> Option<String> {
        let mut chosen_peers = vec![];

        for peer_id in peer_ids {
            let xored_peer_id_hash = entropy ^ (T::Hashing::hash(peer_id.as_bytes()));
            let xored_peer_id_hash_bytes = <[u8; 32]>::try_from(xored_peer_id_hash.as_ref())
                .expect("Hash should be 32 bytes long");
            let difficulty_1_bytes: [u8; 16] = T::PskDifficulty1::get().to_le_bytes();
            let difficulty_2_bytes: [u8; 16] = T::PskDifficulty2::get().to_le_bytes();
            let difficulty_bytes_extended = <[u8; 32]>::try_from([difficulty_1_bytes, difficulty_2_bytes].concat().as_ref())
                .expect("Difficulty should be 32 bytes long");

            if xored_peer_id_hash_bytes.gt(&difficulty_bytes_extended) {
                chosen_peers.push(peer_id);
            }
        };

        log::info!("Chosen peers num: {}", chosen_peers.len());
        match chosen_peers.len() {
            0 => None,
            1 => Some(chosen_peers.first().unwrap().to_string()),
            _ => None
        }
    }
}
