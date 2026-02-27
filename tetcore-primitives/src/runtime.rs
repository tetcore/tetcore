// File: runtime.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Runtime primitives for Tetcore including Header, Transaction, Receipt,
// TransactionAction, Digest, DigestItem, DispatchError, Log, and Block.
// Provides core block structure, transaction types, and execution receipt
// definitions for the deterministic state machine.

use crate::crypto::{Address, Signature};
use crate::hash::Hash32;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Header {
    pub parent_hash: Hash32,
    pub number: u64,
    pub state_root: Hash32,
    pub transaction_root: Hash32,
    pub receipts_root: Hash32,
    pub digest: Digest,
    pub timestamp: u64,
    pub validator_set_id: u64,
}

impl Header {
    pub fn new(
        parent_hash: Hash32,
        number: u64,
        state_root: Hash32,
        transaction_root: Hash32,
    ) -> Self {
        Self {
            parent_hash,
            number,
            state_root,
            transaction_root,
            receipts_root: Hash32::empty(),
            digest: Digest::default(),
            timestamp: 0,
            validator_set_id: 0,
        }
    }

    pub fn hash(&self) -> Hash32 {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(self.encode());
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        Hash32(hash)
    }

    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.parent_hash.as_bytes());
        bytes.extend_from_slice(&self.number.to_le_bytes());
        bytes.extend_from_slice(self.state_root.as_bytes());
        bytes.extend_from_slice(self.transaction_root.as_bytes());
        bytes.extend_from_slice(self.receipts_root.as_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(&self.validator_set_id.to_le_bytes());
        bytes
    }
}

impl Default for Header {
    fn default() -> Self {
        Self {
            parent_hash: Hash32::empty(),
            number: 0,
            state_root: Hash32::empty(),
            transaction_root: Hash32::empty(),
            receipts_root: Hash32::empty(),
            digest: Digest::default(),
            timestamp: 0,
            validator_set_id: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub sender: Address,
    pub recipient: Option<Address>,
    pub nonce: u64,
    pub action: TransactionAction,
    pub payload: Vec<u8>,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub value: u128,
    pub signature: Option<Signature>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TransactionAction {
    Call(Address),
    Create,
    Invoke { module: String, method: String },
}

impl Transaction {
    pub fn new(sender: Address) -> Self {
        Self {
            sender,
            recipient: None,
            nonce: 0,
            action: TransactionAction::Call(Address::from_bytes([0u8; 32])),
            payload: Vec::new(),
            gas_limit: 0,
            gas_price: 0,
            value: 0,
            signature: None,
        }
    }

    pub fn signed(mut self, signature: Signature) -> Self {
        self.signature = Some(signature);
        self
    }

    pub fn is_signed(&self) -> bool {
        self.signature.is_some()
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.sender.as_bytes());
        if let Some(ref recipient) = self.recipient {
            bytes.extend_from_slice(recipient.as_bytes());
        } else {
            bytes.extend_from_slice(&[0u8; 32]);
        }
        bytes.extend_from_slice(&self.nonce.to_le_bytes());
        bytes.extend_from_slice(&self.gas_limit.to_le_bytes());
        bytes.extend_from_slice(&self.gas_price.to_le_bytes());
        bytes.extend_from_slice(&self.value.to_le_bytes());
        bytes.extend_from_slice(&self.payload.len().to_le_bytes());
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    pub fn hash(&self) -> Hash32 {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(self.encode());
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        Hash32(hash)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Receipt {
    pub transaction_hash: Hash32,
    pub transaction_index: u32,
    pub block_hash: Hash32,
    pub block_number: u64,
    pub gas_used: u64,
    pub logs: Vec<Log>,
    pub outcome: TransactionOutcome,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum TransactionOutcome {
    #[default]
    Success,
    Failure {
        error: DispatchError,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Log {
    pub address: Address,
    pub topics: Vec<Hash32>,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Digest {
    pub logs: Vec<DigestItem>,
}

impl Digest {
    pub fn new() -> Self {
        Self { logs: Vec::new() }
    }

    pub fn push(&mut self, item: DigestItem) {
        self.logs.push(item);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DigestItem {
    PreRuntime(Hash32, Vec<u8>),
    Runtime(Hash32, Vec<u8>),
    Seal(Hash32, Vec<u8>),
    Consensus(Hash32, Vec<u8>),
    ChangesTrieRoot(Hash32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DispatchError {
    pub module: Option<u8>,
    pub error: u8,
    pub message: Option<String>,
}

impl DispatchError {
    pub fn new(module: Option<u8>, error: u8) -> Self {
        Self {
            module,
            error,
            message: None,
        }
    }

    pub fn with_message(mut self, message: &str) -> Self {
        self.message = Some(message.to_string());
        self
    }

    pub fn bad_origin() -> Self {
        Self {
            module: None,
            error: 1,
            message: Some("Bad origin".to_string()),
        }
    }

    pub fn module_not_found(index: u8) -> Self {
        Self {
            module: Some(index),
            error: 0,
            message: Some("Module not found".to_string()),
        }
    }
}

impl Default for DispatchError {
    fn default() -> Self {
        Self {
            module: None,
            error: 0,
            message: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    pub header: Header,
    pub transactions: Vec<Transaction>,
    pub receipts: Vec<Receipt>,
}

impl Block {
    pub fn new(header: Header) -> Self {
        Self {
            header,
            transactions: Vec::new(),
            receipts: Vec::new(),
        }
    }

    pub fn hash(&self) -> Hash32 {
        self.header.hash()
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::new(Header::default())
    }
}
