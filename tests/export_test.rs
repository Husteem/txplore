use bitcoin::absolute::LockTime;
use bitcoin::hashes::Hash;
use bitcoin::transaction::Version;
use bitcoin::{
    Amount, Network, OutPoint, PubkeyHash, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid,
    Witness,
};
use txplore::decoder::script::decode_script;
use txplore::decoder::tx::{decode_bitcoin_tx, update_fee_info};
use txplore::model::types::SpentTxOut;
use txplore::render::json::to_json_pretty;
use txplore::render::markdown::generate_markdown_report;
use txplore::render::mermaid::generate_mermaid_diagram;
use txplore::render::table::print_transaction_summary;

fn create_sample_tx() -> txplore::model::types::DecodedTx {
    let dummy_hash = PubkeyHash::from_slice(&[0x11; 20]).unwrap();
    let tx = Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint {
                txid: Txid::from_slice(&[0xaa; 32]).unwrap(),
                vout: 0,
            },
            script_sig: ScriptBuf::new(),
            sequence: Sequence(0xfffffffd),
            witness: Witness::default(),
        }],
        output: vec![
            TxOut {
                value: Amount::from_sat(45_000),
                script_pubkey: ScriptBuf::new_p2pkh(&dummy_hash),
            },
            TxOut {
                value: Amount::from_sat(50_000),
                script_pubkey: ScriptBuf::new_p2pkh(&dummy_hash),
            },
        ],
    };

    let mut decoded = decode_bitcoin_tx(&tx, 200, Network::Bitcoin).unwrap();
    decoded.inputs[0].spent_txout = Some(SpentTxOut {
        value_sats: 100_000,
        script_pubkey: decode_script(&bitcoin::ScriptBuf::new(), Network::Bitcoin),
        address: None,
    });
    update_fee_info(&mut decoded);
    decoded
}

#[test]
fn test_render_json_output() {
    let tx = create_sample_tx();
    let json_str = to_json_pretty(&tx).expect("render json");
    assert!(json_str.contains("\"txid\""));
    assert!(json_str.contains("\"inputs\""));
    assert!(json_str.contains("\"outputs\""));
    assert!(json_str.contains("\"fee_info\""));

    // Verify it is valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("valid json");
    assert_eq!(parsed["version"], 2);
}

#[test]
fn test_render_markdown_output() {
    let tx = create_sample_tx();
    let md = generate_markdown_report(&tx);
    assert!(md.contains("# Bitcoin Transaction Technical Report"));
    assert!(md.contains("## 1. Executive Summary"));
    assert!(md.contains("## 2. Visual Transaction DAG"));
    assert!(md.contains("## 3. Inputs Breakdown"));
    assert!(md.contains("## 4. Outputs Breakdown"));
    assert!(md.contains("45000 sats"));
}

#[test]
fn test_render_mermaid_output() {
    let tx = create_sample_tx();
    let mermaid = generate_mermaid_diagram(&tx);
    assert!(mermaid.contains("graph LR"));
    assert!(mermaid.contains("classDef"));
    assert!(mermaid.contains("TX["));
}

#[test]
fn test_render_cli_tables() {
    let tx = create_sample_tx();
    // Verify print_transaction_summary executes without panicking
    print_transaction_summary(&tx);
}
