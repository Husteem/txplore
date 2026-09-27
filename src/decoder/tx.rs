use anyhow::{Context, Result};
use bitcoin::consensus::deserialize;
use bitcoin::{Network, Transaction};

use crate::decoder::script::{
    decode_script, decode_witness_item, derive_address_from_script, parse_op_return_payload,
};
use crate::model::types::{
    CoinbaseData, DecodedInput, DecodedOutput, DecodedTx, FeeInfo, LocktimeInfo, LocktimeType,
    RbfStatus, RelativeLocktimeInfo, TxClassification,
};

pub fn decode_raw_tx_hex(hex_str: &str, network: Network) -> Result<DecodedTx> {
    let clean_hex = hex_str.trim().trim_start_matches("0x");
    let bytes = hex::decode(clean_hex).context("Failed to parse transaction hex")?;
    decode_raw_tx_bytes(&bytes, network)
}

pub fn decode_raw_tx_bytes(bytes: &[u8], network: Network) -> Result<DecodedTx> {
    let tx: Transaction =
        deserialize(bytes).context("Failed to deserialize Bitcoin transaction")?;
    decode_bitcoin_tx(&tx, bytes.len(), network)
}

pub fn decode_bitcoin_tx(
    tx: &Transaction,
    size_bytes: usize,
    network: Network,
) -> Result<DecodedTx> {
    let txid = tx.compute_txid().to_string();
    let wtxid = tx.compute_wtxid().to_string();
    let version = tx.version.0;

    let is_coinbase = tx.is_coinbase();
    let is_segwit = tx.input.iter().any(|i| !i.witness.is_empty());

    let weight_wu = tx.weight().to_wu();
    let vsize_vb = tx.vsize() as u64;

    // Witness discount ratio: (size * 4 - weight) / (size * 4)
    let total_unstripped_weight = (size_bytes as u64) * 4;
    let discount_ratio = if total_unstripped_weight > weight_wu {
        (total_unstripped_weight - weight_wu) as f64 / total_unstripped_weight as f64
    } else {
        0.0
    };

    let locktime = parse_locktime(tx.lock_time.to_consensus_u32(), tx);
    let rbf_status = determine_rbf_status(tx, is_coinbase);

    // Decode inputs
    let mut inputs = Vec::new();
    for (idx, txin) in tx.input.iter().enumerate() {
        let prevout_txid = txin.previous_output.txid.to_string();
        let prevout_vout = txin.previous_output.vout;
        let sequence = txin.sequence.0;
        let sequence_rbf = sequence < 0xfffffffe;

        let relative_locktime = parse_relative_locktime(sequence);

        let coinbase_data = if is_coinbase {
            Some(parse_coinbase_script(&txin.script_sig))
        } else {
            None
        };

        let script_sig = if !txin.script_sig.is_empty() {
            Some(decode_script(&txin.script_sig, network))
        } else {
            None
        };

        let mut witness_items = Vec::new();
        for (w_idx, item) in txin.witness.iter().enumerate() {
            witness_items.push(decode_witness_item(w_idx, item));
        }

        inputs.push(DecodedInput {
            index: idx,
            prevout_txid,
            prevout_vout,
            sequence,
            sequence_rbf,
            relative_locktime,
            coinbase_data,
            script_sig,
            witness: witness_items,
            spent_txout: None,
        });
    }

    // Decode outputs
    let mut outputs = Vec::new();
    for (idx, txout) in tx.output.iter().enumerate() {
        let value_sats = txout.value.to_sat();
        let value_btc = value_sats as f64 / 100_000_000.0;
        let decoded_script = decode_script(&txout.script_pubkey, network);
        let address = derive_address_from_script(&txout.script_pubkey, network);
        let script_type = decoded_script.script_type;
        let op_return_payload = parse_op_return_payload(&txout.script_pubkey);

        outputs.push(DecodedOutput {
            index: idx,
            value_sats,
            value_btc,
            script_pubkey: decoded_script,
            address,
            script_type,
            op_return_payload,
        });
    }

    let classification = classify_transaction(is_coinbase, &inputs, &outputs);

    Ok(DecodedTx {
        txid,
        wtxid,
        version,
        locktime,
        is_segwit,
        is_coinbase,
        size_bytes,
        weight_wu,
        vsize_vb,
        discount_ratio,
        inputs,
        outputs,
        fee_info: None,
        rbf_status,
        classification,
        network: network.to_string(),
        confirmation_info: None,
    })
}

fn parse_locktime(raw: u32, tx: &Transaction) -> LocktimeInfo {
    let all_final = tx.input.iter().all(|i| i.sequence.0 == 0xffffffff);

    if raw == 0 {
        LocktimeInfo {
            raw,
            locktime_type: LocktimeType::None,
            description: "No locktime enforced (executable immediately in any block)".to_string(),
            is_final: true,
        }
    } else if raw < 500_000_000 {
        LocktimeInfo {
            raw,
            locktime_type: LocktimeType::BlockHeight,
            description: format!("Locked until block height {}", raw),
            is_final: all_final,
        }
    } else {
        LocktimeInfo {
            raw,
            locktime_type: LocktimeType::Timestamp,
            description: format!("Locked until UNIX timestamp {} (UTC)", raw),
            is_final: all_final,
        }
    }
}

fn determine_rbf_status(tx: &Transaction, is_coinbase: bool) -> RbfStatus {
    if is_coinbase {
        return RbfStatus::Coinbase;
    }

    let signaling_count = tx
        .input
        .iter()
        .filter(|i| i.sequence.0 < 0xfffffffe)
        .count();

    if signaling_count == tx.input.len() {
        RbfStatus::Signaling
    } else if signaling_count > 0 {
        RbfStatus::Mixed
    } else {
        RbfStatus::NotSignaling
    }
}

fn parse_relative_locktime(sequence: u32) -> Option<RelativeLocktimeInfo> {
    // BIP68: Bit 31 disables relative locktime if set
    const DISABLE_FLAG: u32 = 1 << 31;
    const TYPE_FLAG: u32 = 1 << 22;
    const MASK: u32 = 0x0000ffff;

    if (sequence & DISABLE_FLAG) != 0 {
        return None;
    }

    let is_time_based = (sequence & TYPE_FLAG) != 0;
    let value = sequence & MASK;

    let human_readable = if is_time_based {
        let seconds = value * 512;
        format!(
            "Locked for {} seconds (~{:.1} minutes)",
            seconds,
            seconds as f64 / 60.0
        )
    } else {
        format!(
            "Locked for {} blocks (~{:.1} minutes)",
            value,
            (value * 10) as f64
        )
    };

    Some(RelativeLocktimeInfo {
        raw: sequence,
        is_time_based,
        value,
        human_readable,
    })
}

fn parse_coinbase_script(script_sig: &bitcoin::Script) -> CoinbaseData {
    let bytes = script_sig.as_bytes();
    let hex_str = hex::encode(bytes);

    // BIP34: Height is encoded as first push data in coinbase scriptSig
    let mut bip34_height = None;
    if !bytes.is_empty() {
        let push_len = bytes[0] as usize;
        if (1..=8).contains(&push_len) && bytes.len() > push_len {
            let mut height_bytes = [0u8; 8];
            for (i, &b) in bytes[1..=push_len].iter().enumerate() {
                height_bytes[i] = b;
            }
            bip34_height = Some(u64::from_le_bytes(height_bytes) as u32);
        }
    }

    // Attempt to extract readable text
    let ascii_text: String = bytes
        .iter()
        .map(|&b| {
            if (32..=126).contains(&b) {
                b as char
            } else {
                '.'
            }
        })
        .collect();

    CoinbaseData {
        hex: hex_str,
        bip34_height,
        text_representation: ascii_text,
    }
}

fn classify_transaction(
    is_coinbase: bool,
    inputs: &[DecodedInput],
    outputs: &[DecodedOutput],
) -> TxClassification {
    use crate::model::types::ScriptType;

    if is_coinbase {
        return TxClassification::Coinbase;
    }

    if outputs
        .iter()
        .any(|o| o.script_type == ScriptType::OpReturn)
    {
        return TxClassification::OpReturnData;
    }

    if inputs
        .iter()
        .any(|i| i.witness.len() == 1 && i.witness[0].size_bytes == 64)
    {
        return TxClassification::TaprootKeySpend;
    }

    if inputs.iter().any(|i| {
        i.witness
            .iter()
            .any(|w| w.inferred_type == crate::model::types::WitnessItemType::TaprootControlBlock)
    }) {
        return TxClassification::TaprootScriptSpend;
    }

    if inputs.len() == 1 && outputs.len() == 2 {
        return TxClassification::SimplePayment;
    }

    if inputs.len() >= 3 && outputs.len() <= 2 {
        return TxClassification::Consolidation;
    }

    if inputs.len() <= 2 && outputs.len() >= 4 {
        return TxClassification::BatchPayment;
    }

    TxClassification::Unknown
}

pub fn update_fee_info(decoded_tx: &mut DecodedTx) {
    let mut total_in = 0u64;
    let mut all_inputs_known = true;

    for input in &decoded_tx.inputs {
        if let Some(ref spent) = input.spent_txout {
            total_in += spent.value_sats;
        } else {
            all_inputs_known = false;
            break;
        }
    }

    if all_inputs_known && !decoded_tx.inputs.is_empty() && !decoded_tx.is_coinbase {
        let total_out: u64 = decoded_tx.outputs.iter().map(|o| o.value_sats).sum();
        let fee_sats = total_in.saturating_sub(total_out);
        let fee_rate_sat_per_vb = fee_sats as f64 / decoded_tx.vsize_vb as f64;
        let fee_percentage = (fee_sats as f64 / total_in as f64) * 100.0;

        decoded_tx.fee_info = Some(FeeInfo {
            total_input_sats: total_in,
            total_output_sats: total_out,
            fee_sats,
            fee_rate_sat_per_vb,
            fee_percentage,
        });
    }
}
