#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{str, vec::Vec};

pub use pallet::*;
use log;
use frame_support::{BoundedSlice, BoundedVec};
use sp_std::collections::vec_deque::VecDeque;
use sp_core::{OpaquePeerId as PeerId, OpaquePeerId};
use sp_api::decl_runtime_apis;
use scale_info::prelude::format;

const LOG_TARGET: &str = "rubtime::hypercube";

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_core::OpaquePeerId;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[pallet::constant]
        type MaxPeerIdLength: Get<u32>;
        #[pallet::constant]
        type MaxPeers: Get<u32>;
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    // #[pallet::storage]
    // pub(super) type PeersCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    // #[pallet::storage]
    // pub(super) type Peers<T: Config> = StorageMap<_, Twox64Concat, u64, [u8; 52]>;

    #[pallet::storage]
    #[pallet::getter(fn peers)]
    pub(super) type Peers<T: Config> = StorageValue<_, BoundedVec<PeerId, T::MaxPeers>, ValueQuery>;

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub peers: Vec<PeerId>,
        pub _config: sp_std::marker::PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            Pallet::<T>::initialize_peers(&self.peers);
        }
    }

    #[pallet::error]
    pub enum Error<T> {
        BoundsOverflow,
        NotFindPeer,
        TooManyPeers,
        PeerIdTooLong,
        AlreadyJoined,
        FailedAddPeer,

    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AddNewPeer { peer: PeerId },
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {

        #[pallet::weight(0)]
        pub fn add_new_peer(origin: OriginFor<T>, peer: PeerId) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            Self::add_peer(peer)?;
            Ok(())
        }

    }

    impl<T: Config> Pallet<T> {

        pub fn initialize_peers(peers: &[PeerId]) {
            if !peers.is_empty() {
                assert!(<Peers<T>>::get().is_empty(), "Authorities are already initialized!");
                let bounded = <BoundedSlice<'_, _, T::MaxPeers>>::try_from(peers)
                    .expect("Initial authority set must be less than T::MaxAuthorities");
                <Peers<T>>::put(bounded);
            }
        }

        pub fn add_peer(peer: PeerId) -> Result<(), DispatchError>{
            let mut peers = Self::peers();

            ensure!(peer.0.len() < T::MaxPeerIdLength::get() as usize, Error::<T>::PeerIdTooLong);
            ensure!(peers.len() < T::MaxPeers::get() as usize, Error::<T>::TooManyPeers);
            ensure!(!peers.contains(&peer), Error::<T>::AlreadyJoined);
            peers.try_push(peer.clone());

            <Peers<T>>::put(peers);
            Self::deposit_event(Event::AddNewPeer {peer: peer.clone()});
            Ok(())
        }

        pub fn get_links(peer: Vec<u8>) -> Result<Vec<PeerId>, DispatchError> {
            let d = OpaquePeerId::new(peer);
            let peers = Self::peers();
            ensure!(peers.contains(&d), Error::<T>::NotFindPeer);
            let mut peers_to_connect: Vec<PeerId> = Vec::new();
            let mut num_of_peer: usize = 0;
            for i in 0..peers.len() {
                if peers.get(i).unwrap().clone() == d {
                    num_of_peer = i;
                    break;
                }
            }
        //     // let peers: Vec<(u64, [u8; 52])> = Peers::<T>::iter().collect();
        //
        //     // let num_of_peer = match peers.iter().find(|&&x| x.1 == peer){
        //     //     Some(item) => item.0,
        //     //     None => return Err(Error::<T>::NotFindPeer.into())
        //     // };
        //
            for i in 0..peers.len() {
                let result_xor = i ^ num_of_peer;
                let bin_str = format!("{:08b}", result_xor);
                let count = bin_str.chars().filter(|&c| c == '1').count();
                if count == 1 {
                    let d = peers.get(i).unwrap().clone();
                    peers_to_connect.push(PeerId::new(d.0));
                }
            }
        //
            Ok(peers_to_connect)
        }
    }



}

decl_runtime_apis! {
	pub trait HypercubeApi {
		fn peers() -> Vec<PeerId>;
        fn links(peer: Vec<u8>) -> Vec<PeerId>;
	}
}