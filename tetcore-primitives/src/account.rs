// File: account.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Core account types for Tetcore including AccountId and AccountData.
// Provides account representation, balance management, and nonce tracking
// for the deterministic state machine.

use crate::hash::Hash32;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy, Default, Serialize, Deserialize)]
pub struct AccountId(pub [u8; 32]);

impl AccountId {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn from_hex(hex_str: &str) -> Result<Self, String> {
        let bytes = hex::decode(hex_str).map_err(|e| e.to_string())?;
        if bytes.len() != 32 {
            return Err("Invalid account id length".to_string());
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(AccountId(arr))
    }

    pub fn zero() -> Self {
        Self([0u8; 32])
    }

    pub fn from_hash(hash: Hash32) -> Self {
        Self(hash.0)
    }

    pub fn into_hash(self) -> Hash32 {
        Hash32(self.0)
    }
}

impl AsRef<[u8]> for AccountId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<[u8; 32]> for AccountId {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AccountData {
    pub balance: u128,
    pub nonce: u64,
    pub contract_code_ref: Option<Hash32>,
    pub contract_storage_root: Option<Hash32>,
}

impl AccountData {
    pub fn new(balance: u128) -> Self {
        Self {
            balance,
            nonce: 0,
            contract_code_ref: None,
            contract_storage_root: None,
        }
    }

    pub fn zero() -> Self {
        Self::new(0)
    }

    pub fn can_withdraw(&self, amount: u128) -> bool {
        self.balance >= amount
    }

    pub fn add_balance(&mut self, amount: u128) {
        self.balance = self.balance.saturating_add(amount);
    }

    pub fn sub_balance(&mut self, amount: u128) -> bool {
        if self.can_withdraw(amount) {
            self.balance = self.balance.saturating_sub(amount);
            true
        } else {
            false
        }
    }

    pub fn inc_nonce(&mut self) {
        self.nonce = self.nonce.saturating_add(1);
    }
}

impl AsRef<AccountData> for AccountData {
    fn as_ref(&self) -> &AccountData {
        self
    }
}

impl AsMut<AccountData> for AccountData {
    fn as_mut(&mut self) -> &mut AccountData {
        self
    }
}
