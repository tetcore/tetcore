// File: inference.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Intelligence Fabric Protocol (IFP) primitives for Tetcore including
// Model, ModelState, PricingPolicy, PricingMode, RevenueSplit, Prompt,
// Receipt, ShardCommitment, ShardMetadata, InferenceRequest, and
// InferenceResponse. Supports AI model registration, versioning, and
// inference request/response handling.

use crate::crypto::Address;
use crate::hash::Hash32;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Model {
    pub model_id: Hash32,
    pub owner: Address,
    pub version: u32,
    pub shard_root: Hash32,
    pub shard_count: u32,
    pub state: ModelState,
    pub pricing_policy: PricingPolicy,
    pub revenue_split: RevenueSplit,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Model {
    pub fn new(
        owner: Address,
        shard_root: Hash32,
        shard_count: u32,
        pricing_policy: PricingPolicy,
        revenue_split: RevenueSplit,
    ) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(owner.as_bytes());
        hasher.update(&1u32.to_le_bytes());
        hasher.update(shard_root.as_bytes());
        let result = hasher.finalize();
        let mut model_id = [0u8; 32];
        model_id.copy_from_slice(&result);

        Self {
            model_id: Hash32(model_id),
            owner,
            version: 1,
            shard_root,
            shard_count,
            state: ModelState::Registered,
            pricing_policy,
            revenue_split,
            created_at: 0,
            updated_at: 0,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ModelState {
    #[default]
    Registered,
    Active,
    Paused,
    Deprecated,
    Disabled,
}

impl ModelState {
    pub fn is_active(&self) -> bool {
        matches!(self, ModelState::Active)
    }

    pub fn can_receive_prompts(&self) -> bool {
        matches!(
            self,
            ModelState::Active | ModelState::Paused | ModelState::Deprecated
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PricingPolicy {
    pub mode: PricingMode,
    pub base_price: u128,
    pub per_token_price: u128,
    pub complexity_multiplier: u32,
    pub latency_multiplier: u32,
}

impl Default for PricingPolicy {
    fn default() -> Self {
        Self {
            mode: PricingMode::Owner,
            base_price: 0,
            per_token_price: 0,
            complexity_multiplier: 100,
            latency_multiplier: 100,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PricingMode {
    Owner,
    Market,
    Hybrid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RevenueSplit {
    pub operator_basis_points: u16,
    pub owner_basis_points: u16,
    pub shard_provider_basis_points: u16,
    pub validator_basis_points: u16,
    pub treasury_basis_points: u16,
}

impl RevenueSplit {
    pub fn new(
        operator: u16,
        owner: u16,
        shard_provider: u16,
        validator: u16,
        treasury: u16,
    ) -> Result<Self, &'static str> {
        let total = operator
            .checked_add(owner)
            .and_then(|s| s.checked_add(shard_provider))
            .and_then(|s| s.checked_add(validator))
            .and_then(|s| s.checked_add(treasury));

        if total != Some(10000) {
            return Err("Revenue split must sum to 10000 basis points");
        }

        Ok(Self {
            operator_basis_points: operator,
            owner_basis_points: owner,
            shard_provider_basis_points: shard_provider,
            validator_basis_points: validator,
            treasury_basis_points: treasury,
        })
    }

    pub fn default_model() -> Self {
        Self {
            operator_basis_points: 7000,
            owner_basis_points: 2000,
            shard_provider_basis_points: 500,
            validator_basis_points: 400,
            treasury_basis_points: 100,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prompt {
    pub prompt_id: Hash32,
    pub model_id: Hash32,
    pub version: u32,
    pub sender: Address,
    pub prompt_data: Vec<u8>,
    pub max_gas: u64,
    pub escrow_amount: u128,
    pub created_at: u64,
    pub expires_at: u64,
}

impl Prompt {
    pub fn new(
        model_id: Hash32,
        version: u32,
        sender: Address,
        prompt_data: Vec<u8>,
        max_gas: u64,
        escrow_amount: u128,
    ) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(model_id.as_bytes());
        hasher.update(&version.to_le_bytes());
        hasher.update(sender.as_bytes());
        hasher.update(&prompt_data);
        hasher.update(&escrow_amount.to_le_bytes());
        let result = hasher.finalize();
        let mut prompt_id = [0u8; 32];
        prompt_id.copy_from_slice(&result);

        Self {
            prompt_id: Hash32(prompt_id),
            model_id,
            version,
            sender,
            prompt_data,
            max_gas,
            escrow_amount,
            created_at: 0,
            expires_at: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub receipt_id: Hash32,
    pub prompt_id: Hash32,
    pub model_id: Hash32,
    pub operator: Address,
    pub inference_output: Vec<u8>,
    pub gas_used: u64,
    pub fee: u128,
    pub created_at: u64,
    pub validated: bool,
}

impl Receipt {
    pub fn new(
        prompt_id: Hash32,
        model_id: Hash32,
        operator: Address,
        inference_output: Vec<u8>,
        gas_used: u64,
        fee: u128,
    ) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(prompt_id.as_bytes());
        hasher.update(operator.as_bytes());
        hasher.update(&inference_output);
        let result = hasher.finalize();
        let mut receipt_id = [0u8; 32];
        receipt_id.copy_from_slice(&result);

        Self {
            receipt_id: Hash32(receipt_id),
            prompt_id,
            model_id,
            operator,
            inference_output,
            gas_used,
            fee,
            created_at: 0,
            validated: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShardCommitment {
    pub shard_index: u32,
    pub shard_hash: Hash32,
    pub merkle_proof: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShardMetadata {
    pub model_id: Hash32,
    pub shard_count: u32,
    pub shard_root: Hash32,
    pub shard_size_bytes: u64,
    pub created_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub model_id: Hash32,
    pub version: u32,
    pub prompt: Vec<u8>,
    pub max_gas: u64,
    pub callback: Option<Address>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub request_id: Hash32,
    pub output: Vec<u8>,
    pub gas_used: u64,
    pub fee: u128,
}
