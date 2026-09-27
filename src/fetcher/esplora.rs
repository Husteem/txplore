use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

use crate::decoder::script::decode_script;
use crate::decoder::tx::{decode_raw_tx_hex, update_fee_info};
use crate::model::types::{ConfirmationInfo, DecodedTx, SpentTxOut};
use bitcoin::Network;

#[derive(Debug, Deserialize)]
struct EsploraTxStatus {
    confirmed: bool,
    block_height: Option<u32>,
    block_hash: Option<String>,
    block_time: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct EsploraPrevout {
    value: u64,
    scriptpubkey: String,
    scriptpubkey_address: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EsploraVin {
    prevout: Option<EsploraPrevout>,
}

#[derive(Debug, Deserialize)]
struct EsploraTxResponse {
    status: EsploraTxStatus,
    vin: Vec<EsploraVin>,
}

pub struct EsploraClient {
    client: Client,
    base_url: String,
    network: Network,
}

impl EsploraClient {
    pub fn new(base_url: Option<&str>, network: Network) -> Self {
        let default_url = match network {
            Network::Bitcoin => "https://mempool.space/api",
            Network::Testnet => "https://mempool.space/testnet/api",
            Network::Signet => "https://mempool.space/signet/api",
            _ => "https://mempool.space/api",
        };

        let url = base_url
            .unwrap_or(default_url)
            .trim_end_matches('/')
            .to_string();

        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        Self {
            client,
            base_url: url,
            network,
        }
    }

    pub async fn fetch_raw_hex(&self, txid: &str) -> Result<String> {
        let url = format!("{}/tx/{}/hex", self.base_url, txid);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context(format!("Failed to connect to Esplora API at {}", url))?;

        if !resp.status().is_success() {
            anyhow::bail!("Esplora returned HTTP {} for txid {}", resp.status(), txid);
        }

        resp.text().await.context("Failed reading response text")
    }

    pub async fn fetch_and_enrich(&self, txid: &str) -> Result<DecodedTx> {
        let raw_hex = self.fetch_raw_hex(txid).await?;
        let mut decoded = decode_raw_tx_hex(&raw_hex, self.network)?;
        self.enrich_with_esplora_data(&mut decoded).await?;
        Ok(decoded)
    }

    pub async fn enrich_with_esplora_data(&self, tx: &mut DecodedTx) -> Result<()> {
        let url = format!("{}/tx/{}", self.base_url, tx.txid);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to query Esplora tx details")?;

        if !resp.status().is_success() {
            return Ok(()); // Non-fatal if node is offline
        }

        let esplora_tx: EsploraTxResponse =
            resp.json().await.context("Failed parsing Esplora JSON")?;

        // Populate confirmations
        tx.confirmation_info = Some(ConfirmationInfo {
            confirmed: esplora_tx.status.confirmed,
            block_height: esplora_tx.status.block_height,
            block_hash: esplora_tx.status.block_hash,
            block_time: esplora_tx.status.block_time,
            confirmations: if esplora_tx.status.confirmed {
                Some(1)
            } else {
                Some(0)
            },
        });

        // Populate inputs spent outputs
        for (idx, vin) in esplora_tx.vin.into_iter().enumerate() {
            if idx < tx.inputs.len() {
                if let Some(prev) = vin.prevout {
                    if let Ok(script_bytes) = hex::decode(&prev.scriptpubkey) {
                        let script = bitcoin::ScriptBuf::from(script_bytes);
                        let decoded_script = decode_script(&script, self.network);
                        tx.inputs[idx].spent_txout = Some(SpentTxOut {
                            value_sats: prev.value,
                            script_pubkey: decoded_script,
                            address: prev.scriptpubkey_address,
                        });
                    }
                }
            }
        }

        update_fee_info(tx);
        Ok(())
    }
}
