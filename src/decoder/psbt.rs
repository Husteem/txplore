use anyhow::{Context, Result};
use bitcoin::base64::prelude::BASE64_STANDARD;
use bitcoin::base64::Engine;
use bitcoin::psbt::Psbt;

use crate::model::types::{PsbtAnalysis, PsbtInputStatus};

pub fn analyze_psbt_str(input_str: &str) -> Result<PsbtAnalysis> {
    let clean = input_str.trim();

    let bytes = if let Ok(b) = hex::decode(clean) {
        b
    } else {
        BASE64_STANDARD
            .decode(clean)
            .context("Failed to decode PSBT from hex or base64 string")?
    };

    let psbt = Psbt::deserialize(&bytes).context("Failed to deserialize PSBT structure")?;
    analyze_psbt(&psbt)
}

pub fn analyze_psbt(psbt: &Psbt) -> Result<PsbtAnalysis> {
    let num_inputs = psbt.inputs.len();
    let num_outputs = psbt.outputs.len();

    let mut total_in = 0u64;
    let mut all_inputs_have_utxo = true;
    let mut signatures_present = 0;
    let mut inputs_status = Vec::new();

    for (idx, input) in psbt.inputs.iter().enumerate() {
        let mut value_sats = None;
        let mut has_utxo = false;

        if let Some(ref witness_utxo) = input.witness_utxo {
            has_utxo = true;
            let val = witness_utxo.value.to_sat();
            value_sats = Some(val);
            total_in += val;
        } else if let Some(ref non_witness_tx) = input.non_witness_utxo {
            has_utxo = true;
            let prev_vout = psbt.unsigned_tx.input[idx].previous_output.vout as usize;
            if prev_vout < non_witness_tx.output.len() {
                let val = non_witness_tx.output[prev_vout].value.to_sat();
                value_sats = Some(val);
                total_in += val;
            }
        } else {
            all_inputs_have_utxo = false;
        }

        let partial_sigs_count = input.partial_sigs.len();
        signatures_present += partial_sigs_count;
        let has_partial_sigs = partial_sigs_count > 0;

        let sighash_type = input.sighash_type.map(|s| format!("{:?}", s));
        let has_witness_script = input.witness_script.is_some();
        let has_redeem_script = input.redeem_script.is_some();
        let has_taproot_internal_key = input.tap_internal_key.is_some();
        let has_taproot_script_tree = !input.tap_scripts.is_empty();
        let is_finalized = input.final_script_sig.is_some() || input.final_script_witness.is_some();

        if input.tap_key_sig.is_some() {
            signatures_present += 1;
        }

        inputs_status.push(PsbtInputStatus {
            index: idx,
            has_utxo,
            value_sats,
            has_partial_sigs,
            partial_sigs_count,
            sighash_type,
            has_witness_script,
            has_redeem_script,
            has_taproot_internal_key,
            has_taproot_script_tree,
            is_finalized,
        });
    }

    let is_all_finalized = inputs_status.iter().all(|i| i.is_finalized);

    let fee_sats = if all_inputs_have_utxo && num_inputs > 0 {
        let total_out: u64 = psbt
            .unsigned_tx
            .output
            .iter()
            .map(|o| o.value.to_sat())
            .sum();
        Some(total_in.saturating_sub(total_out))
    } else {
        None
    };

    let fee_rate_sat_per_vb = fee_sats.map(|fee| {
        let vsize = psbt.unsigned_tx.vsize() as f64;
        fee as f64 / vsize
    });

    let signatures_required = num_inputs;

    Ok(PsbtAnalysis {
        num_inputs,
        num_outputs,
        fee_sats,
        fee_rate_sat_per_vb,
        is_finalized: is_all_finalized,
        signatures_present,
        signatures_required,
        inputs_status,
    })
}
