use bitcoin::absolute::LockTime;
use bitcoin::hashes::Hash;
use bitcoin::psbt::Psbt;
use bitcoin::transaction::Version;
use bitcoin::{Amount, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid};
use txplore::decoder::psbt::{analyze_psbt, analyze_psbt_str};

#[test]
fn test_psbt_analysis_unfinalized() {
    let tx = Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: vec![
            TxIn {
                previous_output: OutPoint {
                    txid: Txid::from_slice(&[0x11; 32]).unwrap(),
                    vout: 0,
                },
                script_sig: ScriptBuf::new(),
                sequence: Sequence::MAX,
                witness: bitcoin::Witness::default(),
            },
            TxIn {
                previous_output: OutPoint {
                    txid: Txid::from_slice(&[0x22; 32]).unwrap(),
                    vout: 1,
                },
                script_sig: ScriptBuf::new(),
                sequence: Sequence::MAX,
                witness: bitcoin::Witness::default(),
            },
        ],
        output: vec![TxOut {
            value: Amount::from_sat(80_000),
            script_pubkey: ScriptBuf::new(),
        }],
    };

    let mut psbt = Psbt::from_unsigned_tx(tx).unwrap();

    // Attach witness UTXOs
    psbt.inputs[0].witness_utxo = Some(TxOut {
        value: Amount::from_sat(50_000),
        script_pubkey: ScriptBuf::new(),
    });
    psbt.inputs[1].witness_utxo = Some(TxOut {
        value: Amount::from_sat(40_000),
        script_pubkey: ScriptBuf::new(),
    });

    let analysis = analyze_psbt(&psbt).expect("analyze psbt");
    assert_eq!(analysis.num_inputs, 2);
    assert_eq!(analysis.num_outputs, 1);
    // Total in: 90,000 sats, total out: 80,000 sats -> Fee: 10,000 sats
    assert_eq!(analysis.fee_sats, Some(10_000));
    assert!(analysis.fee_rate_sat_per_vb.is_some());
    assert!(!analysis.is_finalized);
    assert_eq!(analysis.signatures_present, 0);
    assert_eq!(analysis.signatures_required, 2);

    // Test serialization to base64 and string analysis
    let base64_str = psbt.to_string();
    let analysis_from_str = analyze_psbt_str(&base64_str).expect("analyze psbt from base64");
    assert_eq!(analysis_from_str.fee_sats, Some(10_000));
}

#[test]
fn test_psbt_invalid_string() {
    assert!(analyze_psbt_str("not_a_valid_psbt").is_err());
}
