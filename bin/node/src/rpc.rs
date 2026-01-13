//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;

use jsonrpsee::{core::RpcResult, proc_macros::rpc, RpcModule};
use did_runtime_api::DidRuntimeApi;
use qmc_runtime::{opaque::Block, AccountId, Balance, Nonce};
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_runtime::generic::BlockId;

pub use sc_rpc_api::DenyUnsafe;

#[rpc(server)]
pub trait DidApi {
	#[method(name = "did_getByString")]
	fn did_by_string(&self, did: String) -> RpcResult<Option<did::DidDetails>>;
}

pub struct DidRpc<C> {
	client: Arc<C>,
}

impl<C> DidRpc<C> {
	pub fn new(client: Arc<C>) -> Self {
		Self { client }
	}
}

impl<C> DidApiServer for DidRpc<C>
where
	C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + 'static,
	C::Api: did_runtime_api::DidRuntimeApi<Block>,
{
	fn did_by_string(&self, did: String) -> RpcResult<Option<did::DidDetails>> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;
		api.did_by_string(at, did.into_bytes())
			.map_err(|e| jsonrpsee::core::Error::Custom(format!("Runtime API error: {:?}", e)))
	}
}

/// Full client dependencies.
pub struct FullDeps<C, P> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
}

/// Instantiate all full RPC extensions.
pub fn create_full<C, P>(
	deps: FullDeps<C, P>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
	C: Send + Sync + 'static,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: did_runtime_api::DidRuntimeApi<Block>,
	C::Api: BlockBuilder<Block>,
	P: TransactionPool + 'static,
{
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
	use substrate_frame_rpc_system::{System, SystemApiServer};

	let mut module = RpcModule::new(());
	let FullDeps { client, pool, deny_unsafe } = deps;

	module.merge(System::new(client.clone(), pool, deny_unsafe).into_rpc())?;
	module.merge(TransactionPayment::new(client.clone()).into_rpc())?;
	module.merge(DidApiServer::into_rpc(DidRpc::new(client)))?;

	// Extend this RPC with a custom API by using the following syntax.
	// `YourRpcStruct` should have a reference to a client, which is needed
	// to call into the runtime.
	// `module.merge(YourRpcTrait::into_rpc(YourRpcStruct::new(ReferenceToClient, ...)))?;`

	Ok(module)
}
