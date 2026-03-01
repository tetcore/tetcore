// File: ifp.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Intelligence Fabric Protocol (IFP) implementation for Tetcore.
// Handles model registration, prompt escrow, receipt validation,
// revenue distribution, and inference lifecycle management.

use crate::{Address, Hash32};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub const DEFAULT_PROMPT_TIMEOUT_BLOCKS: u64 = 100;
pub const MAX_OUTPUT_TOKENS: u32 = 8192;
pub const MIN_ESCROW_AMOUNT: u128 = 1000;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelState {
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

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PricingMode {
    Owner,
    Market,
    Hybrid,
}

impl PricingMode {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(PricingMode::Owner),
            1 => Some(PricingMode::Market),
            2 => Some(PricingMode::Hybrid),
            _ => None,
        }
    }

    pub fn to_u8(self) -> u8 {
        match self {
            PricingMode::Owner => 0,
            PricingMode::Market => 1,
            PricingMode::Hybrid => 2,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelayMode {
    Direct,
    AnyOperator,
    PreferredOperator,
}

impl RelayMode {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(RelayMode::Direct),
            1 => Some(RelayMode::AnyOperator),
            2 => Some(RelayMode::PreferredOperator),
            _ => None,
        }
    }

    pub fn to_u8(self) -> u8 {
        match self {
            RelayMode::Direct => 0,
            RelayMode::AnyOperator => 1,
            RelayMode::PreferredOperator => 2,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromptStatus {
    Pending,
    Settled,
    Expired,
    Disputed,
    Cancelled,
}

impl PromptStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, PromptStatus::Pending)
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

impl PricingPolicy {
    pub fn compute_price(&self, input_tokens: u32, output_tokens: u32) -> u128 {
        let base = self.base_price;
        let token_cost = (input_tokens as u128)
            .saturating_add(output_tokens as u128)
            .saturating_mul(self.per_token_price);
        let complexity = (self.complexity_multiplier as u128) * token_cost / 100;
        base.saturating_add(complexity)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RevenueSplit {
    pub operator_bp: u16,
    pub owner_bp: u16,
    pub shard_provider_bp: u16,
    pub validator_bp: u16,
    pub treasury_bp: u16,
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
            operator_bp: operator,
            owner_bp: owner,
            shard_provider_bp: shard_provider,
            validator_bp: validator,
            treasury_bp: treasury,
        })
    }

    pub fn default_ifp() -> Self {
        Self {
            operator_bp: 7000,
            owner_bp: 2000,
            shard_provider_bp: 500,
            validator_bp: 400,
            treasury_bp: 100,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelInfo {
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
    pub total_prompts: u64,
    pub total_revenue: u128,
}

impl ModelInfo {
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
            total_prompts: 0,
            total_revenue: 0,
        }
    }

    pub fn update_version(&mut self, shard_root: Hash32, shard_count: u32) {
        self.version += 1;
        self.shard_root = shard_root;
        self.shard_count = shard_count;
        self.updated_at = 0;
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
        let mut hasher = Sha256::new();
        hasher.update(prompt);
        hasher.update(&salt);
        let result = hasher.finalize();
        let mut commitment = [0u8; 32];
        commitment.copy_from_slice(&result);

        let mut prompt_hasher = Sha256::new();
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
        let mut hasher = Sha256::new();
        hasher.update(prompt);
        hasher.update(&self.salt);
        let result = hasher.finalize();
        let computed: [u8; 32] = result.into();
        computed == *self.commitment.as_bytes()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptEntry {
    pub prompt_id: Hash32,
    pub model_id: Hash32,
    pub version: u32,
    pub sender: Address,
    pub commitment: PromptCommitment,
    pub deadline_height: u64,
    pub escrow_amount: u128,
    pub status: PromptStatus,
    pub relay_mode: RelayMode,
    pub pricing_mode: PricingMode,
    pub max_output_tokens: u32,
    pub operator: Option<Address>,
    pub created_at: u64,
}

impl PromptEntry {
    pub fn new(
        model_id: Hash32,
        version: u32,
        sender: Address,
        commitment: PromptCommitment,
        deadline_height: u64,
        escrow_amount: u128,
        relay_mode: RelayMode,
        pricing_mode: PricingMode,
        max_output_tokens: u32,
    ) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(model_id.as_bytes());
        hasher.update(&version.to_le_bytes());
        hasher.update(sender.as_bytes());
        hasher.update(commitment.commitment.as_bytes());
        let result = hasher.finalize();
        let mut prompt_id = [0u8; 32];
        prompt_id.copy_from_slice(&result);

        Self {
            prompt_id: Hash32(prompt_id),
            model_id,
            version,
            sender,
            commitment,
            deadline_height,
            escrow_amount,
            status: PromptStatus::Pending,
            relay_mode,
            pricing_mode,
            max_output_tokens,
            operator: None,
            created_at: 0,
        }
    }

    pub fn accept(&mut self, operator: Address) {
        self.operator = Some(operator);
    }

    pub fn settle(&mut self) {
        self.status = PromptStatus::Settled;
    }

    pub fn expire(&mut self) {
        self.status = PromptStatus::Expired;
    }

    pub fn dispute(&mut self) {
        self.status = PromptStatus::Disputed;
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
    pub fn distribute(total_fee: u128, split: &RevenueSplit) -> Self {
        let operator_amount = (total_fee * split.operator_bp as u128) / 10000;
        let owner_amount = (total_fee * split.owner_bp as u128) / 10000;
        let shard_provider_amount = (total_fee * split.shard_provider_bp as u128) / 10000;
        let validator_amount = (total_fee * split.validator_bp as u128) / 10000;
        let treasury_amount = (total_fee * split.treasury_bp as u128) / 10000;

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

pub struct InferenceModule {
    models: HashMap<Hash32, ModelInfo>,
    prompts: HashMap<Hash32, PromptEntry>,
    receipts: HashMap<Hash32, ReceiptEntry>,
    operators: HashMap<Address, OperatorInfo>,
    pub treasury_balance: u128,
    pub total_escrowed: u128,
    pub total_revenue: u128,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperatorInfo {
    pub address: Address,
    pub stake: u128,
    pub is_active: bool,
    pub prompts_handled: u64,
    pub total_revenue: u128,
    pub reputation: u32,
}

impl OperatorInfo {
    pub fn new(address: Address, stake: u128) -> Self {
        Self {
            address,
            stake,
            is_active: true,
            prompts_handled: 0,
            total_revenue: 0,
            reputation: 100,
        }
    }

    pub fn record_prompt(&mut self, revenue: u128) {
        self.prompts_handled += 1;
        self.total_revenue = self.total_revenue.saturating_add(revenue);
    }

    pub fn update_reputation(&mut self, rating: i32) {
        let new_rep = (self.reputation as i32 + rating).clamp(0, 100);
        self.reputation = new_rep as u32;
    }
}

impl InferenceModule {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            prompts: HashMap::new(),
            receipts: HashMap::new(),
            operators: HashMap::new(),
            treasury_balance: 0,
            total_escrowed: 0,
            total_revenue: 0,
        }
    }

    pub fn register_model(
        &mut self,
        owner: Address,
        shard_root: Hash32,
        shard_count: u32,
        pricing_policy: PricingPolicy,
        revenue_split: RevenueSplit,
    ) -> Result<ModelInfo, IfpError> {
        if revenue_split
            .operator_bp
            .checked_add(revenue_split.owner_bp)
            .and_then(|s| s.checked_add(revenue_split.shard_provider_bp))
            .and_then(|s| s.checked_add(revenue_split.validator_bp))
            .and_then(|s| s.checked_add(revenue_split.treasury_bp))
            != Some(10000)
        {
            return Err(IfpError::InvalidRevenueSplit);
        }

        let model = ModelInfo::new(
            owner,
            shard_root,
            shard_count,
            pricing_policy,
            revenue_split,
        );

        let model_id = model.model_id;
        self.models.insert(model_id, model.clone());

        Ok(model)
    }

    pub fn update_model(
        &mut self,
        model_id: &Hash32,
        shard_root: Hash32,
        shard_count: u32,
    ) -> Result<ModelInfo, IfpError> {
        let model = self
            .models
            .get_mut(model_id)
            .ok_or(IfpError::ModelNotFound)?;
        model.update_version(shard_root, shard_count);
        Ok(model.clone())
    }

    pub fn activate_model(&mut self, model_id: &Hash32) -> Result<ModelInfo, IfpError> {
        let model = self
            .models
            .get_mut(model_id)
            .ok_or(IfpError::ModelNotFound)?;
        model.state = ModelState::Active;
        model.updated_at = 0;
        Ok(model.clone())
    }

    pub fn pause_model(&mut self, model_id: &Hash32) -> Result<ModelInfo, IfpError> {
        let model = self
            .models
            .get_mut(model_id)
            .ok_or(IfpError::ModelNotFound)?;
        model.state = ModelState::Paused;
        Ok(model.clone())
    }

    pub fn get_model(&self, model_id: &Hash32) -> Option<&ModelInfo> {
        self.models.get(model_id)
    }

    pub fn register_operator(
        &mut self,
        address: Address,
        stake: u128,
    ) -> Result<OperatorInfo, IfpError> {
        if stake < MIN_ESCROW_AMOUNT {
            return Err(IfpError::InsufficientStake);
        }

        let operator = OperatorInfo::new(address, stake);
        self.operators.insert(address, operator.clone());
        Ok(operator)
    }

    pub fn get_operator(&self, address: &Address) -> Option<&OperatorInfo> {
        self.operators.get(address)
    }

    pub fn submit_prompt(
        &mut self,
        model_id: Hash32,
        version: u32,
        sender: Address,
        commitment: PromptCommitment,
        deadline_height: u64,
        escrow_amount: u128,
        relay_mode: RelayMode,
        pricing_mode: PricingMode,
        max_output_tokens: u32,
    ) -> Result<PromptEntry, IfpError> {
        let model = self.models.get(&model_id).ok_or(IfpError::ModelNotFound)?;

        if !model.state.can_receive_prompts() {
            return Err(IfpError::ModelNotActive);
        }

        if escrow_amount < MIN_ESCROW_AMOUNT {
            return Err(IfpError::InsufficientEscrow);
        }

        if max_output_tokens > MAX_OUTPUT_TOKENS {
            return Err(IfpError::OutputTooLarge);
        }

        let prompt = PromptEntry::new(
            model_id,
            version,
            sender,
            commitment,
            deadline_height,
            escrow_amount,
            relay_mode,
            pricing_mode,
            max_output_tokens,
        );

        let prompt_id = prompt.prompt_id;
        self.prompts.insert(prompt_id, prompt.clone());
        self.total_escrowed = self.total_escrowed.saturating_add(escrow_amount);

        Ok(prompt)
    }

    pub fn accept_prompt(&mut self, prompt_id: &Hash32, operator: Address) -> Result<(), IfpError> {
        let prompt = self
            .prompts
            .get_mut(prompt_id)
            .ok_or(IfpError::PromptNotFound)?;

        if !prompt.status.is_active() {
            return Err(IfpError::PromptNotActive);
        }

        if !self.operators.contains_key(&operator) {
            return Err(IfpError::OperatorNotRegistered);
        }

        prompt.accept(operator);
        Ok(())
    }

    pub fn submit_receipt(
        &mut self,
        prompt_id: Hash32,
        operator: Address,
        inference_output: Vec<u8>,
        gas_used: u64,
    ) -> Result<ReceiptEntry, IfpError> {
        let prompt = self
            .prompts
            .get(&prompt_id)
            .ok_or(IfpError::PromptNotFound)?;

        if !prompt.status.is_active() {
            return Err(IfpError::PromptNotActive);
        }

        let model = self
            .models
            .get(&prompt.model_id)
            .ok_or(IfpError::ModelNotFound)?;
        let price = model
            .pricing_policy
            .compute_price(prompt.max_output_tokens / 2, prompt.max_output_tokens);
        let fee = price.max(prompt.escrow_amount);

        let receipt = ReceiptEntry::new(
            prompt_id,
            prompt.model_id,
            operator,
            inference_output,
            gas_used,
            fee,
        );

        let receipt_id = receipt.receipt_id;
        self.receipts.insert(receipt_id, receipt.clone());

        Ok(receipt)
    }

    pub fn validate_receipt(
        &mut self,
        receipt_id: &Hash32,
        current_height: u64,
    ) -> Result<RevenueDistribution, IfpError> {
        let receipt = self
            .receipts
            .get_mut(receipt_id)
            .ok_or(IfpError::ReceiptNotFound)?;

        if receipt.validated {
            return Err(IfpError::AlreadyValidated);
        }

        let prompt = self
            .prompts
            .get(&receipt.prompt_id)
            .ok_or(IfpError::PromptNotFound)?;

        if current_height > prompt.deadline_height {
            return Err(IfpError::PromptExpired);
        }

        let model = self
            .models
            .get(&receipt.model_id)
            .ok_or(IfpError::ModelNotFound)?;
        let revenue_split = &model.revenue_split;

        let distribution = RevenueDistribution::distribute(receipt.fee, revenue_split);

        self.total_escrowed = self.total_escrowed.saturating_sub(receipt.fee);
        self.total_revenue = self.total_revenue.saturating_add(receipt.fee);

        if let Some(p) = self.prompts.get_mut(&receipt.prompt_id) {
            p.settle();
        }

        if let Some(m) = self.models.get_mut(&receipt.model_id) {
            m.total_prompts += 1;
            m.total_revenue = m.total_revenue.saturating_add(receipt.fee);
        }

        self.treasury_balance = self
            .treasury_balance
            .saturating_add(distribution.treasury_amount);

        if let Some(operator_info) = self.operators.get_mut(&receipt.operator) {
            operator_info.record_prompt(distribution.operator_amount);
        }

        receipt.validated = true;

        Ok(distribution)
    }

    pub fn expire_prompts(&mut self, current_height: u64) -> Vec<Hash32> {
        let mut expired = Vec::new();

        for (prompt_id, prompt) in self.prompts.iter_mut() {
            if prompt.status == PromptStatus::Pending && current_height > prompt.deadline_height {
                prompt.expire();
                self.total_escrowed = self.total_escrowed.saturating_sub(prompt.escrow_amount);
                expired.push(*prompt_id);
            }
        }

        expired
    }

    pub fn get_prompt(&self, prompt_id: &Hash32) -> Option<&PromptEntry> {
        self.prompts.get(prompt_id)
    }

    pub fn get_receipt(&self, receipt_id: &Hash32) -> Option<&ReceiptEntry> {
        self.receipts.get(receipt_id)
    }

    pub fn active_operators(&self) -> Vec<&OperatorInfo> {
        self.operators.values().filter(|op| op.is_active).collect()
    }

    pub fn model_count(&self) -> usize {
        self.models.len()
    }

    pub fn pending_prompt_count(&self) -> usize {
        self.prompts
            .values()
            .filter(|p| p.status == PromptStatus::Pending)
            .count()
    }
}

impl Default for InferenceModule {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum IfpError {
    ModelNotFound,
    ModelNotActive,
    PromptNotFound,
    PromptNotActive,
    PromptExpired,
    ReceiptNotFound,
    AlreadyValidated,
    OperatorNotRegistered,
    InsufficientStake,
    InsufficientEscrow,
    OutputTooLarge,
    InvalidRevenueSplit,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_address(i: u8) -> Address {
        let mut bytes = [0u8; 32];
        bytes[31] = i;
        Address(bytes)
    }

    fn create_hash(i: u8) -> Hash32 {
        let mut bytes = [0u8; 32];
        bytes[31] = i;
        Hash32(bytes)
    }

    #[test]
    fn test_model_registration() {
        let mut module = InferenceModule::new();
        let owner = create_address(1);
        let shard_root = create_hash(1);

        let result = module.register_model(
            owner,
            shard_root,
            4,
            PricingPolicy::default(),
            RevenueSplit::default_ifp(),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_prompt_submission() {
        let mut module = InferenceModule::new();
        let owner = create_address(1);
        let sender = create_address(2);
        let shard_root = create_hash(1);

        let model = module
            .register_model(
                owner,
                shard_root,
                4,
                PricingPolicy::default(),
                RevenueSplit::default_ifp(),
            )
            .unwrap();

        module.activate_model(&model.model_id).unwrap();

        let commitment = PromptCommitment::new(b"test prompt", vec![1, 2, 3]);

        let result = module.submit_prompt(
            model.model_id,
            1,
            sender,
            commitment,
            100,
            10000,
            RelayMode::Direct,
            PricingMode::Owner,
            1000,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_revenue_distribution() {
        let split = RevenueSplit::default_ifp();
        let distribution = RevenueDistribution::distribute(10000, &split);

        assert_eq!(distribution.operator_amount, 7000);
        assert_eq!(distribution.owner_amount, 2000);
        assert_eq!(distribution.shard_provider_amount, 500);
        assert_eq!(distribution.validator_amount, 400);
        assert_eq!(distribution.treasury_amount, 100);
    }

    #[test]
    fn test_pricing_computation() {
        let policy = PricingPolicy {
            mode: PricingMode::Owner,
            base_price: 1000,
            per_token_price: 10,
            complexity_multiplier: 100,
            latency_multiplier: 100,
        };

        let price = policy.compute_price(100, 200);
        assert!(price >= 1000);
    }
}
