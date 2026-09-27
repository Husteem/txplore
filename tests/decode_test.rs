use bitcoin::absolute::LockTime;
use bitcoin::consensus::encode::serialize_hex;
use bitcoin::hashes::Hash;
use bitcoin::key::Secp256k1;
use bitcoin::opcodes::all::OP_RETURN;
use bitcoin::script::{Builder, PushBytesBuf};
use bitcoin::transaction::Version;
use bitcoin::{
    Amount, Network, OutPoint, PubkeyHash, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid,
    WPubkeyHash, Witness,
};
use txplore::decoder::script::decode_script;
use txplore::decoder::tx::{decode_bitcoin_tx, decode_raw_tx_hex, update_fee_info};
use txplore::model::types::{
    OpReturnProtocol, RbfStatus, ScriptType, SpentTxOut, TxClassification, WitnessItemType,
};

#[test]
fn test_decode_legacy_p2pkh_tx() {
    let dummy_hash = PubkeyHash::from_slice(&[0x11; 20]).unwrap();
    let p2pkh_script = ScriptBuf::new_p2pkh(&dummy_hash);

    let tx = Transaction {
        version: Version::ONE,
        lock_time: LockTime::from_consensus(0),
        input: vec![TxIn {
            previous_output: OutPoint {
                txid: Txid::from_slice(&[0x01; 32]).unwrap(),
                vout: 0,
            },
            script_sig: ScriptBuf::from(vec![0x00, 0x01, 0x02]),
            sequence: Sequence(0xffffffff),
            witness: Witness::default(),
        }],
        output: vec![
            TxOut {
                value: Amount::from_sat(50_000),
                script_pubkey: p2pkh_script.clone(),
            },
            TxOut {
                value: Amount::from_sat(45_000),
                script_pubkey: p2pkh_script,
            },
        ],
    };

    let raw_hex = serialize_hex(&tx);
    let decoded = decode_raw_tx_hex(&raw_hex, Network::Bitcoin).expect("decode legacy tx");

    assert_eq!(decoded.version, 1);
    assert_eq!(decoded.inputs.len(), 1);
    assert_eq!(decoded.outputs.len(), 2);
    assert_eq!(decoded.classification, TxClassification::SimplePayment);
    assert!(!decoded.is_segwit);
    assert_eq!(decoded.outputs[0].value_sats, 50_000);
    assert_eq!(decoded.outputs[0].script_type, ScriptType::P2pkh);
    assert_eq!(decoded.rbf_status, RbfStatus::NotSignaling);
}

#[test]
fn test_decode_segwit_v0_tx() {
    let dummy_wpkh = WPubkeyHash::from_slice(&[0x22; 20]).unwrap();
    let p2wpkh_script = ScriptBuf::new_p2wpkh(&dummy_wpkh);

    let mut witness = Witness::new();
    witness.push(vec![0x30, 0x44, 0x02]); // mock signature
    witness.push(vec![0x02; 33]); // mock pubkey

    let tx = Transaction {
        version: Version::TWO,
        lock_time: LockTime::from_consensus(500_000),
        input: vec![TxIn {
            previous_output: OutPoint {
                txid: Txid::from_slice(&[0x02; 32]).unwrap(),
                vout: 1,
            },
            script_sig: ScriptBuf::new(),
            sequence: Sequence(0xfffffffd), // RBF signaled
            witness,
        }],
        output: vec![TxOut {
            value: Amount::from_sat(90_000),
            script_pubkey: p2wpkh_script,
        }],
    };

    let raw_hex = serialize_hex(&tx);
    let mut decoded = decode_raw_tx_hex(&raw_hex, Network::Bitcoin).expect("decode segwit tx");

    assert_eq!(decoded.version, 2);
    assert!(decoded.is_segwit);
    assert_eq!(decoded.outputs[0].script_type, ScriptType::P2wpkh);
    assert_eq!(decoded.rbf_status, RbfStatus::Signaling);
    assert_eq!(decoded.inputs[0].witness.len(), 2);

    // Test fee calculation
    decoded.inputs[0].spent_txout = Some(SpentTxOut {
        value_sats: 100_000,
        script_pubkey: decode_script(&bitcoin::ScriptBuf::new(), Network::Bitcoin),
        address: None,
    });
    update_fee_info(&mut decoded);

    assert!(decoded.fee_info.is_some());
    let fee_info = decoded.fee_info.unwrap();
    assert_eq!(fee_info.fee_sats, 10_000);
    assert!(fee_info.fee_rate_sat_per_vb > 0.0);
}

#[test]
fn test_decode_taproot_tx() {
    let secp = Secp256k1::new();
    // secp256k1 Generator point G
    let g_bytes =
        hex::decode("0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798").unwrap();
    let internal_key = bitcoin::secp256k1::PublicKey::from_slice(&g_bytes).unwrap();
    let (xonly, _) = internal_key.x_only_public_key();
    let p2tr_script = ScriptBuf::new_p2tr(&secp, xonly, None);

    let mut witness = Witness::new();
    witness.push(vec![0xaa; 64]); // 64-byte Schnorr signature

    let tx = Transaction {
        version: Version::TWO,
        lock_time: LockTime::from_consensus(0),
        input: vec![TxIn {
            previous_output: OutPoint {
                txid: Txid::from_slice(&[0x03; 32]).unwrap(),
                vout: 0,
            },
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness,
        }],
        output: vec![TxOut {
            value: Amount::from_sat(150_000),
            script_pubkey: p2tr_script,
        }],
    };

    let raw_hex = serialize_hex(&tx);
    let decoded = decode_raw_tx_hex(&raw_hex, Network::Bitcoin).expect("decode taproot tx");

    assert_eq!(decoded.classification, TxClassification::TaprootKeySpend);
    assert_eq!(decoded.outputs[0].script_type, ScriptType::P2tr);
    assert_eq!(decoded.inputs[0].witness.len(), 1);
    assert_eq!(
        decoded.inputs[0].witness[0].inferred_type,
        WitnessItemType::SchnorrSignature
    );
}

#[test]
fn test_decode_op_return_protocols() {
    let runes_payload = vec![b'R', 0x01, 0x02, 0x03];
    let push_buf = PushBytesBuf::try_from(runes_payload).unwrap();
    let runes_script = Builder::new()
        .push_opcode(OP_RETURN)
        .push_opcode(bitcoin::opcodes::all::OP_PUSHNUM_13)
        .push_slice(&push_buf)
        .into_script();

    let tx = Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint {
                txid: Txid::from_slice(&[0x04; 32]).unwrap(),
                vout: 0,
            },
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::default(),
        }],
        output: vec![TxOut {
            value: Amount::ZERO,
            script_pubkey: runes_script,
        }],
    };

    let decoded = decode_bitcoin_tx(&tx, 100, Network::Bitcoin).expect("decode op_return");
    assert_eq!(decoded.outputs[0].script_type, ScriptType::OpReturn);
    assert!(decoded.outputs[0].op_return_payload.is_some());
    let op_ret = decoded.outputs[0].op_return_payload.as_ref().unwrap();
    assert_eq!(op_ret.protocol, OpReturnProtocol::Runes);
}

#[test]
fn test_decode_coinbase_tx() {
    let push_buf = PushBytesBuf::try_from(b"/Mined by Antigravity/".to_vec()).unwrap();
    let coinbase_in = TxIn {
        previous_output: OutPoint::null(),
        script_sig: Builder::new()
            .push_int(840_000) // BIP34 block height
            .push_slice(&push_buf)
            .into_script(),
        sequence: Sequence::MAX,
        witness: Witness::default(),
    };

    let tx = Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: vec![coinbase_in],
        output: vec![TxOut {
            value: Amount::from_sat(312_500_000), // 3.125 BTC subsidy
            script_pubkey: ScriptBuf::new_p2wpkh(&WPubkeyHash::from_slice(&[0x33; 20]).unwrap()),
        }],
    };

    let decoded = decode_bitcoin_tx(&tx, 150, Network::Bitcoin).expect("decode coinbase");
    assert!(decoded.is_coinbase);
    assert_eq!(decoded.classification, TxClassification::Coinbase);
    assert_eq!(decoded.rbf_status, RbfStatus::Coinbase);
    assert!(decoded.inputs[0].coinbase_data.is_some());
    let cb_data = decoded.inputs[0].coinbase_data.as_ref().unwrap();
    assert_eq!(cb_data.bip34_height, Some(840_000));
}

#[test]
fn test_decode_invalid_hex() {
    let invalid_hex = "not_a_valid_hex_string_xyz";
    assert!(decode_raw_tx_hex(invalid_hex, Network::Bitcoin).is_err());

    let truncated_hex = "0100000001";
    assert!(decode_raw_tx_hex(truncated_hex, Network::Bitcoin).is_err());
}
