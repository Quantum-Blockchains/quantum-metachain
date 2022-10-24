//! Psk specific RPC methods.

use base64::decode;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
    types::error::{CallError, ErrorObject},
};
use sc_network::{PeerId, PreSharedKey};
use sc_service::config::NetworkConfiguration;
use serde::{Deserialize, Serialize};

trait ToBytes {
    fn to_bytes(self) -> Result<[u8; 32], hex::FromHexError>;
}

impl ToBytes for PreSharedKey {
    fn to_bytes(self) -> Result<[u8; 32], hex::FromHexError> {
        let psk_string = self.to_string();
        let split = psk_string.split('\n');
        let vec = split.collect::<Vec<&str>>();
        let vec_bytes = hex::decode(vec[2])?;
        let mut bytes: [u8; 32] = [0; 32];
        bytes[..32].copy_from_slice(&vec_bytes[..32]);
        Ok(bytes)
    }
}

/// Structure corrsponding to the data received from the QKD simulator
#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct QkdKey {
    pub key_ID: String,
    pub key: String,
}

#[derive(Deserialize)]
pub struct QkdResponse {
    pub keys: Vec<QkdKey>,
}

/// Psk RPC methods
#[rpc(client, server)]
pub trait PskApi {
    /// Returns the encripted pre-shared key.
    #[method(name = "psk_getKey", aliases = ["getKey"])]
    async fn psk_get_key(&self, peer_id: String) -> RpcResult<QkdKey>;
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
    pub fn new(config: NetworkConfiguration) -> Self {
        Self { config }
    }
}

#[async_trait]
impl PskApiServer for Psk {
    async fn psk_get_key(&self, peer_id: String) -> RpcResult<QkdKey> {
        let _peer_id = peer_id.parse::<PeerId>().map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::ParsePeerIdError.into(),
                "Invalid peer id.",
                Some(e.to_string()),
            ))
        })?;

        let mut qkd_url = String::new();
        let addrs = self.config.qkd_addr.clone();

        if addrs.is_empty(){
            for i in 0..addrs.len(){
                if addrs[i].peer_id == _peer_id {
                    qkd_url.push_str("http://");
                    qkd_url.push_str(&addrs[i].host.to_string());
                    let d: String = addrs[i].path.clone().unwrap();
                    qkd_url.push_str(&d);
                    qkd_url.push_str("/enc_keys?size=256");
                }
            }
        }

        if qkd_url.is_empty() {
            return Err(jsonrpsee::core::error::Error::Custom("The provided peer id doers not have a qkd address configured.".to_string()))
        }

        let psk = self
            .config
            .pre_shared_key
            .clone()
            .into_pre_share_key()
            .map_err(|e| {
                CallError::Custom(ErrorObject::owned(
                    Error::RuntimeError.into(),
                    "Pre-shared key not found.",
                    Some(e.to_string()),
                ))
            })?;
        let mut psk_bytes = psk.to_bytes().map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Error in decoding pre-shared key.",
                Some(e.to_string()),
            ))
        })?;
        let response = reqwest::get(qkd_url).await.map_err(|e| {
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

        let qkd_key: QkdResponse = serde_json::from_str(&body).map_err(|e| {
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
        Ok(QkdKey {
            key_ID: qkd_key.keys[0].key_ID.clone(),
            key: hex::encode(psk_bytes),
        })
    }
}
