#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

mod local_peer_id;

pub use local_peer_id::fetch_n_parse_local_peerid as get_local_peer_id;
