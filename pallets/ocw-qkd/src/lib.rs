#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

use frame_support::traits::Randomness;
use sp_std::vec::Vec;
pub use pallet::*;
use crate::Error::CannotGenerateKeyFromEntropy;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_support::traits::Randomness;
    use frame_system::pallet_prelude::*;

    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Call: From<Call<Self>>;
        type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// QKD offchain worker entry point.
        fn offchain_worker(block_number: T::BlockNumber) {
            log::debug!(
                "Block number: {:?} - generating {:?} single-use keys",
                block_number,
                0
            );
            let result = Self::generate_keys(1);
            if let Err(e) = result {
                log::error!("Key generation failed: {:?}", e);
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
    fn get_node_keys_len() -> usize {
        <KeyByHash<T>>::iter_values().collect::<Vec<_>>().len()
    }

    fn generate_keys(amount: u32) -> Result<(), Error<T>> {
        for n in 0..amount {
            let (seed, _) = T::Randomness::random_seed();
            let key: [u8; 32] = <[u8; 32]>::try_from(seed.as_ref())
                .map_err(|_| CannotGenerateKeyFromEntropy)?;
            log::debug!("Random Key generated: {:?}", &key);
            <KeyByHash<T>>::insert(n, key);
        }
        Ok(())
    }
}
