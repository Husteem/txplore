use colored::Colorize;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};

use crate::model::types::{DecodedTx, RbfStatus, ScriptType};

pub fn print_transaction_summary(tx: &DecodedTx) {
    println!("{}", "=========================================================================================================".cyan().bold());
    println!(
        "  {} {}",
        "BITCOIN TRANSACTION EXPLORER".yellow().bold(),
        format!("(Network: {})", tx.network).white()
    );
    println!("{}", "=========================================================================================================".cyan().bold());

    println!(
        "{:<18} {}",
        "Transaction ID:".bold(),
        tx.txid.green().bold()
    );
    println!("{:<18} {}", "Witness TXID:".bold(), tx.wtxid.dimmed());
    println!(
        "{:<18} {}",
        "Classification:".bold(),
        format!("{}", tx.classification).magenta().bold()
    );
    println!("{:<18} v{}", "Version:".bold(), tx.version);

    let segwit_badge = if tx.is_segwit {
        " SegWit Active ".on_green().white().bold()
    } else {
        " Legacy ".on_blue().white().bold()
    };

    let rbf_badge = match tx.rbf_status {
        RbfStatus::Signaling => " BIP125 RBF Signaling ".on_yellow().black().bold(),
        RbfStatus::NotSignaling => " RBF Inactive (Final) ".on_bright_black().white(),
        RbfStatus::Mixed => " Mixed RBF Inputs ".on_purple().white().bold(),
        RbfStatus::Coinbase => " Coinbase Generation ".on_bright_blue().white().bold(),
    };

    println!("{:<18} {} {}", "Protocols:".bold(), segwit_badge, rbf_badge);
    println!(
        "{:<18} {}",
        "Locktime:".bold(),
        tx.locktime.description.cyan()
    );

    if let Some(ref conf) = tx.confirmation_info {
        let conf_str = if conf.confirmed {
            format!(
                "Confirmed (Height: {}, Confirmations: {})",
                conf.block_height.unwrap_or(0),
                conf.confirmations.unwrap_or(1)
            )
            .green()
            .bold()
        } else {
            "Unconfirmed (In Mempool)".yellow().bold()
        };
        println!("{:<18} {}", "Status:".bold(), conf_str);
    }

    println!();

    // Size & Weight Metrics Table
    let mut metrics_table = Table::new();
    metrics_table
        .load_preset(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Virtual Size (vB)")
                .fg(Color::Yellow)
                .add_attribute(Attribute::Bold),
            Cell::new("Weight Units (WU)")
                .fg(Color::Yellow)
                .add_attribute(Attribute::Bold),
            Cell::new("Serialized Size")
                .fg(Color::Yellow)
                .add_attribute(Attribute::Bold),
            Cell::new("Witness Discount")
                .fg(Color::Yellow)
                .add_attribute(Attribute::Bold),
        ]);

    metrics_table.add_row(vec![
        Cell::new(format!("{} vB", tx.vsize_vb)),
        Cell::new(format!("{} WU", tx.weight_wu)),
        Cell::new(format!("{} bytes", tx.size_bytes)),
        Cell::new(format!("{:.1}% savings", tx.discount_ratio * 100.0)),
    ]);
    println!("{}", metrics_table);

    // Fee Info Table (if present)
    if let Some(ref fee) = tx.fee_info {
        let mut fee_table = Table::new();
        fee_table.load_preset(UTF8_ROUND_CORNERS).set_header(vec![
            Cell::new("Total Input Value")
                .fg(Color::Cyan)
                .add_attribute(Attribute::Bold),
            Cell::new("Total Output Value")
                .fg(Color::Cyan)
                .add_attribute(Attribute::Bold),
            Cell::new("Miner Fee")
                .fg(Color::Cyan)
                .add_attribute(Attribute::Bold),
            Cell::new("Fee Rate")
                .fg(Color::Cyan)
                .add_attribute(Attribute::Bold),
        ]);

        fee_table.add_row(vec![
            Cell::new(format!(
                "{} sats ({:.8} BTC)",
                fee.total_input_sats,
                fee.total_input_sats as f64 / 1e8
            )),
            Cell::new(format!(
                "{} sats ({:.8} BTC)",
                fee.total_output_sats,
                fee.total_output_sats as f64 / 1e8
            )),
            Cell::new(format!(
                "{} sats ({:.2}% of in)",
                fee.fee_sats, fee.fee_percentage
            )),
            Cell::new(format!("{:.2} sat/vB", fee.fee_rate_sat_per_vb))
                .fg(Color::Green)
                .add_attribute(Attribute::Bold),
        ]);
        println!("{}", fee_table);
    }

    // Inputs Table
    println!(
        "\n{}",
        format!("INPUTS ({})", tx.inputs.len()).bold().yellow()
    );
    let mut in_table = Table::new();
    in_table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("#").fg(Color::Yellow),
            Cell::new("Previous OutPoint").fg(Color::Yellow),
            Cell::new("Value (sats)").fg(Color::Yellow),
            Cell::new("ScriptSig / Witness").fg(Color::Yellow),
            Cell::new("Sequence / Lock").fg(Color::Yellow),
        ]);

    for input in &tx.inputs {
        let outpoint_str = if tx.is_coinbase {
            "Coinbase (Newly Mined)".to_string()
        } else {
            format!("{}:{}", input.prevout_txid, input.prevout_vout)
        };

        let val_str = input
            .spent_txout
            .as_ref()
            .map(|s| format!("{} sats", s.value_sats))
            .unwrap_or_else(|| "Unknown (Offline)".dimmed().to_string());

        let mut script_desc = Vec::new();
        if let Some(ref cb) = input.coinbase_data {
            if let Some(h) = cb.bip34_height {
                script_desc.push(format!("BIP34 Height: {}", h));
            }
            if !cb.text_representation.is_empty() {
                script_desc.push(format!("Text: \"{}\"", cb.text_representation));
            }
        }
        if let Some(ref ss) = input.script_sig {
            script_desc.push(format!("ScriptSig ({}): {}", ss.script_type, ss.asm));
        }
        if !input.witness.is_empty() {
            script_desc.push(format!("Witness stack ({} items):", input.witness.len()));
            for w in &input.witness {
                script_desc.push(format!("  [{}] {} ({})", w.index, w.description, w.hex));
            }
        }

        let seq_str = if let Some(ref rel) = input.relative_locktime {
            format!("0x{:08x}\n{}", input.sequence, rel.human_readable)
        } else {
            format!("0x{:08x}", input.sequence)
        };

        in_table.add_row(vec![
            Cell::new(input.index.to_string()),
            Cell::new(outpoint_str),
            Cell::new(val_str),
            Cell::new(script_desc.join("\n")),
            Cell::new(seq_str),
        ]);
    }
    println!("{}", in_table);

    // Outputs Table
    println!(
        "\n{}",
        format!("OUTPUTS ({})", tx.outputs.len()).bold().yellow()
    );
    let mut out_table = Table::new();
    out_table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("#").fg(Color::Yellow),
            Cell::new("Value (sats / BTC)").fg(Color::Yellow),
            Cell::new("Script Type").fg(Color::Yellow),
            Cell::new("Recipient Address / Payload").fg(Color::Yellow),
            Cell::new("ScriptPubKey ASM").fg(Color::Yellow),
        ]);

    for output in &tx.outputs {
        let val_display = format!("{} sats\n({:.8} BTC)", output.value_sats, output.value_btc);
        let type_display = format!("{}", output.script_type);

        let recipient_or_payload = if output.script_type == ScriptType::OpReturn {
            if let Some(ref op) = output.op_return_payload {
                match &op.protocol {
                    crate::model::types::OpReturnProtocol::PlainText(t) => {
                        format!("Text: \"{}\"", t)
                    }
                    crate::model::types::OpReturnProtocol::OmniLayer => {
                        "Omni Layer Token Payload".to_string()
                    }
                    crate::model::types::OpReturnProtocol::OpenTimestamps => {
                        "OpenTimestamps Attestation".to_string()
                    }
                    crate::model::types::OpReturnProtocol::Runes => {
                        "Runes Etching/Minting Protocol".to_string()
                    }
                    crate::model::types::OpReturnProtocol::UnknownProtocol(h) => {
                        format!("Hex: 0x{}", h)
                    }
                }
            } else {
                "OP_RETURN Data".to_string()
            }
        } else {
            output
                .address
                .clone()
                .unwrap_or_else(|| "No Address (Non-standard)".to_string())
        };

        out_table.add_row(vec![
            Cell::new(output.index.to_string()),
            Cell::new(val_display),
            Cell::new(type_display),
            Cell::new(recipient_or_payload),
            Cell::new(output.script_pubkey.asm.clone()),
        ]);
    }
    println!("{}", out_table);
    println!("{}", "=========================================================================================================\n".cyan());
}
