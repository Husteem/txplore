use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

use crate::decoder::script::decode_script;
use crate::decoder::tx::{decode_raw_tx_hex, update_fee_info};
use crate::model::types::{ConfirmationInfo, DecodedTx, SpentTxOut};
use bitcoin::Network;

#[derive(Debug, Serialize)]
struct JsonRpcRequest<'a> {
    jsonrpc: &'static str,
    id: &'static str,
    method: &'a str,
    params: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse<T> {
    result: Option<T>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

#[derive(Debug, Deserialize)]
struct VerboseTxResponse {
    hex: String,
    vin: Vec<VerboseVin>,
    vout: Vec<VerboseVout>,
    blockhash: Option<String>,
    confirmations: Option<u64>,
    time: Option<u64>,
    blocktime: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct VerboseVin {
    txid: Option<String>,
    vout: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct VerboseVout {
    value: f64,
    n: u32,
    script_pub_key: VerboseScriptPubKey,
}

#[derive(Debug, Deserialize)]
struct VerboseScriptPubKey {
    hex: String,
    address: Option<String>,
}

pub struct BitRpcClient {
    client: Client,
    endpoint: String,
    network: Network,
}

impl BitRpcClient {
    pub fn new(api_key: &str, network: Network) -> Self {
        let mut headers = HeaderMap::new();
        if let Ok(mut val) = HeaderValue::from_str(api_key.trim()) {
            val.set_sensitive(true);
            headers.insert("X-API-Key", val);
        }
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(20))
            .connect_timeout(Duration::from_secs(5))
            .user_agent("txplore/0.1.0 (BitRPC Client; Rust for Bitcoin 2.0)")
            .build()
            .unwrap_or_default();

        Self {
            client,
            endpoint: "https://bitrpc.thebuidl.xyz/bitcoin".to_string(),
            network,
        }
    }

    pub fn with_endpoint(mut self, endpoint: &str) -> Self {
        self.endpoint = endpoint.trim_end_matches('/').to_string();
        self
    }

    pub async fn call<T: DeserializeOwned>(&self, method: &str, params: serde_json::Value) -> Result<T> {
        let payload = JsonRpcRequest {
            jsonrpc: "1.0",
            id: "txplore",
            method,
            params,
        };

        let resp = self
            .client
            .post(&self.endpoint)
            .json(&payload)
            .send()
            .await
            .context(format!("Failed to connect to BitRPC at {}", self.endpoint))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("BitRPC HTTP {}: {}", status, body);
        }

        let rpc_res: JsonRpcResponse<T> = resp
            .json()
            .await
            .context("Failed parsing BitRPC JSON-RPC response")?;

        if let Some(err) = rpc_res.error {
            anyhow::bail!("Bitcoin Core RPC error [code {}]: {}", err.code, err.message);
        }

        rpc_res.result.context("Empty result in JSON-RPC response")
    }

    pub async fn get_blockchain_info(&self) -> Result<serde_json::Value> {
        self.call("getblockchaininfo", json!([])).await
    }

    pub async fn fetch_raw_hex(&self, txid: &str) -> Result<String> {
        // Try verbose=false first (returns raw hex string)
        match self.call::<String>("getrawtransaction", json!([txid, false])).await {
            Ok(hex) => Ok(hex),
            Err(_) => {
                // Try verbose=true (returns object with hex field)
                let verbose: VerboseTxResponse = self
                    .call("getrawtransaction", json!([txid, true]))
                    .await
                    .context(format!("Failed to retrieve transaction {} via BitRPC", txid))?;
                Ok(verbose.hex)
            }
        }
    }

    pub async fn fetch_and_enrich(&self, txid: &str) -> Result<DecodedTx> {
        // Call verbose getrawtransaction
        let verbose_res: Result<VerboseTxResponse> =
            self.call("getrawtransaction", json!([txid, true])).await;

        let (raw_hex, verbose_info) = match verbose_res {
            Ok(v) => (v.hex.clone(), Some(v)),
            Err(_) => {
                let hex = self.fetch_raw_hex(txid).await?;
                (hex, None)
            }
        };

        let mut decoded = decode_raw_tx_hex(&raw_hex, self.network)?;

        if let Some(v) = verbose_info {
            decoded.confirmation_info = Some(ConfirmationInfo {
                confirmed: v.confirmations.unwrap_or(0) > 0,
                block_height: None,
                block_hash: v.blockhash,
                block_time: v.blocktime.or(v.time),
                confirmations: v.confirmations.map(|c| c as u32),
            });

            // For each input, resolve spent output value and script via parent transaction
            for (idx, vin) in v.vin.iter().enumerate() {
                if idx >= decoded.inputs.len() || decoded.is_coinbase {
                    continue;
                }

                if let (Some(ref parent_txid), Some(parent_vout)) = (&vin.txid, vin.vout) {
                    if let Ok(parent_verbose) = self
                        .call::<VerboseTxResponse>("getrawtransaction", json!([parent_txid, true]))
                        .await
                    {
                        if let Some(out) = parent_verbose.vout.iter().find(|o| o.n == parent_vout) {
                            let sats = (out.value * 100_000_000.0).round() as u64;
                            if let Ok(script_bytes) = hex::decode(&out.script_pub_key.hex) {
                                let script = bitcoin::ScriptBuf::from(script_bytes);
                                let decoded_script = decode_script(&script, self.network);
                                decoded.inputs[idx].spent_txout = Some(SpentTxOut {
                                    value_sats: sats,
                                    script_pubkey: decoded_script,
                                    address: out.script_pub_key.address.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }

        update_fee_info(&mut decoded);
        Ok(decoded)
    }
}
