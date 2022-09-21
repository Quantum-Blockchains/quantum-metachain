#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

use frame_support::traits::Randomness;
use sp_runtime::offchain::{
    Duration,
    storage::{StorageRetrievalError, StorageValueRef},
    storage_lock::{BlockAndTime, StorageLock},
};
use sp_runtime::traits::Get;
pub use pallet::*;
use sp_std::collections::btree_map::BTreeMap;

use crate::Error::{CannotFetchFromLocalStorage, CannotGenerateKeyFromEntropy};

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

        #[pallet::constant]
        type TargetKeysAmount: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// QKD offchain worker entry point.
        fn offchain_worker(block_number: T::BlockNumber) {
            let storage_persistent = StorageValueRef::persistent(b"ocw-qkd-storage");
            let temp_storage = &mut match storage_persistent.get::<BTreeMap<u8, [u8; 32]>>() {
                Ok(v) => match v {
                    Some(t) => t,
                    None => {
                        <BTreeMap<u8, [u8; 32]>>::default()
                    },
                },
                Err(err) => {
                    log::error!("Couldn't get keys from local storage, {:?}", err);

                    return;
                }
            };



            let amount_to_generate = Self::calculate_amount_to_generate(temp_storage);

            if amount_to_generate > 0 {
                log::debug!(
                "Block number: {:?} - generating {:?} single-use keys",
                &block_number,
                &amount_to_generate
            );
                let result = Self::generate_keys(temp_storage, amount_to_generate);
                if let Err(e) = result {
                    log::error!("Key generation failed: {:?}", e);
                }
            } else {
                log::debug!("Block number: {:?} - skipping keys generation, target keys amount fulfilled",
                    block_number);
                }

            storage_persistent.set(temp_storage);
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    #[pallet::event]
    pub enum Event<T: Config> {}

    #[pallet::error]
    pub enum Error<T> {
        CannotGenerateKeyFromEntropy,
        CannotFetchFromLocalStorage,
    }
}

impl<T: Config> Pallet<T> {
    fn calculate_amount_to_generate(storage: &mut BTreeMap<u8, [u8; 32]>) -> u32 {
        let keys_len =  Self::get_node_keys_len(storage);
        T::TargetKeysAmount::get() - keys_len
    }

    fn get_node_keys_len(storage: &mut BTreeMap<u8, [u8; 32]>) -> u32 {
        <u32>::try_from(storage.len()).unwrap()
    }

    fn generate_keys(storage: &mut BTreeMap<u8, [u8; 32]>, amount: u32) -> Result<(), Error<T>> {
        for n in 0..amount {
            let (seed, _) = T::Randomness::random_seed();
            let key: [u8; 32] =
                <[u8; 32]>::try_from(seed.as_ref()).map_err(|_| CannotGenerateKeyFromEntropy)?;
            log::debug!("Random Key generated: {:?}", &key);

            storage.insert(<u8>::try_from(n).unwrap(), key);
        }
        Ok(())
    }
}
