use std::sync::Arc;

use jsonrpsee::{
	core::{RpcResult, async_trait },
	proc_macros::rpc,
    types::error::{CallError, ErrorObject},
};
use sc_service::config::NetworkConfiguration;
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use serde_json;
use base64::decode;
use hex;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Key {
    pub key_ID: String,
	pub key: String,
}

// #[derive(Deserialize)]
// pub struct ResponseQKD {
//     key_ID: String,
//     key: String,
// }

#[rpc(client, server)]
pub trait PskApi {
	#[method(name = "psk_getKey", aliases = ["getKey"])]
    async fn psk_get_key(&self, peer_id: String) -> RpcResult<Key>;
}

/// Error type of this RPC api.
pub enum ErrorRPC {
	/// The transaction was not decodable.
	PeerIdError,
}

impl From<ErrorRPC> for i32 {
	fn from(e: ErrorRPC) -> i32 {
		match e {
			ErrorRPC::PeerIdError => 1,
		}
	}
}

pub struct Psk<C> {
	client: Arc<C>,
    config: NetworkConfiguration,
}

impl<C> Psk<C> {
	pub fn new(client: Arc<C>, config: NetworkConfiguration) -> Self {
		Self { client, config }
	}
}

#[async_trait]
impl<C>PskApiServer for Psk<C>
where
	C: Send + Sync + 'static,
{
    async fn psk_get_key(&self, peer_id: String) -> RpcResult<Key> {

        let peerId = peer_id.parse::<PeerId>().map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                ErrorRPC::PeerIdError.into(),
                "Invalid peer id.",
                Some(e.to_string()),
            ))
        })?;

        // todo Sprawdzic czy jest dostempny endpoint dla peer_id

        let psk = self.config.pre_shared_key.clone().into_pre_share_key()?;
        let psk_string = psk.to_string();
        let split = psk_string.split("\n");
        let vec = split.collect::<Vec<&str>>();
        let mut psk_bytes = hex::decode(vec[2].to_string()).unwrap();
        

        let url = "http://212.244.177.99:9082/api/v1/keys/AliceSAE/enc_keys?size=256";
        let response = reqwest::get(url).await.map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                ErrorRPC::PeerIdError.into(),
                "Unable to query nonce.",
                Some(e.to_string()),
            ))
        })?;
        let body = response.text().await.map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                ErrorRPC::PeerIdError.into(),
                "Unable to query nonce.",
                Some(e.to_string()),
            ))
        })?;

        let qkd_key: Vec<Key> = serde_json::from_str(&body)?;

        let qkd_key_bytes = decode(qkd_key[0].key.clone()).unwrap();

        for i in 0..32 {
            psk_bytes[i] ^= qkd_key_bytes[i];
        }
        
        Ok(
            Key {
                key_ID: qkd_key[0].key_ID.clone(),
                key: hex::encode(psk_bytes),
            }
        )		
    }
}
