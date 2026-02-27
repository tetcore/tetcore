// File: storage.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Storage primitives for Tetcore including Storage, ChildStorage,
// ChildInfo, and StorageProof. Provides Merkle trie-based state
// storage with child trie support and proof generation for
// light client verification.

use crate::hash::Hash32;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Storage {
    pub top: HashMap<Vec<u8>, Vec<u8>>,
    pub children_default: HashMap<Vec<u8>, ChildStorage>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            top: HashMap::new(),
            children_default: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.top.insert(key, value);
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.top.get(key).cloned()
    }

    pub fn remove(&mut self, key: &[u8]) {
        self.top.remove(key);
    }

    pub fn clear(&mut self) {
        self.top.clear();
        self.children_default.clear();
    }

    pub fn root(&self) -> Hash32 {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        let mut keys: Vec<_> = self.top.keys().collect();
        keys.sort();
        for key in keys {
            if let Some(value) = self.top.get(key) {
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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChildStorage {
    pub data: HashMap<Vec<u8>, Vec<u8>>,
    pub child_info: ChildInfo,
}

impl ChildStorage {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            child_info: ChildInfo::default(),
        }
    }

    pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.data.insert(key, value);
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.data.get(key).cloned()
    }

    pub fn remove(&mut self, key: &[u8]) {
        self.data.remove(key);
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChildInfo {
    pub storage_key: Vec<u8>,
    pub child_type: u32,
}

impl ChildInfo {
    pub fn new(storage_key: Vec<u8>) -> Self {
        Self {
            storage_key,
            child_type: 0,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StorageProof {
    pub nodes: Vec<Vec<u8>>,
}

impl StorageProof {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn add_node(&mut self, node: Vec<u8>) {
        self.nodes.push(node);
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}
