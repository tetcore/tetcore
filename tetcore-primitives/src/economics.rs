// File: economics.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Economic primitives for Tetcore including TokenSupply, TokenBalance,
// Transfer, Escrow, Vault, VaultPosition, FeeParameters, and GasSchedule.
// Implements the TNT token system with 18-decimal precision, vault staking,
// and fee management for the deterministic state machine.

use crate::crypto::Address;
use crate::hash::Hash32;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const DECIMALS: u8 = 18;
pub const TOTAL_SUPPLY: u128 = 100_000_000_000_u128 * 10_u128.pow(DECIMALS as u32);

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TokenSupply {
    pub total: u128,
    pub circulating: u128,
    pub escrowed: u128,
    pub vault_staked: u128,
    pub burned: u128,
    pub locked_collateral: u128,
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
        }
    }

    pub fn total_burned(&self) -> u128 {
        self.burned
    }

    pub fn current_supply(&self) -> u128 {
        self.total.saturating_sub(self.burned)
    }

    pub fn verify_invariant(&self) -> bool {
        let accounted = self
            .circulating
            .saturating_add(self.escrowed)
            .saturating_add(self.vault_staked)
            .saturating_add(self.burned)
            .saturating_add(self.locked_collateral);
        accounted == self.total
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenBalance {
    pub address: Address,
    pub balance: u128,
    pub nonce: u64,
    pub locked: u128,
    pub reserved: u128,
}

impl TokenBalance {
    pub fn new(address: Address, balance: u128) -> Self {
        Self {
            address,
            balance,
            nonce: 0,
            locked: 0,
            reserved: 0,
        }
    }

    pub fn available(&self) -> u128 {
        self.balance
            .saturating_sub(self.locked)
            .saturating_sub(self.reserved)
    }

    pub fn can_transfer(&self, amount: u128) -> bool {
        self.available() >= amount
    }

    pub fn add_balance(&mut self, amount: u128) {
        self.balance = self.balance.saturating_add(amount);
    }

    pub fn sub_balance(&mut self, amount: u128) -> bool {
        if self.can_transfer(amount) {
            self.balance = self.balance.saturating_sub(amount);
            true
        } else {
            false
        }
    }

    pub fn inc_nonce(&mut self) {
        self.nonce = self.nonce.saturating_add(1);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transfer {
    pub from: Address,
    pub to: Address,
    pub amount: u128,
    pub nonce: u64,
    pub fee: u128,
}

impl Transfer {
    pub fn new(from: Address, to: Address, amount: u128, nonce: u64, fee: u128) -> Self {
        Self {
            from,
            to,
            amount,
            nonce,
            fee,
        }
    }

    pub fn total_debit(&self) -> u128 {
        self.amount.saturating_add(self.fee)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Escrow {
    pub escrow_id: Hash32,
    pub sender: Address,
    pub recipient: Address,
    pub amount: u128,
    pub release_height: u64,
    pub created_at: u64,
    pub status: EscrowStatus,
}

impl Escrow {
    pub fn new(sender: Address, recipient: Address, amount: u128, release_height: u64) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(sender.as_bytes());
        hasher.update(recipient.as_bytes());
        hasher.update(&amount.to_le_bytes());
        hasher.update(&release_height.to_le_bytes());
        let result = hasher.finalize();
        let mut escrow_id = [0u8; 32];
        escrow_id.copy_from_slice(&result);

        Self {
            escrow_id: Hash32(escrow_id),
            sender,
            recipient,
            amount,
            release_height,
            created_at: 0,
            status: EscrowStatus::Locked,
        }
    }

    pub fn is_releasable(&self, current_height: u64) -> bool {
        self.status == EscrowStatus::Locked && current_height >= self.release_height
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EscrowStatus {
    #[default]
    Locked,
    Released,
    Cancelled,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vault {
    pub vault_id: Hash32,
    pub model_id: Hash32,
    pub owner: Address,
    pub total_staked: u128,
    pub total_shares: u128,
    pub reward_accumulator: u128,
    pub created_at: u64,
    pub active: bool,
}

impl Vault {
    pub fn new(model_id: Hash32, owner: Address) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(model_id.as_bytes());
        hasher.update(owner.as_bytes());
        let result = hasher.finalize();
        let mut vault_id = [0u8; 32];
        vault_id.copy_from_slice(&result);

        Self {
            vault_id: Hash32(vault_id),
            model_id,
            owner,
            total_staked: 0,
            total_shares: 0,
            reward_accumulator: 0,
            created_at: 0,
            active: true,
        }
    }

    pub fn stake(&mut self, amount: u128) -> u128 {
        if self.total_shares == 0 || self.total_staked == 0 {
            let shares = amount;
            self.total_shares = shares;
            self.total_staked = amount;
            shares
        } else {
            let shares = (amount * self.total_shares) / self.total_staked;
            self.total_shares = self.total_shares.saturating_add(shares);
            self.total_staked = self.total_staked.saturating_add(amount);
            shares
        }
    }

    pub fn unstake(&mut self, shares: u128) -> u128 {
        if shares == 0 || self.total_shares == 0 {
            return 0;
        }
        let amount = (shares * self.total_staked) / self.total_shares;
        self.total_shares = self.total_shares.saturating_sub(shares);
        self.total_staked = self.total_staked.saturating_sub(amount);
        amount
    }

    pub fn share_value(&self, shares: u128) -> u128 {
        if self.total_shares == 0 {
            return 0;
        }
        (shares * self.total_staked) / self.total_shares
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VaultPosition {
    pub vault_id: Hash32,
    pub staker: Address,
    pub shares: u128,
    pub pending_rewards: u128,
    pub last_update: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GasSchedule {
    pub gas_storage_byte: u64,
    pub gas_storage_item: u64,
    pub gas_compute_byte: u64,
    pub gas_compute_math: u64,
    pub gas_compute_mem: u64,
    pub gas_call: u64,
    pub gas_create: u64,
    pub gas_invoke: u64,
}

impl Default for GasSchedule {
    fn default() -> Self {
        Self {
            gas_storage_byte: 1,
            gas_storage_item: 5,
            gas_compute_byte: 2,
            gas_compute_math: 3,
            gas_compute_mem: 1,
            gas_call: 10,
            gas_create: 20,
            gas_invoke: 15,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeeParameters {
    pub gas_price_min: u128,
    pub gas_price_max: u128,
    pub fee_multiplier_numerator: u128,
    pub fee_multiplier_denominator: u128,
    pub burn_percentage: u8,
}

impl Default for FeeParameters {
    fn default() -> Self {
        Self {
            gas_price_min: 1,
            gas_price_max: 1_000_000_000,
            fee_multiplier_numerator: 1,
            fee_multiplier_denominator: 1,
            burn_percentage: 0,
        }
    }
}
