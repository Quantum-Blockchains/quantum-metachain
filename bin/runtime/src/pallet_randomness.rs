use frame_support::traits::Randomness;
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::dispatch::DispatchErrorWithPostInfo;
    use frame_system::Event;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(T::WeightInfo::transfer())]
        pub fn generate_qrng(
            origin: OriginFor<T>,
        ) -> DispatchErrorWithPostInfo {
            Ok(().into())
        }
    }
}

impl<T: Config> Randomness<T::Hash, T::BlockNumber> for Pallet<T> {
    fn random(subject: &[u8]) -> (T::Hash, T::BlockNumber) {
        todo!()
    }
}
