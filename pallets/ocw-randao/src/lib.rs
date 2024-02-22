#![cfg_attr(not(feature = "std"), no_std)]

use codec::alloc::string::ToString;
pub use pallet::*;
use scale_info::prelude::string::String;
use serde::{Deserialize, Deserializer};
use sp_core::{Decode, Encode, Hasher};
use sp_io::offchain::timestamp;
use sp_runtime::offchain::{http::Request, storage::StorageValueRef, Duration};
use sp_std::{str, vec::Vec};

use crate::Error::{DeserializeError, GetNumberQRNGError, HttpFetchError};

const ONCHAIN_COMMITS: &[u8] = b"ocw-randao::commits";
const ONCHAIN_REVEALS: &[u8] = b"ocw-randao::reveals";

#[derive(Debug, Deserialize, Encode, Decode, Default)]
struct CommitData(u64, [u8; 32]);

#[derive(Debug, Deserialize, Encode, Decode, Default)]
struct RevealData(u64, u64);

#[derive(Deserialize, Encode, Decode, Default)]
struct QRNGResponse {
    #[serde(deserialize_with = "de_string_to_bytes")]
    result: Vec<u8>,
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
    use frame_system::pallet_prelude::BlockNumberFor;

    use super::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config + randao::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type RuntimeCall: From<Call<Self>>;
        type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
    where
        u64: From<<<<T as frame_system::Config>::Block as sp_runtime::traits::Block>::Header as sp_runtime::traits::Header>::Number>
    {
        /// RANDAO offchain worker entry point.
        fn offchain_worker(block_number: BlockNumberFor<T>) {
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

            let random_num = match Self::get_random_numner_from_qrng() {
                Ok(secret) => secret,
                Err(err) => {
                    log::error!("[OCW-RANDAO] Failed to get qrng rundom number. {:?}", err);
                    let (random_seed, _) = T::Randomness::random(&b"PSK creator chosing"[..]);
                    let random_number = <u64>::decode(&mut random_seed.as_ref())
                        .expect("[OCW-RANDAO] secure hashes should always be bigger than u32; qed");
                    random_number
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
        GetNumberQRNGError,
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

    fn get_random_numner_from_qrng() -> Result<u64, Error<T>> {
        let storage_qrng_api_url = StorageValueRef::persistent(b"qrng-api-url");
        let qrng_api_url = match storage_qrng_api_url.get::<_>() {
            Ok(Some(bytes)) => match String::from_utf8(bytes) {
                Ok(url) => url,
                Err(err) => {
                    log::error!("[OCW-RANDAO] Failed to convert bytes to string: {:?}", err);
                    return Err(GetNumberQRNGError);
                }
            },
            Ok(None) => "".to_string(),
            Err(err) => {
                log::error!(
                    "[OCW-RANDAO] Error occurred while fetching RPC port from storage. {:?}",
                    err
                );
                return Err(GetNumberQRNGError);
            }
        };

        let qrng_data = match Self::fetch_qrng_data(&qrng_api_url) {
            Ok(qrng_data) => qrng_data,
            Err(err) => {
                log::error!("[OCW-RANDAO] Failed to fetch qrng data. {:?}", err);
                return Err(GetNumberQRNGError);
            }
        };

        let random_num_vec = match Self::parse_qrng_data(&qrng_data) {
            Ok(qrng_data) => qrng_data.result,
            Err(err) => {
                log::error!("[OCW-RANDAO] Failed to parse qrng response. {:?}", err);
                return Err(GetNumberQRNGError);
            }
        };

        let bytes: [u8; 8] = random_num_vec[0..8].try_into().unwrap();
        let random_num = u64::from_le_bytes(bytes);
        Ok(random_num)
    }

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
