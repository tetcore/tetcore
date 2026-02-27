// File: blockchain.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Blockchain primitives for Tetcore including chain configuration, fork
// resolution, chain synchronization, block import/export, and blockchain
// traits. Provides core infrastructure for chain management and
// fork-aware state machine operations.

use crate::crypto::Address;
use crate::hash::Hash32;
use crate::runtime::Header;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_id: u32,
    pub chain_name: String,
    pub chain_type: ChainType,
    pub genesis_hash: Hash32,
    pub fork_config: ForkConfig,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ChainType {
    #[default]
    Development,
    Local,
    Testnet,
    Mainnet,
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self {
            chain_id: 0,
            chain_name: "tetcore-dev".to_string(),
            chain_type: ChainType::Development,
            genesis_hash: Hash32::empty(),
            fork_config: ForkConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ForkConfig {
    pub fork_after_block: Option<u64>,
    pub fork_migration: Option<ForkMigration>,
    pub known_forks: Vec<Fork>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fork {
    pub fork_id: Hash32,
    pub block_number: u64,
    pub parent_hash: Hash32,
    pub state_root: Hash32,
}

impl Fork {
    pub fn new(block_number: u64, parent_hash: Hash32, state_root: Hash32) -> Self {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&block_number.to_le_bytes());
        hasher.update(parent_hash.as_bytes());
        let result = hasher.finalize();
        let mut fork_id = [0u8; 32];
        fork_id.copy_from_slice(&result);

        Self {
            fork_id: Hash32(fork_id),
            block_number,
            parent_hash,
            state_root,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ForkMigration {
    pub from_version: u32,
    pub to_version: u32,
    pub migration_code: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainBlock {
    pub header: Header,
    pub body: Vec<Vec<u8>>,
    pub receipts: Vec<Vec<u8>>,
}

impl ChainBlock {
    pub fn hash(&self) -> Hash32 {
        self.header.hash()
    }

    pub fn number(&self) -> u64 {
        self.header.number
    }

    pub fn parent_hash(&self) -> Hash32 {
        self.header.parent_hash
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainStorage {
    pub blocks: std::collections::HashMap<Hash32, ChainBlock>,
    pub block_by_number: std::collections::HashMap<u64, Hash32>,
    pub best_hash: Hash32,
    pub best_number: u64,
    pub genesis_hash: Hash32,
}

impl ChainStorage {
    pub fn new() -> Self {
        Self {
            blocks: std::collections::HashMap::new(),
            block_by_number: std::collections::HashMap::new(),
            best_hash: Hash32::empty(),
            best_number: 0,
            genesis_hash: Hash32::empty(),
        }
    }

    pub fn insert(&mut self, block: ChainBlock) {
        let hash = block.hash();
        let number = block.number();
        self.blocks.insert(hash, block);
        self.block_by_number.insert(number, hash);
    }

    pub fn get_block(&self, hash: &Hash32) -> Option<&ChainBlock> {
        self.blocks.get(hash)
    }

    pub fn get_block_by_number(&self, number: u64) -> Option<&ChainBlock> {
        self.block_by_number
            .get(&number)
            .and_then(|h| self.blocks.get(h))
    }

    pub fn set_best(&mut self, hash: Hash32, number: u64) {
        self.best_hash = hash;
        self.best_number = number;
    }
}

impl Default for ChainStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ForkChoice {
    pub latest_finalized_hash: Hash32,
    pub latest_finalized_number: u64,
    pub best_chain: Vec<Hash32>,
}

impl ForkChoice {
    pub fn new() -> Self {
        Self {
            latest_finalized_hash: Hash32::empty(),
            latest_finalized_number: 0,
            best_chain: Vec::new(),
        }
    }

    pub fn update_finalized(&mut self, hash: Hash32, number: u64) {
        self.latest_finalized_hash = hash;
        self.latest_finalized_number = number;
    }
}

impl Default for ForkChoice {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImportResult {
    pub hash: Hash32,
    pub number: u64,
    pub is_new_best: bool,
    pub is_finalized: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImportError {
    pub kind: ImportErrorKind,
    pub message: String,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportErrorKind {
    UnknownBlock,
    MissingParent,
    BlockInFuture,
    InvalidStateRoot,
    InvalidTransactionsRoot,
    InvalidReceiptsRoot,
    InvalidSignature,
    BadNonce,
    InsufficientBalance,
    GasLimitExceeded,
    FullBlock,
    ForkChoiceChanged,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockchainImportContext {
    pub header: Header,
    pub just_header: bool,
    pub allow_missing_state: bool,
    pub justification: Option<BlockJustification>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockJustification {
    pub round: u64,
    pub votes: Vec<(Address, Vec<u8>)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FinalityProof {
    pub block_hash: Hash32,
    pub block_number: u64,
    pub justifications: Vec<Vec<u8>>,
    pub ancestor_hash: Hash32,
    pub ancestor_number: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncState {
    pub starting_block: u64,
    pub current_block: u64,
    pub highest_block: Option<u64>,
    pub sync_type: SyncType,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncType {
    Full,
    Light,
    Warp,
}

impl Default for SyncState {
    fn default() -> Self {
        Self {
            starting_block: 0,
            current_block: 0,
            highest_block: None,
            sync_type: SyncType::Full,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockStatus {
    pub hash: Option<Hash32>,
    pub status: BlockAvailability,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockAvailability {
    Unknown,
    Pending,
    InChain,
    Queued,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockRequest {
    pub id: Hash32,
    pub fields: BlockFields,
    pub from: BlockRequestStart,
    pub to: Option<Hash32>,
    pub direction: BlockRequestDirection,
    pub max: u32,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockRequestStart {
    Hash(Hash32),
    Number(u64),
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockRequestDirection {
    Ascending,
    Descending,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockFields {
    pub header: bool,
    pub body: bool,
    pub receipt: bool,
    pub message_queue: bool,
    pub just_ification: bool,
}

impl Default for BlockFields {
    fn default() -> Self {
        Self {
            header: true,
            body: true,
            receipt: true,
            message_queue: false,
            just_ification: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockResponse {
    pub id: Hash32,
    pub blocks: Vec<BlockResponseItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockResponseItem {
    pub hash: Hash32,
    pub header: Option<Header>,
    pub body: Option<Vec<Vec<u8>>>,
    pub receipt: Option<Vec<Vec<u8>>>,
    pub message_queue: Option<Vec<u8>>,
    pub just_ification: Option<Vec<u8>>,
}

pub trait Blockchain {
    fn chain_config(&self) -> &ChainConfig;
    fn storage(&self) -> &ChainStorage;
    fn fork_choice(&self) -> &ForkChoice;

    fn best_block(&self) -> Option<(Hash32, u64)>;
    fn finalized_block(&self) -> Option<(Hash32, u64)>;

    fn get_block(&self, hash: &Hash32) -> Option<&ChainBlock>;
    fn get_block_by_number(&self, number: u64) -> Option<&ChainBlock>;

    fn has_block(&self, hash: &Hash32) -> bool;
    fn is_descendant_of(&self, descendant: &Hash32, ancestor: &Hash32) -> bool;

    fn import_block(&mut self, block: ChainBlock) -> Result<ImportResult, ImportError>;
    fn finalize_block(&mut self, hash: Hash32) -> Result<(), ImportError>;
}

pub trait HeaderMetadata: Send + Sync {
    fn header(&self, hash: &Hash32) -> Option<Header>;
    fn number(&self, hash: &Hash32) -> Option<u64>;
    fn hash(&self, number: u64) -> Option<Hash32>;
    fn parent_hash(&self, hash: &Hash32) -> Option<Hash32>;
}

pub trait ChainSegment: Send + Sync {
    fn segment(&self, start: u64, end: u64) -> Option<Vec<ChainBlock>>;
    fn contains(&self, hash: &Hash32) -> bool;
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChainProperties {
    pub ss58_format: u8,
    pub token_symbol: String,
    pub token_decimals: u8,
    pub chain_type: ChainType,
}

impl ChainProperties {
    pub fn new() -> Self {
        Self {
            ss58_format: 42,
            token_symbol: "TNT".to_string(),
            token_decimals: 18,
            chain_type: ChainType::Development,
        }
    }
}
