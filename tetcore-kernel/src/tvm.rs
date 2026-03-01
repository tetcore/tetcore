// File: tvm.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Tetcore Virtual Machine (TVM) implementation for deterministic smart contract execution.
// Provides gas metering, contract storage, deployment, invocation, and event emission.

use crate::{Address, Hash32};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub const MAX_CONTRACT_CODE_SIZE: usize = 65536;
pub const MAX_STORAGE_SIZE: usize = 1048576;
pub const MAX_STACK_SIZE: usize = 1024;
pub const MAX_CALL_DEPTH: usize = 64;
pub const MAX_MEMORY_PAGES: usize = 256;
pub const MEMORY_PAGE_SIZE: usize = 65536;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Opcode {
    Nop,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Xor,
    Not,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    LtU,
    GtU,
    Min,
    Max,
    MulDiv,
    AddMod,
    MulMod,

    LoadImm,
    LoadConst,
    Store,
    Load,

    Mov,
    Swap,
    Dup,
    Drop,

    Call,
    CallExt,
    Return,
    DelegateCall,

    Jump,
    JumpI,
    Branch,

    PushStack,
    PopStack,

    Gas,
    GasLimit,
    GasPrice,
    Balance,
    Address,
    Origin,
    Caller,
    CallValue,
    CallDataSize,
    CallDataLoad,
    CallDataCopy,
    CodeSize,
    CodeCopy,
    ExtCodeSize,
    ExtCodeCopy,

    Create,
    Create2,
    CallCode,
    SelfDestruct,

    BlockNumber,
    Timestamp,
    Difficulty,
    Coinbase,
    BlockHash,
    BlockGasLimit,

    Log0,
    Log1,
    Log2,
    Log3,
    Log4,

    Revert,
    Invalid,
    Stop,
}

impl Opcode {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x00 => Some(Opcode::Stop),
            0x01 => Some(Opcode::Add),
            0x02 => Some(Opcode::Sub),
            0x03 => Some(Opcode::Mul),
            0x04 => Some(Opcode::Div),
            0x05 => Some(Opcode::Mod),
            0x06 => Some(Opcode::And),
            0x07 => Some(Opcode::Or),
            0x08 => Some(Opcode::Xor),
            0x09 => Some(Opcode::Not),
            0x0a => Some(Opcode::Shl),
            0x0b => Some(Opcode::Shr),
            0x0c => Some(Opcode::AddMod),
            0x0d => Some(Opcode::MulMod),
            0x0e => Some(Opcode::Eq),
            0x0f => Some(Opcode::Ne),
            0x10 => Some(Opcode::Lt),
            0x11 => Some(Opcode::Gt),
            0x12 => Some(Opcode::Le),
            0x13 => Some(Opcode::Ge),
            0x14 => Some(Opcode::LtU),
            0x15 => Some(Opcode::GtU),
            0x16 => Some(Opcode::Min),
            0x17 => Some(Opcode::Max),
            0x18 => Some(Opcode::MulDiv),

            0x20 => Some(Opcode::LoadImm),
            0x21 => Some(Opcode::LoadConst),
            0x22 => Some(Opcode::Store),
            0x23 => Some(Opcode::Load),

            0x30 => Some(Opcode::Address),
            0x31 => Some(Opcode::Balance),
            0x32 => Some(Opcode::Origin),
            0x33 => Some(Opcode::Caller),
            0x34 => Some(Opcode::CallValue),
            0x35 => Some(Opcode::CallDataSize),
            0x36 => Some(Opcode::CallDataLoad),
            0x37 => Some(Opcode::CallDataCopy),
            0x38 => Some(Opcode::CodeSize),
            0x39 => Some(Opcode::CodeCopy),
            0x3a => Some(Opcode::ExtCodeSize),
            0x3b => Some(Opcode::ExtCodeCopy),

            0x40 => Some(Opcode::GasLimit),
            0x41 => Some(Opcode::GasPrice),
            0x42 => Some(Opcode::BlockNumber),
            0x43 => Some(Opcode::Timestamp),
            0x44 => Some(Opcode::Difficulty),
            0x45 => Some(Opcode::Coinbase),
            0x46 => Some(Opcode::BlockHash),
            0x47 => Some(Opcode::BlockGasLimit),

            0x50 => Some(Opcode::PopStack),
            0x51 => Some(Opcode::PushStack),
            0x52 => Some(Opcode::Mov),
            0x53 => Some(Opcode::Swap),
            0x54 => Some(Opcode::Dup),
            0x55 => Some(Opcode::Drop),

            0x60 => Some(Opcode::Gas),

            0x70 => Some(Opcode::Jump),
            0x71 => Some(Opcode::JumpI),
            0x72 => Some(Opcode::Branch),

            0x80 => Some(Opcode::Call),
            0x81 => Some(Opcode::Return),
            0x82 => Some(Opcode::DelegateCall),
            0x83 => Some(Opcode::CallExt),

            0xa0 => Some(Opcode::Log0),
            0xa1 => Some(Opcode::Log1),
            0xa2 => Some(Opcode::Log2),
            0xa3 => Some(Opcode::Log3),
            0xa4 => Some(Opcode::Log4),

            0xf0 => Some(Opcode::Create),
            0xf1 => Some(Opcode::CallCode),
            0xf2 => Some(Opcode::SelfDestruct),
            0xf3 => Some(Opcode::Revert),
            0xfe => Some(Opcode::Invalid),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GasSchedule {
    pub step: u64,
    pub jumpdest: u64,
    pub log: u64,
    pub create: u64,
    pub call: u64,
    pub memory: u64,
    pub storage_read: u64,
    pub storage_write: u64,
    pub call_storage_read: u64,
    pub call_storage_write: u64,
}

impl Default for GasSchedule {
    fn default() -> Self {
        Self {
            step: 1,
            jumpdest: 1,
            log: 20,
            create: 100,
            call: 40,
            memory: 3,
            storage_read: 50,
            storage_write: 100,
            call_storage_read: 30,
            call_storage_write: 60,
        }
    }
}

#[derive(Clone, Debug)]
pub struct VmContext {
    pub address: Address,
    pub caller: Address,
    pub origin: Address,
    pub gas_limit: u64,
    pub gas_price: u128,
    pub call_value: u128,
    pub call_data: Vec<u8>,
    pub code: Vec<u8>,
    pub block_number: u64,
    pub timestamp: u64,
    pub difficulty: u128,
    pub coinbase: Address,
    pub block_hash: Hash32,
    pub block_gas_limit: u64,
}

impl VmContext {
    pub fn new(address: Address, caller: Address, origin: Address, gas_limit: u64) -> Self {
        Self {
            address,
            caller,
            origin,
            gas_limit,
            gas_price: 1,
            call_value: 0,
            call_data: Vec::new(),
            code: Vec::new(),
            block_number: 0,
            timestamp: 0,
            difficulty: 0,
            coinbase: Address([0u8; 32]),
            block_hash: Hash32::empty(),
            block_gas_limit: 10000000,
        }
    }
}

pub struct Vm {
    registers: [u128; 16],
    stack: Vec<u128>,
    memory: Vec<u8>,
    storage: HashMap<Vec<u8>, Vec<u8>>,
    pc: usize,
    gas_remaining: u64,
    gas_limit: u64,
    context: VmContext,
    logs: Vec<VmLog>,
    returned: Vec<u8>,
    error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VmLog {
    pub topics: Vec<Hash32>,
    pub data: Vec<u8>,
}

impl Vm {
    pub fn new(context: VmContext) -> Self {
        Self {
            registers: [0u128; 16],
            stack: Vec::new(),
            memory: Vec::new(),
            storage: HashMap::new(),
            pc: 0,
            gas_remaining: context.gas_limit,
            gas_limit: context.gas_limit,
            context,
            logs: Vec::new(),
            returned: Vec::new(),
            error: None,
        }
    }

    pub fn execute(&mut self, gas_schedule: &GasSchedule) -> VmResult {
        while self.pc < self.context.code.len() && self.gas_remaining > 0 {
            if let Err(e) = self.execute_instruction(gas_schedule) {
                self.error = Some(e);
                break;
            }
        }

        VmResult {
            success: self.error.is_none(),
            gas_used: self.gas_limit.saturating_sub(self.gas_remaining),
            returned: self.returned.clone(),
            logs: self.logs.clone(),
            storage_changes: HashMap::new(),
        }
    }

    fn execute_instruction(&mut self, gas_schedule: &GasSchedule) -> Result<(), String> {
        let opcode = self.context.code[self.pc];

        if self.gas_remaining < gas_schedule.step {
            return Err("Out of gas".to_string());
        }
        self.gas_remaining -= gas_schedule.step;
        self.pc += 1;

        match Opcode::from_u8(opcode) {
            Some(Opcode::Stop) => return Err("Stop".to_string()),
            Some(Opcode::Add) => self.binary_op(|a, b| a.wrapping_add(b)),
            Some(Opcode::Sub) => self.binary_op(|a, b| a.wrapping_sub(b)),
            Some(Opcode::Mul) => self.binary_op(|a, b| a.wrapping_mul(b)),
            Some(Opcode::Div) => self.binary_op(|a, b| if b == 0 { 0 } else { a / b }),
            Some(Opcode::Mod) => self.binary_op(|a, b| if b == 0 { 0 } else { a % b }),
            Some(Opcode::And) => self.binary_op(|a, b| a & b),
            Some(Opcode::Or) => self.binary_op(|a, b| a | b),
            Some(Opcode::Xor) => self.binary_op(|a, b| a ^ b),
            Some(Opcode::Not) => self.unary_op(|a| !a),
            Some(Opcode::Shl) => self.binary_op(|a, b| a << (b as u32)),
            Some(Opcode::Shr) => self.binary_op(|a, b| a >> (b as u32)),
            Some(Opcode::Eq) => self.binary_op(|a, b| if a == b { 1 } else { 0 }),
            Some(Opcode::Ne) => self.binary_op(|a, b| if a != b { 1 } else { 0 }),
            Some(Opcode::Lt) => {
                self.binary_op(|a, b| if (a as i128) < (b as i128) { 1 } else { 0 })
            }
            Some(Opcode::Gt) => {
                self.binary_op(|a, b| if (a as i128) > (b as i128) { 1 } else { 0 })
            }
            Some(Opcode::Le) => {
                self.binary_op(|a, b| if (a as i128) <= (b as i128) { 1 } else { 0 })
            }
            Some(Opcode::Ge) => {
                self.binary_op(|a, b| if (a as i128) >= (b as i128) { 1 } else { 0 })
            }
            Some(Opcode::LtU) => self.binary_op(|a, b| if a < b { 1 } else { 0 }),
            Some(Opcode::GtU) => self.binary_op(|a, b| if a > b { 1 } else { 0 }),
            Some(Opcode::LoadImm) => self.load_imm(),
            Some(Opcode::Dup) => self.dup(),
            Some(Opcode::Swap) => self.swap(),
            Some(Opcode::Drop) => self.drop(),
            Some(Opcode::PushStack) => self.push_stack(),
            Some(Opcode::PopStack) => self.pop_stack(),
            Some(Opcode::Jump) => self.jump()?,
            Some(Opcode::JumpI) => self.jumpi()?,
            Some(Opcode::Gas) => self.push(self.gas_remaining as u128),
            Some(Opcode::GasLimit) => self.push(self.gas_limit as u128),
            Some(Opcode::GasPrice) => self.push(self.context.gas_price),
            Some(Opcode::Address) => self.push(u128::from_le_bytes(
                self.context.address.0[..16].try_into().unwrap_or([0u8; 16]),
            )),
            Some(Opcode::Caller) => self.push(u128::from_le_bytes(
                self.context.caller.0[..16].try_into().unwrap_or([0u8; 16]),
            )),
            Some(Opcode::CallValue) => self.push(self.context.call_value),
            Some(Opcode::CallDataSize) => self.push(self.context.call_data.len() as u128),
            Some(Opcode::BlockNumber) => self.push(self.context.block_number as u128),
            Some(Opcode::Timestamp) => self.push(self.context.timestamp as u128),
            Some(Opcode::Difficulty) => self.push(self.context.difficulty),
            Some(Opcode::Coinbase) => self.push(u128::from_le_bytes(
                self.context.coinbase.0[..16]
                    .try_into()
                    .unwrap_or([0u8; 16]),
            )),
            Some(Opcode::BlockGasLimit) => self.push(self.context.block_gas_limit as u128),
            Some(Opcode::Return) => return Err("Return".to_string()),
            Some(Opcode::Revert) => return Err("Revert".to_string()),
            Some(Opcode::SelfDestruct) => return Err("SelfDestruct".to_string()),
            Some(Opcode::Create) => self.create(),
            Some(Opcode::Log0) | Some(Opcode::Log1) | Some(Opcode::Log2) | Some(Opcode::Log3)
            | Some(Opcode::Log4) => self.log(opcode - 0xa0)?,
            Some(Opcode::Invalid) => return Err("Invalid instruction".to_string()),
            Some(Opcode::Call) => self.call()?,
            Some(Opcode::CallCode) => self.callcode()?,
            Some(Opcode::DelegateCall) => self.delegate_call()?,
            Some(Opcode::Store) => self.store()?,
            Some(Opcode::Load) => self.load()?,
            Some(Opcode::CallDataLoad) => self.call_data_load()?,
            Some(Opcode::CallDataCopy) => self.call_data_copy()?,
            Some(Opcode::Mov) => self.mov(),
            _ => return Err(format!("Unknown opcode: {:02x}", opcode)),
        }

        Ok(())
    }

    fn binary_op<F>(&mut self, op: F)
    where
        F: FnOnce(u128, u128) -> u128,
    {
        let b = self.stack.pop().unwrap_or(0);
        let a = self.stack.pop().unwrap_or(0);
        self.stack.push(op(a, b));
    }

    fn unary_op<F>(&mut self, op: F)
    where
        F: FnOnce(u128) -> u128,
    {
        let a = self.stack.pop().unwrap_or(0);
        self.stack.push(op(a));
    }

    fn push(&mut self, value: u128) {
        if self.stack.len() >= MAX_STACK_SIZE {
            self.error = Some("Stack overflow".to_string());
            return;
        }
        self.stack.push(value);
    }

    fn pop(&mut self) -> u128 {
        self.stack.pop().unwrap_or(0)
    }

    fn load_imm(&mut self) {
        if self.pc + 16 > self.context.code.len() {
            return;
        }
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&self.context.code[self.pc..self.pc + 16]);
        self.pc += 16;
        self.stack.push(u128::from_le_bytes(bytes));
    }

    fn dup(&mut self) {
        if let Some(&top) = self.stack.last() {
            self.stack.push(top);
        }
    }

    fn swap(&mut self) {
        if self.stack.len() >= 2 {
            let len = self.stack.len();
            self.stack.swap(len - 1, len - 2);
        }
    }

    fn drop(&mut self) {
        self.stack.pop();
    }

    fn push_stack(&mut self) {
        let value = self.pop();
        self.registers[0] = value;
    }

    fn pop_stack(&mut self) {
        let value = self.registers[0];
        self.stack.push(value);
    }

    fn mov(&mut self) {
        if self.stack.len() >= 2 {
            let src = self.stack.pop().unwrap();
            let dst = self.stack.pop().unwrap() as usize;
            if dst < 16 {
                self.registers[dst] = src;
            }
        }
    }

    fn jump(&mut self) -> Result<(), String> {
        let target = self.pop() as usize;
        if target >= self.context.code.len() {
            return Err("Invalid jump target".to_string());
        }
        self.pc = target;
        Ok(())
    }

    fn jumpi(&mut self) -> Result<(), String> {
        let target = self.pop() as usize;
        let condition = self.pop();
        if condition != 0 && target < self.context.code.len() {
            self.pc = target;
        }
        Ok(())
    }

    fn call(&mut self) -> Result<(), String> {
        let gas = self.pop();
        let addr = self.pop();
        let args_offset = self.pop() as usize;
        let args_size = self.pop() as usize;
        let ret_offset = self.pop() as usize;
        let ret_size = self.pop() as usize;

        if gas as u64 > self.gas_remaining {
            return Err("Out of gas for call".to_string());
        }

        self.gas_remaining -= gas as u64;
        Ok(())
    }

    fn callcode(&self) -> Result<(), String> {
        Ok(())
    }

    fn delegate_call(&self) -> Result<(), String> {
        Ok(())
    }

    fn create(&mut self) {
        let value = self.pop();
        let offset = self.pop() as usize;
        let size = self.pop() as usize;

        if offset + size > self.memory.len() {
            self.error = Some("Invalid creation".to_string());
            return;
        }

        let mut hasher = Sha256::new();
        hasher.update(&self.memory[offset..offset + size]);
        hasher.update(&[0u8; 32]);
        let result = hasher.finalize();

        self.stack.push(u128::from_le_bytes(
            result[..16].try_into().unwrap_or([0u8; 16]),
        ));
    }

    fn store(&mut self) -> Result<(), String> {
        let key = self.pop();
        let value = self.pop();

        let mut key_bytes = vec![0u8; 16];
        key_bytes.copy_from_slice(&key.to_le_bytes());

        let mut value_bytes = vec![0u8; 16];
        value_bytes.copy_from_slice(&value.to_le_bytes());

        self.storage.insert(key_bytes, value_bytes);
        Ok(())
    }

    fn load(&mut self) -> Result<(), String> {
        let key = self.pop();
        let mut key_bytes = vec![0u8; 16];
        key_bytes.copy_from_slice(&key.to_le_bytes());

        let value = self.storage.get(&key_bytes).cloned().unwrap_or_default();
        let value_u128 = u128::from_le_bytes(value.as_slice().try_into().unwrap_or([0u8; 16]));

        self.stack.push(value_u128);
        Ok(())
    }

    fn call_data_load(&mut self) -> Result<(), String> {
        let offset = self.pop() as usize;
        if offset + 16 <= self.context.call_data.len() {
            let mut bytes = [0u8; 16];
            bytes.copy_from_slice(
                &self.context.call_data
                    [offset..offset + 16.min(self.context.call_data.len() - offset)],
            );
            self.stack.push(u128::from_le_bytes(bytes));
        } else {
            self.stack.push(0);
        }
        Ok(())
    }

    fn call_data_copy(&mut self) -> Result<(), String> {
        let dest_offset = self.pop() as usize;
        let offset = self.pop() as usize;
        let size = self.pop() as usize;

        if dest_offset + size > self.memory.len() {
            self.memory.resize(dest_offset + size, 0);
        }

        let end = (offset + size).min(self.context.call_data.len());
        self.memory[dest_offset..dest_offset + size]
            .copy_from_slice(&self.context.call_data[offset..end]);

        Ok(())
    }

    fn log(&mut self, num_topics: u8) -> Result<(), String> {
        let offset = self.pop() as usize;
        let size = self.pop() as usize;

        if offset + size > self.memory.len() {
            return Err("Invalid log".to_string());
        }

        let mut topics = Vec::new();
        for _ in 0..num_topics {
            let topic = self.pop();
            let mut bytes = [0u8; 32];
            let topic_bytes = topic.to_le_bytes();
            bytes[..16].copy_from_slice(&topic_bytes);
            topics.push(Hash32(bytes));
        }

        let data = self.memory[offset..offset + size].to_vec();

        self.logs.push(VmLog { topics, data });

        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VmResult {
    pub success: bool,
    pub gas_used: u64,
    pub returned: Vec<u8>,
    pub logs: Vec<VmLog>,
    pub storage_changes: HashMap<Vec<u8>, Vec<u8>>,
}

pub struct ContractModule {
    pub contracts: HashMap<Address, ContractInstance>,
    pub code_store: HashMap<Hash32, Vec<u8>>,
    pub gas_schedule: GasSchedule,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractInstance {
    pub address: Address,
    pub owner: Address,
    pub code_hash: Hash32,
    pub storage: HashMap<Vec<u8>, Vec<u8>>,
    pub balance: u128,
    pub nonce: u64,
    pub frozen: bool,
}

impl ContractInstance {
    pub fn new(address: Address, owner: Address, code_hash: Hash32) -> Self {
        Self {
            address,
            owner,
            code_hash,
            storage: HashMap::new(),
            balance: 0,
            nonce: 0,
            frozen: false,
        }
    }
}

impl ContractModule {
    pub fn new() -> Self {
        Self {
            contracts: HashMap::new(),
            code_store: HashMap::new(),
            gas_schedule: GasSchedule::default(),
        }
    }

    pub fn deploy(
        &mut self,
        owner: Address,
        code: Vec<u8>,
        value: u128,
    ) -> Result<Address, ContractError> {
        if code.len() > MAX_CONTRACT_CODE_SIZE {
            return Err(ContractError::CodeTooLarge);
        }

        let code_hash = Self::compute_code_hash(&code);

        let mut address = Address([0u8; 32]);
        let mut hasher = Sha256::new();
        hasher.update(owner.as_bytes());
        hasher.update(code_hash.as_bytes());
        let result = hasher.finalize();
        address.0.copy_from_slice(&result[..32]);

        self.code_store.insert(code_hash, code);

        let mut contract = ContractInstance::new(address, owner, code_hash);
        contract.balance = value;

        self.contracts.insert(address, contract);

        Ok(address)
    }

    pub fn call(
        &mut self,
        contract_addr: &Address,
        caller: Address,
        method: &[u8],
        call_data: Vec<u8>,
        gas_limit: u64,
        value: u128,
    ) -> Result<VmResult, ContractError> {
        let contract = self
            .contracts
            .get_mut(contract_addr)
            .ok_or(ContractError::ContractNotFound)?;

        if contract.frozen {
            return Err(ContractError::ContractFrozen);
        }

        let code = self
            .code_store
            .get(&contract.code_hash)
            .ok_or(ContractError::CodeNotFound)?;

        let mut context = VmContext::new(*contract_addr, caller, caller, gas_limit);
        context.call_data = call_data;
        context.call_value = value;
        context.code = code.clone();

        let mut vm = Vm::new(context);
        vm.storage = contract.storage.clone();

        let result = vm.execute(&self.gas_schedule);

        contract.storage = vm.storage.clone();
        contract.nonce += 1;

        Ok(result)
    }

    pub fn get_contract(&self, address: &Address) -> Option<&ContractInstance> {
        self.contracts.get(address)
    }

    pub fn get_balance(&self, address: &Address) -> u128 {
        self.contracts.get(address).map(|c| c.balance).unwrap_or(0)
    }

    pub fn transfer(
        &mut self,
        from: &Address,
        to: &Address,
        amount: u128,
    ) -> Result<(), ContractError> {
        let from_contract = self
            .contracts
            .get_mut(from)
            .ok_or(ContractError::ContractNotFound)?;

        if from_contract.balance < amount {
            return Err(ContractError::InsufficientBalance);
        }

        from_contract.balance = from_contract.balance.saturating_sub(amount);

        let to_contract = self
            .contracts
            .entry(*to)
            .or_insert_with(|| ContractInstance::new(*to, *to, Hash32::empty()));
        to_contract.balance = to_contract.balance.saturating_add(amount);

        Ok(())
    }

    fn compute_code_hash(code: &[u8]) -> Hash32 {
        let mut hasher = Sha256::new();
        hasher.update(code);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        Hash32(hash)
    }
}

impl Default for ContractModule {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum ContractError {
    ContractNotFound,
    ContractFrozen,
    CodeNotFound,
    CodeTooLarge,
    InsufficientBalance,
    OutOfGas,
    InvalidCall,
    StorageError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_deployment() {
        let mut module = ContractModule::new();
        let owner = Address([1u8; 32]);
        let code = vec![0x00];

        let result = module.deploy(owner, code, 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_gas_schedule() {
        let schedule = GasSchedule::default();
        assert_eq!(schedule.step, 1);
        assert_eq!(schedule.storage_read, 50);
    }

    #[test]
    fn test_vm_context() {
        let address = Address([1u8; 32]);
        let caller = Address([2u8; 32]);
        let context = VmContext::new(address, caller, caller, 1000);

        assert_eq!(context.gas_limit, 1000);
    }
}
