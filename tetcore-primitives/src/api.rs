// File: api.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// API primitives for Tetcore including endpoints, requests, responses,
// prompt management, and shard storage ledger. Supports the Intelligence
// Fabric Protocol with commitment-based privacy and availability verification.

use crate::crypto::Address;
use crate::hash::Hash32;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiEndpoint {
    pub path: String,
    pub method: ApiMethod,
    pub auth_required: bool,
    pub rate_limit: Option<RateLimit>,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApiMethod {
    Get,
    Post,
    Put,
    Delete,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RateLimit {
    pub max_requests: u32,
    pub window_blocks: u32,
}

impl Default for RateLimit {
    fn default() -> Self {
        Self {
            max_requests: 1000,
            window_blocks: 100,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiRequest {
    pub request_id: Hash32,
    pub endpoint: String,
    pub sender: Address,
    pub payload: Vec<u8>,
    pub timestamp: u64,
    pub block_number: u64,
    pub gas_limit: u64,
}

impl ApiRequest {
    pub fn new(endpoint: String, sender: Address, payload: Vec<u8>, gas_limit: u64) -> Self {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(endpoint.as_bytes());
        hasher.update(sender.as_bytes());
        hasher.update(&payload);
        hasher.update(&gas_limit.to_le_bytes());
        let result = hasher.finalize();
        let mut request_id = [0u8; 32];
        request_id.copy_from_slice(&result);

        Self {
            request_id: Hash32(request_id),
            endpoint,
            sender,
            payload,
            timestamp: 0,
            block_number: 0,
            gas_limit,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub request_id: Hash32,
    pub status: ApiResponseStatus,
    pub data: Vec<u8>,
    pub gas_used: u64,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApiResponseStatus {
    Success,
    InvalidRequest,
    Unauthorized,
    NotFound,
    RateLimited,
    GasExceeded,
    InternalError,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptCommitment {
    pub commitment: Hash32,
    pub salt: Vec<u8>,
    pub prompt_hash: Hash32,
}

impl PromptCommitment {
    pub fn new(prompt: &[u8], salt: Vec<u8>) -> Self {
        use sha2::{Digest, Sha256};

        let mut prompt_hasher = Sha256::new();
        prompt_hasher.update(prompt);
        prompt_hasher.update(&salt);
        let prompt_result = prompt_hasher.finalize();
        let mut prompt_hash = [0u8; 32];
        prompt_hash.copy_from_slice(&prompt_result);

        let mut commitment_hasher = Sha256::new();
        commitment_hasher.update(&prompt_hash);
        commitment_hasher.update(&salt);
        let commitment_result = commitment_hasher.finalize();
        let mut commitment = [0u8; 32];
        commitment.copy_from_slice(&commitment_result);

        Self {
            commitment: Hash32(commitment),
            salt,
            prompt_hash: Hash32(prompt_hash),
        }
    }

    pub fn verify(&self, prompt: &[u8]) -> bool {
        use sha2::{Digest, Sha256};
        let mut prompt_hasher = Sha256::new();
        prompt_hasher.update(prompt);
        prompt_hasher.update(&self.salt);
        let prompt_result = prompt_hasher.finalize();

        self.prompt_hash.as_bytes() == &prompt_result[..]
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptEntry {
    pub prompt_id: Hash32,
    pub model_id: Hash32,
    pub version: u32,
    pub sender: Address,
    pub prompt_commitment: Hash32,
    pub max_output_tokens: u32,
    pub pricing_mode: u8,
    pub relay_mode: u8,
    pub deadline_height: u64,
    pub escrow_amount: u128,
    pub status: PromptStatus,
    pub created_at: u64,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromptStatus {
    Pending,
    Settled,
    Expired,
    Disputed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShardStorageEntry {
    pub model_id: Hash32,
    pub version: u32,
    pub shard_index: u32,
    pub storage_nodes: Vec<Address>,
    pub collateral_locked: Vec<(Address, u128)>,
    pub registered_at: u64,
}

impl ShardStorageEntry {
    pub fn new(model_id: Hash32, version: u32, shard_index: u32) -> Self {
        Self {
            model_id,
            version,
            shard_index,
            storage_nodes: Vec::new(),
            collateral_locked: Vec::new(),
            registered_at: 0,
        }
    }

    pub fn register_node(&mut self, node: Address, collateral: u128) {
        if !self.storage_nodes.contains(&node) {
            self.storage_nodes.push(node);
            self.collateral_locked.push((node, collateral));
        }
    }

    pub fn deregister_node(&mut self, node: &Address) -> Option<u128> {
        if let Some(pos) = self.storage_nodes.iter().position(|n| n == node) {
            self.storage_nodes.remove(pos);
            if let Some(collateral_pos) = self.collateral_locked.iter().position(|(n, _)| n == node)
            {
                let (_, collateral) = self.collateral_locked.remove(collateral_pos);
                Some(collateral)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShardStorageLedger {
    pub entries: Vec<ShardStorageEntry>,
}

impl ShardStorageLedger {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get_entry(
        &self,
        model_id: Hash32,
        version: u32,
        shard_index: u32,
    ) -> Option<&ShardStorageEntry> {
        self.entries.iter().find(|e| {
            e.model_id == model_id && e.version == version && e.shard_index == shard_index
        })
    }

    pub fn get_entry_mut(
        &mut self,
        model_id: Hash32,
        version: u32,
        shard_index: u32,
    ) -> Option<&mut ShardStorageEntry> {
        self.entries.iter_mut().find(|e| {
            e.model_id == model_id && e.version == version && e.shard_index == shard_index
        })
    }

    pub fn add_entry(&mut self, entry: ShardStorageEntry) {
        if self
            .get_entry(entry.model_id, entry.version, entry.shard_index)
            .is_none()
        {
            self.entries.push(entry);
        }
    }
}

impl Default for ShardStorageLedger {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AvailabilityChallenge {
    pub challenge_id: Hash32,
    pub model_id: Hash32,
    pub version: u32,
    pub shard_index: u32,
    pub challenged_node: Address,
    pub challenger: Address,
    pub challenge_window: u32,
    pub status: ChallengeStatus,
    pub created_at: u64,
}

impl AvailabilityChallenge {
    pub fn new(
        model_id: Hash32,
        version: u32,
        shard_index: u32,
        challenged_node: Address,
        challenger: Address,
        challenge_window: u32,
    ) -> Self {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(model_id.as_bytes());
        hasher.update(&version.to_le_bytes());
        hasher.update(&shard_index.to_le_bytes());
        hasher.update(challenged_node.as_bytes());
        let result = hasher.finalize();
        let mut challenge_id = [0u8; 32];
        challenge_id.copy_from_slice(&result);

        Self {
            challenge_id: Hash32(challenge_id),
            model_id,
            version,
            shard_index,
            challenged_node,
            challenger,
            challenge_window,
            status: ChallengeStatus::Pending,
            created_at: 0,
        }
    }

    pub fn is_expired(&self, current_block: u64) -> bool {
        current_block > self.created_at + self.challenge_window as u64
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChallengeStatus {
    Pending,
    Success,
    Failed,
    Expired,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShardProof {
    pub challenge_id: Hash32,
    pub shard_hash: Hash32,
    pub merkle_proof: Vec<u8>,
    pub signature: Vec<u8>,
    pub submitted_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiRoute {
    pub route_id: Hash32,
    pub path: String,
    pub handler_module: String,
    pub handler_method: String,
    pub auth_level: AuthLevel,
    pub enabled: bool,
}

impl ApiRoute {
    pub fn new(path: String, handler_module: String, handler_method: String) -> Self {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(path.as_bytes());
        hasher.update(handler_module.as_bytes());
        hasher.update(handler_method.as_bytes());
        let result = hasher.finalize();
        let mut route_id = [0u8; 32];
        route_id.copy_from_slice(&result);

        Self {
            route_id: Hash32(route_id),
            path,
            handler_module,
            handler_method,
            auth_level: AuthLevel::Public,
            enabled: true,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthLevel {
    Public,
    User,
    Operator,
    Governance,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelayConfig {
    pub relay_address: Address,
    pub supported_endpoints: Vec<String>,
    pub fee_bps: u16,
    pub active: bool,
}

impl Default for RelayConfig {
    fn default() -> Self {
        Self {
            relay_address: Address::from_bytes([0u8; 32]),
            supported_endpoints: Vec::new(),
            fee_bps: 0,
            active: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiRegistry {
    pub routes: Vec<ApiRoute>,
    pub relays: Vec<RelayConfig>,
}

impl ApiRegistry {
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            relays: Vec::new(),
        }
    }

    pub fn register_route(&mut self, route: ApiRoute) {
        if !self.routes.iter().any(|r| r.path == route.path) {
            self.routes.push(route);
        }
    }

    pub fn get_route(&self, path: &str) -> Option<&ApiRoute> {
        self.routes.iter().find(|r| r.path == path && r.enabled)
    }

    pub fn register_relay(&mut self, relay: RelayConfig) {
        if !self
            .relays
            .iter()
            .any(|r| r.relay_address == relay.relay_address)
        {
            self.relays.push(relay);
        }
    }
}

impl Default for ApiRegistry {
    fn default() -> Self {
        Self::new()
    }
}
