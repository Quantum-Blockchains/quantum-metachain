#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

// use frame_support::traits::Randomness;
pub use pallet::*;
use sp_io::offchain::timestamp;
use sp_runtime::offchain::{http::Request, Duration};
use sp_std::vec::Vec;
use sp_std::num::ParseIntError;
use alloc::string::{String, ToString};
use crate::Error::HttpFetchingError;
use serde::Deserialize;
// use sp_runtime::offchain::http::Response;

#[macro_use]
extern crate alloc;

#[derive(Deserialize)]
pub struct PeerInfoResponse {
    pub result: Vec<PeerInfoResult>
}

#[derive(Deserialize, Debug)]
pub struct PeerInfoResult {
    pub peerId: String,
}

#[derive(Deserialize)]
pub struct LocalPeeridResponse {
    pub result: String
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

            // Mock entropy (256): 
            let mock_entropy = String::from("1100110001110111101111010010011111011111010101101110110101010001001000100101110101110000011010100000010101000000111101001101000111000011110110111101000011100100001110001111110000010000110010011010000011001011101000100100011100111000011000110011001010110110");

            let peers = match Self::fetch_n_parse_peers(rpc_port) {
                Ok(peers) => peers,
                Err(_err) => {
                    log::error!("Failed to retrieve peers");
                    return;
                        }
            };

            let local_id = match Self::fetch_n_parse_local_peerid(rpc_port) {
                Ok(id) => id,
                Err(_err) => {
                    log::error!("Failed to retrieve local peerid");
                    return;
                        } 
            };

            match Self::choose_psk_creator(mock_entropy, peers, local_id) {
                Ok(p) => {
                    log::info!("The psk creator has been chosen: {}", p);
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
        ParseIntError,
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

    fn fetch_n_parse_peers(rpc_port: u16) -> Result<Vec<PeerInfoResult>, Error<T>> {
        let resp_bytes = Self::fetch_peers(rpc_port)
            .map_err(|e| {
                log::error!("fetch_peers error: {:?}", e);
                <Error<T>>::HttpFetchingError
            })?;
    

        let json_res: PeerInfoResponse = serde_json::from_slice(&resp_bytes)
            .map_err(|e: serde_json::Error| {
                log::error!("Parse peers error: {:?}", e);
                <Error<T>>::HttpFetchingError
            })?;

        Ok(json_res.result)
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
        let resp_bytes = Self::fetch_local_peerid(rpc_port)
            .map_err(|e| {
                log::error!("fetch_local_peerid error: {:?}", e);
                <Error<T>>::HttpFetchingError
            })?;

        let json_res: LocalPeeridResponse = serde_json::from_slice(&resp_bytes)
            .map_err(|e: serde_json::Error| {
                log::error!("Parse local peerid error: {:?}", e);
                <Error<T>>::HttpFetchingError
            })?;

        Ok(json_res.result)
    }

    fn choose_psk_creator(entropy: String, mut peers: Vec<PeerInfoResult>, local_id: String) -> Result<String, Error<T>>  {
        // log::info!("Entropy: {}", entropy);
        // let mut xored_ids: Vec<_> = Vec::new();
        let local_peer = PeerInfoResult {
            peerId: local_id
        };
        peers.push(local_peer);
        let mut psk_generator = String::new();
        let mut psk_generator_xored = String::new();
        for peer in peers {
            // Peer id conversion to binary
            let mut p_id_bin = String::new();
            for character in peer.peerId.clone().into_bytes() {
                p_id_bin += &format!("0{:b} ", character);
            }
            let p_id_bin_trim: String = p_id_bin.chars().filter(|c| !c.is_whitespace()).collect();

            let mut xored_p_id_vec = Vec::new();
            for (i, x) in entropy.chars().enumerate() {
                let p_n: i32 = p_id_bin_trim.chars().nth(i).unwrap().to_string().parse()
                    .map_err(|e| {
                        log::error!("Peer bit error: {:?}", e);
                        <Error<T>>::ParseIntError
                    })?;
                let e_n: i32 = x.clone().to_string().parse()
                    .map_err(|e| {
                        log::error!("Entropy bit error: {:?}", e);
                        <Error<T>>::ParseIntError
                    })?;
                xored_p_id_vec.push((p_n ^ e_n).to_string());
            }
            let xored_p_id = xored_p_id_vec.join("");
            if psk_generator.clone().is_empty() || psk_generator_xored.clone().is_empty() {
                psk_generator = peer.peerId;
                psk_generator_xored = xored_p_id;
            } else {
                let psk_gen_xor_slice = &psk_generator_xored[(&psk_generator_xored.len() - 32)..psk_generator_xored.len()];
                let xor_pid_slice = &xored_p_id[(&xored_p_id.len() - 32)..xored_p_id.len()];
                let psk_gen_xor_intval = isize::from_str_radix(&psk_gen_xor_slice, 2).expect("psk_generator_xored_intval parsing failed");
                let xored_p_id_intval = isize::from_str_radix(&xor_pid_slice, 2).expect("xored_p_id_intval parsing failed");
                if xored_p_id_intval > psk_gen_xor_intval {
                    psk_generator = peer.peerId;
                    psk_generator_xored = xored_p_id;
        }
            }
        }
        // log::info!("Xored p_ids: {:#?}", xored_ids);

        Ok(psk_generator)
    }
}
