use bitcoin::blockdata::opcodes;
use bitcoin::blockdata::script::Instruction;
use bitcoin::hashes::{hash160, ripemd160, sha256, Hash};
use bitcoin::Script;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptExecutionTrace {
    pub initial_stack: Vec<String>,
    pub steps: Vec<ExecutionStep>,
    pub final_stack: Vec<String>,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub step_index: usize,
    pub instruction: String,
    pub stack_before: Vec<String>,
    pub stack_after: Vec<String>,
    pub description: String,
}

pub struct ScriptVm {
    stack: Vec<Vec<u8>>,
    alt_stack: Vec<Vec<u8>>,
}

impl Default for ScriptVm {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptVm {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            alt_stack: Vec::new(),
        }
    }

    pub fn with_initial_stack(initial_items: &[Vec<u8>]) -> Self {
        Self {
            stack: initial_items.to_vec(),
            alt_stack: Vec::new(),
        }
    }

    pub fn simulate(&mut self, script: &Script) -> ScriptExecutionTrace {
        let initial_stack_formatted: Vec<String> = self.stack.iter().map(hex::encode).collect();
        let mut steps = Vec::new();
        let mut step_counter = 0;
        let mut success = true;
        let mut error_message = None;

        let instructions: Vec<_> = script.instructions().collect();
        let mut pc = 0;
        let mut condition_stack: Vec<bool> = Vec::new();

        while pc < instructions.len() {
            let inst = match &instructions[pc] {
                Ok(i) => i,
                Err(e) => {
                    success = false;
                    error_message = Some(format!("Script parse error: {:?}", e));
                    break;
                }
            };

            let executing = condition_stack.iter().all(|&c| c);

            let stack_before_formatted: Vec<String> = self.stack.iter().map(hex::encode).collect();

            match inst {
                Instruction::PushBytes(bytes) => {
                    if executing {
                        self.stack.push(bytes.as_bytes().to_vec());
                        steps.push(ExecutionStep {
                            step_index: step_counter,
                            instruction: format!("PUSHBYTES ({})", bytes.len()),
                            stack_before: stack_before_formatted,
                            stack_after: self.stack.iter().map(hex::encode).collect(),
                            description: format!(
                                "Pushed 0x{} onto stack",
                                hex::encode(bytes.as_bytes())
                            ),
                        });
                    }
                }
                Instruction::Op(op) => {
                    let opcode_byte = op.to_u8();

                    // Handle control flow opcodes
                    if opcode_byte == 0x63 {
                        // OP_IF
                        if executing {
                            if let Some(top) = self.stack.pop() {
                                let branch = !is_zero(&top);
                                condition_stack.push(branch);
                                steps.push(ExecutionStep {
                                    step_index: step_counter,
                                    instruction: "OP_IF".to_string(),
                                    stack_before: stack_before_formatted,
                                    stack_after: self.stack.iter().map(hex::encode).collect(),
                                    description: format!("Evaluated OP_IF condition: {}", branch),
                                });
                            } else {
                                success = false;
                                error_message = Some("Stack underflow on OP_IF".to_string());
                                break;
                            }
                        } else {
                            condition_stack.push(false);
                            steps.push(ExecutionStep {
                                step_index: step_counter,
                                instruction: "OP_IF (skipped)".to_string(),
                                stack_before: stack_before_formatted.clone(),
                                stack_after: stack_before_formatted,
                                description: "Skipping non-executing branch".to_string(),
                            });
                        }
                        step_counter += 1;
                        pc += 1;
                        continue;
                    } else if opcode_byte == 0x64 {
                        // OP_NOTIF
                        if executing {
                            if let Some(top) = self.stack.pop() {
                                let branch = is_zero(&top);
                                condition_stack.push(branch);
                                steps.push(ExecutionStep {
                                    step_index: step_counter,
                                    instruction: "OP_NOTIF".to_string(),
                                    stack_before: stack_before_formatted,
                                    stack_after: self.stack.iter().map(hex::encode).collect(),
                                    description: format!(
                                        "Evaluated OP_NOTIF condition: {}",
                                        branch
                                    ),
                                });
                            } else {
                                success = false;
                                error_message = Some("Stack underflow on OP_NOTIF".to_string());
                                break;
                            }
                        } else {
                            condition_stack.push(false);
                        }
                        step_counter += 1;
                        pc += 1;
                        continue;
                    } else if opcode_byte == 0x67 {
                        // OP_ELSE
                        if let Some(last) = condition_stack.pop() {
                            let parent_exec = condition_stack.iter().all(|&c| c);
                            if parent_exec {
                                condition_stack.push(!last);
                            } else {
                                condition_stack.push(false);
                            }
                            steps.push(ExecutionStep {
                                step_index: step_counter,
                                instruction: "OP_ELSE".to_string(),
                                stack_before: stack_before_formatted.clone(),
                                stack_after: stack_before_formatted,
                                description: "Switched branch in OP_ELSE".to_string(),
                            });
                        } else {
                            success = false;
                            error_message = Some("Unmatched OP_ELSE without OP_IF".to_string());
                            break;
                        }
                        step_counter += 1;
                        pc += 1;
                        continue;
                    } else if opcode_byte == 0x68 {
                        // OP_ENDIF
                        if condition_stack.pop().is_some() {
                            steps.push(ExecutionStep {
                                step_index: step_counter,
                                instruction: "OP_ENDIF".to_string(),
                                stack_before: stack_before_formatted.clone(),
                                stack_after: stack_before_formatted,
                                description: "Closed conditional block (OP_ENDIF)".to_string(),
                            });
                        } else {
                            success = false;
                            error_message = Some("Unmatched OP_ENDIF without OP_IF".to_string());
                            break;
                        }
                        step_counter += 1;
                        pc += 1;
                        continue;
                    }

                    if !executing {
                        step_counter += 1;
                        pc += 1;
                        continue;
                    }

                    // Standard Opcode Execution
                    let res = self.execute_opcode(*op);
                    let (desc, ok) = match res {
                        Ok(d) => (d, true),
                        Err(e) => (e, false),
                    };

                    steps.push(ExecutionStep {
                        step_index: step_counter,
                        instruction: format!("{:?}", op),
                        stack_before: stack_before_formatted,
                        stack_after: self.stack.iter().map(hex::encode).collect(),
                        description: desc.clone(),
                    });

                    if !ok {
                        success = false;
                        error_message = Some(desc);
                        break;
                    }
                }
            }

            step_counter += 1;
            pc += 1;
        }

        if success && !condition_stack.is_empty() {
            success = false;
            error_message = Some("Unclosed conditional OP_IF/OP_NOTIF block".to_string());
        }

        if success {
            if let Some(top) = self.stack.last() {
                if is_zero(top) {
                    success = false;
                    error_message =
                        Some("Script evaluated to false: top stack element is 0".to_string());
                }
            } else {
                success = false;
                error_message = Some("Script stack is empty upon termination".to_string());
            }
        }

        let final_stack_formatted: Vec<String> = self.stack.iter().map(hex::encode).collect();

        ScriptExecutionTrace {
            initial_stack: initial_stack_formatted,
            steps,
            final_stack: final_stack_formatted,
            success,
            error_message,
        }
    }

    fn execute_opcode(&mut self, op: opcodes::Opcode) -> Result<String, String> {
        let b = op.to_u8();
        match b {
            0x00 => {
                self.stack.push(vec![]);
                Ok("Pushed empty vector (OP_0) onto stack".to_string())
            }
            0x51 => {
                self.stack.push(vec![0x01]);
                Ok("Pushed 1 (OP_1 / true) onto stack".to_string())
            }
            0x52..=0x60 => {
                let n = (b - 0x51) + 1;
                self.stack.push(vec![n]);
                Ok(format!("Pushed {} onto stack", n))
            }
            // Stack Operations
            0x76 => {
                // OP_DUP
                let top = self
                    .stack
                    .last()
                    .ok_or("Stack underflow on OP_DUP")?
                    .clone();
                self.stack.push(top);
                Ok("Duplicated top stack element (OP_DUP)".to_string())
            }
            0x75 => {
                // OP_DROP
                self.stack.pop().ok_or("Stack underflow on OP_DROP")?;
                Ok("Dropped top stack element (OP_DROP)".to_string())
            }
            0x7c => {
                // OP_SWAP
                let len = self.stack.len();
                if len < 2 {
                    return Err("Stack underflow on OP_SWAP".to_string());
                }
                self.stack.swap(len - 1, len - 2);
                Ok("Swapped top two stack elements (OP_SWAP)".to_string())
            }
            0x78 => {
                // OP_OVER
                let len = self.stack.len();
                if len < 2 {
                    return Err("Stack underflow on OP_OVER".to_string());
                }
                let item = self.stack[len - 2].clone();
                self.stack.push(item);
                Ok("Copied second item to top of stack (OP_OVER)".to_string())
            }
            0x7b => {
                // OP_ROT
                let len = self.stack.len();
                if len < 3 {
                    return Err("Stack underflow on OP_ROT".to_string());
                }
                let item = self.stack.remove(len - 3);
                self.stack.push(item);
                Ok("Rotated top three items (OP_ROT)".to_string())
            }
            0x6e => {
                // OP_2DUP
                let len = self.stack.len();
                if len < 2 {
                    return Err("Stack underflow on OP_2DUP".to_string());
                }
                let item1 = self.stack[len - 2].clone();
                let item2 = self.stack[len - 1].clone();
                self.stack.push(item1);
                self.stack.push(item2);
                Ok("Duplicated top two items (OP_2DUP)".to_string())
            }
            0x6d => {
                // OP_2DROP
                if self.stack.len() < 2 {
                    return Err("Stack underflow on OP_2DROP".to_string());
                }
                self.stack.pop();
                self.stack.pop();
                Ok("Dropped top two stack elements (OP_2DROP)".to_string())
            }
            // Arithmetic Operations
            0x8b => {
                // OP_1ADD
                let top = self.stack.pop().ok_or("Stack underflow on OP_1ADD")?;
                let val = bytes_to_i64(&top);
                let res = val + 1;
                self.stack.push(i64_to_bytes(res));
                Ok(format!("Incremented {} to {} (OP_1ADD)", val, res))
            }
            0x8c => {
                // OP_1SUB
                let top = self.stack.pop().ok_or("Stack underflow on OP_1SUB")?;
                let val = bytes_to_i64(&top);
                let res = val - 1;
                self.stack.push(i64_to_bytes(res));
                Ok(format!("Decremented {} to {} (OP_1SUB)", val, res))
            }
            0x8f => {
                // OP_NEGATE
                let top = self.stack.pop().ok_or("Stack underflow on OP_NEGATE")?;
                let val = bytes_to_i64(&top);
                self.stack.push(i64_to_bytes(-val));
                Ok(format!("Negated {} to {} (OP_NEGATE)", val, -val))
            }
            0x90 => {
                // OP_ABS
                let top = self.stack.pop().ok_or("Stack underflow on OP_ABS")?;
                let val = bytes_to_i64(&top);
                self.stack.push(i64_to_bytes(val.abs()));
                Ok(format!("Computed absolute value of {} (OP_ABS)", val))
            }
            0x93 => {
                // OP_ADD
                let b_item = self.stack.pop().ok_or("Stack underflow on OP_ADD")?;
                let a_item = self.stack.pop().ok_or("Stack underflow on OP_ADD")?;
                let a = bytes_to_i64(&a_item);
                let b = bytes_to_i64(&b_item);
                let sum = a + b;
                self.stack.push(i64_to_bytes(sum));
                Ok(format!("Added {} + {} = {} (OP_ADD)", a, b, sum))
            }
            0x94 => {
                // OP_SUB
                let b_item = self.stack.pop().ok_or("Stack underflow on OP_SUB")?;
                let a_item = self.stack.pop().ok_or("Stack underflow on OP_SUB")?;
                let a = bytes_to_i64(&a_item);
                let b = bytes_to_i64(&b_item);
                let diff = a - b;
                self.stack.push(i64_to_bytes(diff));
                Ok(format!("Subtracted {} - {} = {} (OP_SUB)", a, b, diff))
            }
            0x9a => {
                // OP_NUMEQUAL
                let b_item = self.stack.pop().ok_or("Stack underflow on OP_NUMEQUAL")?;
                let a_item = self.stack.pop().ok_or("Stack underflow on OP_NUMEQUAL")?;
                let eq = bytes_to_i64(&a_item) == bytes_to_i64(&b_item);
                self.stack.push(if eq { vec![0x01] } else { vec![] });
                Ok(format!("Compared numeric equality: {}", eq))
            }
            0x9f => {
                // OP_LESSTHAN
                let b_item = self.stack.pop().ok_or("Stack underflow on OP_LESSTHAN")?;
                let a_item = self.stack.pop().ok_or("Stack underflow on OP_LESSTHAN")?;
                let lt = bytes_to_i64(&a_item) < bytes_to_i64(&b_item);
                self.stack.push(if lt { vec![0x01] } else { vec![] });
                Ok(format!("Evaluated numeric less than: {}", lt))
            }
            0xa0 => {
                // OP_GREATERTHAN
                let b_item = self
                    .stack
                    .pop()
                    .ok_or("Stack underflow on OP_GREATERTHAN")?;
                let a_item = self
                    .stack
                    .pop()
                    .ok_or("Stack underflow on OP_GREATERTHAN")?;
                let gt = bytes_to_i64(&a_item) > bytes_to_i64(&b_item);
                self.stack.push(if gt { vec![0x01] } else { vec![] });
                Ok(format!("Evaluated numeric greater than: {}", gt))
            }
            // Hashes
            0xa9 => {
                let top = self.stack.pop().ok_or("Stack underflow on OP_HASH160")?;
                let digest = hash160::Hash::hash(&top);
                self.stack.push(digest.to_byte_array().to_vec());
                Ok(format!(
                    "Hash160 computed: 0x{}",
                    hex::encode(digest.to_byte_array())
                ))
            }
            0xa8 => {
                let top = self.stack.pop().ok_or("Stack underflow on OP_SHA256")?;
                let digest = sha256::Hash::hash(&top);
                self.stack.push(digest.to_byte_array().to_vec());
                Ok(format!(
                    "SHA-256 computed: 0x{}",
                    hex::encode(digest.to_byte_array())
                ))
            }
            0xaa => {
                let top = self.stack.pop().ok_or("Stack underflow on OP_HASH256")?;
                let d1 = sha256::Hash::hash(&top);
                let d2 = sha256::Hash::hash(d1.as_byte_array());
                self.stack.push(d2.to_byte_array().to_vec());
                Ok(format!(
                    "Double SHA-256 computed: 0x{}",
                    hex::encode(d2.to_byte_array())
                ))
            }
            0xa6 => {
                let top = self.stack.pop().ok_or("Stack underflow on OP_RIPEMD160")?;
                let digest = ripemd160::Hash::hash(&top);
                self.stack.push(digest.to_byte_array().to_vec());
                Ok(format!(
                    "RIPEMD-160 computed: 0x{}",
                    hex::encode(digest.to_byte_array())
                ))
            }
            0x87 => {
                let a = self.stack.pop().ok_or("Stack underflow on OP_EQUAL")?;
                let b = self.stack.pop().ok_or("Stack underflow on OP_EQUAL")?;
                let eq = a == b;
                self.stack.push(if eq { vec![0x01] } else { vec![] });
                Ok(format!("Compared equality: {}", eq))
            }
            0x88 => {
                let a = self
                    .stack
                    .pop()
                    .ok_or("Stack underflow on OP_EQUALVERIFY")?;
                let b = self
                    .stack
                    .pop()
                    .ok_or("Stack underflow on OP_EQUALVERIFY")?;
                if a == b {
                    Ok("Elements matched equality check; verified and consumed".to_string())
                } else {
                    Err("OP_EQUALVERIFY failed: elements do not match".to_string())
                }
            }
            0xac => {
                let pubkey = self
                    .stack
                    .pop()
                    .ok_or("Stack underflow (missing pubkey) on OP_CHECKSIG")?;
                let sig = self
                    .stack
                    .pop()
                    .ok_or("Stack underflow (missing sig) on OP_CHECKSIG")?;
                let is_valid = !sig.is_empty() && !pubkey.is_empty();
                self.stack.push(if is_valid { vec![0x01] } else { vec![] });
                Ok(format!(
                    "Simulated OP_CHECKSIG: verified pubkey ({} bytes) against signature ({} bytes)",
                    pubkey.len(),
                    sig.len()
                ))
            }
            0xad => {
                let pubkey = self
                    .stack
                    .pop()
                    .ok_or("Stack underflow on OP_CHECKSIGVERIFY")?;
                let sig = self
                    .stack
                    .pop()
                    .ok_or("Stack underflow on OP_CHECKSIGVERIFY")?;
                if !sig.is_empty() && !pubkey.is_empty() {
                    Ok("OP_CHECKSIGVERIFY succeeded".to_string())
                } else {
                    Err("OP_CHECKSIGVERIFY failed: invalid signature or key".to_string())
                }
            }
            0x6a => Err("OP_RETURN executed: script explicitly failed as unspendable".to_string()),
            0x69 => {
                let top = self.stack.pop().ok_or("Stack underflow on OP_VERIFY")?;
                if !is_zero(&top) {
                    Ok("OP_VERIFY succeeded: top element was true".to_string())
                } else {
                    Err("OP_VERIFY failed: top element was false".to_string())
                }
            }
            0x6b => {
                let top = self.stack.pop().ok_or("Stack underflow on OP_TOALTSTACK")?;
                self.alt_stack.push(top);
                Ok("Moved top stack element to alt-stack".to_string())
            }
            0x6c => {
                let top = self
                    .alt_stack
                    .pop()
                    .ok_or("Alt-stack underflow on OP_FROMALTSTACK")?;
                self.stack.push(top);
                Ok("Moved top alt-stack element back to main stack".to_string())
            }
            _ => Ok(format!("Executed opcode {:?}", op)),
        }
    }
}

fn is_zero(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return true;
    }
    bytes.iter().all(|&b| b == 0)
}

fn bytes_to_i64(bytes: &[u8]) -> i64 {
    if bytes.is_empty() {
        return 0;
    }
    let mut val = 0i64;
    for (i, &b) in bytes.iter().enumerate().take(8) {
        if i == bytes.len() - 1 {
            let magnitude = (b & 0x7f) as i64;
            val |= magnitude << (8 * i);
            if (b & 0x80) != 0 {
                val = -val;
            }
        } else {
            val |= (b as i64) << (8 * i);
        }
    }
    val
}

fn i64_to_bytes(n: i64) -> Vec<u8> {
    if n == 0 {
        return vec![];
    }
    let negative = n < 0;
    let mut abs_n = n.unsigned_abs();
    let mut bytes = Vec::new();
    while abs_n > 0 {
        bytes.push((abs_n & 0xff) as u8);
        abs_n >>= 8;
    }
    if (bytes.last().unwrap() & 0x80) != 0 {
        bytes.push(if negative { 0x80 } else { 0x00 });
    } else if negative {
        let last = bytes.len() - 1;
        bytes[last] |= 0x80;
    }
    bytes
}
