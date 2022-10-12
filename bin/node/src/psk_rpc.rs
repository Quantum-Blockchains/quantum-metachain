//! Psk specific RPC methods.

use base64::decode;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
    types::error::{CallError, ErrorObject},
};
use sc_network::PeerId;
use sc_service::config::NetworkConfiguration;
use serde::{Deserialize, Serialize};

/// Structure corrsponding to the data received from the QKD simulator
#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Key {
    pub key_ID: String,
    pub key: String,
}

#[derive(Deserialize)]
pub struct Keys {
    pub keys: Vec<Key>,
}

/// Psk RPC methods
#[rpc(client, server)]
pub trait PskApi {
    /// Returns the encripted pre-shared key.
    #[method(name = "psk_getKey", aliases = ["getKey"])]
    async fn psk_get_key(&self, peer_id: String) -> RpcResult<Key>;
}

/// Error type of this RPC api.
pub enum Error {
    /// Parse peer id failed.
    ParsePeerIdError,
    /// The call to runtime failed.
    RuntimeError,
}

impl From<Error> for i32 {
    fn from(e: Error) -> i32 {
        match e {
            Error::ParsePeerIdError => 1,
            Error::RuntimeError => 2,
        }
    }
}

/// An implementation of Psk-specific RPC methods on full client.
pub struct Psk {
    config: NetworkConfiguration,
}

impl Psk {
    /// Create new `FullSystem` given client and configuration.
    pub fn new( config: NetworkConfiguration) -> Self {
        Self { config }
    }
}

#[async_trait]
impl PskApiServer for Psk
{
    async fn psk_get_key(&self, peer_id: String) -> RpcResult<Key> {
        let _peer_id = peer_id.parse::<PeerId>().map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::ParsePeerIdError.into(),
                "Invalid peer id.",
                Some(e.to_string()),
            ))
        })?;

        // TODO Get URL from configuration by peer id.

        let url = "http://212.244.177.99:9082/api/v1/keys/AliceSAE/enc_keys?size=256";
        let psk = self
            .config
            .pre_shared_key
            .clone()
            .into_pre_share_key()
            .map_err(|e| {
                CallError::Custom(ErrorObject::owned(
                    Error::RuntimeError.into(),
                    "Pre-shared key not",
                    Some(e.to_string()),
                ))
            })?;
        let psk_string = psk.to_string();
        let split = psk_string.split("\n");
        let vec = split.collect::<Vec<&str>>();
        let mut psk_bytes = hex::decode(vec[2].to_string()).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Error in decoding pre-shared key.",
                Some(e.to_string()),
            ))
        })?;
        let response = reqwest::get(url).await.map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Error in getting the key from the QKD simulator.",
                Some(e.to_string()),
            ))
        })?;
        let body = response.text().await.map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Error in getting the key from the QKD simulator.",
                Some(e.to_string()),
            ))
        })?;

        let qkd_key: Keys = serde_json::from_str(&body).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Error in deserialization of data received from QKD simulator",
                Some(e.to_string()),
            ))
        })?;

        let qkd_key_bytes = decode(qkd_key.keys[0].key.clone()).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Error in decoding of QKD key.",
                Some(e.to_string()),
            ))
        })?;

        for i in 0..32 {
            psk_bytes[i] ^= qkd_key_bytes[i];
        }
        Ok(Key {
            key_ID: qkd_key.keys[0].key_ID.clone(),
            key: hex::encode(psk_bytes),
        })
    }
}
