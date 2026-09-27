use crate::model::types::{DecodedTx, ScriptType};

pub fn generate_mermaid_diagram(tx: &DecodedTx) -> String {
    let mut diagram = String::new();
    diagram.push_str("```mermaid\ngraph LR\n");
    diagram.push_str(
        "    classDef inputNode fill:#1e293b,stroke:#38bdf8,stroke-width:2px,color:#f8fafc;\n",
    );
    diagram.push_str(
        "    classDef txNode fill:#0f172a,stroke:#eab308,stroke-width:3px,color:#fef08a;\n",
    );
    diagram.push_str(
        "    classDef outputNode fill:#1e293b,stroke:#22c55e,stroke-width:2px,color:#f8fafc;\n",
    );
    diagram.push_str(
        "    classDef opReturnNode fill:#3b0764,stroke:#d8b4fe,stroke-width:2px,color:#f3e8ff;\n\n",
    );

    // Central Transaction Node
    let short_txid = if tx.txid.len() > 16 {
        format!("{}...{}", &tx.txid[..8], &tx.txid[tx.txid.len() - 8..])
    } else {
        tx.txid.clone()
    };

    let fee_str = if let Some(ref fee) = tx.fee_info {
        format!(
            "Fee: {} sats<br/>{:.1} sat/vB",
            fee.fee_sats, fee.fee_rate_sat_per_vb
        )
    } else {
        "Fee: Unknown (Offline)".to_string()
    };

    diagram.push_str(&format!(
        "    TX[\"<b>Transaction</b><br/><code>{}</code><br/>{} vB | {} WU<br/>{}\"]:::txNode\n\n",
        short_txid, tx.vsize_vb, tx.weight_wu, fee_str
    ));

    // Inputs
    for input in &tx.inputs {
        let node_id = format!("IN_{}", input.index);
        let val_str = input
            .spent_txout
            .as_ref()
            .map(|s| format!("{} sats", s.value_sats))
            .unwrap_or_else(|| "Unknown".to_string());

        let short_outpoint = if tx.is_coinbase {
            "Coinbase (Block Reward)".to_string()
        } else if input.prevout_txid.len() > 12 {
            format!("{}...:{}", &input.prevout_txid[..6], input.prevout_vout)
        } else {
            format!("{}:{}", input.prevout_txid, input.prevout_vout)
        };

        diagram.push_str(&format!(
            "    {}[\"<b>Input #{}</b><br/><code>{}</code><br/>{}\"]:::inputNode\n",
            node_id, input.index, short_outpoint, val_str
        ));
        diagram.push_str(&format!("    {} --> TX\n", node_id));
    }

    diagram.push('\n');

    // Outputs
    for output in &tx.outputs {
        let node_id = format!("OUT_{}", output.index);
        let is_op_return = output.script_type == ScriptType::OpReturn;
        let class_name = if is_op_return {
            "opReturnNode"
        } else {
            "outputNode"
        };

        let addr_display = if is_op_return {
            "OP_RETURN Data".to_string()
        } else if let Some(ref a) = output.address {
            if a.len() > 14 {
                format!("{}...{}", &a[..7], &a[a.len() - 7..])
            } else {
                a.clone()
            }
        } else {
            "Non-standard Script".to_string()
        };

        diagram.push_str(&format!(
            "    {}[\"<b>Output #{}</b> ({})<br/><code>{}</code><br/>{} sats ({:.6} BTC)\"]:::{}\n",
            node_id,
            output.index,
            output.script_type,
            addr_display,
            output.value_sats,
            output.value_btc,
            class_name
        ));
        diagram.push_str(&format!("    TX --> {}\n", node_id));
    }

    diagram.push_str("```\n");
    diagram
}
