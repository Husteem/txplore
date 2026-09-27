use crate::model::types::{DecodedTx, ScriptType};
use crate::render::mermaid::generate_mermaid_diagram;

pub fn generate_markdown_report(tx: &DecodedTx) -> String {
    let mut doc = String::new();

    doc.push_str(&format!(
        "# Bitcoin Transaction Technical Report: `{}`\n\n",
        tx.txid
    ));
    doc.push_str(&format!(
        "**Network**: {} | **Classification**: {} | **Version**: {}\n\n",
        tx.network, tx.classification, tx.version
    ));

    doc.push_str("## 1. Executive Summary\n\n");
    doc.push_str("| Metric | Value |\n| :--- | :--- |\n");
    doc.push_str(&format!("| **Txid** | `{}` |\n", tx.txid));
    doc.push_str(&format!("| **Wtxid** | `{}` |\n", tx.wtxid));
    doc.push_str(&format!(
        "| **Virtual Size (vsize)** | {} vB |\n",
        tx.vsize_vb
    ));
    doc.push_str(&format!(
        "| **Weight Units (WU)** | {} WU |\n",
        tx.weight_wu
    ));
    doc.push_str(&format!(
        "| **Raw Serialized Size** | {} bytes |\n",
        tx.size_bytes
    ));
    doc.push_str(&format!(
        "| **Witness Discount** | {:.1}% |\n",
        tx.discount_ratio * 100.0
    ));
    doc.push_str(&format!(
        "| **SegWit Status** | {} |\n",
        if tx.is_segwit {
            "Active (v0/v1)"
        } else {
            "Legacy (v0 inactive)"
        }
    ));
    doc.push_str(&format!(
        "| **BIP125 RBF Signaling** | {:?} |\n",
        tx.rbf_status
    ));
    doc.push_str(&format!(
        "| **Locktime Specification** | {} |\n",
        tx.locktime.description
    ));

    if let Some(ref fee) = tx.fee_info {
        doc.push_str(&format!(
            "| **Total Input Value** | {} sats ({:.8} BTC) |\n",
            fee.total_input_sats,
            fee.total_input_sats as f64 / 1e8
        ));
        doc.push_str(&format!(
            "| **Total Output Value** | {} sats ({:.8} BTC) |\n",
            fee.total_output_sats,
            fee.total_output_sats as f64 / 1e8
        ));
        doc.push_str(&format!(
            "| **Miner Fee** | {} sats ({:.2}% of in) |\n",
            fee.fee_sats, fee.fee_percentage
        ));
        doc.push_str(&format!(
            "| **Fee Rate** | **{:.2} sat/vB** |\n",
            fee.fee_rate_sat_per_vb
        ));
    }
    doc.push('\n');

    doc.push_str("## 2. Visual Transaction DAG\n\n");
    doc.push_str(&generate_mermaid_diagram(tx));
    doc.push('\n');

    doc.push_str("## 3. Inputs Breakdown\n\n");
    doc.push_str("| # | Previous OutPoint | Value | Sequence | ScriptSig / Witness Details |\n| :--- | :--- | :--- | :--- | :--- |\n");

    for input in &tx.inputs {
        let outpoint = if tx.is_coinbase {
            "Coinbase Generation".to_string()
        } else {
            format!("`{}:{}`", input.prevout_txid, input.prevout_vout)
        };

        let val_str = input
            .spent_txout
            .as_ref()
            .map(|s| format!("{} sats", s.value_sats))
            .unwrap_or_else(|| "Unknown".to_string());

        let mut details = Vec::new();
        if let Some(ref ss) = input.script_sig {
            details.push(format!("**ScriptSig**: `{}`", ss.asm));
        }
        if !input.witness.is_empty() {
            details.push(format!("**Witness Items ({})**:", input.witness.len()));
            for w in &input.witness {
                details.push(format!("- [{}] *{}*: `{}`", w.index, w.description, w.hex));
            }
        }

        doc.push_str(&format!(
            "| {} | {} | {} | `0x{:08x}` | {} |\n",
            input.index,
            outpoint,
            val_str,
            input.sequence,
            details.join("<br/>")
        ));
    }
    doc.push('\n');

    doc.push_str("## 4. Outputs Breakdown\n\n");
    doc.push_str("| # | Value | Script Type | Address / Payload | ScriptPubKey Disassembly |\n| :--- | :--- | :--- | :--- | :--- |\n");

    for output in &tx.outputs {
        let recipient = if output.script_type == ScriptType::OpReturn {
            if let Some(ref op) = output.op_return_payload {
                format!("{:?}", op.protocol)
            } else {
                "OP_RETURN Data".to_string()
            }
        } else {
            output.address.clone().unwrap_or_else(|| "None".to_string())
        };

        doc.push_str(&format!(
            "| {} | {} sats ({:.6} BTC) | {} | `{}` | `{}` |\n",
            output.index,
            output.value_sats,
            output.value_btc,
            output.script_type,
            recipient,
            output.script_pubkey.asm
        ));
    }

    doc
}
