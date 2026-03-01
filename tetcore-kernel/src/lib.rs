// File: lib.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Tetcore kernel providing core protocol logic including block
// construction, transaction validation, state root computation,
// receipt generation, and consensus integration. The kernel is the
// heart of the deterministic state machine.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use thiserror::Error;

pub mod consensus;
pub mod economics;
pub mod governance;
pub mod ifp;
pub mod network;
pub mod runtime;
pub mod sdk;
pub mod tests;
pub mod tvm;

pub const MAX_TRANSACTIONS_PER_BLOCK: usize = 10000;
pub const MAX_GAS_PER_BLOCK: u64 = 100_000_000;
pub const MAX_STORAGE_SIZE: u64 = 1_073_741_824;
pub const MIN_BALANCE: u128 = 1_000_000_000_000_000_000;

#[derive(Error, Debug, Clone)]
pub enum KernelError {
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Insufficient balance: required {0}, available {1}")]
    InsufficientBalance(u128, u128),
    #[error("Invalid nonce: expected {0}, got {1}")]
    InvalidNonce(u64, u64),
    #[error("Account not found: {0}")]
    AccountNotFound(String),
    #[error("Invalid transaction type")]
    InvalidTransactionType,
    #[error("Module dispatch error: {0}")]
    ModuleDispatchError(String),
    #[error("Insufficient gas: required {0}, provided {1}")]
    InsufficientGas(u64, u64),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("State root mismatch")]
    StateRootMismatch,
    #[error("Invalid state transition")]
    InvalidStateTransition,
    #[error("Arithmetic overflow")]
    Overflow,
    #[error("Division by zero")]
    DivisionByZero,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer,
    DeployContract,
    CallContract,
    InvokeModule,
    SubmitPrompt,
    SubmitReceipt,
    Governance,
    Batch,
}

impl TransactionType {
    pub fn to_u8(self) -> u8 {
        match self {
            TransactionType::Transfer => 0,
            TransactionType::DeployContract => 1,
            TransactionType::CallContract => 2,
            TransactionType::InvokeModule => 3,
            TransactionType::SubmitPrompt => 4,
            TransactionType::SubmitReceipt => 5,
            TransactionType::Governance => 6,
            TransactionType::Batch => 7,
        }
    }

    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(TransactionType::Transfer),
            1 => Some(TransactionType::DeployContract),
            2 => Some(TransactionType::CallContract),
            3 => Some(TransactionType::InvokeModule),
            4 => Some(TransactionType::SubmitPrompt),
            5 => Some(TransactionType::SubmitReceipt),
            6 => Some(TransactionType::Governance),
            7 => Some(TransactionType::Batch),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub transaction: Transaction,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub tx_type: TransactionType,
    pub sender: [u8; 32],
    pub nonce: u64,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub value: u128,
    pub payload: Vec<u8>,
    pub chain_id: u32,
}

impl Transaction {
    pub fn new(sender: [u8; 32]) -> Self {
        Self {
            tx_type: TransactionType::Transfer,
            sender,
            nonce: 0,
            gas_limit: 21000,
            gas_price: 1,
            value: 0,
            payload: Vec::new(),
            chain_id: 0,
        }
    }

    pub fn sender_address(&self) -> Address {
        Address(self.sender)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.tx_type.to_u8());
        bytes.extend_from_slice(&self.sender);
        bytes.extend_from_slice(&self.nonce.to_le_bytes());
        bytes.extend_from_slice(&self.gas_limit.to_le_bytes());
        bytes.extend_from_slice(&self.gas_price.to_le_bytes());
        bytes.extend_from_slice(&self.value.to_le_bytes());
        bytes.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.payload);
        bytes.extend_from_slice(&self.chain_id.to_le_bytes());
        bytes
    }

    pub fn hash(&self) -> Hash32 {
        let encoded = self.encode();
        let digest = Sha256::digest(&encoded);
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&digest[..32]);
        Hash32(hash)
    }

    pub fn fee(&self) -> u128 {
        (self.gas_limit as u128) * (self.gas_price as u128)
    }
}

#[derive(Clone, Debug, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address(pub [u8; 32]);

impl Address {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn as_mut_bytes(&mut self) -> &mut [u8; 32] {
        &mut self.0
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }

    pub fn from_public_key(pk: &[u8; 32]) -> Self {
        let digest = Sha256::digest(pk);
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&digest[..32]);
        Self(hash)
    }
}

impl From<[u8; 32]> for Address {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

#[derive(Clone, Debug, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash32(pub [u8; 32]);

impl Hash32 {
    pub fn empty() -> Self {
        Self([0u8; 32])
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        let mut hash = [0u8; 32];
        let len = slice.len().min(32);
        hash[..len].copy_from_slice(&slice[..len]);
        Self(hash)
    }
}

impl From<[u8; 32]> for Hash32 {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AccountData {
    pub balance: u128,
    pub nonce: u64,
    pub storage: HashMap<Vec<u8>, Vec<u8>>,
    pub code_hash: Option<Hash32>,
}

impl AccountData {
    pub fn new(balance: u128) -> Self {
        Self {
            balance,
            nonce: 0,
            storage: HashMap::new(),
            code_hash: None,
        }
    }

    pub fn can_transfer(&self, amount: u128, fee: u128) -> bool {
        self.balance >= amount.saturating_add(fee)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StorageValue {
    pub value: Vec<u8>,
    pub hash: Hash32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Storage {
    pub values: HashMap<Vec<u8>, StorageValue>,
    pub size: u64,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            size: 0,
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<&StorageValue> {
        self.values.get(key)
    }

    pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>) -> Option<Vec<u8>> {
        let old_value = self.values.remove(&key).map(|sv| sv.value);
        let hash = Self::compute_value_hash(&value);
        let size_delta = if let Some(ref old) = old_value {
            (value.len() as i64) - (old.len() as i64)
        } else {
            value.len() as i64
        };
        self.size = (self.size as i64 + size_delta) as u64;
        self.values.insert(key, StorageValue { value, hash });
        old_value
    }

    pub fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        if let Some(sv) = self.values.remove(key) {
            self.size = self.size.saturating_sub(sv.value.len() as u64);
            Some(sv.value)
        } else {
            None
        }
    }

    pub fn root(&self) -> Hash32 {
        self.merkle_root()
    }

    pub fn merkle_root(&self) -> Hash32 {
        let mut keys: Vec<_> = self.values.keys().collect();
        keys.sort();

        if keys.is_empty() {
            return Hash32::empty();
        }

        let mut hashes: Vec<Hash32> = keys
            .iter()
            .map(|k| {
                let sv = self.values.get(*k).unwrap();
                let mut data = Vec::new();
                data.extend_from_slice(k);
                data.extend_from_slice(&sv.value);
                let digest = Sha256::digest(&data);
                let mut h = [0u8; 32];
                h.copy_from_slice(&digest[..32]);
                Hash32(h)
            })
            .collect();

        while hashes.len() > 1 {
            if hashes.len() % 2 == 1 {
                hashes.push(Hash32::empty());
            }
            let mut next_level = Vec::new();
            for pair in hashes.chunks(2) {
                let mut data = Vec::new();
                data.extend_from_slice(pair[0].as_bytes());
                data.extend_from_slice(pair[1].as_bytes());
                let digest = Sha256::digest(&data);
                let mut h = [0u8; 32];
                h.copy_from_slice(&digest[..32]);
                next_level.push(Hash32(h));
            }
            hashes = next_level;
        }

        hashes[0]
    }

    fn compute_value_hash(value: &[u8]) -> Hash32 {
        let digest = Sha256::digest(value);
        let mut h = [0u8; 32];
        h.copy_from_slice(&digest[..32]);
        Hash32(h)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub accounts: HashMap<Address, AccountData>,
    pub storage: Storage,
}

impl State {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            storage: Storage::new(),
        }
    }

    pub fn create_account(&mut self, address: Address, balance: u128) {
        self.accounts.insert(address, AccountData::new(balance));
    }

    pub fn account_exists(&self, address: &Address) -> bool {
        self.accounts.contains_key(address)
    }

    pub fn get_account(&self, address: &Address) -> Option<&AccountData> {
        self.accounts.get(address)
    }

    pub fn get_account_mut(&mut self, address: &Address) -> Option<&mut AccountData> {
        self.accounts.get_mut(address)
    }

    pub fn transfer(
        &mut self,
        from: &Address,
        to: &Address,
        amount: u128,
    ) -> Result<(), KernelError> {
        let from_account = self
            .accounts
            .get_mut(from)
            .ok_or_else(|| KernelError::AccountNotFound(format!("{:?}", from)))?;

        if from_account.balance < amount {
            return Err(KernelError::InsufficientBalance(
                amount,
                from_account.balance,
            ));
        }

        from_account.balance = from_account.balance.saturating_sub(amount);

        let to_account = self
            .accounts
            .entry(*to)
            .or_insert_with(|| AccountData::new(0));
        to_account.balance = to_account.balance.saturating_add(amount);

        Ok(())
    }

    pub fn root(&self) -> Hash32 {
        self.merkle_root()
    }

    pub fn merkle_root(&self) -> Hash32 {
        let mut account_keys: Vec<_> = self.accounts.keys().collect();
        account_keys.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        if account_keys.is_empty() && self.storage.values.is_empty() {
            return Hash32::empty();
        }

        let mut all_hashes = Vec::new();

        for addr in account_keys {
            if let Some(acc) = self.accounts.get(addr) {
                let mut data = Vec::new();
                data.extend_from_slice(addr.as_bytes());
                data.extend_from_slice(&acc.balance.to_le_bytes());
                data.extend_from_slice(&acc.nonce.to_le_bytes());
                let digest = Sha256::digest(&data);
                let mut h = [0u8; 32];
                h.copy_from_slice(&digest[..32]);
                all_hashes.push(Hash32(h));
            }
        }

        let storage_root = self.storage.merkle_root();
        all_hashes.push(storage_root);

        while all_hashes.len() > 1 {
            if all_hashes.len() % 2 == 1 {
                all_hashes.push(Hash32::empty());
            }
            let mut next_level = Vec::new();
            for pair in all_hashes.chunks(2) {
                let mut data = Vec::new();
                data.extend_from_slice(pair[0].as_bytes());
                data.extend_from_slice(pair[1].as_bytes());
                let digest = Sha256::digest(&data);
                let mut h = [0u8; 32];
                h.copy_from_slice(&digest[..32]);
                next_level.push(Hash32(h));
            }
            all_hashes = next_level;
        }

        all_hashes[0]
    }
}

pub struct Kernel {
    state: State,
    gas_schedule: GasSchedule,
    chain_id: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GasSchedule {
    pub tx_base_gas: u64,
    pub transfer_gas: u64,
    pub contract_deploy_gas: u64,
    pub contract_call_gas: u64,
    pub storage_write_gas: u64,
    pub storage_read_gas: u64,
    pub sload_gas: u64,
    pub per_byte_gas: u64,
}

impl Default for GasSchedule {
    fn default() -> Self {
        Self {
            tx_base_gas: 21000,
            transfer_gas: 21000,
            contract_deploy_gas: 100000,
            contract_call_gas: 50000,
            storage_write_gas: 50000,
            storage_read_gas: 5000,
            sload_gas: 5000,
            per_byte_gas: 1,
        }
    }
}

impl Kernel {
    pub fn new(chain_id: u32) -> Self {
        Self {
            state: State::new(),
            gas_schedule: GasSchedule::default(),
            chain_id,
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    pub fn chain_id(&self) -> u32 {
        self.chain_id
    }

    pub fn create_account(&mut self, address: Address, balance: u128) {
        self.state.create_account(address, balance);
    }

    pub fn account_exists(&self, address: &Address) -> bool {
        self.state.account_exists(address)
    }

    pub fn get_balance(&self, address: &Address) -> u128 {
        self.state
            .get_account(address)
            .map(|a| a.balance)
            .unwrap_or(0)
    }

    pub fn get_nonce(&self, address: &Address) -> u64 {
        self.state
            .get_account(address)
            .map(|a| a.nonce)
            .unwrap_or(0)
    }

    pub fn validate_transaction(&self, tx: &Transaction) -> Result<Validation, KernelError> {
        if tx.chain_id != self.chain_id {
            return Err(KernelError::InvalidTransactionType);
        }

        let account = self
            .state
            .get_account(&tx.sender_address())
            .ok_or_else(|| KernelError::AccountNotFound(format!("{:?}", tx.sender_address())))?;

        if account.nonce != tx.nonce {
            return Err(KernelError::InvalidNonce(account.nonce, tx.nonce));
        }

        let fee = tx.fee();
        if account.balance < tx.value.saturating_add(fee) {
            return Err(KernelError::InsufficientBalance(
                tx.value.saturating_add(fee),
                account.balance,
            ));
        }

        if tx.gas_limit > MAX_GAS_PER_BLOCK {
            return Err(KernelError::InsufficientGas(0, tx.gas_limit));
        }

        Ok(Validation {
            sender_nonce: account.nonce,
            sender_balance: account.balance,
            fee,
        })
    }

    pub fn execute_transaction(
        &mut self,
        tx: &Transaction,
    ) -> Result<ExecutionReceipt, KernelError> {
        let validation = self.validate_transaction(tx)?;

        let sender = tx.sender_address();
        let account = self.state.get_account_mut(&sender).unwrap();

        let fee = tx.fee();
        account.balance = account.balance.saturating_sub(fee);
        if tx.value > 0 {
            account.balance = account.balance.saturating_sub(tx.value);
        }
        account.nonce = account.nonce.saturating_add(1);

        let mut gas_used = self.gas_schedule.tx_base_gas;

        match tx.tx_type {
            TransactionType::Transfer => {
                gas_used += self.gas_schedule.transfer_gas;
                if tx.value > 0 && !tx.payload.is_empty() {
                    let mut bytes = [0u8; 32];
                    let len = 32.min(tx.payload.len());
                    bytes[..len].copy_from_slice(&tx.payload[..len]);
                    let recipient = Address(bytes);
                    if !recipient.is_zero() {
                        let to_account = self
                            .state
                            .accounts
                            .entry(recipient)
                            .or_insert_with(|| AccountData::new(0));
                        to_account.balance = to_account.balance.saturating_add(tx.value);
                    }
                }
            }
            TransactionType::DeployContract => {
                gas_used += self.gas_schedule.contract_deploy_gas;
                gas_used += tx.payload.len() as u64 * self.gas_schedule.per_byte_gas;
            }
            TransactionType::CallContract => {
                gas_used += self.gas_schedule.contract_call_gas;
                gas_used += tx.payload.len() as u64 * self.gas_schedule.per_byte_gas;
            }
            _ => {}
        }

        let state_root = self.state.root();

        Ok(ExecutionReceipt {
            tx_hash: tx.hash(),
            gas_used,
            state_root,
            success: true,
            output: Vec::new(),
            events: Vec::new(),
        })
    }

    pub fn apply_block(&mut self, txs: &[Transaction]) -> Result<BlockReceipt, KernelError> {
        let mut receipts = Vec::new();
        let mut total_gas = 0u64;

        for tx in txs {
            if total_gas + tx.gas_limit > MAX_GAS_PER_BLOCK {
                break;
            }

            match self.execute_transaction(tx) {
                Ok(receipt) => {
                    total_gas = total_gas.saturating_add(receipt.gas_used);
                    receipts.push(receipt);
                }
                Err(e) => {
                    let receipt = ExecutionReceipt {
                        tx_hash: tx.hash(),
                        gas_used: tx.gas_limit,
                        state_root: self.state.root(),
                        success: false,
                        output: Vec::new(),
                        events: Vec::new(),
                    };
                    receipts.push(receipt);
                }
            }
        }

        let state_root = self.state.root();
        let receipts_root = compute_receipts_root(&receipts);

        Ok(BlockReceipt {
            state_root,
            receipts_root,
            gas_used: total_gas,
            transaction_count: receipts.len() as u32,
        })
    }
}

impl Default for Kernel {
    fn default() -> Self {
        Self::new(0)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Validation {
    pub sender_nonce: u64,
    pub sender_balance: u128,
    pub fee: u128,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionReceipt {
    pub tx_hash: Hash32,
    pub gas_used: u64,
    pub state_root: Hash32,
    pub success: bool,
    pub output: Vec<u8>,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub phase: u8,
    pub address: Address,
    pub topics: Vec<Hash32>,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockReceipt {
    pub state_root: Hash32,
    pub receipts_root: Hash32,
    pub gas_used: u64,
    pub transaction_count: u32,
}

pub fn compute_state_root(accounts: &HashMap<Address, AccountData>) -> Hash32 {
    let mut keys: Vec<_> = accounts.keys().collect();
    keys.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

    if keys.is_empty() {
        return Hash32::empty();
    }

    let hashes: Vec<Hash32> = keys
        .iter()
        .filter_map(|addr| {
            accounts.get(addr).map(|acc| {
                let mut data = Vec::new();
                data.extend_from_slice(addr.as_bytes());
                data.extend_from_slice(&acc.balance.to_le_bytes());
                data.extend_from_slice(&acc.nonce.to_le_bytes());
                let digest = Sha256::digest(&data);
                let mut h = [0u8; 32];
                h.copy_from_slice(&digest[..32]);
                Hash32(h)
            })
        })
        .collect();

    if hashes.is_empty() {
        return Hash32::empty();
    }

    let mut current = hashes;
    while current.len() > 1 {
        if current.len() % 2 == 1 {
            current.push(Hash32::empty());
        }
        let mut next = Vec::new();
        for pair in current.chunks(2) {
            let mut data = Vec::new();
            data.extend_from_slice(pair[0].as_bytes());
            data.extend_from_slice(pair[1].as_bytes());
            let digest = Sha256::digest(&data);
            let mut h = [0u8; 32];
            h.copy_from_slice(&digest[..32]);
            next.push(Hash32(h));
        }
        current = next;
    }

    current[0]
}

fn compute_receipts_root(receipts: &[ExecutionReceipt]) -> Hash32 {
    if receipts.is_empty() {
        return Hash32::empty();
    }

    let hashes: Vec<Hash32> = receipts
        .iter()
        .map(|r| {
            let mut data = Vec::new();
            data.extend_from_slice(r.tx_hash.as_bytes());
            data.extend_from_slice(&r.gas_used.to_le_bytes());
            data.push(if r.success { 1 } else { 0 });
            let digest = Sha256::digest(&data);
            let mut h = [0u8; 32];
            h.copy_from_slice(&digest[..32]);
            Hash32(h)
        })
        .collect();

    let mut current = hashes;
    while current.len() > 1 {
        if current.len() % 2 == 1 {
            current.push(Hash32::empty());
        }
        let mut next = Vec::new();
        for pair in current.chunks(2) {
            let mut data = Vec::new();
            data.extend_from_slice(pair[0].as_bytes());
            data.extend_from_slice(pair[1].as_bytes());
            let digest = Sha256::digest(&data);
            let mut h = [0u8; 32];
            h.copy_from_slice(&digest[..32]);
            next.push(Hash32(h));
        }
        current = next;
    }

    current[0]
}
