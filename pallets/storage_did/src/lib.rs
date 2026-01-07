#![cfg_attr(not(feature = "std"), no_std)]

// use ed25519_zebra::*;
// use bs58;
use frame_support::ensure;
pub use pallet::*;
use sp_std::vec::Vec;

#[frame_support::pallet]
pub mod pallet {

    use super::*;
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
    use frame_system::pallet_prelude::OriginFor;

    #[pallet::pallet]
    #[pallet::without_storage_info] //TODO zobaczyc co to powaduje
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::storage]
    pub(super) type Storage<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, Vec<u8>, OptionQuery>;

    #[pallet::error]
    pub enum Error<T> {
        DataAlreadyExists,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        SetData { id: Vec<u8> },
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight({0})]
        pub fn set_data(_origin: OriginFor<T>, did: Vec<u8>, data: Vec<u8>) -> DispatchResult {
            ensure!(
                !Storage::<T>::contains_key(did.clone()),
                Error::<T>::DataAlreadyExists
            );
            Storage::<T>::insert(did.clone(), data);
            Self::deposit_event(Event::SetData { id: did });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {}
}

