use bitcoin::hashes::{hash160, sha256, Hash};
use bitcoin::opcodes::all::*;
use bitcoin::script::{Builder, PushBytesBuf};
use txplore::decoder::evaluator::ScriptVm;

#[test]
fn test_stack_manipulation_opcodes() {
    let mut vm = ScriptVm::new();
    let script = Builder::new()
        .push_int(10)
        .push_opcode(OP_DUP)
        .push_int(20)
        .push_opcode(OP_SWAP)
        .push_opcode(OP_DROP)
        .into_script();

    let trace = vm.simulate(&script);
    assert!(
        trace.success,
        "Simulation should succeed: {:?}",
        trace.error_message
    );
    assert_eq!(trace.final_stack.len(), 2);
}

#[test]
fn test_arithmetic_opcodes() {
    let mut vm = ScriptVm::new();
    let script = Builder::new()
        .push_int(7)
        .push_int(3)
        .push_opcode(OP_ADD)
        .push_int(4)
        .push_opcode(OP_SUB)
        .push_opcode(OP_1ADD)
        .into_script();

    let trace = vm.simulate(&script);
    assert!(
        trace.success,
        "Simulation failed: {:?}",
        trace.error_message
    );
    assert_eq!(trace.final_stack.len(), 1);
    let val_hex = &trace.final_stack[0];
    let val_bytes = hex::decode(val_hex).unwrap();
    assert_eq!(val_bytes, vec![0x07]);
}

#[test]
fn test_conditional_execution_true_branch() {
    let mut vm = ScriptVm::new();
    let script = Builder::new()
        .push_opcode(OP_PUSHNUM_1)
        .push_opcode(OP_IF)
        .push_int(42)
        .push_opcode(OP_ELSE)
        .push_int(99)
        .push_opcode(OP_ENDIF)
        .into_script();

    let trace = vm.simulate(&script);
    assert!(trace.success);
    assert_eq!(trace.final_stack.len(), 1);
    let val_bytes = hex::decode(&trace.final_stack[0]).unwrap();
    assert_eq!(val_bytes, vec![42]);
}

#[test]
fn test_conditional_execution_false_branch() {
    let mut vm = ScriptVm::new();
    let script = Builder::new()
        .push_opcode(OP_PUSHBYTES_0)
        .push_opcode(OP_IF)
        .push_int(42)
        .push_opcode(OP_ELSE)
        .push_int(99)
        .push_opcode(OP_ENDIF)
        .into_script();

    let trace = vm.simulate(&script);
    assert!(trace.success);
    assert_eq!(trace.final_stack.len(), 1);
    let val_bytes = hex::decode(&trace.final_stack[0]).unwrap();
    assert_eq!(val_bytes, vec![99]);
}

#[test]
fn test_hash_opcodes() {
    let data = b"bitcoin";
    let expected_sha256 = sha256::Hash::hash(data).to_byte_array();
    let expected_hash160 = hash160::Hash::hash(data).to_byte_array();

    let push_buf = PushBytesBuf::try_from(data.to_vec()).unwrap();
    let mut vm = ScriptVm::new();
    let script = Builder::new()
        .push_slice(&push_buf)
        .push_opcode(OP_SHA256)
        .into_script();

    let trace = vm.simulate(&script);
    assert!(trace.success);
    assert_eq!(trace.final_stack.len(), 1);
    assert_eq!(trace.final_stack[0], hex::encode(expected_sha256));

    let mut vm160 = ScriptVm::new();
    let script160 = Builder::new()
        .push_slice(&push_buf)
        .push_opcode(OP_HASH160)
        .into_script();

    let trace160 = vm160.simulate(&script160);
    assert!(trace160.success);
    assert_eq!(trace160.final_stack[0], hex::encode(expected_hash160));
}

#[test]
fn test_verify_failure() {
    let mut vm = ScriptVm::new();
    let script = Builder::new()
        .push_int(1)
        .push_int(2)
        .push_opcode(OP_EQUALVERIFY) // 1 != 2, should fail
        .into_script();

    let trace = vm.simulate(&script);
    assert!(!trace.success);
    assert!(trace.error_message.is_some());
}

#[test]
fn test_stack_underflow() {
    let mut vm = ScriptVm::new();
    let script = Builder::new()
        .push_opcode(OP_DROP) // Drop on empty stack
        .into_script();

    let trace = vm.simulate(&script);
    assert!(!trace.success);
    assert!(trace.error_message.unwrap().contains("underflow"));
}
