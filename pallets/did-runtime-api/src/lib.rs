#![cfg_attr(not(feature = "std"), no_std)]

use sp_api::decl_runtime_apis;
use sp_std::vec::Vec;

use did::DidDetails;

decl_runtime_apis! {
    pub trait DidRuntimeApi {
        fn did_by_string(did: Vec<u8>) -> Option<DidDetails>;
    }
}
