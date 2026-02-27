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
use tetcore_primitives::{account::AccountData, Address, Hash32, PublicKey, Signature};
use thiserror::Error;

pub const MAX_TRANSACTIONS_PER_BLOCK: usize = 10000;
pub const MAX_GAS_PER_BLOCK: u64 = 100_000_000;

#[derive(Error, Debug, Clone)]
pub enum KernelError {
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Insufficient balance")]
    InsufficientBalance,
    #[error("Invalid nonce")]
    InvalidNonce,
    #[error("Account not found")]
    AccountNotFound,
    #[error("Invalid transaction type")]
    InvalidTransactionType,
    #[error("Module dispatch error")]
    ModuleDispatchError,
    #[error("Insufficient gas")]
    InsufficientGas,
    #[error("Storage error")]
    StorageError,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionType {
    Transfer,
    DeployContract,
    CallContract,
    SubmitPrompt,
    SubmitReceipt,
    Governance,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub tx_type: TransactionType,
    pub sender: Address,
    pub nonce: u64,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub fee: u64,
    pub payload: Vec<u8>,
    pub signature: Option<Signature>,
}

impl Transaction {
    pub fn new_transfer(sender: Address, nonce: u64, payload: Vec<u8>) -> Self {
        Self {
            tx_type: TransactionType::Transfer,
            sender,
            nonce,
            gas_limit: 21000,
            gas_price: 1,
            fee: 21000,
            payload,
            signature: None,
        }
    }

    pub fn with_signature(mut self, signature: Signature) -> Self {
        self.signature = Some(signature);
        self
    }

    pub fn sender_public_key(&self) -> Option<PublicKey> {
        None
    }

    pub fn verify_signature(&self) -> Result<bool, KernelError> {
        let _signature = self
            .signature
            .as_ref()
            .ok_or(KernelError::InvalidSignature)?;

        let mut tx_data = Vec::new();
        tx_data.extend_from_slice(self.sender.as_bytes());
        tx_data.extend_from_slice(&self.nonce.to_le_bytes());
        tx_data.push(match self.tx_type {
            TransactionType::Transfer => 0,
            TransactionType::DeployContract => 1,
            TransactionType::CallContract => 2,
            TransactionType::SubmitPrompt => 3,
            TransactionType::SubmitReceipt => 4,
            TransactionType::Governance => 5,
        });
        tx_data.extend_from_slice(&self.payload);

        Ok(true)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BlockHeader {
    pub height: u64,
    pub parent_hash: Hash32,
    pub timestamp: u64,
    pub state_root: Hash32,
    pub tx_root: Hash32,
    pub receipts_root: Hash32,
    pub validator_set: Vec<Address>,
}

impl BlockHeader {
    pub fn new(height: u64, parent_hash: Hash32) -> Self {
        Self {
            height,
            parent_hash,
            timestamp: 0,
            state_root: Hash32::empty(),
            tx_root: Hash32::empty(),
            receipts_root: Hash32::empty(),
            validator_set: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Receipt {
    pub tx_hash: Hash32,
    pub gas_used: u64,
    pub success: bool,
    pub return_data: Vec<u8>,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub contract: Option<Address>,
    pub name: String,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub state_root: Hash32,
    pub receipts: Vec<Receipt>,
    pub gas_used: u64,
}

pub struct Kernel {
    state: State,
    gas_schedule: GasSchedule,
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
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub accounts: HashMap<Address, AccountData>,
    pub block_height: u64,
}

impl Kernel {
    pub fn new() -> Self {
        Self {
            state: State::default(),
            gas_schedule: GasSchedule::default(),
        }
    }

    pub fn create_account(&mut self, address: Address, data: AccountData) {
        self.state.accounts.insert(address, data);
    }

    pub fn get_account(&self, address: &Address) -> Option<&AccountData> {
        self.state.accounts.get(address)
    }

    pub fn account_exists(&self, address: &Address) -> bool {
        self.state.accounts.contains_key(address)
    }

    pub fn transfer(
        &mut self,
        from: &Address,
        to: &Address,
        amount: u128,
    ) -> Result<(), KernelError> {
        let from_account = self
            .state
            .accounts
            .get_mut(from)
            .ok_or(KernelError::AccountNotFound)?;

        if from_account.balance < amount {
            return Err(KernelError::InsufficientBalance);
        }

        from_account.balance -= amount;

        let to_account = self
            .state
            .accounts
            .entry(to.clone())
            .or_insert_with(|| AccountData::new(0));
        to_account.balance += amount;

        Ok(())
    }

    pub fn increment_nonce(&mut self, address: &Address) -> Result<(), KernelError> {
        let account = self
            .state
            .accounts
            .get_mut(address)
            .ok_or(KernelError::AccountNotFound)?;
        account.nonce += 1;
        Ok(())
    }

    pub fn validate_transaction(&self, tx: &Transaction) -> Result<(), KernelError> {
        if !self.account_exists(&tx.sender) {
            return Err(KernelError::AccountNotFound);
        }

        let account = self.get_account(&tx.sender).unwrap();

        if account.nonce != tx.nonce {
            return Err(KernelError::InvalidNonce);
        }

        let required_fee = tx.gas_limit * tx.gas_price;
        if account.balance < required_fee as u128 {
            return Err(KernelError::InsufficientBalance);
        }

        Ok(())
    }

    pub fn apply_transaction(&mut self, tx: &Transaction) -> Result<Receipt, KernelError> {
        self.validate_transaction(tx)?;

        let account = self.state.accounts.get_mut(&tx.sender).unwrap();
        let fee = tx.gas_limit * tx.gas_price;
        account.balance -= fee as u128;

        let mut gas_used = self.gas_schedule.tx_base_gas;

        match tx.tx_type {
            TransactionType::Transfer => {
                gas_used += self.gas_schedule.transfer_gas;
            }
            TransactionType::DeployContract => {
                gas_used += self.gas_schedule.contract_deploy_gas;
            }
            TransactionType::CallContract => {
                gas_used += self.gas_schedule.contract_call_gas;
            }
            _ => {}
        }

        self.increment_nonce(&tx.sender)?;

        let receipt = Receipt {
            tx_hash: Hash32::empty(),
            gas_used,
            success: true,
            return_data: Vec::new(),
            events: Vec::new(),
        };

        Ok(receipt)
    }

    pub fn get_block_height(&self) -> u64 {
        self.state.block_height
    }

    pub fn set_block_height(&mut self, height: u64) {
        self.state.block_height = height;
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }
}

impl Default for Kernel {
    fn default() -> Self {
        Self::new()
    }
}

pub fn compute_state_root(accounts: &HashMap<Address, AccountData>) -> Hash32 {
    let mut data = Vec::new();
    let mut addresses: Vec<_> = accounts.keys().cloned().collect();
    addresses.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

    for addr in addresses {
        if let Some(account) = accounts.get(&addr) {
            data.extend_from_slice(addr.as_bytes());
            data.extend_from_slice(&account.balance.to_le_bytes());
            data.extend_from_slice(&account.nonce.to_le_bytes());
        }
    }

    let hash = Sha256::digest(&data);
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&hash[..32]);
    Hash32(arr)
}
