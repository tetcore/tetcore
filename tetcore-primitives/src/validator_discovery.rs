// File: validator_discovery.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Validator discovery primitives for Tetcore including ValidatorInfo,
// ValidatorEndpoint, ValidatorRegistry, ValidatorSetSnapshot, DiscoveryRequest,
// DiscoveryResponse, ValidatorPerformance, ValidatorSessionInfo, and
// network topology types. Supports validator peer discovery and networking.

use crate::crypto::Address;
use crate::hash::Hash32;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub account_id: Address,
    pub stake: u128,
    pub commission: u8,
    pub active: bool,
    pub discovered_at: u64,
    pub last_heartbeat: u64,
    pub peer_id: Option<String>,
    pub listen_addresses: Vec<String>,
    pub endpoint: Option<ValidatorEndpoint>,
}

impl ValidatorInfo {
    pub fn new(account_id: Address, stake: u128) -> Self {
        Self {
            account_id,
            stake,
            commission: 0,
            active: true,
            discovered_at: 0,
            last_heartbeat: 0,
            peer_id: None,
            listen_addresses: Vec::new(),
            endpoint: None,
        }
    }

    pub fn is_online(&self, current_block: u64, heartbeat_timeout: u64) -> bool {
        current_block.saturating_sub(self.last_heartbeat) < heartbeat_timeout
    }

    pub fn update_heartbeat(&mut self, block_number: u64) {
        self.last_heartbeat = block_number;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorEndpoint {
    pub rpc_address: String,
    pub p2p_address: String,
    pub wasm_runtime: Option<String>,
    pub api_versions: Vec<ApiVersion>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiVersion {
    pub spec_version: u32,
    pub impl_version: u32,
    pub apis: Vec<([u8; 8], u32)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorSetSnapshot {
    pub set_id: u64,
    pub validators: Vec<ValidatorInfo>,
    pub total_stake: u128,
    pub block_number: u64,
    pub root_hash: Hash32,
}

impl ValidatorSetSnapshot {
    pub fn new(set_id: u64, validators: Vec<ValidatorInfo>) -> Self {
        let total_stake = validators.iter().map(|v| v.stake).sum();
        Self {
            set_id,
            validators,
            total_stake,
            block_number: 0,
            root_hash: Hash32::empty(),
        }
    }

    pub fn compute_root(&mut self) {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&self.set_id.to_le_bytes());
        for v in &self.validators {
            hasher.update(v.account_id.as_bytes());
            hasher.update(&v.stake.to_le_bytes());
            hasher.update(&v.commission.to_le_bytes());
        }
        hasher.update(&self.total_stake.to_le_bytes());
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        self.root_hash = Hash32(hash);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscoveryRequest {
    pub requester: Address,
    pub request_type: DiscoveryRequestType,
    pub max_results: u32,
    pub filter: Option<DiscoveryFilter>,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoveryRequestType {
    All,
    Active,
    ByStake,
    ByRegion,
    ByPerformance,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscoveryFilter {
    pub min_stake: Option<u128>,
    pub max_commission: Option<u8>,
    pub region: Option<String>,
    pub active_only: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscoveryResponse {
    pub request_id: Hash32,
    pub validators: Vec<ValidatorInfo>,
    pub total_found: u32,
    pub block_number: u64,
}

impl DiscoveryResponse {
    pub fn new(request_id: Hash32, validators: Vec<ValidatorInfo>, block_number: u64) -> Self {
        let total_found = validators.len() as u32;
        Self {
            request_id,
            validators,
            total_found,
            block_number,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub addresses: Vec<String>,
    pub connected: bool,
    pub latency_ms: Option<u32>,
    pub validator: Option<Address>,
    pub version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkTopology {
    pub connected_peers: u32,
    pub validator_peers: u32,
    pub discovered_at: u64,
    pub region: String,
}

impl NetworkTopology {
    pub fn new(region: String) -> Self {
        Self {
            connected_peers: 0,
            validator_peers: 0,
            discovered_at: 0,
            region,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorRegistry {
    pub entries: Vec<ValidatorInfo>,
    pub next_set_id: u64,
}

impl ValidatorRegistry {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_set_id: 1,
        }
    }

    pub fn register(&mut self, validator: ValidatorInfo) {
        if !self
            .entries
            .iter()
            .any(|v| v.account_id == validator.account_id)
        {
            self.entries.push(validator);
        }
    }

    pub fn deregister(&mut self, account_id: &Address) -> Option<ValidatorInfo> {
        if let Some(pos) = self
            .entries
            .iter()
            .position(|v| v.account_id == *account_id)
        {
            Some(self.entries.remove(pos))
        } else {
            None
        }
    }

    pub fn get(&self, account_id: &Address) -> Option<&ValidatorInfo> {
        self.entries.iter().find(|v| v.account_id == *account_id)
    }

    pub fn get_mut(&mut self, account_id: &Address) -> Option<&mut ValidatorInfo> {
        self.entries
            .iter_mut()
            .find(|v| v.account_id == *account_id)
    }

    pub fn active_validators(&self) -> Vec<&ValidatorInfo> {
        self.entries.iter().filter(|v| v.active).collect()
    }

    pub fn sort_by_stake(&mut self) {
        self.entries.sort_by(|a, b| b.stake.cmp(&a.stake));
    }
}

impl Default for ValidatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorSessionInfo {
    pub session_id: u64,
    pub validators: Vec<Address>,
    pub block_number: u64,
    pub root_hash: Hash32,
}

impl ValidatorSessionInfo {
    pub fn new(session_id: u64, validators: Vec<Address>, block_number: u64) -> Self {
        Self {
            session_id,
            validators,
            block_number,
            root_hash: Hash32::empty(),
        }
    }

    pub fn contains(&self, account_id: &Address) -> bool {
        self.validators.contains(account_id)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorPerformance {
    pub account_id: Address,
    pub blocks_proposed: u64,
    pub blocks_authored: u64,
    pub votes_cast: u64,
    pub missed_blocks: u64,
    pub uptime_percentage: u32,
    pub last_evaluated: u64,
}

impl ValidatorPerformance {
    pub fn new(account_id: Address) -> Self {
        Self {
            account_id,
            blocks_proposed: 0,
            blocks_authored: 0,
            votes_cast: 0,
            missed_blocks: 0,
            uptime_percentage: 100,
            last_evaluated: 0,
        }
    }

    pub fn record_block_proposed(&mut self) {
        self.blocks_proposed += 1;
    }

    pub fn record_block_authored(&mut self) {
        self.blocks_authored += 1;
    }

    pub fn record_vote(&mut self) {
        self.votes_cast += 1;
    }

    pub fn record_missed_block(&mut self) {
        self.missed_blocks += 1;
    }

    pub fn calculate_uptime(&mut self) {
        let total = self.blocks_proposed + self.missed_blocks;
        if total > 0 {
            self.uptime_percentage = ((self.blocks_proposed as u64 * 100) / total) as u32;
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorDiscoveryConfig {
    pub discovery_enabled: bool,
    pub discovery_interval_blocks: u32,
    pub heartbeat_timeout_blocks: u64,
    pub max_discovered_validators: u32,
    pub min_stake_threshold: u128,
    pub region_preference: Option<String>,
}

impl Default for ValidatorDiscoveryConfig {
    fn default() -> Self {
        Self {
            discovery_enabled: true,
            discovery_interval_blocks: 100,
            heartbeat_timeout_blocks: 600,
            max_discovered_validators: 100,
            min_stake_threshold: 0,
            region_preference: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscoveryAnnounce {
    pub account_id: Address,
    pub peer_id: String,
    pub listen_addresses: Vec<String>,
    pub endpoint: Option<ValidatorEndpoint>,
    pub signature: Vec<u8>,
}

impl DiscoveryAnnounce {
    pub fn new(account_id: Address, peer_id: String, listen_addresses: Vec<String>) -> Self {
        Self {
            account_id,
            peer_id,
            listen_addresses,
            endpoint: None,
            signature: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthoritySetTransition {
    pub old_set_id: u64,
    pub new_set_id: u64,
    pub added: Vec<Address>,
    pub removed: Vec<Address>,
    pub block_number: u64,
}
