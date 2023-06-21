use alloc::string::String;

use serde::Deserialize;
use sp_io::offchain::timestamp;
use sp_runtime::offchain::{http::Request, Duration};
use sp_std::vec::Vec;

#[derive(Deserialize)]
pub struct LocalPeerIdResponse {
    pub result: String,
}

fn fetch_local_peerid(rpc_port: u16) -> Result<Vec<u8>, &'static str> {
    let url = format!("http://localhost:{}", rpc_port);

    let mut vec_body: Vec<&[u8]> = Vec::new();
    let data = b"{\"id\": 1, \"jsonrpc\": \"2.0\", \"method\": \"system_localPeerId\"}";
    vec_body.push(data);

    let request = Request::post(&url, vec_body);
    let timeout = timestamp().add(Duration::from_millis(3000));

    let pending = request
        .add_header("Content-Type", "application/json")
        .deadline(timeout)
        .send()
        .map_err(|_| "HttpFetchingError")?;

    let response = pending
        .try_wait(timeout)
        .map_err(|_| "HttpFetchingError")?
        .map_err(|_| "HttpFetchingError")?;

    if response.code != 200 {
        log::error!("Unexpected http request status code: {}", response.code);
        return Err("HttpFetchingError");
    }

    Ok(response.body().collect::<Vec<u8>>())
}

pub fn fetch_n_parse_local_peerid(rpc_port: u16) -> Result<String, &'static str> {
    let resp_bytes = fetch_local_peerid(rpc_port).map_err(|e| {
        log::error!("fetch_local_peerid error: {:?}", e);
        e
    })?;

    let json_res: LocalPeerIdResponse =
        serde_json::from_slice(&resp_bytes).map_err(|e: serde_json::Error| {
            log::error!("Parse local peerid error: {:?}", e);
            "HttpFetchingError"
        })?;

    Ok(json_res.result)
}
