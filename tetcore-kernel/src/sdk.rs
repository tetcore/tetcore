// File: sdk.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Tetcore SDK Blueprint Framework for composing custom runtimes.
// Provides module traits, genesis configuration, network identity,
// and deterministic runtime generation.

use crate::{Address, Hash32};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub const MAX_MODULES: usize = 64;
pub const MAX_GENESIS_ACCOUNTS: usize = 1000;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModuleId(pub [u8; 4]);

impl ModuleId {
    pub fn from_name(name: &str) -> Self {
        let mut id = [0u8; 4];
        let hash = Sha256::digest(name.as_bytes());
        id.copy_from_slice(&hash[..4]);
        ModuleId(id)
    }
}

pub trait TetcoreModule: Send + Sync {
    fn module_id(&self) -> ModuleId;
    fn name(&self) -> &str;
    fn version(&self) -> (u32, u32, u32);

    fn initialize(&self, genesis: &mut GenesisState) -> Result<(), ModuleError>;
    fn apply(
        &self,
        tx: &ModuleTransaction,
        state: &mut ModuleState,
    ) -> Result<ModuleResult, ModuleError>;
    fn on_block_end(&self, state: &mut ModuleState) -> Result<(), ModuleError>;

    fn get_storage(&self, state: &ModuleState, key: &[u8]) -> Option<Vec<u8>>;
    fn set_storage(&self, state: &mut ModuleState, key: Vec<u8>, value: Vec<u8>);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModuleTransaction {
    pub module_id: ModuleId,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModuleResult {
    pub success: bool,
    pub output: Vec<u8>,
    pub events: Vec<ModuleEvent>,
    pub state_changes: HashMap<Vec<u8>, Option<Vec<u8>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModuleEvent {
    pub module_id: ModuleId,
    pub event_type: u32,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModuleState {
    pub storage: HashMap<Vec<u8>, Vec<u8>>,
}

impl ModuleState {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<&Vec<u8>> {
        self.storage.get(key)
    }

    pub fn set(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.storage.insert(key, value);
    }

    pub fn remove(&mut self, key: &[u8]) {
        self.storage.remove(key);
    }

    pub fn root(&self) -> Hash32 {
        let mut keys: Vec<_> = self.storage.keys().collect();
        keys.sort();

        if keys.is_empty() {
            return Hash32::empty();
        }

        let hashes: Vec<Hash32> = keys
            .iter()
            .filter_map(|k| {
                self.storage.get(*k).map(|v| {
                    let mut data = Vec::new();
                    data.extend_from_slice(k);
                    data.extend_from_slice(v);
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
}

impl Default for ModuleState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenesisAccount {
    pub address: Address,
    pub balance: u128,
    pub code: Option<Vec<u8>>,
    pub storage: HashMap<Vec<u8>, Vec<u8>>,
}

impl GenesisAccount {
    pub fn new(address: Address, balance: u128) -> Self {
        Self {
            address,
            balance,
            code: None,
            storage: HashMap::new(),
        }
    }

    pub fn with_code(mut self, code: Vec<u8>) -> Self {
        self.code = Some(code);
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenesisConfig {
    pub name: String,
    pub id: String,
    pub chain_id: u32,
    pub genesis_time: u64,
    pub initial_validators: Vec<Address>,
    pub accounts: Vec<GenesisAccount>,
    pub parameters: RuntimeParameters,
}

impl GenesisConfig {
    pub fn new(name: String, id: String, chain_id: u32) -> Self {
        Self {
            name,
            id,
            chain_id,
            genesis_time: 0,
            initial_validators: Vec::new(),
            accounts: Vec::new(),
            parameters: RuntimeParameters::default(),
        }
    }

    pub fn with_validator(mut self, address: Address) -> Self {
        self.initial_validators.push(address);
        self
    }

    pub fn with_account(mut self, account: GenesisAccount) -> Self {
        self.accounts.push(account);
        self
    }

    pub fn with_parameter<T: Into<RuntimeValue>>(&mut self, key: &str, value: T) {
        self.parameters.set(key, value);
    }

    pub fn verify(&self) -> Result<(), GenesisError> {
        if self.accounts.len() > MAX_GENESIS_ACCOUNTS {
            return Err(GenesisError::TooManyAccounts);
        }

        let mut total_balance: u128 = 0;
        for account in &self.accounts {
            total_balance = total_balance.saturating_add(account.balance);
        }

        Ok(())
    }

    pub fn root(&self) -> Hash32 {
        let mut data = Vec::new();

        data.extend_from_slice(self.name.as_bytes());
        data.extend_from_slice(self.id.as_bytes());
        data.extend_from_slice(&self.chain_id.to_le_bytes());

        let mut account_hashes: Vec<u8> = Vec::new();
        for account in &self.accounts {
            account_hashes.extend_from_slice(account.address.as_bytes());
            account_hashes.extend_from_slice(&account.balance.to_le_bytes());
        }

        let account_hash = Sha256::digest(&account_hashes);
        data.extend_from_slice(&account_hash);

        let digest = Sha256::digest(&data);
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&digest[..32]);
        Hash32(hash)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RuntimeParameters {
    values: HashMap<String, RuntimeValue>,
}

impl RuntimeParameters {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn set<T: Into<RuntimeValue>>(&mut self, key: &str, value: T) {
        self.values.insert(key.to_string(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&RuntimeValue> {
        self.values.get(key)
    }

    pub fn get_u64(&self, key: &str, default: u64) -> u64 {
        self.values
            .get(key)
            .and_then(|v| v.as_u64())
            .unwrap_or(default)
    }

    pub fn get_u128(&self, key: &str, default: u128) -> u128 {
        self.values
            .get(key)
            .and_then(|v| v.as_u128())
            .unwrap_or(default)
    }

    pub fn get_bool(&self, key: &str, default: bool) -> bool {
        self.values
            .get(key)
            .and_then(|v| v.as_bool())
            .unwrap_or(default)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RuntimeValue {
    U64(u64),
    U128(u128),
    Bool(bool),
    String(String),
    Bytes(Vec<u8>),
}

impl RuntimeValue {
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            RuntimeValue::U64(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_u128(&self) -> Option<u128> {
        match self {
            RuntimeValue::U128(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            RuntimeValue::Bool(v) => Some(*v),
            _ => None,
        }
    }
}

impl From<u64> for RuntimeValue {
    fn from(v: u64) -> Self {
        RuntimeValue::U64(v)
    }
}

impl From<u128> for RuntimeValue {
    fn from(v: u128) -> Self {
        RuntimeValue::U128(v)
    }
}

impl From<bool> for RuntimeValue {
    fn from(v: bool) -> Self {
        RuntimeValue::Bool(v)
    }
}

impl From<String> for RuntimeValue {
    fn from(v: String) -> Self {
        RuntimeValue::String(v)
    }
}

impl From<&str> for RuntimeValue {
    fn from(v: &str) -> Self {
        RuntimeValue::String(v.to_string())
    }
}

impl From<Vec<u8>> for RuntimeValue {
    fn from(v: Vec<u8>) -> Self {
        RuntimeValue::Bytes(v)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenesisState {
    pub accounts: HashMap<Address, GenesisAccount>,
    pub validators: Vec<Address>,
    pub parameters: RuntimeParameters,
    pub module_states: HashMap<ModuleId, ModuleState>,
}

impl GenesisState {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            validators: Vec::new(),
            parameters: RuntimeParameters::new(),
            module_states: HashMap::new(),
        }
    }

    pub fn add_account(&mut self, account: GenesisAccount) {
        self.accounts.insert(account.address, account);
    }

    pub fn add_validator(&mut self, address: Address) {
        if !self.validators.contains(&address) {
            self.validators.push(address);
        }
    }

    pub fn get_account(&self, address: &Address) -> Option<&GenesisAccount> {
        self.accounts.get(address)
    }

    pub fn root(&self) -> Hash32 {
        let mut data = Vec::new();

        let mut account_keys: Vec<_> = self.accounts.keys().collect();
        account_keys.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        for addr in account_keys {
            data.extend_from_slice(addr.as_bytes());
            if let Some(acc) = self.accounts.get(addr) {
                data.extend_from_slice(&acc.balance.to_le_bytes());
            }
        }

        for val in &self.validators {
            data.extend_from_slice(val.as_bytes());
        }

        let digest = Sha256::digest(&data);
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&digest[..32]);
        Hash32(hash)
    }
}

impl Default for GenesisState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkIdentity {
    pub network_id: String,
    pub chain_id: u32,
    pub genesis_hash: Hash32,
    pub protocol_version: u32,
}

impl NetworkIdentity {
    pub fn new(network_id: String, chain_id: u32, genesis_config: &GenesisConfig) -> Self {
        let genesis_hash = genesis_config.root();

        Self {
            network_id,
            chain_id,
            genesis_hash,
            protocol_version: 1,
        }
    }

    pub fn verify(&self, other: &NetworkIdentity) -> bool {
        self.chain_id == other.chain_id && self.genesis_hash == other.genesis_hash
    }
}

pub struct Blueprint {
    pub name: String,
    pub version: (u32, u32, u32),
    pub modules: Vec<Box<dyn TetcoreModule>>,
    pub genesis_config: Option<GenesisConfig>,
}

impl Blueprint {
    pub fn new(name: String, version: (u32, u32, u32)) -> Self {
        Self {
            name,
            version,
            modules: Vec::new(),
            genesis_config: None,
        }
    }

    pub fn with_module(mut self, module: Box<dyn TetcoreModule>) -> Result<Self, BlueprintError> {
        if self.modules.len() >= MAX_MODULES {
            return Err(BlueprintError::TooManyModules);
        }

        if self
            .modules
            .iter()
            .any(|m| m.module_id() == module.module_id())
        {
            return Err(BlueprintError::DuplicateModule);
        }

        self.modules.push(module);
        Ok(self)
    }

    pub fn with_genesis(mut self, config: GenesisConfig) -> Self {
        self.genesis_config = Some(config);
        self
    }

    pub fn initialize(&self) -> Result<GenesisState, BlueprintError> {
        let mut state = GenesisState::new();

        if let Some(ref config) = self.genesis_config {
            for account in &config.accounts {
                state.add_account(account.clone());
            }

            for validator in &config.initial_validators {
                state.add_validator(*validator);
            }

            state.parameters = config.parameters.clone();
        }

        for module in &self.modules {
            module
                .initialize(&mut state)
                .map_err(|_| BlueprintError::InvalidGenesis)?;
        }

        Ok(state)
    }

    pub fn module_count(&self) -> usize {
        self.modules.len()
    }

    pub fn get_module(&self, id: &ModuleId) -> Option<&Box<dyn TetcoreModule>> {
        self.modules.iter().find(|m| m.module_id() == *id)
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum ModuleError {
    InvalidTransaction,
    StorageError,
    InsufficientBalance,
    NotFound,
    AlreadyExists,
    InvalidState,
    ModuleNotFound,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum GenesisError {
    InvalidConfig,
    TooManyAccounts,
    InvalidAccount,
    DuplicateAccount,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum BlueprintError {
    ModuleNotFound,
    DuplicateModule,
    TooManyModules,
    InvalidGenesis,
    InvalidConfiguration,
}

pub struct Runtime {
    pub chain_id: u32,
    pub version: (u32, u32, u32),
    pub modules: HashMap<ModuleId, Box<dyn TetcoreModule>>,
    pub state: HashMap<ModuleId, ModuleState>,
}

impl Runtime {
    pub fn new(chain_id: u32, version: (u32, u32, u32)) -> Self {
        Self {
            chain_id,
            version,
            modules: HashMap::new(),
            state: HashMap::new(),
        }
    }

    pub fn register_module(
        &mut self,
        module: Box<dyn TetcoreModule>,
    ) -> Result<(), BlueprintError> {
        let id = module.module_id();

        if self.modules.contains_key(&id) {
            return Err(BlueprintError::DuplicateModule);
        }

        self.state.insert(id, ModuleState::new());
        self.modules.insert(id, module);

        Ok(())
    }

    pub fn execute(&mut self, tx: &ModuleTransaction) -> Result<ModuleResult, ModuleError> {
        let module = self
            .modules
            .get(&tx.module_id)
            .ok_or(ModuleError::ModuleNotFound)?;
        let state = self
            .state
            .get_mut(&tx.module_id)
            .ok_or(ModuleError::ModuleNotFound)?;

        module.apply(tx, state)
    }

    pub fn on_block_end(&mut self) -> Result<(), ModuleError> {
        for (id, state) in self.state.iter_mut() {
            if let Some(module) = self.modules.get(id) {
                module.on_block_end(state)?;
            }
        }
        Ok(())
    }

    pub fn state_root(&self) -> Hash32 {
        let mut roots: Vec<Hash32> = self.state.values().map(|s| s.root()).collect();
        roots.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        if roots.is_empty() {
            return Hash32::empty();
        }

        while roots.len() > 1 {
            if roots.len() % 2 == 1 {
                roots.push(Hash32::empty());
            }
            let mut next = Vec::new();
            for pair in roots.chunks(2) {
                let mut data = Vec::new();
                data.extend_from_slice(pair[0].as_bytes());
                data.extend_from_slice(pair[1].as_bytes());
                let digest = Sha256::digest(&data);
                let mut h = [0u8; 32];
                h.copy_from_slice(&digest[..32]);
                next.push(Hash32(h));
            }
            roots = next;
        }

        roots[0]
    }

    pub fn module_count(&self) -> usize {
        self.modules.len()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RuntimeParametersBuilder {
    params: RuntimeParameters,
}

impl RuntimeParametersBuilder {
    pub fn new() -> Self {
        Self {
            params: RuntimeParameters::new(),
        }
    }

    pub fn set<T: Into<RuntimeValue>>(mut self, key: &str, value: T) -> Self {
        self.params.set(key, value);
        self
    }

    pub fn build(self) -> RuntimeParameters {
        self.params
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_config() {
        let config = GenesisConfig::new("test".to_string(), "testnet".to_string(), 1)
            .with_validator(Address([1u8; 32]))
            .with_account(GenesisAccount::new(Address([2u8; 32]), 1000));

        assert!(config.verify().is_ok());
    }

    #[test]
    fn test_runtime_parameters() {
        let params = RuntimeParametersBuilder::new()
            .set("max_validators", 100u64)
            .set("block_time", 5000u64)
            .set("enable_inflation", false)
            .build();

        assert_eq!(params.get_u64("max_validators", 0), 100);
        assert_eq!(params.get_bool("enable_inflation", true), false);
    }

    #[test]
    fn test_module_state_root() {
        let mut state = ModuleState::new();
        state.set(b"key1".to_vec(), b"value1".to_vec());
        state.set(b"key2".to_vec(), b"value2".to_vec());

        let root = state.root();
        assert!(!root.0.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_network_identity() {
        let config = GenesisConfig::new("test".to_string(), "testnet".to_string(), 1);
        let identity = NetworkIdentity::new("testnet".to_string(), 1, &config);

        assert_eq!(identity.chain_id, 1);
    }
}
