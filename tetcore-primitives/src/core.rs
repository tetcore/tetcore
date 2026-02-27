// File: core.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Core primitives for Tetcore including system types, error definitions,
// version information, chain identifiers, system events, and fundamental
// constants used throughout the runtime.

use crate::crypto::Address;
use crate::hash::Hash32;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ChainId(pub u32);

impl ChainId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }

    pub fn is_mainnet(self) -> bool {
        self.0 == 1
    }

    pub fn is_testnet(self) -> bool {
        self.0 > 1 && self.0 < 1000
    }

    pub fn is_development(self) -> bool {
        self.0 == 0
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemVersion(pub u32);

impl SystemVersion {
    pub fn new(major: u16, minor: u8, patch: u8) -> Self {
        Self(((major as u32) << 16) | ((minor as u32) << 8) | (patch as u32))
    }

    pub fn major(self) -> u16 {
        (self.0 >> 16) as u16
    }

    pub fn minor(self) -> u8 {
        ((self.0 >> 8) & 0xFF) as u8
    }

    pub fn patch(self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }
}

impl Default for SystemVersion {
    fn default() -> Self {
        Self(1 << 16)
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BlockNumber(pub u64);

impl BlockNumber {
    pub fn new(n: u64) -> Self {
        Self(n)
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }

    pub fn saturating_add(self, other: u64) -> Self {
        Self(self.0.saturating_add(other))
    }

    pub fn saturating_sub(self, other: u64) -> Self {
        Self(self.0.saturating_sub(other))
    }
}

impl From<u64> for BlockNumber {
    fn from(n: u64) -> Self {
        Self(n)
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Nonce(pub u64);

impl Nonce {
    pub fn new(n: u64) -> Self {
        Self(n)
    }

    pub fn increment(self) -> Self {
        Self(self.0 + 1)
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<u64> for Nonce {
    fn from(n: u64) -> Self {
        Self(n)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemError {
    pub kind: SystemErrorKind,
    pub message: String,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemErrorKind {
    Unknown,
    InvalidTransaction,
    InvalidSignature,
    InsufficientFunds,
    InsufficientGas,
    NonceTooLow,
    NonceTooHigh,
    ChainIdMismatch,
    InvalidBlock,
    InvalidState,
    StorageOverflow,
    StorageUnderflow,
    ModuleNotFound,
    MethodNotFound,
    CallFailed,
    ContractNotFound,
    ContractCodeNotFound,
    ContractExecutionFailed,
    InvalidCall,
    InvalidStorageKey,
    InvalidArgument,
    ArithmeticOverflow,
    ArithmeticUnderflow,
    DivisionByZero,
    BadOrigin,
    TokenTransferFailed,
    VaultNotFound,
    VaultInsufficientStake,
    ModelNotFound,
    ModelNotActive,
    PromptNotFound,
    PromptExpired,
    ReceiptAlreadySubmitted,
    GovernanceProposalNotFound,
    GovernanceProposalExpired,
    GovernanceVotingPeriod,
    ValidatorNotFound,
    ValidatorAlreadyExists,
    ValidatorInsufficientStake,
    Slashed,
    Jailed,
}

impl SystemError {
    pub fn new(kind: SystemErrorKind) -> Self {
        Self {
            kind,
            message: String::new(),
        }
    }

    pub fn with_message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    pub fn code(&self) -> u32 {
        match self.kind {
            SystemErrorKind::Unknown => 0,
            SystemErrorKind::InvalidTransaction => 1,
            SystemErrorKind::InvalidSignature => 2,
            SystemErrorKind::InsufficientFunds => 3,
            SystemErrorKind::InsufficientGas => 4,
            SystemErrorKind::NonceTooLow => 5,
            SystemErrorKind::NonceTooHigh => 6,
            SystemErrorKind::ChainIdMismatch => 7,
            SystemErrorKind::InvalidBlock => 10,
            SystemErrorKind::InvalidState => 11,
            SystemErrorKind::StorageOverflow => 12,
            SystemErrorKind::StorageUnderflow => 13,
            SystemErrorKind::ModuleNotFound => 20,
            SystemErrorKind::MethodNotFound => 21,
            SystemErrorKind::CallFailed => 22,
            SystemErrorKind::ContractNotFound => 30,
            SystemErrorKind::ContractCodeNotFound => 31,
            SystemErrorKind::ContractExecutionFailed => 32,
            SystemErrorKind::InvalidCall => 33,
            SystemErrorKind::InvalidStorageKey => 34,
            SystemErrorKind::InvalidArgument => 35,
            SystemErrorKind::ArithmeticOverflow => 40,
            SystemErrorKind::ArithmeticUnderflow => 41,
            SystemErrorKind::DivisionByZero => 42,
            SystemErrorKind::BadOrigin => 50,
            SystemErrorKind::TokenTransferFailed => 60,
            SystemErrorKind::VaultNotFound => 70,
            SystemErrorKind::VaultInsufficientStake => 71,
            SystemErrorKind::ModelNotFound => 80,
            SystemErrorKind::ModelNotActive => 81,
            SystemErrorKind::PromptNotFound => 90,
            SystemErrorKind::PromptExpired => 91,
            SystemErrorKind::ReceiptAlreadySubmitted => 92,
            SystemErrorKind::GovernanceProposalNotFound => 100,
            SystemErrorKind::GovernanceProposalExpired => 101,
            SystemErrorKind::GovernanceVotingPeriod => 102,
            SystemErrorKind::ValidatorNotFound => 110,
            SystemErrorKind::ValidatorAlreadyExists => 111,
            SystemErrorKind::ValidatorInsufficientStake => 112,
            SystemErrorKind::Slashed => 120,
            SystemErrorKind::Jailed => 121,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventId(pub [u8; 32]);

impl EventId {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemEvent {
    pub event_id: EventId,
    pub phase: EventPhase,
    pub module: String,
    pub event_type: SystemEventType,
    pub topics: Vec<Hash32>,
    pub data: Vec<u8>,
    pub block_number: u64,
    pub transaction_hash: Option<Hash32>,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventPhase {
    Initialization,
    ApplyExtrinsic(u32),
    Finalization,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SystemEventType {
    ExtrinsicSuccess,
    ExtrinsicFailed {
        error: SystemError,
    },
    CodeUpdated,
    RuntimeUpdated,
    NewAccount {
        account: Address,
    },
    KilledAccount {
        account: Address,
    },
    Transfer {
        from: Address,
        to: Address,
        amount: u128,
    },
    Deposit {
        account: Address,
        amount: u128,
    },
    Withdrawal {
        account: Address,
        amount: u128,
    },
    BalanceSet {
        account: Address,
        free: u128,
        reserved: u128,
    },
    EventEmitted {
        module: String,
        event: String,
    },
    GasConsumed {
        consumer: Address,
        gas: u64,
    },
    GasRefunded {
        consumer: Address,
        gas: u64,
    },
    StorageChange {
        key: Vec<u8>,
        old_value: Option<Vec<u8>>,
        new_value: Option<Vec<u8>>,
    },
    InvalidTransaction {
        reason: SystemErrorKind,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeVersion {
    pub spec_name: String,
    pub impl_name: String,
    pub authoring_version: u32,
    pub spec_version: u32,
    pub impl_version: u32,
    pub apis: Vec<(String, u32)>,
}

impl Default for RuntimeVersion {
    fn default() -> Self {
        Self {
            spec_name: "tetcore".to_string(),
            impl_name: "tetcore".to_string(),
            authoring_version: 1,
            spec_version: 1,
            impl_version: 1,
            apis: Vec::new(),
        }
    }
}

impl RuntimeVersion {
    pub fn new(spec_name: String, spec_version: u32) -> Self {
        Self {
            spec_name,
            impl_name: "tetcore".to_string(),
            authoring_version: 1,
            spec_version,
            impl_version: 1,
            apis: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemProperties {
    pub ss58_format: u8,
    pub token_symbol: String,
    pub token_decimals: u8,
    pub chain_type: CoreChainType,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CoreChainType {
    #[default]
    Development,
    Local,
    Testnet,
    Mainnet,
}

impl Default for SystemProperties {
    fn default() -> Self {
        Self {
            ss58_format: 42,
            token_symbol: "TNT".to_string(),
            token_decimals: 18,
            chain_type: CoreChainType::Development,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Constants {
    pub max_block_length: u32,
    pub max_transaction_size: u32,
    pub max_call_depth: u32,
    pub max_storage_size: u64,
    pub block_gas_limit: u64,
    pub transaction_gas_limit: u64,
    pub storage_gas_per_byte: u64,
    pub evm_gas_per_nanosecond: u64,
    pub min_balance: u128,
    pub max_authorities: u32,
    pub min_validator_stake: u128,
    pub epochs_per_era: u64,
    pub blocks_per_epoch: u64,
}

impl Default for Constants {
    fn default() -> Self {
        Self {
            max_block_length: 10_485_760,
            max_transaction_size: 2_097_152,
            max_call_depth: 1024,
            max_storage_size: 1_073_741_824,
            block_gas_limit: 100_000_000,
            transaction_gas_limit: 50_000_000,
            storage_gas_per_byte: 1,
            evm_gas_per_nanosecond: 1,
            min_balance: 1_000_000_000_000_000_000,
            max_authorities: 1000,
            min_validator_stake: 100_000_000_000_000_000_000_000,
            epochs_per_era: 1,
            blocks_per_epoch: 2400,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Upgrades {
    pub schedule_version: u32,
    pub active_version: RuntimeVersion,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub chain_id: ChainId,
    pub version: RuntimeVersion,
    pub properties: SystemProperties,
    pub constants: Constants,
}

impl SystemInfo {
    pub fn new(chain_id: ChainId) -> Self {
        Self {
            chain_id,
            version: RuntimeVersion::default(),
            properties: SystemProperties::default(),
            constants: Constants::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoreEventRecord {
    pub phase: EventPhase,
    pub module: String,
    pub event: SystemEventType,
    pub topics: Vec<Hash32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EventEmitter {
    pub events: Vec<CoreEventRecord>,
}

impl EventEmitter {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn emit(&mut self, module: String, event: SystemEventType) {
        self.events.push(CoreEventRecord {
            phase: EventPhase::ApplyExtrinsic(0),
            module,
            event,
            topics: Vec::new(),
        });
    }

    pub fn drain(&mut self) -> Vec<CoreEventRecord> {
        std::mem::take(&mut self.events)
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timestamp(u64);

impl Timestamp {
    pub fn now() -> Self {
        Self(0)
    }

    pub fn from_millis(ms: u64) -> Self {
        Self(ms)
    }

    pub fn as_millis(self) -> u64 {
        self.0
    }

    pub fn as_secs(self) -> u64 {
        self.0 / 1000
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self(0)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventFilter {
    pub module: Option<String>,
    pub event_type: Option<String>,
    pub from_block: Option<BlockNumber>,
    pub to_block: Option<BlockNumber>,
    pub from_address: Option<Address>,
    pub to_address: Option<Address>,
}

impl Default for EventFilter {
    fn default() -> Self {
        Self {
            module: None,
            event_type: None,
            from_block: None,
            to_block: None,
            from_address: None,
            to_address: None,
        }
    }
}

impl EventFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_module(mut self, module: String) -> Self {
        self.module = Some(module);
        self
    }

    pub fn with_from_block(mut self, block: BlockNumber) -> Self {
        self.from_block = Some(block);
        self
    }

    pub fn with_to_block(mut self, block: BlockNumber) -> Self {
        self.to_block = Some(block);
        self
    }
}
