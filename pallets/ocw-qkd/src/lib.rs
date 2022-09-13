#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

use frame_support::traits::Randomness;
use sp_runtime::traits::Get;
pub use pallet::*;

use crate::Error::CannotGenerateKeyFromEntropy;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::Randomness};
    use frame_system::pallet_prelude::*;

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
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// QKD offchain worker entry point.
        fn offchain_worker(block_number: T::BlockNumber) {
            let amount_to_generate = Self::calculate_amount_to_generate();

            if amount_to_generate > 0 {
                log::debug!(
                "Block number: {:?} - generating {:?} single-use keys",
                &block_number,
                &amount_to_generate
            );
                let result = Self::generate_keys(amount_to_generate);
                if let Err(e) = result {
                    log::error!("Key generation failed: {:?}", e);
                }
            } else {
                log::debug!("Block number: {:?} - skipping keys generation, target keys amount fulfilled",
                    block_number);
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
    }

    #[pallet::storage]
    #[pallet::getter(fn key_by_hash)]
    pub(super) type KeyByHash<T> = StorageMap<_, Blake2_128Concat, u32, [u8; 32], ValueQuery>;
}

impl<T: Config> Pallet<T> {

    fn calculate_amount_to_generate() -> u32 {
        let keys_len =  Self::get_node_keys_len();
        T::TargetKeysAmount::get() - keys_len
    }

    fn get_node_keys_len() -> u32 {
        <u32>::try_from(<KeyByHash<T>>::iter_values().count()).unwrap()
    }

    fn generate_keys(amount: u32) -> Result<(), Error<T>> {
        for n in 0..amount {
            let (seed, _) = T::Randomness::random_seed();
            let key: [u8; 32] =
                <[u8; 32]>::try_from(seed.as_ref()).map_err(|_| CannotGenerateKeyFromEntropy)?;
            log::debug!("Random Key generated: {:?}", &key);
            <KeyByHash<T>>::mutate(|n, &mut keys_by_hash| n, keys_by_hash.insert(n, key));

            // <KeyByHash<T>>::insert(n, key);
        }
        let keys_len = Self::get_node_keys_len();
        Ok(())
    }
}
