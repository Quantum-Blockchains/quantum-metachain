//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

use jsonrpsee::RpcModule;
use sc_client_db::offchain::LocalStorage;
pub use sc_rpc_api::DenyUnsafe;
use sc_service::config::NetworkConfiguration;
use qmc_runtime::{opaque::Block, AccountId, Balance, Index, BlockNumber, Hash};
use std::sync::Arc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sc_client_api::AuxStore;

/// Full client dependencies.
pub struct FullDeps<C> {
    pub client: Arc<C>,
    /// Network configuration
    pub config: NetworkConfiguration,
    /// Offchain storage
    pub storage: LocalStorage,
}

/// Instantiate all full RPC extensions.
pub fn create_full<C>(
    deps: FullDeps<C>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
    where
        C: ProvideRuntimeApi<Block>
		    + sc_client_api::BlockBackend<Block>
		    + HeaderBackend<Block>
		    + AuxStore
		    + HeaderMetadata<Block, Error = BlockChainError>
		    + Sync
		    + Send
		    + 'static,
        C::Api: pallet_contracts_rpc::ContractsRuntimeApi<Block, AccountId, Balance, BlockNumber, Hash>,
{
    use crate::psk_rpc::{Psk, PskApiServer};
    use pallet_contracts_rpc::{Contracts, ContractsApiServer};

    let mut module = RpcModule::new(());
    let FullDeps { client, config, storage } = deps;

    module.merge(PskApiServer::into_rpc(Psk::new(config, storage)))?;

    module.merge(Contracts::new(client.clone()).into_rpc())?;

    Ok(module)
}
