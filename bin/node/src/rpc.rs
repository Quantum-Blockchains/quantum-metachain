//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

//TODO
// #![warn(missing_docs)]

use jsonrpsee::RpcModule;
use sc_client_db::offchain::LocalStorage;
pub use sc_rpc_api::DenyUnsafe;
use sc_service::config::NetworkConfiguration;

/// Full client dependencies.
pub struct FullDeps {
    /// Network configuration
    pub config: NetworkConfiguration,
    /// Offchain storage
    pub storage: LocalStorage,
}

/// Instantiate all full RPC extensions.
pub fn create_full(
    deps: FullDeps,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>> {
    use crate::psk_rpc::{Psk, PskApiServer};

    let mut module = RpcModule::new(());
    let FullDeps { config, storage } = deps;

    module.merge(PskApiServer::into_rpc(Psk::new(config, storage)))?;

    Ok(module)
}
