// File: contracts.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Smart contract primitives for Tetcore including Contract, ContractCode,
// ContractCall, ContractMethod, ContractStorage, ContractEvent, and
// ContractMetadata. Supports TVM contract deployment, invocation,
// and event emission.

use crate::crypto::Address;
use crate::hash::Hash32;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contract {
    pub contract_id: Hash32,
    pub owner: Address,
    pub code_hash: Hash32,
    pub storage_root: Hash32,
    pub created_at: u64,
    pub frozen: bool,
}

impl Contract {
    pub fn new(owner: Address, code_hash: Hash32) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(owner.as_bytes());
        hasher.update(code_hash.as_bytes());
        let result = hasher.finalize();
        let mut contract_id = [0u8; 32];
        contract_id.copy_from_slice(&result);

        Self {
            contract_id: Hash32(contract_id),
            owner,
            code_hash,
            storage_root: Hash32::empty(),
            created_at: 0,
            frozen: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractCode {
    pub code_hash: Hash32,
    pub code: Vec<u8>,
    pub metadata: ContractMetadata,
}

impl ContractCode {
    pub fn new(code: Vec<u8>, metadata: ContractMetadata) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(&code);
        let result = hasher.finalize();
        let mut code_hash = [0u8; 32];
        code_hash.copy_from_slice(&result);

        Self {
            code_hash: Hash32(code_hash),
            code,
            metadata,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractMetadata {
    pub abi: Vec<ContractMethod>,
    pub language: ContractLanguage,
    pub compiler_version: String,
    pub gas_limit: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractMethod {
    pub name: String,
    pub arguments: Vec<ContractArg>,
    pub return_type: Option<ContractType>,
    pub gas_limit: Option<u64>,
    pub is_view: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractArg {
    pub name: String,
    pub arg_type: ContractType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ContractType {
    Address,
    Uint256,
    Uint128,
    Uint64,
    Uint32,
    Int256,
    Bool,
    Bytes,
    String,
    Array(Box<ContractType>),
    Tuple(Vec<ContractType>),
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractLanguage {
    TCL,
    Solidity,
    WASM,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractCall {
    pub contract_id: Hash32,
    pub caller: Address,
    pub method: String,
    pub args: Vec<Vec<u8>>,
    pub gas_limit: u64,
    pub gas_price: u128,
    pub value: u128,
    pub nonce: u64,
}

impl ContractCall {
    pub fn new(
        contract_id: Hash32,
        caller: Address,
        method: String,
        args: Vec<Vec<u8>>,
        gas_limit: u64,
    ) -> Self {
        Self {
            contract_id,
            caller,
            method,
            args,
            gas_limit,
            gas_price: 0,
            value: 0,
            nonce: 0,
        }
    }

    pub fn with_value(mut self, value: u128) -> Self {
        self.value = value;
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractResult {
    pub success: bool,
    pub return_data: Vec<u8>,
    pub gas_used: u64,
    pub gas_refunded: u64,
    pub logs: Vec<ContractLog>,
}

impl ContractResult {
    pub fn success(return_data: Vec<u8>, gas_used: u64) -> Self {
        Self {
            success: true,
            return_data,
            gas_used,
            gas_refunded: 0,
            logs: Vec::new(),
        }
    }

    pub fn failure(error: &str, gas_used: u64) -> Self {
        Self {
            success: false,
            return_data: error.as_bytes().to_vec(),
            gas_used,
            gas_refunded: 0,
            logs: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractLog {
    pub address: Address,
    pub topics: Vec<Hash32>,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractStorage {
    pub contract_id: Hash32,
    pub storage: Vec<(Vec<u8>, Vec<u8>)>,
}

impl ContractStorage {
    pub fn new(contract_id: Hash32) -> Self {
        Self {
            contract_id,
            storage: Vec::new(),
        }
    }

    pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>) {
        if let Some(pos) = self.storage.iter().position(|(k, _)| k == &key) {
            self.storage[pos] = (key, value);
        } else {
            self.storage.push((key, value));
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.storage
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }

    pub fn remove(&mut self, key: &[u8]) {
        self.storage.retain(|(k, _)| k != key);
    }

    pub fn root(&self) -> Hash32 {
        let mut hasher = Sha256::new();
        let mut keys: Vec<_> = self.storage.iter().map(|(k, _)| k).collect();
        keys.sort();
        for key in keys {
            if let Some((_, value)) = self.storage.iter().find(|(k, _)| k == key) {
                hasher.update(key);
                hasher.update(value);
            }
        }
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        Hash32(hash)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractEvent {
    pub event_id: Hash32,
    pub contract_id: Hash32,
    pub name: String,
    pub args: Vec<Vec<u8>>,
    pub timestamp: u64,
}
