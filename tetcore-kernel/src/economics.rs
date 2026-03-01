// File: economics.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Economics module implementing token supply, staking, treasury, and fee mechanisms.
// Includes validator staking, inflation control, treasury management, and fee distribution.

use crate::Address;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const DECIMALS: u8 = 18;
pub const TOTAL_SUPPLY: u128 = 100_000_000_000_u128 * 10_u128.pow(DECIMALS as u32);
pub const BLOCKS_PER_YEAR: u64 = 5256000;
pub const DEFAULT_INFLATION_RATE_BPS: u64 = 0;
pub const MAX_INFLATION_RATE_BPS: u64 = 1000;
pub const TREASURY_ADDRESS: [u8; 32] = [0u8; 32];

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InflationState {
    Disabled,
    Enabled,
    Paused,
}

pub struct InflationConfig {
    pub state: InflationState,
    pub rate_bps: u64,
    pub cap: u128,
    pub start_block: u64,
    pub end_block: u64,
    pub treasury_share_bps: u64,
    pub validator_share_bps: u64,
}

impl Default for InflationConfig {
    fn default() -> Self {
        Self {
            state: InflationState::Disabled,
            rate_bps: 0,
            cap: 0,
            start_block: 0,
            end_block: 0,
            treasury_share_bps: 2000,
            validator_share_bps: 8000,
        }
    }
}

impl InflationConfig {
    pub fn new(rate_bps: u64) -> Self {
        Self {
            state: InflationState::Enabled,
            rate_bps: rate_bps.min(MAX_INFLATION_RATE_BPS),
            cap: 0,
            start_block: 0,
            end_block: 0,
            treasury_share_bps: 2000,
            validator_share_bps: 8000,
        }
    }

    pub fn is_active(&self, current_block: u64) -> bool {
        self.state == InflationState::Enabled
            && current_block >= self.start_block
            && (self.end_block == 0 || current_block < self.end_block)
    }

    pub fn compute_mint(&self, current_supply: u128) -> u128 {
        if !matches!(self.state, InflationState::Enabled) {
            return 0;
        }

        let annual_mint = current_supply
            .saturating_mul(self.rate_bps as u128)
            .saturating_div(10000);

        let per_block_mint = annual_mint.saturating_div(BLOCKS_PER_YEAR as u128);

        if self.cap > 0 {
            per_block_mint.min(self.cap)
        } else {
            per_block_mint
        }
    }

    pub fn distribute(&self, amount: u128) -> (u128, u128) {
        let treasury = amount
            .saturating_mul(self.treasury_share_bps as u128)
            .saturating_div(10000);
        let validators = amount.saturating_sub(treasury);
        (treasury, validators)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenSupply {
    pub total: u128,
    pub circulating: u128,
    pub escrowed: u128,
    pub vault_staked: u128,
    pub burned: u128,
    pub locked_collateral: u128,
    pub treasury_balance: u128,
    pub validator_rewards: u128,
}

impl TokenSupply {
    pub fn new() -> Self {
        Self {
            total: TOTAL_SUPPLY,
            circulating: TOTAL_SUPPLY,
            escrowed: 0,
            vault_staked: 0,
            burned: 0,
            locked_collateral: 0,
            treasury_balance: 0,
            validator_rewards: 0,
        }
    }

    pub fn current_supply(&self) -> u128 {
        self.total.saturating_sub(self.burned)
    }

    pub fn verify_invariant(&self) -> bool {
        let accounted = self
            .circulating
            .saturating_add(self.escrowed)
            .saturating_add(self.vault_staked)
            .saturating_add(self.locked_collateral)
            .saturating_add(self.treasury_balance)
            .saturating_add(self.validator_rewards);
        accounted.saturating_add(self.burned) == self.total
    }

    pub fn add_to_circulating(&mut self, amount: u128) {
        self.circulating = self.circulating.saturating_add(amount);
    }

    pub fn remove_from_circulating(&mut self, amount: u128) {
        self.circulating = self.circulating.saturating_sub(amount);
    }

    pub fn add_burned(&mut self, amount: u128) {
        self.burned = self.burned.saturating_add(amount);
    }

    pub fn add_treasury(&mut self, amount: u128) {
        self.treasury_balance = self.treasury_balance.saturating_add(amount);
    }

    pub fn add_validator_rewards(&mut self, amount: u128) {
        self.validator_rewards = self.validator_rewards.saturating_add(amount);
    }
}

impl Default for TokenSupply {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StakerInfo {
    pub staker: Address,
    pub staked: u128,
    pub shares: u128,
    pub pending_rewards: u128,
    pub last_stake_height: u64,
}

impl StakerInfo {
    pub fn new(staker: Address) -> Self {
        Self {
            staker,
            staked: 0,
            shares: 0,
            pending_rewards: 0,
            last_stake_height: 0,
        }
    }

    pub fn add_stake(
        &mut self,
        amount: u128,
        total_staked: &mut u128,
        total_shares: &mut u128,
    ) -> u128 {
        let shares = if *total_shares == 0 || *total_staked == 0 {
            amount
        } else {
            (amount * *total_shares) / *total_staked
        };

        self.shares = self.shares.saturating_add(shares);
        self.staked = self.staked.saturating_add(amount);
        *total_staked = total_staked.saturating_add(amount);
        *total_shares = total_shares.saturating_add(shares);

        shares
    }

    pub fn remove_stake(
        &mut self,
        shares: u128,
        total_staked: &mut u128,
        total_shares: &mut u128,
    ) -> u128 {
        if shares == 0 || *total_shares == 0 {
            return 0;
        }

        let amount = (shares * *total_staked) / *total_shares;
        self.shares = self.shares.saturating_sub(shares);
        self.staked = self.staked.saturating_sub(amount);
        *total_staked = total_staked.saturating_sub(amount);
        *total_shares = total_shares.saturating_sub(shares);

        amount
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorStake {
    pub validator: Address,
    pub own_stake: u128,
    pub delegated_stake: u128,
    pub total_stake: u128,
    pub commission_bps: u16,
    pub pending_rewards: u128,
    pub active: bool,
    pub jailed: bool,
}

impl ValidatorStake {
    pub fn new(validator: Address) -> Self {
        Self {
            validator,
            own_stake: 0,
            delegated_stake: 0,
            total_stake: 0,
            commission_bps: 1000,
            pending_rewards: 0,
            active: false,
            jailed: false,
        }
    }

    pub fn add_stake(&mut self, amount: u128, is_own: bool) {
        if is_own {
            self.own_stake = self.own_stake.saturating_add(amount);
        } else {
            self.delegated_stake = self.delegated_stake.saturating_add(amount);
        }
        self.total_stake = self.own_stake.saturating_add(self.delegated_stake);
    }

    pub fn remove_stake(&mut self, amount: u128, is_own: bool) -> bool {
        if is_own {
            if self.own_stake >= amount {
                self.own_stake = self.own_stake.saturating_sub(amount);
                self.total_stake = self.own_stake.saturating_add(self.delegated_stake);
                true
            } else {
                false
            }
        } else {
            if self.delegated_stake >= amount {
                self.delegated_stake = self.delegated_stake.saturating_sub(amount);
                self.total_stake = self.own_stake.saturating_add(self.delegated_stake);
                true
            } else {
                false
            }
        }
    }

    pub fn distribute_rewards(&mut self, total_reward: u128) -> (u128, u128) {
        let commission = total_reward
            .saturating_mul(self.commission_bps as u128)
            .saturating_div(10000);

        let delegator_reward = total_reward.saturating_sub(commission);

        self.pending_rewards = self.pending_rewards.saturating_add(commission);

        (commission, delegator_reward)
    }

    pub fn weight(&self) -> u128 {
        self.total_stake
    }
}

pub struct StakingModule {
    pub total_staked: u128,
    pub total_shares: u128,
    pub stakers: HashMap<Address, StakerInfo>,
    pub validators: HashMap<Address, ValidatorStake>,
    pub inflation_config: InflationConfig,
    pub token_supply: TokenSupply,
}

impl StakingModule {
    pub fn new() -> Self {
        Self {
            total_staked: 0,
            total_shares: 0,
            stakers: HashMap::new(),
            validators: HashMap::new(),
            inflation_config: InflationConfig::default(),
            token_supply: TokenSupply::new(),
        }
    }

    pub fn stake(
        &mut self,
        staker: Address,
        amount: u128,
        block_height: u64,
    ) -> Result<u128, StakingError> {
        if amount == 0 {
            return Err(StakingError::ZeroAmount);
        }

        self.token_supply.remove_from_circulating(amount);

        let staker_info = self
            .stakers
            .entry(staker)
            .or_insert_with(|| StakerInfo::new(staker));
        let shares = staker_info.add_stake(amount, &mut self.total_staked, &mut self.total_shares);

        staker_info.last_stake_height = block_height;

        Ok(shares)
    }

    pub fn unstake(
        &mut self,
        staker: Address,
        shares: u128,
        block_height: u64,
    ) -> Result<u128, StakingError> {
        if shares == 0 {
            return Err(StakingError::ZeroAmount);
        }

        let staker_info = self
            .stakers
            .get_mut(&staker)
            .ok_or(StakingError::NotStaker)?;

        if staker_info.shares < shares {
            return Err(StakingError::InsufficientShares);
        }

        let amount =
            staker_info.remove_stake(shares, &mut self.total_staked, &mut self.total_shares);

        self.token_supply.add_to_circulating(amount);

        if staker_info.shares == 0 {
            self.stakers.remove(&staker);
        }

        Ok(amount)
    }

    pub fn claim_rewards(&mut self, staker: Address) -> Result<u128, StakingError> {
        let staker_info = self
            .stakers
            .get_mut(&staker)
            .ok_or(StakingError::NotStaker)?;

        let rewards = staker_info.pending_rewards;
        if rewards == 0 {
            return Ok(0);
        }

        staker_info.pending_rewards = 0;
        staker_info.staked = staker_info.staked.saturating_add(rewards);

        self.token_supply.add_to_circulating(rewards);

        Ok(rewards)
    }

    pub fn register_validator(
        &mut self,
        validator: Address,
        commission_bps: u16,
    ) -> Result<(), StakingError> {
        if commission_bps > 10000 {
            return Err(StakingError::InvalidCommission);
        }

        if self.validators.contains_key(&validator) {
            return Err(StakingError::AlreadyRegistered);
        }

        let mut stake = ValidatorStake::new(validator);
        stake.commission_bps = commission_bps;
        stake.active = true;

        self.validators.insert(validator, stake);
        Ok(())
    }

    pub fn validator_stake(
        &mut self,
        validator: Address,
        amount: u128,
    ) -> Result<(), StakingError> {
        let stake = self
            .validators
            .get_mut(&validator)
            .ok_or(StakingError::UnknownValidator)?;

        stake.add_stake(amount, true);

        self.total_staked = self.total_staked.saturating_add(amount);

        Ok(())
    }

    pub fn delegate(
        &mut self,
        delegator: Address,
        validator: Address,
        amount: u128,
    ) -> Result<(), StakingError> {
        if !self.validators.contains_key(&validator) {
            return Err(StakingError::UnknownValidator);
        }

        self.token_supply.remove_from_circulating(amount);

        let stake = self.validators.get_mut(&validator).unwrap();
        stake.add_stake(amount, false);

        self.total_staked = self.total_staked.saturating_add(amount);

        let delegator_info = self
            .stakers
            .entry(delegator)
            .or_insert_with(|| StakerInfo::new(delegator));
        let shares =
            delegator_info.add_stake(amount, &mut self.total_staked, &mut self.total_shares);

        let _ = shares;

        Ok(())
    }

    pub fn distribute_block_rewards(&mut self, reward: u128) {
        if reward == 0 {
            return;
        }

        let mut remaining = reward;

        for (validator_addr, validator) in self.validators.iter_mut() {
            if !validator.active || validator.jailed {
                continue;
            }

            let stake_ratio = if self.total_staked > 0 {
                (validator.total_stake as u128 * 10000) / self.total_staked
            } else {
                0
            };

            let validator_reward = reward.saturating_mul(stake_ratio).saturating_div(10000);

            let (commission, _) = validator.distribute_rewards(validator_reward);
            remaining = remaining.saturating_sub(validator_reward);

            let _ = commission;
        }

        self.token_supply
            .add_validator_rewards(reward.saturating_sub(remaining));
    }

    pub fn process_inflation(&mut self, block_height: u64) -> (u128, u128) {
        if !self.inflation_config.is_active(block_height) {
            return (0, 0);
        }

        let current_supply = self.token_supply.current_supply();
        let mint_amount = self.inflation_config.compute_mint(current_supply);

        if mint_amount == 0 {
            return (0, 0);
        }

        let (treasury_amount, validator_amount) = self.inflation_config.distribute(mint_amount);

        self.token_supply.total = self.token_supply.total.saturating_add(mint_amount);
        self.token_supply.add_treasury(treasury_amount);
        self.token_supply.add_validator_rewards(validator_amount);

        self.total_staked = self.total_staked.saturating_add(validator_amount);

        (treasury_amount, validator_amount)
    }

    pub fn get_staker_info(&self, staker: &Address) -> Option<&StakerInfo> {
        self.stakers.get(staker)
    }

    pub fn get_validator_info(&self, validator: &Address) -> Option<&ValidatorStake> {
        self.validators.get(validator)
    }

    pub fn top_validators(&self, limit: usize) -> Vec<(Address, u128)> {
        let mut validators: Vec<_> = self
            .validators
            .iter()
            .filter(|(_, v)| v.active && !v.jailed)
            .map(|(addr, v)| (*addr, v.total_stake))
            .collect();

        validators.sort_by(|a, b| b.1.cmp(&a.1));
        validators.truncate(limit);
        validators
    }

    pub fn enable_inflation(&mut self, rate_bps: u64, start_block: u64, end_block: u64) {
        self.inflation_config.state = InflationState::Enabled;
        self.inflation_config.rate_bps = rate_bps.min(MAX_INFLATION_RATE_BPS);
        self.inflation_config.start_block = start_block;
        self.inflation_config.end_block = end_block;
    }

    pub fn disable_inflation(&mut self) {
        self.inflation_config.state = InflationState::Disabled;
    }
}

impl Default for StakingModule {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum StakingError {
    ZeroAmount,
    InsufficientShares,
    NotStaker,
    AlreadyRegistered,
    UnknownValidator,
    InvalidCommission,
    InsufficientBalance,
}

pub struct Treasury {
    pub balance: u128,
    pub spent: u128,
    pub proposal_count: u64,
    pub spend_limit_per_proposal: u128,
}

impl Treasury {
    pub fn new() -> Self {
        Self {
            balance: 0,
            spent: 0,
            proposal_count: 0,
            spend_limit_per_proposal: TOTAL_SUPPLY / 1000,
        }
    }

    pub fn deposit(&mut self, amount: u128) {
        self.balance = self.balance.saturating_add(amount);
    }

    pub fn spend(&mut self, amount: u128) -> Result<(), TreasuryError> {
        if amount > self.spend_limit_per_proposal {
            return Err(TreasuryError::ExceedsLimit);
        }

        if self.balance < amount {
            return Err(TreasuryError::InsufficientFunds);
        }

        self.balance = self.balance.saturating_sub(amount);
        self.spent = self.spent.saturating_add(amount);
        self.proposal_count += 1;

        Ok(())
    }

    pub fn burn_unspent(&mut self) {
        self.balance = 0;
    }
}

impl Default for Treasury {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum TreasuryError {
    InsufficientFunds,
    ExceedsLimit,
    InvalidProposal,
}

pub struct FeeModule {
    pub base_fee: u128,
    pub min_gas_price: u128,
    pub max_gas_price: u128,
    pub burn_percentage: u8,
    pub treasury_share: u8,
    pub validator_share: u8,
    pub congestion_multiplier: u128,
    pub target_utilization: u64,
}

impl Default for FeeModule {
    fn default() -> Self {
        Self {
            base_fee: 1000,
            min_gas_price: 1,
            max_gas_price: 1_000_000_000,
            burn_percentage: 0,
            treasury_share: 10,
            validator_share: 90,
            congestion_multiplier: 1000,
            target_utilization: 60,
        }
    }
}

impl FeeModule {
    pub fn compute_fee(&self, gas_limit: u64) -> u128 {
        let congestion_price = (self.base_fee as u128)
            .saturating_mul(self.congestion_multiplier)
            .saturating_div(1000);

        let gas_price = self
            .min_gas_price
            .max(self.max_gas_price.min(congestion_price));

        (gas_limit as u128) * gas_price
    }

    pub fn distribute_fee(&self, amount: u128) -> (u128, u128, u128) {
        let burn = amount
            .saturating_mul(self.burn_percentage as u128)
            .saturating_div(100);
        let remaining = amount.saturating_sub(burn);

        let treasury = remaining
            .saturating_mul(self.treasury_share as u128)
            .saturating_div(100);
        let validators = remaining.saturating_sub(treasury);

        (burn, treasury, validators)
    }

    pub fn set_congestion(&mut self, utilization: u64) {
        if utilization > self.target_utilization {
            let excess = utilization - self.target_utilization;
            self.congestion_multiplier = 1000 + excess.min(5000) as u128;
        } else {
            self.congestion_multiplier = 1000;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_address(i: u8) -> Address {
        let mut bytes = [0u8; 32];
        bytes[31] = i;
        Address(bytes)
    }

    #[test]
    fn test_token_supply_invariant() {
        let supply = TokenSupply::new();
        assert!(supply.verify_invariant());
    }

    #[test]
    fn test_staking_basic() {
        let mut staking = StakingModule::new();
        let staker = create_address(1);

        staking.token_supply.circulating = 1000;
        let shares = staking.stake(staker, 100, 1).unwrap();

        assert_eq!(shares, 100);
        assert_eq!(staking.total_staked, 100);
    }

    #[test]
    fn test_validator_registration() {
        let mut staking = StakingModule::new();
        let validator = create_address(1);

        staking.register_validator(validator, 1000).unwrap();

        let info = staking.get_validator_info(&validator).unwrap();
        assert!(info.active);
        assert_eq!(info.commission_bps, 1000);
    }

    #[test]
    fn test_inflation_disabled_by_default() {
        let config = InflationConfig::default();
        assert_eq!(config.state, InflationState::Disabled);
        assert_eq!(config.compute_mint(1000), 0);
    }

    #[test]
    fn test_fee_distribution() {
        let fee_module = FeeModule::default();
        let (burn, treasury, validators) = fee_module.distribute_fee(1000);

        assert_eq!(burn, 0);
        assert_eq!(treasury, 100);
        assert_eq!(validators, 900);
    }

    #[test]
    fn test_treasury_spend() {
        let mut treasury = Treasury::new();
        treasury.deposit(1000);

        treasury.spend(100).unwrap();
        assert_eq!(treasury.balance, 900);
    }

    #[test]
    fn test_treasury_spend_limit() {
        let mut treasury = Treasury::new();
        treasury.deposit(TOTAL_SUPPLY);

        let result = treasury.spend(treasury.spend_limit_per_proposal + 1);
        assert!(result.is_err());
    }
}
