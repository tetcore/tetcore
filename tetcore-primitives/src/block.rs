// File: block.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Block building primitives for Tetcore runtime. Provides BlockBuilder
// for constructing blocks with transaction inclusion, gas metering, and
// root computation. Includes strategies for block production, statistics
// tracking, and import context for block execution.

use crate::crypto::Signature;
use crate::hash::Hash32;
use crate::runtime::{Digest, Header, Receipt, Transaction};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockBuilderConfig {
    pub max_gas: u64,
    pub max_transactions: u32,
    pub max_size_bytes: u32,
    pub min_timestamp: Option<u64>,
}

impl Default for BlockBuilderConfig {
    fn default() -> Self {
        Self {
            max_gas: 50_000_000,
            max_transactions: 1000,
            max_size_bytes: 5_242_880,
            min_timestamp: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockBuilder {
    pub parent_hash: Hash32,
    pub number: u64,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub receipts: Vec<Receipt>,
    pub gas_used: u64,
    pub size_bytes: u32,
    pub digest: Digest,
    pub config: BlockBuilderConfig,
}

impl BlockBuilder {
    pub fn new(parent_hash: Hash32, number: u64) -> Self {
        Self {
            parent_hash,
            number,
            timestamp: 0,
            transactions: Vec::new(),
            receipts: Vec::new(),
            gas_used: 0,
            size_bytes: 0,
            digest: Digest::default(),
            config: BlockBuilderConfig::default(),
        }
    }

    pub fn with_config(parent_hash: Hash32, number: u64, config: BlockBuilderConfig) -> Self {
        Self {
            parent_hash,
            number,
            timestamp: 0,
            transactions: Vec::new(),
            receipts: Vec::new(),
            gas_used: 0,
            size_bytes: 0,
            digest: Digest::default(),
            config,
        }
    }

    pub fn timestamp(mut self, timestamp: u64) -> Self {
        if let Some(min_ts) = self.config.min_timestamp {
            self.timestamp = timestamp.max(min_ts);
        } else {
            self.timestamp = timestamp;
        }
        self
    }

    pub fn push_transaction(mut self, transaction: Transaction) -> Result<Self, BlockBuilderError> {
        if self.transactions.len() >= self.config.max_transactions as usize {
            return Err(BlockBuilderError::TooManyTransactions);
        }

        let tx_size = transaction.encode().len() as u32;
        if self.size_bytes + tx_size > self.config.max_size_bytes {
            return Err(BlockBuilderError::BlockTooBig);
        }

        if self.gas_used + transaction.gas_limit > self.config.max_gas {
            return Err(BlockBuilderError::GasExceeded);
        }

        self.size_bytes += tx_size;
        self.transactions.push(transaction);
        Ok(self)
    }

    pub fn push_receipt(mut self, receipt: Receipt) -> Self {
        self.gas_used += receipt.gas_used;
        self.receipts.push(receipt);
        self
    }

    pub fn push_digest_item(mut self, item: crate::runtime::DigestItem) -> Self {
        self.digest.push(item);
        self
    }

    pub fn build(self) -> BuildableBlock {
        let state_root = Hash32::empty();
        let transaction_root = self.compute_transaction_root();
        let receipts_root = self.compute_receipts_root();

        let header = Header {
            parent_hash: self.parent_hash,
            number: self.number,
            state_root,
            transaction_root,
            receipts_root,
            digest: self.digest,
            timestamp: self.timestamp,
            validator_set_id: 0,
        };

        BuildableBlock {
            header,
            transactions: self.transactions,
            receipts: self.receipts,
        }
    }

    fn compute_transaction_root(&self) -> Hash32 {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        for tx in &self.transactions {
            hasher.update(tx.hash().as_bytes());
        }
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        Hash32(hash)
    }

    fn compute_receipts_root(&self) -> Hash32 {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        for receipt in &self.receipts {
            hasher.update(receipt.transaction_hash.as_bytes());
            hasher.update(&receipt.gas_used.to_le_bytes());
        }
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        Hash32(hash)
    }

    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }

    pub fn transaction_count(&self) -> u32 {
        self.transactions.len() as u32
    }

    pub fn size_bytes(&self) -> u32 {
        self.size_bytes
    }

    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildableBlock {
    pub header: Header,
    pub transactions: Vec<Transaction>,
    pub receipts: Vec<Receipt>,
}

impl BuildableBlock {
    pub fn hash(&self) -> Hash32 {
        self.header.hash()
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.header.hash().as_bytes());

        bytes.extend_from_slice(&(self.transactions.len() as u32).to_le_bytes());
        for tx in &self.transactions {
            bytes.extend_from_slice(&tx.encode());
        }

        bytes.extend_from_slice(&(self.receipts.len() as u32).to_le_bytes());
        for receipt in &self.receipts {
            bytes.extend_from_slice(&receipt.gas_used.to_le_bytes());
        }

        bytes
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn transactions(&self) -> &[Transaction] {
        &self.transactions
    }

    pub fn receipts(&self) -> &[Receipt] {
        &self.receipts
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockBuilderError {
    TooManyTransactions,
    BlockTooBig,
    GasExceeded,
    InvalidTransaction,
    SealAlreadySet,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SealedBlock {
    pub header: Header,
    pub transactions: Vec<Transaction>,
    pub receipts: Vec<Receipt>,
    pub signature: Option<Signature>,
}

impl SealedBlock {
    pub fn new(block: BuildableBlock) -> Self {
        Self {
            header: block.header,
            transactions: block.transactions,
            receipts: block.receipts,
            signature: None,
        }
    }

    pub fn seal(mut self, signature: Signature) -> Self {
        self.signature = Some(signature);
        self
    }

    pub fn hash(&self) -> Hash32 {
        self.header.hash()
    }

    pub fn is_sealed(&self) -> bool {
        self.signature.is_some()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockImportContext {
    pub parent_block: Header,
    pub state_root: Hash32,
    pub storage_changes: Vec<(Vec<u8>, Option<Vec<u8>>)>,
    pub events: Vec<crate::runtime::Log>,
}

impl BlockImportContext {
    pub fn new(parent_block: Header) -> Self {
        Self {
            parent_block,
            state_root: Hash32::empty(),
            storage_changes: Vec::new(),
            events: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockExport {
    pub block: SealedBlock,
    pub justifications: Vec<Justification>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Justification {
    pub validator: crate::crypto::Address,
    pub signature: Signature,
    pub round: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BlockStats {
    pub transaction_count: u32,
    pub gas_used: u64,
    pub storage_read_count: u64,
    pub storage_write_count: u64,
    pub event_count: u32,
    pub size_bytes: u32,
}

impl BlockStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_transaction(&mut self) {
        self.transaction_count += 1;
    }

    pub fn record_gas(&mut self, gas: u64) {
        self.gas_used += gas;
    }

    pub fn record_storage_read(&mut self) {
        self.storage_read_count += 1;
    }

    pub fn record_storage_write(&mut self) {
        self.storage_write_count += 1;
    }

    pub fn record_event(&mut self) {
        self.event_count += 1;
    }

    pub fn finalize(&mut self, size_bytes: u32) {
        self.size_bytes = size_bytes;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockMetadata {
    pub hash: Hash32,
    pub number: u64,
    pub parent_hash: Hash32,
    pub state_root: Hash32,
    pub transaction_count: u32,
    pub gas_used: u64,
    pub timestamp: u64,
    pub validator_set_id: u64,
}

impl BlockMetadata {
    pub fn from_block(block: &SealedBlock) -> Self {
        Self {
            hash: block.header.hash(),
            number: block.header.number,
            parent_hash: block.header.parent_hash,
            state_root: block.header.state_root,
            transaction_count: block.transactions.len() as u32,
            gas_used: block.receipts.iter().map(|r| r.gas_used).sum(),
            timestamp: block.header.timestamp,
            validator_set_id: block.header.validator_set_id,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EmptyBlockStrategy;

impl BlockBuildingStrategy for EmptyBlockStrategy {
    fn should_build_block(&self, _current_timestamp: u64, _tx_pool_size: usize) -> bool {
        true
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TimedBlockStrategy {
    pub target_interval_ms: u64,
    pub last_block_timestamp: u64,
}

impl TimedBlockStrategy {
    pub fn new(interval_ms: u64) -> Self {
        Self {
            target_interval_ms: interval_ms,
            last_block_timestamp: 0,
        }
    }
}

impl BlockBuildingStrategy for TimedBlockStrategy {
    fn should_build_block(&self, current_timestamp: u64, _tx_pool_size: usize) -> bool {
        if self.last_block_timestamp == 0 {
            return true;
        }
        current_timestamp.saturating_sub(self.last_block_timestamp) >= self.target_interval_ms
    }
}

pub trait BlockBuildingStrategy: Send + Sync {
    fn should_build_block(&self, current_timestamp: u64, tx_pool_size: usize) -> bool;
}
