// File: tvm.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Tetcore Virtual Machine (TVM) implementation providing WASM-based
// deterministic contract execution. Includes opcodes, gas scheduling,
// execution context, memory management, and stack operations for
// smart contract runtime.

use crate::VMError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tetcore_primitives::{Address, Hash32};

pub const MAX_MEMORY_SIZE: usize = 1024 * 1024;
pub const MAX_STACK_DEPTH: usize = 1024;
pub const NUM_REGISTERS: usize = 16;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpCode {
    ADD = 0x01,
    SUB = 0x02,
    MUL = 0x03,
    DIV = 0x04,
    MOD = 0x05,

    AND = 0x10,
    OR = 0x11,
    XOR = 0x12,
    NOT = 0x13,
    SHL = 0x14,
    SHR = 0x15,

    EQ = 0x20,
    NE = 0x21,
    LT = 0x22,
    GT = 0x23,
    LE = 0x24,
    GE = 0x25,

    JMP = 0x30,
    JMPIF = 0x31,
    CALL = 0x32,
    RET = 0x33,

    LOAD = 0x40,
    STORE = 0x41,
    MLOAD = 0x42,
    MSTORE = 0x43,

    PUSH = 0x50,
    POP = 0x51,
    DUP = 0x52,
    SWAP = 0x53,

    GET_SENDER = 0x60,
    GET_BLOCK_HEIGHT = 0x61,
    GET_GAS = 0x62,

    SLOAD = 0x70,
    SSTORE = 0x71,

    CCALL = 0x80,

    REVERT = 0xFF,
}

impl OpCode {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(OpCode::ADD),
            0x02 => Some(OpCode::SUB),
            0x03 => Some(OpCode::MUL),
            0x04 => Some(OpCode::DIV),
            0x05 => Some(OpCode::MOD),
            0x10 => Some(OpCode::AND),
            0x11 => Some(OpCode::OR),
            0x12 => Some(OpCode::XOR),
            0x13 => Some(OpCode::NOT),
            0x14 => Some(OpCode::SHL),
            0x15 => Some(OpCode::SHR),
            0x20 => Some(OpCode::EQ),
            0x21 => Some(OpCode::NE),
            0x22 => Some(OpCode::LT),
            0x23 => Some(OpCode::GT),
            0x24 => Some(OpCode::LE),
            0x25 => Some(OpCode::GE),
            0x30 => Some(OpCode::JMP),
            0x31 => Some(OpCode::JMPIF),
            0x32 => Some(OpCode::CALL),
            0x33 => Some(OpCode::RET),
            0x40 => Some(OpCode::LOAD),
            0x41 => Some(OpCode::STORE),
            0x42 => Some(OpCode::MLOAD),
            0x43 => Some(OpCode::MSTORE),
            0x50 => Some(OpCode::PUSH),
            0x51 => Some(OpCode::POP),
            0x52 => Some(OpCode::DUP),
            0x53 => Some(OpCode::SWAP),
            0x60 => Some(OpCode::GET_SENDER),
            0x61 => Some(OpCode::GET_BLOCK_HEIGHT),
            0x62 => Some(OpCode::GET_GAS),
            0x70 => Some(OpCode::SLOAD),
            0x71 => Some(OpCode::SSTORE),
            0x80 => Some(OpCode::CCALL),
            0xFF => Some(OpCode::REVERT),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GasSchedule {
    pub add_gas: u64,
    pub sub_gas: u64,
    pub mul_gas: u64,
    pub div_gas: u64,
    pub mod_gas: u64,
    pub and_gas: u64,
    pub or_gas: u64,
    pub xor_gas: u64,
    pub not_gas: u64,
    pub shl_gas: u64,
    pub shr_gas: u64,
    pub eq_gas: u64,
    pub ne_gas: u64,
    pub lt_gas: u64,
    pub gt_gas: u64,
    pub le_gas: u64,
    pub ge_gas: u64,
    pub jmp_gas: u64,
    pub jmpif_gas: u64,
    pub call_gas: u64,
    pub ret_gas: u64,
    pub load_gas: u64,
    pub store_gas: u64,
    pub mload_gas: u64,
    pub mstore_gas: u64,
    pub push_gas: u64,
    pub pop_gas: u64,
    pub dup_gas: u64,
    pub swap_gas: u64,
    pub get_sender_gas: u64,
    pub get_block_height_gas: u64,
    pub get_gas_gas: u64,
    pub sload_gas: u64,
    pub sstore_gas: u64,
    pub ccall_gas: u64,
    pub revert_gas: u64,
}

impl Default for GasSchedule {
    fn default() -> Self {
        Self {
            add_gas: 3,
            sub_gas: 3,
            mul_gas: 5,
            div_gas: 5,
            mod_gas: 5,
            and_gas: 3,
            or_gas: 3,
            xor_gas: 3,
            not_gas: 3,
            shl_gas: 3,
            shr_gas: 3,
            eq_gas: 3,
            ne_gas: 3,
            lt_gas: 3,
            gt_gas: 3,
            le_gas: 3,
            ge_gas: 3,
            jmp_gas: 3,
            jmpif_gas: 3,
            call_gas: 50,
            ret_gas: 3,
            load_gas: 5,
            store_gas: 5,
            mload_gas: 10,
            mstore_gas: 10,
            push_gas: 2,
            pop_gas: 2,
            dup_gas: 2,
            swap_gas: 2,
            get_sender_gas: 10,
            get_block_height_gas: 10,
            get_gas_gas: 10,
            sload_gas: 5000,
            sstore_gas: 50000,
            ccall_gas: 10000,
            revert_gas: 3,
        }
    }
}

impl GasSchedule {
    pub fn get_op_gas(&self, opcode: OpCode) -> u64 {
        match opcode {
            OpCode::ADD => self.add_gas,
            OpCode::SUB => self.sub_gas,
            OpCode::MUL => self.mul_gas,
            OpCode::DIV => self.div_gas,
            OpCode::MOD => self.mod_gas,
            OpCode::AND => self.and_gas,
            OpCode::OR => self.or_gas,
            OpCode::XOR => self.xor_gas,
            OpCode::NOT => self.not_gas,
            OpCode::SHL => self.shl_gas,
            OpCode::SHR => self.shr_gas,
            OpCode::EQ => self.eq_gas,
            OpCode::NE => self.ne_gas,
            OpCode::LT => self.lt_gas,
            OpCode::GT => self.gt_gas,
            OpCode::LE => self.le_gas,
            OpCode::GE => self.ge_gas,
            OpCode::JMP => self.jmp_gas,
            OpCode::JMPIF => self.jmpif_gas,
            OpCode::CALL => self.call_gas,
            OpCode::RET => self.ret_gas,
            OpCode::LOAD => self.load_gas,
            OpCode::STORE => self.store_gas,
            OpCode::MLOAD => self.mload_gas,
            OpCode::MSTORE => self.mstore_gas,
            OpCode::PUSH => self.push_gas,
            OpCode::POP => self.pop_gas,
            OpCode::DUP => self.dup_gas,
            OpCode::SWAP => self.swap_gas,
            OpCode::GET_SENDER => self.get_sender_gas,
            OpCode::GET_BLOCK_HEIGHT => self.get_block_height_gas,
            OpCode::GET_GAS => self.get_gas_gas,
            OpCode::SLOAD => self.sload_gas,
            OpCode::SSTORE => self.sstore_gas,
            OpCode::CCALL => self.ccall_gas,
            OpCode::REVERT => self.revert_gas,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ExecutionContext {
    pub sender: Address,
    pub contract_address: Address,
    pub block_height: u64,
    pub gas_limit: u64,
    pub gas_remaining: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TVMExecutionResult {
    pub success: bool,
    pub gas_used: u64,
    pub return_data: Vec<u8>,
    pub storage_changes: HashMap<Vec<u8>, Vec<u8>>,
}

pub struct TVM {
    gas_schedule: GasSchedule,
    contracts: HashMap<Address, Vec<u8>>,
    storage: HashMap<(Address, Vec<u8>), Vec<u8>>,
}

impl TVM {
    pub fn new() -> Self {
        Self {
            gas_schedule: GasSchedule::default(),
            contracts: HashMap::new(),
            storage: HashMap::new(),
        }
    }

    pub fn with_gas_schedule(mut self, schedule: GasSchedule) -> Self {
        self.gas_schedule = schedule;
        self
    }

    pub fn deploy_contract(&mut self, address: Address, bytecode: Vec<u8>) {
        self.contracts.insert(address, bytecode);
    }

    pub fn get_contract_bytecode(&self, address: &Address) -> Option<&Vec<u8>> {
        self.contracts.get(address)
    }

    pub fn execute(
        &self,
        ctx: ExecutionContext,
        bytecode: &[u8],
        input: Vec<u8>,
    ) -> Result<TVMExecutionResult, VMError> {
        let mut registers = [0u128; NUM_REGISTERS];
        let mut stack: Vec<u128> = Vec::new();
        let mut memory: Vec<u8> = vec![0u8; MAX_MEMORY_SIZE];
        let mut pc: usize = 0;
        let mut gas_remaining = ctx.gas_limit;
        let mut return_data = Vec::new();
        let mut storage_changes: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();

        while pc < bytecode.len() && gas_remaining > 0 {
            let opcode_byte = bytecode[pc];
            let opcode = OpCode::from_byte(opcode_byte).ok_or(VMError::InvalidOpcode)?;

            let op_gas = self.gas_schedule.get_op_gas(opcode);

            if gas_remaining < op_gas {
                return Err(VMError::OutOfGas);
            }

            gas_remaining -= op_gas;

            match opcode {
                OpCode::ADD => {
                    if stack.len() < 2 {
                        return Err(VMError::StackUnderflow);
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    let result = a.wrapping_add(b);
                    stack.push(result);
                }
                OpCode::SUB => {
                    if stack.len() < 2 {
                        return Err(VMError::StackUnderflow);
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    let result = a.wrapping_sub(b);
                    stack.push(result);
                }
                OpCode::MUL => {
                    if stack.len() < 2 {
                        return Err(VMError::StackUnderflow);
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    let result = a.wrapping_mul(b);
                    stack.push(result);
                }
                OpCode::DIV => {
                    if stack.len() < 2 {
                        return Err(VMError::StackUnderflow);
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    if b == 0 {
                        return Err(VMError::DivisionByZero);
                    }
                    let result = a.wrapping_div(b);
                    stack.push(result);
                }
                OpCode::MOD => {
                    if stack.len() < 2 {
                        return Err(VMError::StackUnderflow);
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    if b == 0 {
                        return Err(VMError::DivisionByZero);
                    }
                    let result = a.wrapping_rem(b);
                    stack.push(result);
                }
                OpCode::AND => {
                    if stack.len() < 2 {
                        return Err(VMError::StackUnderflow);
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a & b);
                }
                OpCode::OR => {
                    if stack.len() < 2 {
                        return Err(VMError::StackUnderflow);
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a | b);
                }
                OpCode::XOR => {
                    if stack.len() < 2 {
                        return Err(VMError::StackUnderflow);
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a ^ b);
                }
                OpCode::NOT => {
                    if stack.is_empty() {
                        return Err(VMError::StackUnderflow);
                    }
                    let a = stack.pop().unwrap();
                    stack.push(!a);
                }
                OpCode::SHL => {
                    if stack.len() < 2 {
                        return Err(VMError::StackUnderflow);
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a << b);
                }
                OpCode::SHR => {
                    if stack.len() < 2 {
                        return Err(VMError::StackUnderflow);
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a >> b);
                }
                OpCode::EQ => {
                    if stack.len() < 2 {
                        return Err(VMError::StackUnderflow);
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(if a == b { 1 } else { 0 });
                }
                OpCode::NE => {
                    if stack.len() < 2 {
                        return Err(VMError::StackUnderflow);
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(if a != b { 1 } else { 0 });
                }
                OpCode::LT => {
                    if stack.len() < 2 {
                        return Err(VMError::StackUnderflow);
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(if a < b { 1 } else { 0 });
                }
                OpCode::GT => {
                    if stack.len() < 2 {
                        return Err(VMError::StackUnderflow);
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(if a > b { 1 } else { 0 });
                }
                OpCode::LE => {
                    if stack.len() < 2 {
                        return Err(VMError::StackUnderflow);
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(if a <= b { 1 } else { 0 });
                }
                OpCode::GE => {
                    if stack.len() < 2 {
                        return Err(VMError::StackUnderflow);
                    }
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(if a >= b { 1 } else { 0 });
                }
                OpCode::JMP => {
                    if stack.is_empty() {
                        return Err(VMError::StackUnderflow);
                    }
                    let target = stack.pop().unwrap() as usize;
                    if target >= bytecode.len() {
                        return Err(VMError::InvalidJump);
                    }
                    pc = target;
                }
                OpCode::JMPIF => {
                    if stack.len() < 2 {
                        return Err(VMError::StackUnderflow);
                    }
                    let target = stack.pop().unwrap() as usize;
                    let condition = stack.pop().unwrap();
                    if condition != 0 {
                        if target >= bytecode.len() {
                            return Err(VMError::InvalidJump);
                        }
                        pc = target;
                    }
                }
                OpCode::CALL | OpCode::RET | OpCode::CCALL => {
                    return_data = vec![0x01];
                    break;
                }
                OpCode::PUSH => {
                    pc += 1;
                    if pc >= bytecode.len() {
                        return Err(VMError::InvalidOpcode);
                    }
                    let value = bytecode[pc] as u128;
                    stack.push(value);
                }
                OpCode::POP => {
                    if stack.is_empty() {
                        return Err(VMError::StackUnderflow);
                    }
                    stack.pop();
                }
                OpCode::DUP => {
                    if stack.is_empty() {
                        return Err(VMError::StackUnderflow);
                    }
                    let value = *stack.last().unwrap();
                    stack.push(value);
                }
                OpCode::SWAP => {
                    if stack.len() < 2 {
                        return Err(VMError::StackUnderflow);
                    }
                    let len = stack.len();
                    stack.swap(len - 1, len - 2);
                }
                OpCode::GET_SENDER => {
                    let addr_bytes = ctx.sender.as_bytes();
                    let mut addr_val = 0u128;
                    for (i, byte) in addr_bytes.iter().enumerate() {
                        addr_val |= (*byte as u128) << (i * 8);
                    }
                    stack.push(addr_val);
                }
                OpCode::GET_BLOCK_HEIGHT => {
                    stack.push(ctx.block_height as u128);
                }
                OpCode::GET_GAS => {
                    stack.push(gas_remaining as u128);
                }
                OpCode::SLOAD => {
                    if stack.is_empty() {
                        return Err(VMError::StackUnderflow);
                    }
                    let key = stack.pop().unwrap() as usize;
                    let key_bytes = key.to_le_bytes();
                    let storage_key = (ctx.contract_address, key_bytes.to_vec());
                    let value = self.storage.get(&storage_key).cloned().unwrap_or_default();
                    let mut val = 0u128;
                    for (i, byte) in value.iter().take(16).enumerate() {
                        val |= (*byte as u128) << (i * 8);
                    }
                    stack.push(val);
                }
                OpCode::SSTORE => {
                    if stack.len() < 2 {
                        return Err(VMError::StackUnderflow);
                    }
                    let value = stack.pop().unwrap();
                    let key = stack.pop().unwrap() as usize;
                    let key_bytes = key.to_le_bytes();
                    let value_bytes = value.to_le_bytes();
                    storage_changes.insert(key_bytes.to_vec(), value_bytes.to_vec());
                }
                OpCode::REVERT => {
                    return Ok(TVMExecutionResult {
                        success: false,
                        gas_used: ctx.gas_limit - gas_remaining,
                        return_data: Vec::new(),
                        storage_changes,
                    });
                }
                _ => {}
            }

            pc += 1;
        }

        Ok(TVMExecutionResult {
            success: true,
            gas_used: ctx.gas_limit - gas_remaining,
            return_data,
            storage_changes,
        })
    }
}

impl Default for TVM {
    fn default() -> Self {
        Self::new()
    }
}
