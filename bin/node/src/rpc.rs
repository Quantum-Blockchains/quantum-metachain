//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.
use std::sync::Arc;

use jsonrpsee::RpcModule;
use qmc_runtime::{opaque::Block, AccountId, Balance, BlockNumber, Hash};
use sc_client_api::AuxStore;
pub use sc_rpc_api::DenyUnsafe;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

/// Full client dependencies.
pub struct FullDeps<C> {
    pub client: Arc<C>,
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
    use pallet_contracts_rpc::{Contracts, ContractsApiServer};

    let mut module = RpcModule::new(());
    let FullDeps { client } = deps;

    module.merge(Contracts::new(client).into_rpc())?;

    Ok(module)
}
