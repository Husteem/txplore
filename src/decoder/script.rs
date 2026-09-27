use bitcoin::blockdata::opcodes;
use bitcoin::blockdata::script::Instruction;
use bitcoin::{Address, Network, Script};

use crate::model::types::{
    DecodedScript, OpReturnPayload, OpReturnProtocol, ParsedOpcode, ScriptType, WitnessItem,
    WitnessItemType,
};

pub fn decode_script(script: &Script, _network: Network) -> DecodedScript {
    let raw_hex = hex::encode(script.as_bytes());
    let script_type = classify_script(script);
    let mut opcodes = Vec::new();
    let mut asm_parts = Vec::new();

    let mut offset = 0;
    for instruction in script.instructions() {
        match instruction {
            Ok(Instruction::Op(op)) => {
                let name = format!("{:?}", op);
                let op_hex = format!("{:02x}", op.to_u8());
                let meaning = explain_opcode(op);
                asm_parts.push(name.clone());
                opcodes.push(ParsedOpcode {
                    offset,
                    name,
                    hex: op_hex,
                    is_push: false,
                    push_data_hex: None,
                    meaning,
                });
                offset += 1;
            }
            Ok(Instruction::PushBytes(data)) => {
                let data_bytes = data.as_bytes();
                let push_hex = hex::encode(data_bytes);
                let name = format!("OP_PUSHBYTES_{}", data_bytes.len());
                let meaning = format!("Push {} bytes onto the execution stack", data_bytes.len());
                asm_parts.push(name.clone());
                asm_parts.push(push_hex.clone());

                let push_op_len = if data_bytes.len() < 76 {
                    1
                } else if data_bytes.len() <= 255 {
                    2
                } else {
                    3
                };

                opcodes.push(ParsedOpcode {
                    offset,
                    name,
                    hex: push_hex.clone(),
                    is_push: true,
                    push_data_hex: Some(push_hex),
                    meaning,
                });
                offset += push_op_len + data_bytes.len();
            }
            Err(_) => {
                let err_name = "INVALID_OPCODE".to_string();
                asm_parts.push(err_name.clone());
                opcodes.push(ParsedOpcode {
                    offset,
                    name: err_name,
                    hex: "".to_string(),
                    is_push: false,
                    push_data_hex: None,
                    meaning: "Malformed opcode or push data".to_string(),
                });
                break;
            }
        }
    }

    let asm = asm_parts.join(" ");

    DecodedScript {
        hex: raw_hex,
        asm,
        opcodes,
        script_type,
    }
}

pub fn derive_address_from_script(script: &Script, network: Network) -> Option<String> {
    Address::from_script(script, network)
        .ok()
        .map(|a| a.to_string())
}

pub fn classify_script(script: &Script) -> ScriptType {
    if script.is_p2pkh() {
        ScriptType::P2pkh
    } else if script.is_p2sh() {
        ScriptType::P2sh
    } else if script.is_p2wpkh() {
        ScriptType::P2wpkh
    } else if script.is_p2wsh() {
        ScriptType::P2wsh
    } else if script.is_p2tr() {
        ScriptType::P2tr
    } else if script.is_op_return() {
        ScriptType::OpReturn
    } else if is_ephemeral_anchor(script) {
        ScriptType::Anchor
    } else if is_multisig(script) {
        ScriptType::Multisig
    } else {
        ScriptType::NonStandard
    }
}

fn is_ephemeral_anchor(script: &Script) -> bool {
    let bytes = script.as_bytes();
    bytes == [0x51, 0x02, 0x4e, 0x73]
}

fn is_multisig(script: &Script) -> bool {
    let bytes = script.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    bytes[bytes.len() - 1] == opcodes::all::OP_CHECKMULTISIG.to_u8()
        || bytes[bytes.len() - 1] == opcodes::all::OP_CHECKMULTISIGVERIFY.to_u8()
}

pub fn parse_op_return_payload(script: &Script) -> Option<OpReturnPayload> {
    if !script.is_op_return() {
        return None;
    }

    let mut payload_bytes = Vec::new();
    for inst in script.instructions() {
        if let Ok(Instruction::PushBytes(bytes)) = inst {
            payload_bytes.extend_from_slice(bytes.as_bytes());
        }
    }

    let hex_str = hex::encode(&payload_bytes);
    let ascii_str = std::str::from_utf8(&payload_bytes)
        .ok()
        .map(|s| s.to_string());

    let protocol = if payload_bytes.starts_with(b"omni") {
        OpReturnProtocol::OmniLayer
    } else if payload_bytes.starts_with(&[0x03, 0xb1, 0xe7, 0x18, 0x00])
        || payload_bytes.starts_with(b"OTS")
    {
        OpReturnProtocol::OpenTimestamps
    } else if payload_bytes.starts_with(&[0x14]) || payload_bytes.starts_with(b"R") {
        OpReturnProtocol::Runes
    } else if let Some(ref text) = ascii_str {
        if text
            .chars()
            .all(|c| !c.is_control() || c == '\n' || c == '\t')
        {
            OpReturnProtocol::PlainText(text.clone())
        } else {
            OpReturnProtocol::UnknownProtocol(hex_str.clone())
        }
    } else {
        OpReturnProtocol::UnknownProtocol(hex_str.clone())
    };

    Some(OpReturnPayload {
        hex: hex_str,
        ascii: ascii_str,
        protocol,
    })
}

pub fn decode_witness_item(index: usize, bytes: &[u8]) -> WitnessItem {
    let hex_str = hex::encode(bytes);
    let size = bytes.len();

    if size == 0 {
        return WitnessItem {
            index,
            hex: hex_str,
            size_bytes: size,
            inferred_type: WitnessItemType::Empty,
            description:
                "Empty witness stack element (e.g. OP_CHECKMULTISIG bug workaround or null flag)"
                    .to_string(),
        };
    }

    if size == 64 {
        return WitnessItem {
            index,
            hex: hex_str,
            size_bytes: size,
            inferred_type: WitnessItemType::SchnorrSignature,
            description:
                "BIP340 Schnorr signature (Taproot Key-Spend or Script-Spend, SIGHASH_DEFAULT)"
                    .to_string(),
        };
    } else if size == 65 && bytes[64] != 0 {
        return WitnessItem {
            index,
            hex: hex_str,
            size_bytes: size,
            inferred_type: WitnessItemType::SchnorrSignature,
            description: format!(
                "BIP340 Schnorr signature with sighash byte 0x{:02x}",
                bytes[64]
            ),
        };
    }

    if (70..=73).contains(&size) && bytes[0] == 0x30 {
        let sighash_byte = bytes[size - 1];
        let sighash_str = match sighash_byte {
            0x01 => "SIGHASH_ALL",
            0x02 => "SIGHASH_NONE",
            0x03 => "SIGHASH_SINGLE",
            0x81 => "SIGHASH_ALL | ANYONECANPAY",
            0x82 => "SIGHASH_NONE | ANYONECANPAY",
            0x83 => "SIGHASH_SINGLE | ANYONECANPAY",
            _ => "CUSTOM SIGHASH",
        };
        return WitnessItem {
            index,
            hex: hex_str,
            size_bytes: size,
            inferred_type: WitnessItemType::EcdsaSignature,
            description: format!("DER ECDSA signature ({})", sighash_str),
        };
    }

    if size == 33 && (bytes[0] == 0x02 || bytes[0] == 0x03) {
        return WitnessItem {
            index,
            hex: hex_str,
            size_bytes: size,
            inferred_type: WitnessItemType::PublicKey,
            description: format!(
                "Compressed secp256k1 public key (prefix 0x{:02x})",
                bytes[0]
            ),
        };
    }

    if size >= 33 && (size - 33).is_multiple_of(32) {
        let leaf_version = bytes[0] & 0xfe;
        let parity = bytes[0] & 0x01;
        let num_hashes = (size - 33) / 32;
        return WitnessItem {
            index,
            hex: hex_str,
            size_bytes: size,
            inferred_type: WitnessItemType::TaprootControlBlock,
            description: format!(
                "Taproot Control Block: leaf version 0x{:02x}, parity {}, {} merkle branch proof(s)",
                leaf_version, parity, num_hashes
            ),
        };
    }

    if size == 32 {
        return WitnessItem {
            index,
            hex: hex_str,
            size_bytes: size,
            inferred_type: WitnessItemType::Preimage,
            description:
                "32-byte data element (SHA-256 hash preimage, Taproot x-only pubkey, or secret)"
                    .to_string(),
        };
    }

    WitnessItem {
        index,
        hex: hex_str,
        size_bytes: size,
        inferred_type: WitnessItemType::WitnessScript,
        description: format!("Witness payload/script element ({} bytes)", size),
    }
}

pub fn explain_opcode(op: opcodes::Opcode) -> String {
    let b = op.to_u8();
    match b {
        0x00 => "Push empty byte array (0 or false) onto the stack".to_string(),
        0x51 => "Push number 1 (or true) onto the stack".to_string(),
        0x52..=0x60 => format!("Push number {} onto the stack", b - 0x51 + 1),
        0x76 => "Duplicate the top stack item (OP_DUP)".to_string(),
        0x75 => "Remove the top stack item (OP_DROP)".to_string(),
        0x7c => "Swap the top two stack items (OP_SWAP)".to_string(),
        0xa9 => "Hash top stack item twice: SHA-256 followed by RIPEMD-160 (OP_HASH160)".to_string(),
        0xa8 => "Hash top stack item with SHA-256 (OP_SHA256)".to_string(),
        0xaa => "Hash top stack item twice with SHA-256 (OP_HASH256)".to_string(),
        0xa6 => "Hash top stack item with RIPEMD-160 (OP_RIPEMD160)".to_string(),
        0x87 => "Verify top two stack items are bitwise identical; push 1 if true, 0 if false (OP_EQUAL)".to_string(),
        0x88 => "Verify top two items are identical; abort and fail script if false (OP_EQUALVERIFY)".to_string(),
        0xac => "Verify signature against public key for transaction sighash (OP_CHECKSIG)".to_string(),
        0xad => "Verify signature against public key; abort and fail script if false (OP_CHECKSIGVERIFY)".to_string(),
        0xae => "Verify m-of-n multisignature signatures against public keys (OP_CHECKMULTISIG)".to_string(),
        0xaf => "Verify m-of-n multisignature; abort and fail script if false (OP_CHECKMULTISIGVERIFY)".to_string(),
        0x6a => "Mark output provably unspendable; any data following is stored on-chain (OP_RETURN)".to_string(),
        0xb1 => "Verify transaction locktime is greater than or equal to stack item (BIP65 OP_CHECKLOCKTIMEVERIFY)".to_string(),
        0xb2 => "Verify input sequence number relative locktime is satisfied (BIP112 OP_CHECKSEQUENCEVERIFY)".to_string(),
        0x63 => "Execute following statements if top stack item is non-zero (OP_IF)".to_string(),
        0x64 => "Execute following statements if top stack item is zero (OP_NOTIF)".to_string(),
        0x67 => "Execute alternate branch of preceding OP_IF / OP_NOTIF (OP_ELSE)".to_string(),
        0x68 => "Terminate an OP_IF / OP_ELSE block (OP_ENDIF)".to_string(),
        0x93 => "Add top two integers on the stack (OP_ADD)".to_string(),
        0x94 => "Subtract top integer from second integer (OP_SUB)".to_string(),
        0x69 => "Fail script immediately if top stack item is false (OP_VERIFY)".to_string(),
        _ => format!("Bitcoin opcode 0x{:02x} ({:?})", b, op),
    }
}
