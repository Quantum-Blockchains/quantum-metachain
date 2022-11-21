#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

// use frame_support::traits::Randomness;
pub use pallet::*;
use sp_io::offchain::timestamp;
use sp_runtime::offchain::{http::Request, Duration};
use sp_std::vec::Vec;
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

            let mock_entropy = String::from("1100110001110111101111010010011111011111010101101110110101010001001000100101110101110000011010100000010101000000111101001101000111000011110110111101000011100100001110001111110000010000110010011010000011001011101000100100011100111000011000110011001010110110");

            match Self::fetch_n_parse(rpc_port) {
                Ok(res) => {
                    log::info!("Peers are parsed: {:?}", res);
                    match Self::choose_psk_creator(mock_entropy, res) {
                        Ok(_) => {
                            log::info!("The psk creator has been chosen");
                        }
                        Err(err) => {
                            log::error!("Error: {:?}", err);
                        } 
                    }
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

    fn fetch_n_parse(rpc_port: u16) -> Result<Vec<PeerInfoResult>, Error<T>> {
        let resp_bytes = Self::fetch_peers(rpc_port)
            .map_err(|e| {
                log::error!("fetch_peers error: {:?}", e);
                <Error<T>>::HttpFetchingError
            })?;
    

        let json_res: PeerInfoResponse = serde_json::from_slice(&resp_bytes).unwrap();

        Ok(json_res.result)
    }

    fn choose_psk_creator(entropy: String, peers: Vec<PeerInfoResult>) -> Result<(), Error<T>>  {
        let mut xored_ids: Vec<Vec<String, String>> = Vec::new();
        for peer in peers {
            // Peer id conversion to binary
            let mut p_id_bin = String::from("");
            for character in peer.peerId.clone().into_bytes() {
                p_id_bin += &format!("0{:b} ", character);
            }
            let p_id_bin_trim: String = p_id_bin.chars().filter(|c| !c.is_whitespace()).collect();

            let mut xored_p_id_vec = Vec::new();
            for (i, x) in entropy.chars().enumerate() {
                let p_n = p_id_bin_trim.chars().nth(i).unwrap().to_string().parse::<i32>().unwrap();
                let e_n = x.clone().to_string().parse::<i32>().unwrap();
                xored_p_id_vec.push((p_n ^ e_n).to_string());
            }
            let xored_p_id = xored_p_id_vec.join("");
        }
        Ok(())
    }
}
