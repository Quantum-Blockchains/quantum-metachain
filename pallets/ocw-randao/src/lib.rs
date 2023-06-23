#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use serde::Deserialize;
use sp_core::{Decode, Encode, Hasher};
use sp_io::offchain::timestamp;
use sp_runtime::offchain::{http::Request, Duration};
use sp_std::{str, vec::Vec};

use crate::Error::{DeserializeError, HttpFetchError};

const ONCHAIN_COMMITS: &[u8] = b"ocw-randao::commits";
const ONCHAIN_REVEALS: &[u8] = b"ocw-randao::reveals";

#[derive(Debug, Deserialize, Encode, Decode, Default)]
struct CommitData(u64, [u8; 32]);

#[derive(Debug, Deserialize, Encode, Decode, Default)]
struct RevealData(u64, u64);

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
    pub trait Config: frame_system::Config + randao::Config {
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
                "[OCW-RANDAO] Running offchain worker in block: {:?}",
                block_number
            );

            let storage_rpc_port = StorageValueRef::persistent(b"rpc-port");
            let rpc_port = match storage_rpc_port.get::<u16>() {
                Ok(p) => match p {
                    Some(port) => port,
                    None => {
                        log::error!(
                            "[OCW-RANDAO] The RPC port is not passed to the offchain worker."
                        );
                        return;
                    }
                },
                Err(err) => {
                    log::error!(
                        "[OCW-RANDAO] Error occurred while fetching RPC port from storage. {:?}",
                        err
                    );
                    return;
                }
            };

            let local_peer_id = match support::get_local_peer_id(rpc_port) {
                Ok(id) => id.into_bytes(),
                Err(err) => {
                    log::error!("[OCW-RANDAO] Failed to retrieve local peer id. {:?}", err);
                    return;
                }
            };

            let local_peer_id_bytes: [u8; 52] = local_peer_id
                .try_into()
                .expect("[OCW-RANDAO] Vector length doesn't match the target array");

            let qrng_data = match Self::fetch_qrng_data() {
                Ok(qrng_data) => qrng_data,
                Err(err) => {
                    log::error!("[OCW-RANDAO] Failed to fetch qrng data. {:?}", err);
                    return;
                }
            };
            let random_num = match Self::parse_qrng_data(&qrng_data) {
                Ok(random_num) => random_num.data.result[0],
                Err(err) => {
                    log::error!("[OCW-RANDAO] Failed to parse qrng response. {:?}", err);
                    return;
                }
            };

            let hashed_random_num = Self::hash_random_num(random_num);

            let block_num: u64 = block_number.into();
            let block_number_for_commit = block_num + 2;
            let block_num_for_reveal = block_num + 6;

            let mut key_for_commit = Self::derived_key(block_number_for_commit, ONCHAIN_COMMITS);
            let mut key_for_reveal = Self::derived_key(block_num_for_reveal, ONCHAIN_REVEALS);

            let mut storage_ref_com = StorageValueRef::persistent(&key_for_commit);
            let mut storage_ref_rev = StorageValueRef::persistent(&key_for_reveal);

            let data_for_commit = CommitData(block_num + 10, hashed_random_num);
            let data_for_reveal = RevealData(block_num + 10, random_num);
            storage_ref_com.set(&data_for_commit);
            storage_ref_rev.set(&data_for_reveal);

            key_for_commit = Self::derived_key(block_num, ONCHAIN_COMMITS);
            key_for_reveal = Self::derived_key(block_num, ONCHAIN_REVEALS);

            storage_ref_com = StorageValueRef::persistent(&key_for_commit);
            storage_ref_rev = StorageValueRef::persistent(&key_for_reveal);

            if let Ok(Some(data)) = storage_ref_com.get::<CommitData>() {
                match <randao::Pallet<T>>::commit_and_raw_unsigned(
                    local_peer_id_bytes,
                    data.0,
                    data.1,
                ) {
                    Ok(_) => {}
                    Err(err) => log::info!(
                        "[OCW-RANDAO] Commit hash of random number failed: {:?}",
                        err
                    ),
                }
            }

            if let Ok(Some(data)) = storage_ref_rev.get::<RevealData>() {
                match <randao::Pallet<T>>::reveal_and_raw_unsigned(
                    local_peer_id_bytes,
                    data.0,
                    data.1,
                ) {
                    Ok(_) => {}
                    Err(err) => log::info!("[OCW-RANDAO] Reveal random number failed: {:?}", err),
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
        HttpFetchError,
        DeserializeError,
    }
}

impl<T: Config> Pallet<T> {
    #[deny(clippy::clone_double_ref)]
    fn derived_key(block_number: u64, prefix: &[u8]) -> Vec<u8> {
        block_number.using_encoded(|encoded_bn| {
            prefix
                .iter()
                .chain(b"/".iter())
                .chain(encoded_bn)
                .copied()
                .collect::<Vec<u8>>()
        })
    }

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
                "[OCW-RANDAO] Failed to deserialize qrng data: {:?} to string, err: {:?}",
                qrng_data,
                err
            );
            DeserializeError
        })?;
        let qrng_response: QRNGResponse = serde_json::from_str(resp_str).map_err(|err| {
            log::error!(
                "[OCW-RANDAO] Failed to deserialize qrng data: {:?} to object, err: {:?}",
                resp_str,
                err
            );
            DeserializeError
        })?;
        Ok(qrng_response)
    }

    fn hash_random_num(num: u64) -> [u8; 32] {
        let data: [u8; 8] = num.to_le_bytes();
        let hashed_random_num = <T>::Hashing::hash(&data);
        Self::vec_to_bytes_array(hashed_random_num.encode())
    }

    fn vec_to_bytes_array(vec: Vec<u8>) -> [u8; 32] {
        vec.try_into()
            .expect("[OCW-RANDAO] Vector length doesn't match the target array")
    }
}
