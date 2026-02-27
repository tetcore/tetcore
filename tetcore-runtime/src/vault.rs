use crate::RuntimeError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tetcore_primitives::{Address, Hash32};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum VaultState {
    Active,
    Paused,
    Closed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vault {
    pub vault_id: Hash32,
    pub model_id: Hash32,
    pub owner: Address,
    pub state: VaultState,
    pub total_staked: u128,
    pub share_token_supply: u128,
    pub reward_accumulator: u128,
    pub created_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VaultShare {
    pub holder: Address,
    pub vault_id: Hash32,
    pub staked_amount: u128,
    pub share_count: u128,
    pub last_reward_update: u64,
}

pub struct VaultModule {
    vaults: HashMap<Hash32, Vault>,
    shares: HashMap<Address, Vec<VaultShare>>,
    vault_counter: u64,
}

impl VaultModule {
    pub fn new() -> Self {
        Self {
            vaults: HashMap::new(),
            shares: HashMap::new(),
            vault_counter: 0,
        }
    }

    pub fn create_vault(
        &mut self,
        model_id: Hash32,
        owner: Address,
        initial_stake: u128,
        current_height: u64,
    ) -> Result<Hash32, RuntimeError> {
        self.vault_counter += 1;

        let mut data = Vec::new();
        data.extend_from_slice(&self.vault_counter.to_le_bytes());
        data.extend_from_slice(model_id.as_bytes());
        data.extend_from_slice(owner.as_bytes());

        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(&data);
        let mut id = [0u8; 32];
        id.copy_from_slice(&hash[..32]);

        let vault = Vault {
            vault_id: Hash32(id),
            model_id,
            owner,
            state: VaultState::Active,
            total_staked: initial_stake,
            share_token_supply: initial_stake,
            reward_accumulator: 0,
            created_at: current_height,
        };

        self.vaults.insert(Hash32(id), vault);

        Ok(Hash32(id))
    }

    pub fn stake(
        &mut self,
        vault_id: &Hash32,
        staker: Address,
        amount: u128,
    ) -> Result<u128, RuntimeError> {
        let vault = self
            .vaults
            .get_mut(vault_id)
            .ok_or(RuntimeError::InvalidState)?;

        if vault.state != VaultState::Active {
            return Err(RuntimeError::InvalidState);
        }

        let share_price = if vault.share_token_supply > 0 {
            (vault.total_staked as u128 * 1_000_000) / vault.share_token_supply
        } else {
            1_000_000
        };

        let shares_to_mint = (amount * 1_000_000) / share_price;

        vault.total_staked += amount;
        vault.share_token_supply += shares_to_mint;

        let holder_shares = self.shares.entry(staker).or_insert_with(Vec::new);

        if let Some(existing) = holder_shares.iter_mut().find(|s| s.vault_id == *vault_id) {
            existing.staked_amount += amount;
            existing.share_count += shares_to_mint;
        } else {
            holder_shares.push(VaultShare {
                holder: staker,
                vault_id: *vault_id,
                staked_amount: amount,
                share_count: shares_to_mint,
                last_reward_update: vault.created_at,
            });
        }

        Ok(shares_to_mint)
    }

    pub fn unstake(
        &mut self,
        vault_id: &Hash32,
        staker: &Address,
        share_count: u128,
    ) -> Result<u128, RuntimeError> {
        let vault = self
            .vaults
            .get_mut(vault_id)
            .ok_or(RuntimeError::InvalidState)?;

        let holder_shares = self
            .shares
            .get_mut(staker)
            .ok_or(RuntimeError::InvalidState)?;

        let share_entry = holder_shares
            .iter_mut()
            .find(|s| s.vault_id == *vault_id)
            .ok_or(RuntimeError::InvalidState)?;

        if share_entry.share_count < share_count {
            return Err(RuntimeError::InvalidState);
        }

        let share_ratio = (share_count as u128 * 1_000_000) / vault.share_token_supply;
        let withdraw_amount = (vault.total_staked * share_ratio) / 1_000_000;

        share_entry.share_count -= share_count;
        share_entry.staked_amount -= withdraw_amount;

        vault.total_staked -= withdraw_amount;
        vault.share_token_supply -= share_count;

        if share_entry.share_count == 0 {
            holder_shares.retain(|s| s.vault_id != *vault_id);
        }

        Ok(withdraw_amount)
    }

    pub fn distribute_reward(
        &mut self,
        vault_id: &Hash32,
        reward_amount: u128,
    ) -> Result<(), RuntimeError> {
        let vault = self
            .vaults
            .get_mut(vault_id)
            .ok_or(RuntimeError::InvalidState)?;

        if vault.state != VaultState::Active {
            return Err(RuntimeError::InvalidState);
        }

        vault.reward_accumulator += reward_amount;

        Ok(())
    }

    pub fn claim_rewards(
        &mut self,
        vault_id: &Hash32,
        staker: &Address,
    ) -> Result<u128, RuntimeError> {
        let vault = self
            .vaults
            .get(vault_id)
            .ok_or(RuntimeError::InvalidState)?;

        let holder_shares = self.shares.get(staker).ok_or(RuntimeError::InvalidState)?;

        let share_entry = holder_shares
            .iter()
            .find(|s| s.vault_id == *vault_id)
            .ok_or(RuntimeError::InvalidState)?;

        let reward =
            (vault.reward_accumulator * share_entry.share_count) / vault.share_token_supply;

        Ok(reward)
    }

    pub fn pause_vault(&mut self, vault_id: &Hash32) -> Result<(), RuntimeError> {
        let vault = self
            .vaults
            .get_mut(vault_id)
            .ok_or(RuntimeError::InvalidState)?;
        vault.state = VaultState::Paused;
        Ok(())
    }

    pub fn resume_vault(&mut self, vault_id: &Hash32) -> Result<(), RuntimeError> {
        let vault = self
            .vaults
            .get_mut(vault_id)
            .ok_or(RuntimeError::InvalidState)?;

        if vault.state != VaultState::Paused {
            return Err(RuntimeError::InvalidState);
        }

        vault.state = VaultState::Active;

        Ok(())
    }

    pub fn get_vault(&self, vault_id: &Hash32) -> Option<&Vault> {
        self.vaults.get(vault_id)
    }

    pub fn get_shares(&self, holder: &Address) -> Option<&Vec<VaultShare>> {
        self.shares.get(holder)
    }

    pub fn all_vaults(&self) -> &HashMap<Hash32, Vault> {
        &self.vaults
    }
}

impl Default for VaultModule {
    fn default() -> Self {
        Self::new()
    }
}
