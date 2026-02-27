// File: traits.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Core traits for Tetcore including ValidateTransaction, ApplyTransaction,
// Dispatchable, Module, BlockTrait, HeaderTrait, MerkleTrie, Store,
// TransactionValidity, and lifecycle hooks (OnInitialize, OnFinalize,
// OnRuntimeUpgrade). Defines the runtime module interface.

use crate::crypto::Address;
use crate::hash::Hash32;
use crate::runtime::DispatchError;

pub trait ValidateTransaction {
    type Validation;

    fn validate_transaction(&self, transaction: &crate::runtime::Transaction) -> Self::Validation;
}

pub trait ApplyTransaction {
    type ApplyResult;

    fn apply_transaction(&mut self, transaction: &crate::runtime::Transaction)
        -> Self::ApplyResult;
}

pub trait Module: Send + Sync {
    fn name(&self) -> &str;

    fn on_initialize(&mut self, _block_number: u64) -> Result<(), DispatchError> {
        Ok(())
    }

    fn on_finalize(&mut self, _block_number: u64) -> Result<(), DispatchError> {
        Ok(())
    }

    fn on_runtime_upgrade(&mut self) -> Result<(), DispatchError> {
        Ok(())
    }
}

pub trait OnInitialize {
    fn on_initialize(block_number: u64) -> Result<(), DispatchError>;
}

pub trait OnFinalize {
    fn on_finalize(block_number: u64) -> Result<(), DispatchError>;
}

pub trait OnRuntimeUpgrade {
    fn on_runtime_upgrade() -> Result<(), DispatchError>;
}

pub trait Store {
    fn store(&self, key: &[u8], value: &[u8]) -> Result<(), DispatchError>;
    fn load(&self, key: &[u8]) -> Result<Option<Vec<u8>>, DispatchError>;
    fn remove(&self, key: &[u8]) -> Result<(), DispatchError>;
}

pub trait Commit {
    fn commit(&mut self) -> Result<(), DispatchError>;
    fn rollback(&mut self);
}

pub trait MerkleTrie: Send + Sync {
    fn root(&self) -> Hash32;
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
    fn insert(&mut self, key: &[u8], value: &[u8]);
    fn remove(&mut self, key: &[u8]);
}

pub type DispatchResult = Result<(), DispatchError>;

pub trait Dispatchable {
    fn dispatch(self, origin: Address) -> DispatchResult;
}

#[derive(Clone, Debug)]
pub enum TransactionValidityError {
    Invalid(TransactionValidity),
    Unknown(DispatchError),
}

#[derive(Clone, Debug)]
pub enum TransactionValidity {
    Valid {
        priority: u64,
        requires: Vec<Vec<u8>>,
        provides: Vec<Vec<u8>>,
        longevity: u64,
    },
    Invalid {
        code: u64,
        message: Option<String>,
    },
    Unknown {
        code: u64,
        message: Option<String>,
    },
}

impl TransactionValidity {
    pub fn valid() -> Self {
        TransactionValidity::Valid {
            priority: 0,
            requires: Vec::new(),
            provides: Vec::new(),
            longevity: 64,
        }
    }

    pub fn invalid(code: u64) -> Self {
        TransactionValidity::Invalid {
            code,
            message: None,
        }
    }

    pub fn unknown(code: u64) -> Self {
        TransactionValidity::Unknown {
            code,
            message: None,
        }
    }
}

pub trait HeaderTrait: Send + Sync {
    type Hash: AsRef<[u8]> + Default + Clone + PartialEq + Eq;
    type Number: Default + Copy + PartialOrd + Send + Sync;

    fn number(&self) -> Self::Number;
    fn hash(&self) -> Self::Hash;
    fn parent_hash(&self) -> Self::Hash;
}

pub trait BlockTrait: Send + Sync {
    type Header: HeaderTrait;
    type Transaction: Send + Sync;
}
