use frame_support::dispatch::DispatchResultWithPostInfo;
use frame_support::sp_tracing::event::Event;
use frame_support::storage::child::put;
use frame_support::traits::Randomness;
use frame_system::ensure_signed;
use frame_system::pallet_prelude::OriginFor;

#[pallet::config]
pub trait Config: frame_system::Config {
    type MyRandomness: Randomness<Self::Hash, Self::BlockNumber>; // qRNG?

}

pub struct Pallet<T: Config>;

impl <T: Config> Pallet<T> {
    fn get_and_increment_nonce() -> Vec<u8> {
        let nonce = Nonce::<T>::get();
        Nonce::<T>::put(nonce.wrapping_add(1));
        nonce.encode()
    }

    #[pallet::weight(100)]
    pub fn create_unique(
        origin: OriginFor<T>)
        -> DispatchResultWithPostInfo {
        // Account calling this dispatchable.
        let _sender = ensure_signed(origin)?;
        // Random value.
        let nonce = Self::get_and_increment_nonce();
        let (randomValue, _) = T::MyRandomness::random(&nonce);
        // Write the random value to storage.
        <MyStorageItem<T>>::put(randomValue);
        Self::deposit_event(Event::UniqueCreated(randomValue));
    }
}


