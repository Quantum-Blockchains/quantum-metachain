#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

use alloc::string::{String, ToString};

pub use pallet::*;
use serde::{Deserialize, Serialize};
use sp_core::Hasher;
use sp_io::offchain::timestamp;
use sp_runtime::{DispatchError, offchain::{http::Request, Duration}, SaturatedConversion, traits::Get};
use sp_core::{OpaquePeerId as PeerId, OpaquePeerId};
use sp_runtime::offchain::storage::StorageValueRef;
use sp_std::vec::Vec;
use sp_std::collections::btree_map::BTreeMap;
use frame_support::dispatch::DispatchResult;
use frame_support::ensure;
use frame_support::traits::Randomness;
use frame_system::offchain::{
    AppCrypto, CreateSignedTransaction, SendSignedTransaction,
    Signer, SignedPayload, SubmitTransaction
};
use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction};
use sp_core::{crypto::KeyTypeId};


#[cfg(test)]
mod tests;

const BLOCK_NUM_FOR_PSK_ROTATION: u64 = 60;
const BLOCK_NUM_FOR_STARTING_KEY_ROTATION: u64 = 10;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"demo");

pub mod crypto {
    use super::KEY_TYPE;
    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::Verify, MultiSignature, MultiSigner
    };
    app_crypto!(sr25519, KEY_TYPE);

    pub struct TestAuthId;

    // implemented for runtime
    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }
}

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
pub struct LocalPeerIdResponse {
    pub result: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct PskRotationRequest {
    peer_id: String,
    is_local_peer: bool,
    block_num: u64,
}

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::Randomness};
    use frame_system::offchain::{SendTransactionTypes, Signer};
    use frame_system::pallet_prelude::*;
    use sp_runtime::offchain::storage::StorageValueRef;

    use super::*;

    #[pallet::config]
    pub trait Config: CreateSignedTransaction<Call<Self>> + frame_system::Config + randao::Config + hypercube::Config {
        type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type RuntimeCall: From<Call<Self>>;
        type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

        // Max const value is u128 16 bytes, but entropy is u256 by default, hence we need to
        // concat two 16 bytes long slices to get proper difficulty
        #[pallet::constant]
        type PskDifficulty1: Get<u128>;
        #[pallet::constant]
        type PskDifficulty2: Get<u128>;

        #[pallet::constant]
        type UnsignedPriority: Get<TransactionPriority>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::type_value]
    pub(super) fn NumBlockForRestartDefault<T: Config>() -> u64 { 0u64 }
    #[pallet::storage]
    #[pallet::getter(fn num_block_for_restart)]
    pub(super) type NumBlockForRestart<T> = StorageValue<Value = u64, QueryKind = ValueQuery, OnEmpty = NumBlockForRestartDefault<T>>;

    #[pallet::storage]
    pub(super) type InBlock<T: Config> = StorageMap<_, Twox64Concat, u64, bool>;

    #[pallet::storage]
    pub(super) type SelectedPeers<T: Config> = StorageDoubleMap<_, Twox64Concat, u64, Twox64Concat, [u8; 52], [u8; 52]>;


    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
    where
        u64: From<<<<T as frame_system::Config>::Block as sp_runtime::traits::Block>::Header as sp_runtime::traits::Header>::Number>
    {
        /// PSK offchain worker entry point.
        fn offchain_worker(block_number: BlockNumberFor<T>) {

            log::info!("[OCW-PSK] Running PSK offchain worker...");

            let current_block_number: u64 = block_number.into();

            let storage_rpc_port = StorageValueRef::persistent(b"rpc-port");
            let rpc_port = match storage_rpc_port.get::<u16>() {
                Ok(p) => match p {
                    Some(port) => port,
                    None => {
                        log::error!("[OCW-PSK] The RPC port is not passed to the offchain worker.");
                        return;
                    }
                },
                Err(err) => {
                    log::error!(
                        "[OCW-PSK] Error occurred while fetching RPC port from storage. {:?}",
                        err
                    );
                    return;
                }
            };

            let storage_runner_port = StorageValueRef::persistent(b"runner-port");
            let runner_port = match storage_runner_port.get::<u16>() {
                Ok(p) => p.unwrap_or(5001),
                Err(err) => {
                    log::error!(
                        "[OCW-PSK] Error occurred while fetching runner port from storage. {:?}",
                        err
                    );
                    return;
                }
            };

            let block_number = NumBlockForRestart::<T>::get();

            if block_number == current_block_number {
                // Restart node
                match Self::send_restart_node_request(runner_port) {
                    Ok(()) => {
                        log::info!("[OCW-PSK] Restart node request sent");
                        return;
                    }
                    Err(err) => {
                        log::error!("[OCW-PSK] Failed to send restart node request. {:?}", err)
                    }
                };
            }
            else if block_number < current_block_number {
                if InBlock::<T>::contains_key(current_block_number) {
                    let tmp = match Self::start_rotation_key(current_block_number, runner_port, rpc_port) {
                        Ok(t) => t,
                        Err(err) => {
                            log::info!("[OCW-PSK] Error: {:?}", err);
                            return;
                        }
                    };
                    if !tmp {
                        Self::start_chosen_node(current_block_number, rpc_port);
                    }
                }
                else {
                    Self::start_chosen_node(current_block_number, rpc_port);
                }
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight({0})]
        pub fn submit_num_block_for_restart(origin: OriginFor<T>, num_block: u64) -> DispatchResultWithPostInfo {
            ensure_none(origin)?;
            Self::set_num_block_for_restart(num_block.clone());
            Ok(().into())
        }

        #[pallet::call_index(1)]
        #[pallet::weight({0})]
        pub fn submit_in_block(origin: OriginFor<T>, num_block: u64) -> DispatchResultWithPostInfo {
            ensure_none(origin)?;
            Self::set_in_block(num_block.clone());
            Ok(().into())
        }

        #[pallet::call_index(2)]
        #[pallet::weight({0})]
        pub fn submit_selected_peers(origin: OriginFor<T>, num_block: u64, peer_id: [u8; 52], selected_peer: [u8; 52]) -> DispatchResultWithPostInfo {
            ensure_none(origin)?;
            Self::set_selected_peers(num_block.clone(), peer_id, selected_peer);
            Ok(().into())
        }
    }

    #[pallet::event]
    pub enum Event<T: Config> {}

    #[pallet::error]
    pub enum Error<T> {
        HttpFetchingError,
        PortNumberFetchingError,
        TimeLineCheck,
        NumberBlockForRestartAlreadyExists,
        SelectedPeerIsAlreadyThere,
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        /// Validate unsigned call to this module.
        ///
        /// By default unsigned transactions are disallowed, but implementing the validator
        /// here we make sure that some particular calls (the ones produced by offchain worker)
        /// are being whitelisted and marked as valid.
        fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            let valid_tx = |provide| ValidTransaction::with_tag_prefix("my-pallet")
                .priority(TransactionPriority::max_value()) // please define `UNSIGNED_TXS_PRIORITY` before this line
                .and_provides([&provide])
                .longevity(3)
                .propagate(true)
                .build();
            match call {
                Call::submit_num_block_for_restart { num_block: current_block_number } => valid_tx(b"my_unsigned_tx1".to_vec()),
                Call::submit_in_block { num_block: current_block_number } => valid_tx(b"my_unsigned_tx2".to_vec()),
                Call::submit_selected_peers { num_block: current_block_number, peer_id: peer_id, selected_peer: selected_peer } => valid_tx(b"my_unsigned_tx3".to_vec()),
                _ => InvalidTransaction::Call.into(),
            }
        }
    }
}

impl<T: Config> Pallet<T> {

    fn get_current_block_num() -> u64 {
        let block = frame_system::Pallet::<T>::block_number();
        block.saturated_into::<u64>()
    }

    fn start_chosen_node(current_block_number: u64, rpc_port: u16) {
        let entropy = match <randao::Pallet<T>>::get_secret(current_block_number) {
            Ok(secret) => T::Hashing::hash(&secret.to_le_bytes()),
            Err(err) => {
                log::info!(
                    "[OCW-PSK] There is no random number for this block: {:?}",
                    err
                );
                let (ent, _) = T::Randomness::random(&b"PSK creator chosing"[..]);
                log::debug!("[OCW-PSK] Entropy in block {:?}: {:?}", current_block_number, ent);
                ent
            }
        };
        let peer_ids = <hypercube::Pallet<T>>::peers()
            .iter()
            .map(|peer| String::from_utf8(peer.0.to_vec()).expect("ASIA"))
            .collect();
        match Self::choose_psk_creator(entropy, peer_ids) {
            Some(psk_creator) => {
                let num_block_for_starting_key_rotation = current_block_number + BLOCK_NUM_FOR_STARTING_KEY_ROTATION;
                if !InBlock::<T>::contains_key(current_block_number) {
                    let call = crate::pallet::Call::submit_in_block { num_block: num_block_for_starting_key_rotation };
                    SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
                        .map_err(|_| {
                            log::error!("Failed in offchain_unsigned_tx");
                        });
                }
                let local_peer_id_bytes:[u8; 52] = match support::get_local_peer_id(rpc_port) {
                    Ok(id) => id.into_bytes().try_into().expect("[OCW-RANDAO] Vector length doesn't match the target array"),
                    Err(err) => {
                        log::error!("[OCW-PSK] Failed to retrieve local peer id. {:?}", err);
                        return;
                    }
                };
                let psk_creator_bytes: [u8; 52] = psk_creator
                    .into_bytes()
                    .try_into()
                    .expect("[OCW-RANDAO] Vector length doesn't match the target array");
                let call = crate::pallet::Call::submit_selected_peers { num_block: num_block_for_starting_key_rotation, peer_id: local_peer_id_bytes, selected_peer: psk_creator_bytes };
                SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
                    .map_err(|_| {
                        log::error!("Failed in offchain_unsigned_tx");
                    });
            }
            None => {
                log::info!(
                    "[OCW-PSK] Psk creator not chosen in block {:?}",
                    current_block_number
                )
            },
        }
    }

    /// This function records the block number for restarting nodes in the blockchain
    /// The resulting block number is checked.
    /// The block number must be greater than the current block number
    /// and the previous recorded number must be less than the one we are trying to record.
    fn set_num_block_for_restart(num_block: u64) -> Result<(), DispatchError> {
        let current_block_num: u64 = Self::get_current_block_num();
        ensure!(current_block_num < num_block, Error::<T>::TimeLineCheck);
        ensure!(NumBlockForRestart::<T>::get() < current_block_num, Error::<T>::NumberBlockForRestartAlreadyExists);
        NumBlockForRestart::<T>::set(num_block);
        Ok(())
    }

    /// This function writes to the blockchain the block number in which
    /// it is necessary to try to generate a new key.
    /// If num_block is less than or equal to the current number then returns an error
    fn set_in_block(num_block: u64) -> Result<(), DispatchError> {
        let current_block_num: u64 = Self::get_current_block_num();
        ensure!(current_block_num < num_block, Error::<T>::TimeLineCheck);
        if !InBlock::<T>::contains_key(num_block) {
            InBlock::<T>::insert(num_block, true);
        }
        Ok(())
    }

    /// This function records the selected peer for num_block
    /// if there was no recording before
    fn set_selected_peers(num_block: u64, peer_id: [u8; 52], selected_peer: [u8; 52]) -> Result<(), DispatchError> {
        ensure!(!SelectedPeers::<T>::contains_key(num_block, peer_id),Error::<T>::SelectedPeerIsAlreadyThere);
        if !SelectedPeers::<T>::contains_key(num_block, peer_id) {
            SelectedPeers::<T>::insert(num_block, peer_id, selected_peer);
        }
        Ok(())
    }

    fn get_ports() -> Result<(u16, u16), Error<T>>{
        let storage_rpc_port = StorageValueRef::persistent(b"rpc-port");
        let rpc_port = match storage_rpc_port.get::<u16>() {
            Ok(p) => match p {
                Some(port) => port,
                None => {
                    // The RPC port is not passed to the offchain worker
                    return Err(Error::PortNumberFetchingError)
                }
            },
            Err(err) => {
                // Error occurred while fetching RPC port from storage
                return Err(Error::PortNumberFetchingError)
            }
        };
        let storage_runner_port = StorageValueRef::persistent(b"runner-port");
        let runner_port = match storage_runner_port.get::<u16>() {
            Ok(p) => p.unwrap_or(5001),
            Err(err) => {
                // Error occurred while fetching runner port from storage
                return Err(Error::PortNumberFetchingError)
            }
        };
        Ok((runner_port, rpc_port))
    }

    fn start_rotation_key(current_block_number: u64, runner_port: u16, rpc_port: u16) -> Result<bool, Error<T>> {
        let selected_peers: Vec<([u8;52], [u8; 52])> = SelectedPeers::<T>::iter_prefix(current_block_number).collect();
        let peers = <hypercube::Pallet<T>>::peers().to_vec();
        if selected_peers.len() == 0 || selected_peers.len() < (peers.len() / 2) + 1 {
            return Ok(false)
        }
        // Couting of votes
        let mut map: BTreeMap<[u8; 52], u32> = BTreeMap::new();
        for item in selected_peers {
            if let Some(x) = map.get_mut(&item.1){
                *x = *x + 1;
            } else {
                map.insert(item.1, 1);
            }
        }
        // Finding the node with the most votes
        let mut tmp: u32 = 0;
        let mut  peer: [u8; 52] = [0; 52];
        for item in map {
            if item.1 > tmp {
                tmp = item.1;
                peer = item.0;
            }
        }
        if tmp < ((peers.len() / 2 ) + 1) as u32 {
            return Ok(false)
        }
        let num_block_restart = current_block_number + BLOCK_NUM_FOR_PSK_ROTATION;
        let local_peer_id = match support::get_local_peer_id(rpc_port) {
            Ok(id) => id,
            Err(err) => {
                log::error!("[OCW-PSK] Failed to retrieve local peer id. {:?}", err);
                return Err(Error::HttpFetchingError);
            }
        };
        let psk_creator = String::from_utf8(peer.to_vec()).unwrap();
        let request = PskRotationRequest {
            peer_id: psk_creator.to_string(),
            is_local_peer: psk_creator == local_peer_id,
            block_num: current_block_number,
        };
        log::debug!("[OCW-PSK] chosen psk creator: {:?}", request);
        match Self::send_psk_rotation_request(runner_port, request) {
            Ok(()) => {
                let call = crate::pallet::Call::submit_num_block_for_restart { num_block: num_block_restart };
                SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
                    .map_err(|_| {
                        log::error!("Failed in offchain_unsigned_tx");
                    });
                log::info!("[OCW-PSK] Psk rotation request sent")
            }
            Err(err) => {
                log::error!(
                    "[OCW-PSK] Failed to send psk rotation request. {:?}",
                    err
                )
            }
        };
        Ok(true)
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
            .map_err(|_| Error::HttpFetchingError)?;

        let response = pending
            .try_wait(timeout)
            .map_err(|_| Error::HttpFetchingError)?
            .map_err(|_| Error::HttpFetchingError)?;

        if response.code != 200 {
            log::error!(
                "[OCW-PSK] Unexpected http request status code: {}",
                response.code
            );
            return Err(Error::HttpFetchingError);
        }

        Ok(response.body().collect::<Vec<u8>>())
    }

    fn send_restart_node_request(runner_port: u16) -> Result<(), Error<T>> {
        let url = format!("http://localhost:{}/restart", runner_port);
        let request = Request::get(&url);
        let timeout = timestamp().add(Duration::from_millis(3000));
        let pending = request
            .deadline(timeout)
            .send()
            .map_err(|_| Error::HttpFetchingError)?;

        let response = pending
            .try_wait(timeout)
            .map_err(|_| Error::HttpFetchingError)?
            .map_err(|_| Error::HttpFetchingError)?;

        if response.code != 200 {
            log::error!(
                "[OCW-PSK] Unexpected http request status code: {}",
                response.code
            );
            return Err(Error::HttpFetchingError);
        }

        Ok(())
    }

    fn send_psk_rotation_request(runner_port: u16, request_body: PskRotationRequest) -> Result<(), Error<T>> {
        let url = format!("http://localhost:{}/psk", runner_port);

        let mut vec_body: Vec<&[u8]> = Vec::new();
        let data = serde_json::to_string(&request_body).unwrap();
        vec_body.push(data.as_bytes());

        let request = Request::post(&url, vec_body);
        let timeout = timestamp().add(Duration::from_millis(3000));

        let pending = request
            .add_header("Content-Type", "application/json")
            .deadline(timeout)
            .send()
            .map_err(|_| Error::HttpFetchingError)?;

        let response = pending
            .try_wait(timeout)
            .map_err(|_| Error::HttpFetchingError)?
            .map_err(|_| Error::HttpFetchingError)?;

        if response.code != 200 {
            log::error!(
                "[OCW-PSK] Unexpected http request status code: {}",
                response.code
            );
            return Err(Error::HttpFetchingError);
        }

        Ok(())
    }

    fn choose_psk_creator(entropy: T::Hash, peer_ids: Vec<String>) -> Option<String> {
        let mut chosen_peers = vec![];

        for peer_id in peer_ids {
            let xored_peer_id_hash = entropy ^ (T::Hashing::hash(peer_id.as_bytes()));
            let xored_peer_id_hash_bytes = <[u8; 32]>::try_from(xored_peer_id_hash.as_ref())
                .expect("[OCW-PSK] Hash should be 32 bytes long");
            let difficulty_1_bytes: [u8; 16] = T::PskDifficulty1::get().to_le_bytes();
            let difficulty_2_bytes: [u8; 16] = T::PskDifficulty2::get().to_le_bytes();
            let difficulty_bytes_extended =
                <[u8; 32]>::try_from([difficulty_1_bytes, difficulty_2_bytes].concat().as_ref())
                    .expect("[OCW-PSK] Difficulty should be 32 bytes long");

            if xored_peer_id_hash_bytes.gt(&difficulty_bytes_extended) {
                chosen_peers.push(peer_id);
            }
        }

        log::info!("[OCW-PSK] Chosen peers num: {}", chosen_peers.len());
        match chosen_peers.len() {
            0 => None,
            1 => Some(chosen_peers.first().unwrap().to_string()),
            _ => None,
        }
    }

    fn choose_psk_creator_test(entropy: T::Hash, peer_ids: Vec<[u8; 52]>) -> Option<[u8; 52]> {
        let mut chosen_peers = vec![];

        for peer_id in peer_ids {
            let xored_peer_id_hash = entropy ^ (T::Hashing::hash(peer_id.as_slice()));
            let xored_peer_id_hash_bytes = <[u8; 32]>::try_from(xored_peer_id_hash.as_ref())
                .expect("[OCW-PSK] Hash should be 32 bytes long");
            let difficulty_1_bytes: [u8; 16] = T::PskDifficulty1::get().to_le_bytes();
            let difficulty_2_bytes: [u8; 16] = T::PskDifficulty2::get().to_le_bytes();
            let difficulty_bytes_extended =
                <[u8; 32]>::try_from([difficulty_1_bytes, difficulty_2_bytes].concat().as_ref())
                    .expect("[OCW-PSK] Difficulty should be 32 bytes long");

            if xored_peer_id_hash_bytes.gt(&difficulty_bytes_extended) {
                chosen_peers.push(peer_id);
            }
        }

        log::info!("[OCW-PSK] Chosen peers num: {}", chosen_peers.len());
        match chosen_peers.len() {
            0 => None,
            1 => Some(*chosen_peers.first().unwrap()),
            _ => None,
        }
    }

}
