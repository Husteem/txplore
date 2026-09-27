use anyhow::{Context, Result};
use bitcoin::Network;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

use crate::decoder::script::decode_script;
use crate::decoder::tx::{decode_raw_tx_hex, update_fee_info};
use crate::model::types::{DecodedTx, SpentTxOut};

pub fn load_tx_from_source(
    hex_or_path: &str,
    network: Network,
    input_values: Option<&[u64]>,
) -> Result<DecodedTx> {
    let raw_hex = if hex_or_path == "-" {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("Failed reading hex from stdin")?;
        buffer.trim().to_string()
    } else if Path::new(hex_or_path).exists() {
        fs::read_to_string(hex_or_path)
            .context("Failed reading transaction hex file")?
            .trim()
            .to_string()
    } else {
        hex_or_path.trim().to_string()
    };

    let mut decoded = decode_raw_tx_hex(&raw_hex, network)?;

    if let Some(values) = input_values {
        for (idx, &val) in values.iter().enumerate() {
            if idx < decoded.inputs.len() {
                decoded.inputs[idx].spent_txout = Some(SpentTxOut {
                    value_sats: val,
                    script_pubkey: decode_script(&bitcoin::ScriptBuf::new(), network),
                    address: None,
                });
            }
        }
        update_fee_info(&mut decoded);
    }

    Ok(decoded)
}
