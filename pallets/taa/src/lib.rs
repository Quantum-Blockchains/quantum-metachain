#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::vec::Vec;
pub use pallet::*;
use frame_support::ensure;

#[frame_support::pallet]
pub mod pallet {

    use super::*;
    use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
    use frame_support::dispatch::DispatchResult;
    use frame_system::ensure_signed;

    #[pallet::pallet]
    #[pallet::without_storage_info] //TODO zobaczyc co to powaduje
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::storage]
    pub(super) type Taa<T: Config> = StorageValue<Value = Vec<u8>, QueryKind = OptionQuery>;

    #[pallet::storage]
    pub(super) type AcceptedTaa<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, bool, OptionQuery>;

    #[pallet::error]
    pub enum Error<T> {
        TaaNotSet,
        AlreadyAccepted,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        TaaUpdated { taa: Vec<u8> },
        TaaAccepted { account_id: T::AccountId }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {

        #[pallet::call_index(0)]
        #[pallet::weight({0})]
        pub fn set_taa(
            origin: OriginFor<T>,
            taa: Vec<u8>,
        ) -> DispatchResult {
            let _admin = ensure_signed(origin)?;
            Taa::<T>::set(Some(taa.clone()));
            Self::deposit_event(Event::TaaUpdated { taa: taa });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight({0})]
        pub fn accept_taa(
            origin: OriginFor<T>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(Taa::<T>::exists(), Error::<T>::TaaNotSet);
            ensure!(!AcceptedTaa::<T>::contains_key(&sender), Error::<T>::AlreadyAccepted);
            AcceptedTaa::<T>::insert(&sender, true);
            Self::deposit_event(Event::TaaAccepted { account_id: sender });
            Ok(())
        }
    }
}

