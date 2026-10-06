use anyhow::Result;
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
    endpoints: Vec<String>,
    network: Network,
}

impl EsploraClient {
    pub fn new(base_url: Option<&str>, network: Network) -> Self {
        let mut endpoints = Vec::new();

        if let Some(custom) = base_url {
            let trimmed = custom.trim_end_matches('/').to_string();
            if !trimmed.is_empty() {
                endpoints.push(trimmed);
            }
        }

        // Add network defaults with automatic mirror fallback
        match network {
            Network::Bitcoin => {
                let mirrors = [
                    "https://mempool.space/api",
                    "https://blockstream.info/api",
                    "https://mempool.emzy.de/api",
                ];
                for m in mirrors {
                    let s = m.to_string();
                    if !endpoints.contains(&s) {
                        endpoints.push(s);
                    }
                }
            }
            Network::Testnet => {
                let mirrors = [
                    "https://mempool.space/testnet/api",
                    "https://blockstream.info/testnet/api",
                ];
                for m in mirrors {
                    let s = m.to_string();
                    if !endpoints.contains(&s) {
                        endpoints.push(s);
                    }
                }
            }
            Network::Signet => {
                let mirrors = [
                    "https://mempool.space/signet/api",
                ];
                for m in mirrors {
                    let s = m.to_string();
                    if !endpoints.contains(&s) {
                        endpoints.push(s);
                    }
                }
            }
            _ => {
                let mirrors = [
                    "https://mempool.space/api",
                    "https://blockstream.info/api",
                    "https://mempool.emzy.de/api",
                ];
                for m in mirrors {
                    let s = m.to_string();
                    if !endpoints.contains(&s) {
                        endpoints.push(s);
                    }
                }
            }
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(4))
            .user_agent("txplore/0.1.0 (Bitcoin Transaction Explorer; +https://github.com/Husteem/txplore)")
            .build()
            .unwrap_or_default();

        Self {
            client,
            endpoints,
            network,
        }
    }

    pub fn endpoints(&self) -> &[String] {
        &self.endpoints
    }

    pub async fn fetch_raw_hex(&self, txid: &str) -> Result<String> {
        let mut errors = Vec::new();

        for base in &self.endpoints {
            let url = format!("{}/tx/{}/hex", base, txid);
            match self.client.get(&url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.text().await {
                        Ok(hex_str) => {
                            let trimmed = hex_str.trim().to_string();
                            if !trimmed.is_empty() {
                                return Ok(trimmed);
                            }
                        }
                        Err(e) => {
                            errors.push(format!("{}: failed reading body ({})", base, e));
                        }
                    }
                }
                Ok(resp) => {
                    errors.push(format!("{}: HTTP {}", base, resp.status()));
                }
                Err(e) => {
                    errors.push(format!("{}: {}", base, e));
                }
            }
        }

        anyhow::bail!(
            "Failed to retrieve transaction {} across all configured endpoints: {}",
            txid,
            errors.join(" | ")
        )
    }

    pub async fn fetch_and_enrich(&self, txid: &str) -> Result<DecodedTx> {
        let raw_hex = self.fetch_raw_hex(txid).await?;
        let mut decoded = decode_raw_tx_hex(&raw_hex, self.network)?;
        self.enrich_with_esplora_data(&mut decoded).await?;
        Ok(decoded)
    }

    pub async fn enrich_with_esplora_data(&self, tx: &mut DecodedTx) -> Result<()> {
        let mut found_response = None;

        for base in &self.endpoints {
            let url = format!("{}/tx/{}", base, tx.txid);
            if let Ok(resp) = self.client.get(&url).send().await {
                if resp.status().is_success() {
                    if let Ok(esplora_tx) = resp.json::<EsploraTxResponse>().await {
                        found_response = Some(esplora_tx);
                        break;
                    }
                }
            }
        }

        let esplora_tx = match found_response {
            Some(res) => res,
            None => return Ok(()), // Non-fatal if offline
        };

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
