// File: runtime.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Tetcore runtime module that integrates all kernel components including
// consensus, economics, governance, IFP, TVM, and SDK. Provides the main
// runtime for block production and transaction execution.

use crate::consensus::{ConsensusEngine, FinalitySignature, ValidatorInfo};
use crate::economics::{FeeModule, StakingModule, Treasury};
use crate::governance::{GovernanceModule, ProposalType, VoteChoice};
use crate::ifp::InferenceModule;
use crate::network::{BlockAnnounce, NetworkMessage, PeerId};
use crate::sdk::{
    Blueprint, GenesisConfig, GenesisState, ModuleId, Runtime as SdkRuntime, TetcoreModule,
};
use crate::tvm::ContractModule;
use crate::{Address, Hash32};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub const SYSTEM_MODULE_ID: [u8; 4] = [0x53, 0x59, 0x53, 0x54];
pub const ACCOUNTS_MODULE_ID: [u8; 4] = [0x41, 0x43, 0x43, 0x54];
pub const CONSENSUS_MODULE_ID: [u8; 4] = [0x43, 0x4f, 0x4e, 0x53];
pub const ECONOMICS_MODULE_ID: [u8; 4] = [0x45, 0x43, 0x4f, 0x4e];
pub const GOVERNANCE_MODULE_ID: [u8; 4] = [0x47, 0x4f, 0x56, 0x45];
pub const IFP_MODULE_ID: [u8; 4] = [0x49, 0x46, 0x50, 0x31];
pub const TVM_MODULE_ID: [u8; 4] = [0x54, 0x56, 0x4d, 0x31];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockHeader {
    pub parent_hash: Hash32,
    pub state_root: Hash32,
    pub receipts_root: Hash32,
    pub block_number: u64,
    pub timestamp: u64,
    pub validator: Address,
    pub consensus_digest: Vec<u8>,
    pub extra: Vec<u8>,
}

impl BlockHeader {
    pub fn new(parent_hash: Hash32, block_number: u64) -> Self {
        Self {
            parent_hash,
            state_root: Hash32::empty(),
            receipts_root: Hash32::empty(),
            block_number,
            timestamp: 0,
            validator: Address([0u8; 32]),
            consensus_digest: Vec::new(),
            extra: Vec::new(),
        }
    }

    pub fn hash(&self) -> Hash32 {
        let mut data = Vec::new();
        data.extend_from_slice(self.parent_hash.as_bytes());
        data.extend_from_slice(self.state_root.as_bytes());
        data.extend_from_slice(&self.block_number.to_le_bytes());
        data.extend_from_slice(&self.timestamp.to_le_bytes());
        data.extend_from_slice(self.validator.as_bytes());

        let digest = Sha256::digest(&data);
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&digest[..32]);
        Hash32(hash)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SealedBlock {
    pub header: BlockHeader,
    pub transactions: Vec<Vec<u8>>,
    pub receipts: Vec<Vec<u8>>,
}

impl SealedBlock {
    pub fn new(header: BlockHeader, transactions: Vec<Vec<u8>>, receipts: Vec<Vec<u8>>) -> Self {
        Self {
            header,
            transactions,
            receipts,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionPool {
    pub pending: Vec<PoolTransaction>,
    pub ready: Vec<PoolTransaction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PoolTransaction {
    pub tx_hash: Hash32,
    pub sender: Address,
    pub nonce: u64,
    pub gas_price: u128,
    pub data: Vec<u8>,
}

impl TransactionPool {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            ready: Vec::new(),
        }
    }

    pub fn add(&mut self, tx: PoolTransaction) {
        self.pending.push(tx);
    }

    pub fn remove(&mut self, tx_hash: &Hash32) -> Option<PoolTransaction> {
        if let Some(pos) = self.pending.iter().position(|t| &t.tx_hash == tx_hash) {
            Some(self.pending.remove(pos))
        } else if let Some(pos) = self.ready.iter().position(|t| &t.tx_hash == tx_hash) {
            Some(self.ready.remove(pos))
        } else {
            None
        }
    }

    pub fn update_nonce(&mut self, sender: &Address, nonce: u64) {
        self.pending
            .retain(|t| !(t.sender == *sender && t.nonce < nonce));
        self.ready
            .retain(|t| !(t.sender == *sender && t.nonce < nonce));
    }
}

impl Default for TransactionPool {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TetcoreRuntime {
    pub block_number: u64,
    pub block_hash: Hash32,
    pub parent_hash: Hash32,
    pub state_root: Hash32,
    pub timestamp: u64,
    pub chain_id: u32,
    pub accounts: HashMap<Address, AccountState>,
    pub consensus: ConsensusEngine,
    pub staking: StakingModule,
    pub treasury: Treasury,
    pub governance: GovernanceModule,
    pub inference: InferenceModule,
    pub contracts: ContractModule,
    pub fee_module: FeeModule,
    pub tx_pool: TransactionPool,
    pub storage: HashMap<Vec<u8>, Vec<u8>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountState {
    pub balance: u128,
    pub nonce: u64,
    pub code_hash: Option<Hash32>,
    pub storage_root: Hash32,
}

impl AccountState {
    pub fn new(balance: u128) -> Self {
        Self {
            balance,
            nonce: 0,
            code_hash: None,
            storage_root: Hash32::empty(),
        }
    }
}

impl TetcoreRuntime {
    pub fn new(chain_id: u32) -> Self {
        Self {
            block_number: 0,
            block_hash: Hash32::empty(),
            parent_hash: Hash32::empty(),
            state_root: Hash32::empty(),
            timestamp: 0,
            chain_id,
            accounts: HashMap::new(),
            consensus: ConsensusEngine::new(),
            staking: StakingModule::new(),
            treasury: Treasury::new(),
            governance: GovernanceModule::new(0),
            inference: InferenceModule::new(),
            contracts: ContractModule::new(),
            fee_module: FeeModule::default(),
            tx_pool: TransactionPool::new(),
            storage: HashMap::new(),
        }
    }

    pub fn create_account(&mut self, address: Address, balance: u128) {
        self.accounts.insert(address, AccountState::new(balance));
    }

    pub fn get_balance(&self, address: &Address) -> u128 {
        self.accounts.get(address).map(|a| a.balance).unwrap_or(0)
    }

    pub fn transfer(
        &mut self,
        from: &Address,
        to: &Address,
        amount: u128,
    ) -> Result<(), RuntimeError> {
        let from_account = self
            .accounts
            .get_mut(from)
            .ok_or(RuntimeError::AccountNotFound)?;

        if from_account.balance < amount {
            return Err(RuntimeError::InsufficientBalance);
        }

        from_account.balance = from_account.balance.saturating_sub(amount);

        let to_account = self
            .accounts
            .entry(*to)
            .or_insert_with(|| AccountState::new(0));
        to_account.balance = to_account.balance.saturating_add(amount);

        Ok(())
    }

    pub fn execute_block(&mut self, txs: Vec<Vec<u8>>) -> Result<SealedBlock, RuntimeError> {
        let mut receipts = Vec::new();
        let mut total_fees = 0u128;

        for tx_data in &txs {
            let result = self.execute_transaction(tx_data);
            match result {
                Ok(receipt) => {
                    total_fees = total_fees.saturating_add(receipt.1);
                    receipts.push(receipt.0);
                }
                Err(_) => {
                    receipts.push(Vec::new());
                }
            }
        }

        let (_, treasury_fee, validator_fee) = self.fee_module.distribute_fee(total_fees);
        self.treasury.deposit(treasury_fee);
        self.staking.distribute_block_rewards(validator_fee);

        let receipts_root = self.compute_receipts_root(&receipts);
        self.state_root = self.compute_state_root();

        let mut header = BlockHeader::new(self.parent_hash, self.block_number);
        header.state_root = self.state_root;
        header.receipts_root = receipts_root;
        header.timestamp = self.timestamp;
        header.validator = self.consensus.get_proposer().unwrap_or(Address([0u8; 32]));

        let block_hash = header.hash();
        self.block_hash = block_hash;

        self.block_number += 1;

        Ok(SealedBlock::new(header, txs, receipts))
    }

    fn execute_transaction(&mut self, tx_data: &[u8]) -> Result<(Vec<u8>, u128), RuntimeError> {
        if tx_data.len() < 32 + 8 + 8 {
            return Err(RuntimeError::InvalidTransaction);
        }

        let mut sender = Address([0u8; 32]);
        sender.0.copy_from_slice(&tx_data[..32]);

        let nonce = u64::from_le_bytes(tx_data[32..40].try_into().unwrap_or([0u8; 8]));
        let gas_limit = u64::from_le_bytes(tx_data[40..48].try_into().unwrap_or([0u8; 8]));

        let account = self
            .accounts
            .get_mut(&sender)
            .ok_or(RuntimeError::AccountNotFound)?;

        if account.nonce != nonce {
            return Err(RuntimeError::InvalidNonce);
        }

        let fee = self.fee_module.compute_fee(gas_limit);
        if account.balance < fee {
            return Err(RuntimeError::InsufficientBalance);
        }

        account.balance = account.balance.saturating_sub(fee);
        account.nonce += 1;

        Ok((Vec::new(), fee))
    }

    fn compute_state_root(&self) -> Hash32 {
        let mut keys: Vec<_> = self.accounts.keys().collect();
        keys.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        if keys.is_empty() {
            return Hash32::empty();
        }

        let hashes: Vec<Hash32> = keys
            .iter()
            .filter_map(|addr| {
                self.accounts.get(addr).map(|acc| {
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

    fn compute_receipts_root(&self, receipts: &[Vec<u8>]) -> Hash32 {
        if receipts.is_empty() {
            return Hash32::empty();
        }

        let hashes: Vec<Hash32> = receipts
            .iter()
            .map(|r| {
                let digest = Sha256::digest(r);
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

    pub fn set_timestamp(&mut self, timestamp: u64) {
        self.timestamp = timestamp;
    }

    pub fn add_validator(&mut self, address: Address, stake: u128) {
        self.consensus
            .add_validator(ValidatorInfo::new(address, stake));
    }

    pub fn initialize_genesis(&mut self, state: &GenesisState) {
        for (address, account) in &state.accounts {
            self.accounts
                .insert(*address, AccountState::new(account.balance));
        }

        for validator in &state.validators {
            self.staking.register_validator(*validator, 1000).ok();
        }

        self.governance
            .update_total_supply(state.accounts.values().map(|a| a.balance).sum());
    }

    pub fn process_inflation(&mut self) {
        let (treasury_amount, _) = self.staking.process_inflation(self.block_number);
        self.treasury.deposit(treasury_amount);
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum RuntimeError {
    AccountNotFound,
    InsufficientBalance,
    InvalidNonce,
    InvalidTransaction,
    ContractNotFound,
    ConsensusError,
    GovernanceError,
    StorageError,
}

pub struct RuntimeBuilder {
    chain_id: u32,
    initial_balances: Vec<(Address, u128)>,
    initial_validators: Vec<(Address, u128)>,
    enable_governance: bool,
    enable_ifp: bool,
    enable_tvm: bool,
}

impl RuntimeBuilder {
    pub fn new(chain_id: u32) -> Self {
        Self {
            chain_id,
            initial_balances: Vec::new(),
            initial_validators: Vec::new(),
            enable_governance: true,
            enable_ifp: true,
            enable_tvm: true,
        }
    }

    pub fn with_account(mut self, address: Address, balance: u128) -> Self {
        self.initial_balances.push((address, balance));
        self
    }

    pub fn with_validator(mut self, address: Address, stake: u128) -> Self {
        self.initial_validators.push((address, stake));
        self
    }

    pub fn with_governance(mut self, enabled: bool) -> Self {
        self.enable_governance = enabled;
        self
    }

    pub fn with_ifp(mut self, enabled: bool) -> Self {
        self.enable_ifp = enabled;
        self
    }

    pub fn with_tvm(mut self, enabled: bool) -> Self {
        self.enable_tvm = enabled;
        self
    }

    pub fn build(self) -> TetcoreRuntime {
        let mut runtime = TetcoreRuntime::new(self.chain_id);

        for (address, balance) in self.initial_balances {
            runtime.create_account(address, balance);
        }

        for (address, stake) in self.initial_validators {
            runtime.add_validator(address, stake);
            runtime.staking.stake(address, stake, 0).ok();
        }

        runtime
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = RuntimeBuilder::new(1)
            .with_account(Address([1u8; 32]), 1000)
            .with_validator(Address([2u8; 32]), 500)
            .build();

        assert_eq!(runtime.chain_id, 1);
        assert_eq!(runtime.get_balance(&Address([1u8; 32])), 1000);
    }

    #[test]
    fn test_account_transfer() {
        let mut runtime = RuntimeBuilder::new(1)
            .with_account(Address([1u8; 32]), 1000)
            .with_account(Address([2u8; 32]), 0)
            .build();

        runtime
            .transfer(&Address([1u8; 32]), &Address([2u8; 32]), 500)
            .unwrap();

        assert_eq!(runtime.get_balance(&Address([1u8; 32])), 500);
        assert_eq!(runtime.get_balance(&Address([2u8; 32])), 500);
    }

    #[test]
    fn test_block_execution() {
        let mut runtime = RuntimeBuilder::new(1)
            .with_account(Address([1u8; 32]), 10000)
            .build();

        let tx_data = vec![1u8; 48];
        let result = runtime.execute_block(vec![tx_data]);

        assert!(result.is_ok());
        assert_eq!(runtime.block_number, 1);
    }
}
