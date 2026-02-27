// File: inference_protocol.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Extended Intelligence Fabric Protocol primitives including
// SubmitPromptPayload, SubmitReceiptPayload, pricing modes, relay
// modes, and protocol-specific data structures for inference request
// lifecycle management.

use crate::crypto::Address;
use crate::hash::Hash32;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmitPromptPayload {
    pub model_id: Hash32,
    pub version: u32,
    pub prompt_commitment: Hash32,
    pub max_output_tokens: u32,
    pub pricing_mode: u8,
    pub relay_mode: u8,
    pub deadline_height: u64,
    pub fee_limit: u128,
}

impl SubmitPromptPayload {
    pub fn new(
        model_id: Hash32,
        version: u32,
        prompt_commitment: Hash32,
        max_output_tokens: u32,
        pricing_mode: u8,
        relay_mode: u8,
        deadline_height: u64,
        fee_limit: u128,
    ) -> Self {
        Self {
            model_id,
            version,
            prompt_commitment,
            max_output_tokens,
            pricing_mode,
            relay_mode,
            deadline_height,
            fee_limit,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmitReceiptPayload {
    pub prompt_tx_hash: Hash32,
    pub inference_output: Vec<u8>,
    pub gas_used: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptEntry {
    pub prompt_id: Hash32,
    pub model_id: Hash32,
    pub version: u32,
    pub sender: Address,
    pub prompt_commitment: Hash32,
    pub deadline_height: u64,
    pub escrow_amount: u128,
    pub status: PromptStatus,
    pub relay_mode: u8,
    pub pricing_mode: u8,
    pub max_output_tokens: u32,
    pub created_at: u64,
}

impl PromptEntry {
    pub fn new(
        model_id: Hash32,
        version: u32,
        sender: Address,
        prompt_commitment: Hash32,
        deadline_height: u64,
        escrow_amount: u128,
        relay_mode: u8,
        pricing_mode: u8,
        max_output_tokens: u32,
    ) -> Self {
        let mut hasher = sha2::Sha256::new();
        hasher.update(model_id.as_bytes());
        hasher.update(&version.to_le_bytes());
        hasher.update(sender.as_bytes());
        hasher.update(prompt_commitment.as_bytes());
        let result = hasher.finalize();
        let mut prompt_id = [0u8; 32];
        prompt_id.copy_from_slice(&result);

        Self {
            prompt_id: Hash32(prompt_id),
            model_id,
            version,
            sender,
            prompt_commitment,
            deadline_height,
            escrow_amount,
            status: PromptStatus::Pending,
            relay_mode,
            pricing_mode,
            max_output_tokens,
            created_at: 0,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromptStatus {
    Pending,
    Settled,
    Expired,
    Disputed,
}

impl PromptStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, PromptStatus::Pending)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptCommitment {
    pub commitment: Hash32,
    pub salt: Vec<u8>,
    pub prompt_hash: Hash32,
}

impl PromptCommitment {
    pub fn new(prompt: &[u8], salt: Vec<u8>) -> Self {
        let mut hasher = sha2::Sha256::new();
        hasher.update(prompt);
        hasher.update(&salt);
        let result = hasher.finalize();
        let mut commitment = [0u8; 32];
        commitment.copy_from_slice(&result);

        let mut prompt_hasher = sha2::Sha256::new();
        prompt_hasher.update(prompt);
        let prompt_result = prompt_hasher.finalize();
        let mut prompt_hash = [0u8; 32];
        prompt_hash.copy_from_slice(&prompt_result);

        Self {
            commitment: Hash32(commitment),
            salt,
            prompt_hash: Hash32(prompt_hash),
        }
    }

    pub fn verify(&self, prompt: &[u8]) -> bool {
        let mut hasher = sha2::Sha256::new();
        hasher.update(prompt);
        hasher.update(&self.salt);
        let result = hasher.finalize();
        let computed_commitment: [u8; 32] = result.into();
        computed_commitment == self.commitment.as_bytes()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReceiptEntry {
    pub receipt_id: Hash32,
    pub prompt_id: Hash32,
    pub model_id: Hash32,
    pub operator: Address,
    pub inference_output: Vec<u8>,
    pub gas_used: u64,
    pub fee: u128,
    pub validated: bool,
    pub created_at: u64,
}

impl ReceiptEntry {
    pub fn new(
        prompt_id: Hash32,
        model_id: Hash32,
        operator: Address,
        inference_output: Vec<u8>,
        gas_used: u64,
        fee: u128,
    ) -> Self {
        let mut hasher = sha2::Sha256::new();
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
            validated: false,
            created_at: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RevenueDistribution {
    pub operator_amount: u128,
    pub owner_amount: u128,
    pub shard_provider_amount: u128,
    pub validator_amount: u128,
    pub treasury_amount: u128,
}

impl RevenueDistribution {
    pub fn distribute(total_fee: u128, revenue_split: &super::inference::RevenueSplit) -> Self {
        let operator_amount = (total_fee * revenue_split.operator_basis_points as u128) / 10000;
        let owner_amount = (total_fee * revenue_split.owner_basis_points as u128) / 10000;
        let shard_provider_amount =
            (total_fee * revenue_split.shard_provider_basis_points as u128) / 10000;
        let validator_amount = (total_fee * revenue_split.validator_basis_points as u128) / 10000;
        let treasury_amount = (total_fee * revenue_split.treasury_basis_points as u128) / 10000;

        Self {
            operator_amount,
            owner_amount,
            shard_provider_amount,
            validator_amount,
            treasury_amount,
        }
    }

    pub fn total(&self) -> u128 {
        self.operator_amount
            .saturating_add(self.owner_amount)
            .saturating_add(self.shard_provider_amount)
            .saturating_add(self.validator_amount)
            .saturating_add(self.treasury_amount)
    }
}

use sha2::{Digest, Sha256};
