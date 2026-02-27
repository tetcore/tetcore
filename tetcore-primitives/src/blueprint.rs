// File: blueprint.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Blueprint and genesis configuration primitives for Tetcore SDK.
// Defines GenesisConfig for initial state setup, RuntimeVersion,
// ChainProperties, NetworkConfig, BlueprintManifest, and host function
// definitions for custom runtime composition.

use crate::consensus::{AuthoritySet, ConsensusParams, ValidatorSet};
use crate::economics::{FeeParameters, GasSchedule, TokenSupply};
use crate::governance::GovernanceParameters;
use crate::hash::Hash32;
use crate::inference::Model;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenesisConfig {
    pub chain_id: u32,
    pub chain_name: String,
    pub initial_runtime_version: RuntimeVersion,
    pub initial_validators: ValidatorSet,
    pub genesis_balances: Vec<(String, u128)>,
    pub initial_models: Vec<Model>,
    pub governance_parameters: GovernanceParameters,
    pub consensus_parameters: ConsensusParams,
    pub fee_parameters: FeeParameters,
    pub gas_schedule: GasSchedule,
    pub token_supply: TokenSupply,
    pub initial_authority_set: AuthoritySet,
}

impl GenesisConfig {
    pub fn new(chain_name: String) -> Self {
        Self {
            chain_id: 0,
            chain_name,
            initial_runtime_version: RuntimeVersion::default(),
            initial_validators: ValidatorSet::new(),
            genesis_balances: Vec::new(),
            initial_models: Vec::new(),
            governance_parameters: GovernanceParameters::default(),
            consensus_parameters: ConsensusParams::default(),
            fee_parameters: FeeParameters::default(),
            gas_schedule: GasSchedule::default(),
            token_supply: TokenSupply::new(),
            initial_authority_set: AuthoritySet::new(Vec::new()),
        }
    }

    pub fn add_balance(&mut self, address: String, balance: u128) {
        self.genesis_balances.push((address, balance));
    }

    pub fn total_balance(&self) -> u128 {
        self.genesis_balances.iter().map(|(_, b)| b).sum()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeVersion {
    pub spec_name: String,
    pub impl_name: String,
    pub spec_version: u32,
    pub impl_version: u32,
    pub apis: Vec<([u8; 8], u32)>,
}

impl Default for RuntimeVersion {
    fn default() -> Self {
        Self {
            spec_name: "tetcore".to_string(),
            impl_name: "tetcore".to_string(),
            spec_version: 1,
            impl_version: 1,
            apis: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainProperties {
    pub ss58_format: u8,
    pub token_symbol: String,
    pub token_decimals: u8,
    pub chain_type: ChainType,
}

impl Default for ChainProperties {
    fn default() -> Self {
        Self {
            ss58_format: 42,
            token_symbol: "TNT".to_string(),
            token_decimals: 18,
            chain_type: ChainType::Development,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainType {
    Development,
    Local,
    Live,
    Testnet,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub listen_addresses: Vec<String>,
    pub public_addresses: Vec<String>,
    pub max_connections: u32,
    pub force_announce: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addresses: vec!["/ip4/0.0.0.0/tcp/30333".to_string()],
            public_addresses: Vec::new(),
            max_connections: 100,
            force_announce: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlueprintManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub modules: Vec<String>,
    pub parameters: Vec<BlueprintParameter>,
    pub genesis_config: GenesisConfig,
    pub runtime_version: RuntimeVersion,
    pub chain_properties: ChainProperties,
}

impl BlueprintManifest {
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            description: String::new(),
            modules: Vec::new(),
            parameters: Vec::new(),
            genesis_config: GenesisConfig::new(String::new()),
            runtime_version: RuntimeVersion::default(),
            chain_properties: ChainProperties::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlueprintParameter {
    pub name: String,
    pub value_type: String,
    pub default_value: Option<String>,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostFunctions {
    pub storage: Vec<HostFunction>,
    pub crypto: Vec<HostFunction>,
    pub inference: Vec<HostFunction>,
    pub misc: Vec<HostFunction>,
}

impl Default for HostFunctions {
    fn default() -> Self {
        Self {
            storage: Vec::new(),
            crypto: Vec::new(),
            inference: Vec::new(),
            misc: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostFunction {
    pub name: String,
    pub signature: String,
    pub description: String,
}
