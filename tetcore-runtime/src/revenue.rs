// File: revenue.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Revenue module for Tetcore runtime. Manages fee distribution,
// revenue routing to operators, model owners, shard providers,
// validators, and treasury. Implements the revenue split mechanism.

use crate::RuntimeError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tetcore_primitives::{Address, Hash32};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RevenueRoute {
    pub recipient: Address,
    pub basis_points: u16,
    pub recipient_type: RecipientType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecipientType {
    ModelOwner,
    Operator,
    ShardProvider,
    Validator,
    Vault,
    Treasury,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RevenueDistribution {
    pub distribution_id: Hash32,
    pub source: Address,
    pub routes: Vec<RevenueRoute>,
    pub total_amount: u128,
    pub distributed: bool,
    pub block_height: u64,
}

pub struct RevenueModule {
    distributions: HashMap<Hash32, RevenueDistribution>,
    pending_distributions: Vec<RevenueDistribution>,
    distribution_counter: u64,
    treasury_address: Address,
}

impl RevenueModule {
    pub fn new() -> Self {
        Self {
            distributions: HashMap::new(),
            pending_distributions: Vec::new(),
            distribution_counter: 0,
            treasury_address: Address::from_bytes([0u8; 32]),
        }
    }

    pub fn set_treasury(&mut self, address: Address) {
        self.treasury_address = address;
    }

    pub fn create_distribution(
        &mut self,
        source: Address,
        routes: Vec<RevenueRoute>,
        total_amount: u128,
        current_height: u64,
    ) -> Result<Hash32, RuntimeError> {
        let total_bps: u16 = routes.iter().map(|r| r.basis_points).sum();

        if total_bps != 10000 {
            return Err(RuntimeError::InvalidState);
        }

        self.distribution_counter += 1;

        let mut data = Vec::new();
        data.extend_from_slice(&self.distribution_counter.to_le_bytes());
        data.extend_from_slice(source.as_bytes());
        data.extend_from_slice(&total_amount.to_le_bytes());

        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(&data);
        let mut id = [0u8; 32];
        id.copy_from_slice(&hash[..32]);

        let distribution = RevenueDistribution {
            distribution_id: Hash32(id),
            source,
            routes,
            total_amount,
            distributed: false,
            block_height: current_height,
        };

        self.distributions.insert(Hash32(id), distribution.clone());
        self.pending_distributions.push(distribution);

        Ok(Hash32(id))
    }

    pub fn distribute(
        &mut self,
        distribution_id: &Hash32,
        balances: &mut HashMap<Address, u128>,
    ) -> Result<Vec<(Address, u128)>, RuntimeError> {
        let distribution = self
            .distributions
            .get_mut(distribution_id)
            .ok_or(RuntimeError::InvalidState)?;

        if distribution.distributed {
            return Err(RuntimeError::InvalidState);
        }

        let mut results = Vec::new();

        for route in &distribution.routes {
            let amount = (distribution.total_amount * route.basis_points as u128) / 10000;

            let balance = balances.entry(route.recipient).or_insert(0);
            *balance += amount;

            results.push((route.recipient, amount));
        }

        distribution.distributed = true;

        Ok(results)
    }

    pub fn calculate_fee(
        &self,
        prompt_tokens: u64,
        output_tokens: u64,
        base_fee: u64,
        token_fee: u64,
    ) -> u64 {
        let total_tokens = prompt_tokens + output_tokens;
        base_fee + (total_tokens * token_fee)
    }

    pub fn route_revenue(
        &self,
        amount: u128,
        split: &crate::model_registry::RevenueSplit,
        model_owner: Address,
        operator: Address,
        validator: Address,
    ) -> Vec<RevenueRoute> {
        vec![
            RevenueRoute {
                recipient: model_owner,
                basis_points: split.model_owner_bps,
                recipient_type: RecipientType::ModelOwner,
            },
            RevenueRoute {
                recipient: operator,
                basis_points: split.operator_bps,
                recipient_type: RecipientType::Operator,
            },
            RevenueRoute {
                recipient: validator,
                basis_points: split.validator_bps,
                recipient_type: RecipientType::Validator,
            },
            RevenueRoute {
                recipient: self.treasury_address,
                basis_points: (split.shard_provider_bps + split.vault_bps) as u16,
                recipient_type: RecipientType::Treasury,
            },
        ]
    }

    pub fn get_distribution(&self, distribution_id: &Hash32) -> Option<&RevenueDistribution> {
        self.distributions.get(distribution_id)
    }

    pub fn pending_count(&self) -> usize {
        self.pending_distributions.len()
    }

    pub fn all_distributions(&self) -> &HashMap<Hash32, RevenueDistribution> {
        &self.distributions
    }
}

impl Default for RevenueModule {
    fn default() -> Self {
        Self::new()
    }
}
