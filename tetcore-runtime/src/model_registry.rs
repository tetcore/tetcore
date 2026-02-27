use crate::RuntimeError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tetcore_primitives::{Address, Hash32};

pub const REVENUE_SPLIT_BASIS_POINTS: u32 = 10000;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelState {
    Registered,
    Active,
    Paused,
    Deprecated,
    Disabled,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PricingPolicy {
    pub mode: PricingMode,
    pub base_fee: u64,
    pub token_per_char: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PricingMode {
    Owner,
    Market,
    Hybrid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RevenueSplit {
    pub model_owner_bps: u16,
    pub operator_bps: u16,
    pub shard_provider_bps: u16,
    pub validator_bps: u16,
    pub vault_bps: u16,
}

impl Default for RevenueSplit {
    fn default() -> Self {
        Self {
            model_owner_bps: 3000,
            operator_bps: 3000,
            shard_provider_bps: 2000,
            validator_bps: 1000,
            vault_bps: 1000,
        }
    }
}

impl RevenueSplit {
    pub fn is_valid(&self) -> bool {
        let total = self.model_owner_bps as u32
            + self.operator_bps as u32
            + self.shard_provider_bps as u32
            + self.validator_bps as u32
            + self.vault_bps as u32;
        total == REVENUE_SPLIT_BASIS_POINTS
    }
}

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
    pub vault_enabled: bool,
}

pub struct ModelRegistryModule {
    models: HashMap<Hash32, Model>,
    model_versions: HashMap<Hash32, Vec<Hash32>>,
}

impl ModelRegistryModule {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            model_versions: HashMap::new(),
        }
    }

    pub fn register_model(&mut self, model: Model) -> Result<(), RuntimeError> {
        if !model.revenue_split.is_valid() {
            return Err(RuntimeError::InvalidState);
        }

        self.models.insert(model.model_id.clone(), model);
        Ok(())
    }

    pub fn get_model(&self, model_id: &Hash32) -> Option<&Model> {
        self.models.get(model_id)
    }

    pub fn get_model_mut(&mut self, model_id: &Hash32) -> Option<&mut Model> {
        self.models.get_mut(model_id)
    }

    pub fn update_version(
        &mut self,
        model_id: &Hash32,
        new_version: u32,
        new_shard_root: Hash32,
    ) -> Result<(), RuntimeError> {
        let model = self
            .models
            .get_mut(model_id)
            .ok_or(RuntimeError::InvalidState)?;
        model.version = new_version;
        model.shard_root = new_shard_root;
        Ok(())
    }

    pub fn set_model_state(
        &mut self,
        model_id: &Hash32,
        state: ModelState,
    ) -> Result<(), RuntimeError> {
        let model = self
            .models
            .get_mut(model_id)
            .ok_or(RuntimeError::InvalidState)?;
        model.state = state;
        Ok(())
    }

    pub fn update_pricing(
        &mut self,
        model_id: &Hash32,
        policy: PricingPolicy,
    ) -> Result<(), RuntimeError> {
        let model = self
            .models
            .get_mut(model_id)
            .ok_or(RuntimeError::InvalidState)?;
        model.pricing_policy = policy;
        Ok(())
    }

    pub fn update_revenue_split(
        &mut self,
        model_id: &Hash32,
        split: RevenueSplit,
    ) -> Result<(), RuntimeError> {
        if !split.is_valid() {
            return Err(RuntimeError::InvalidState);
        }
        let model = self
            .models
            .get_mut(model_id)
            .ok_or(RuntimeError::InvalidState)?;
        model.revenue_split = split;
        Ok(())
    }

    pub fn transfer_ownership(
        &mut self,
        model_id: &Hash32,
        new_owner: Address,
    ) -> Result<(), RuntimeError> {
        let model = self
            .models
            .get_mut(model_id)
            .ok_or(RuntimeError::InvalidState)?;
        model.owner = new_owner;
        Ok(())
    }

    pub fn all_models(&self) -> &HashMap<Hash32, Model> {
        &self.models
    }

    pub fn is_active(&self, model_id: &Hash32) -> bool {
        self.models
            .get(model_id)
            .map(|m| m.state == ModelState::Active)
            .unwrap_or(false)
    }
}

impl Default for ModelRegistryModule {
    fn default() -> Self {
        Self::new()
    }
}
