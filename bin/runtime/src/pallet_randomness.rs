use frame_support::decl_error;
use frame_support::traits::Randomness;
use sp_runtime::offchain;
use sp_io::offchain_index;
pub use pallet::*;

decl_error! {
	pub enum Error {
		HttpFetchingError,
	}
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::dispatch::DispatchResult;
    use frame_system::Event;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn test(origin: OriginFor<T>) -> DispatchResult {
            let (_result, _res) = Self::random(&b"mycontext"[..]);
            Ok(())
        }
    }
}

impl<T: Config> Randomness<T::Hash, T::BlockNumber> for Pallet<T> {
    fn random(subject: &[u8]) -> (T::Hash, T::BlockNumber) {
        let timeout = sp_io::offchain::timestamp()
            .add(offchain::Duration::from_millis(5000));

        let request = offchain::http::Request::get("https://qrng.qbck.io/4148e4a4-ff17-4c77-a8a1-80d8bef3ea3b/qbck/block/bigint?size=1");

        let pending = request
            .deadline(timeout)
            .send()
            .map_err(|| <Error<>)?;

        todo!()
    }
}
