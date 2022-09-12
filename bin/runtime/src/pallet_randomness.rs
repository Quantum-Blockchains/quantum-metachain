use frame_support::decl_error;
use frame_support::traits::Randomness;
use sp_runtime::offchain;
use sp_io;
use frame_support::log::debug;
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::dispatch::DispatchResult;
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
        let block_number = <frame_system::Pallet<T>>::block_number();
        let timeout = sp_io::offchain::timestamp()
            .add(offchain::Duration::from_millis(5000));

        let request = offchain::http::Request::get("https://qrng.qbck.io/4148e4a4-ff17-4c77-a8a1-80d8bef3ea3b/qbck/block/bigint?size=1");

        let pending = match request
            .deadline(timeout)
            .send() {
            Ok(t) => t,
            Err(_err) => {
                debug!("Couldn't send request to QRNG");

                return (T::Hash::default(), block_number);
            }
        };

        let result = match pending.try_wait(timeout) {
            Ok(t) => t,
            Err(_err) => {
                debug!("Couldn't send request to QRNG");

                return (T::Hash::default(), block_number);
            }
        };
        let response = match result {
            Ok(t) => t,
            Err(_err) => {
                debug!("Couldn't send request to QRNG");

                return (T::Hash::default(), block_number);
            }
        };

        if response.code != 200 {
            debug!("Unexpected status code: {}", response.code);

            return (T::Hash::default(), block_number);
        }


        (T::Hash::default(), block_number)
    }
}
