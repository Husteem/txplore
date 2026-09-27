use anyhow::{Context, Result};
use bitcoin::{Network, Txid};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use std::path::PathBuf;
use std::str::FromStr;

use crate::decoder::script::decode_script;
use crate::decoder::tx::{decode_raw_tx_bytes, update_fee_info};
use crate::model::types::{ConfirmationInfo, DecodedTx, SpentTxOut};

pub struct BitcoinCoreRpcClient {
    client: Client,
    network: Network,
}

impl BitcoinCoreRpcClient {
    pub fn new(
        url: &str,
        user: Option<String>,
        pass: Option<String>,
        cookie_file: Option<PathBuf>,
        network: Network,
    ) -> Result<Self> {
        let auth = if let Some(cookie) = cookie_file {
            Auth::CookieFile(cookie)
        } else if let (Some(u), Some(p)) = (user, pass) {
            Auth::UserPass(u, p)
        } else {
            Auth::None
        };

        let client = Client::new(url, auth).context("Failed to connect to Bitcoin Core RPC")?;
        Ok(Self { client, network })
    }

    pub fn fetch_and_enrich(&self, txid_str: &str) -> Result<DecodedTx> {
        let txid = Txid::from_str(txid_str).context("Invalid txid format")?;
        let tx = self.client.get_raw_transaction(&txid, None).context(
            "Failed to get raw transaction from Bitcoin Core (ensure txindex=1 if older)",
        )?;

        let consensus_bytes = bitcoin::consensus::serialize(&tx);
        let mut decoded = decode_raw_tx_bytes(&consensus_bytes, self.network)?;

        // Query confirmation info if available
        if let Ok(info) = self.client.get_raw_transaction_info(&txid, None) {
            decoded.confirmation_info = Some(ConfirmationInfo {
                confirmed: info.confirmations.unwrap_or(0) > 0,
                block_height: None,
                block_hash: info.blockhash.map(|b| b.to_string()),
                block_time: info.blocktime.map(|t| t as u64),
                confirmations: info.confirmations,
            });
        }

        // Enrich spent inputs via get_tx_out or parent tx
        for (idx, txin) in tx.input.iter().enumerate() {
            if idx >= decoded.inputs.len() || decoded.is_coinbase {
                continue;
            }

            if let Ok(Some(txout_res)) = self.client.get_tx_out(
                &txin.previous_output.txid,
                txin.previous_output.vout,
                Some(true),
            ) {
                let script_pubkey = txout_res.script_pub_key.script().unwrap_or_default();
                let decoded_script = decode_script(&script_pubkey, self.network);
                let address = txout_res
                    .script_pub_key
                    .address
                    .map(|a| a.assume_checked().to_string());

                decoded.inputs[idx].spent_txout = Some(SpentTxOut {
                    value_sats: txout_res.value.to_sat(),
                    script_pubkey: decoded_script,
                    address,
                });
            } else if let Ok(parent_tx) = self
                .client
                .get_raw_transaction(&txin.previous_output.txid, None)
            {
                let vout_idx = txin.previous_output.vout as usize;
                if vout_idx < parent_tx.output.len() {
                    let parent_out = &parent_tx.output[vout_idx];
                    let decoded_script = decode_script(&parent_out.script_pubkey, self.network);
                    let address = crate::decoder::script::derive_address_from_script(
                        &parent_out.script_pubkey,
                        self.network,
                    );

                    decoded.inputs[idx].spent_txout = Some(SpentTxOut {
                        value_sats: parent_out.value.to_sat(),
                        script_pubkey: decoded_script,
                        address,
                    });
                }
            }
        }

        update_fee_info(&mut decoded);
        Ok(decoded)
    }
}
