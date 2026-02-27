use crate::RuntimeError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tetcore_primitives::{account::AccountData, Address, Hash32};

pub struct AccountsModule {
    accounts: HashMap<Address, AccountData>,
}

impl AccountsModule {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }

    pub fn create_account(&mut self, address: Address, balance: u128) {
        self.accounts.insert(address, AccountData::new(balance));
    }

    pub fn get_account(&self, address: &Address) -> Option<&AccountData> {
        self.accounts.get(address)
    }

    pub fn get_account_mut(&mut self, address: &Address) -> Option<&mut AccountData> {
        self.accounts.get_mut(address)
    }

    pub fn account_exists(&self, address: &Address) -> bool {
        self.accounts.contains_key(address)
    }

    pub fn transfer(
        &mut self,
        from: &Address,
        to: &Address,
        amount: u128,
    ) -> Result<(), RuntimeError> {
        let from_account = self
            .accounts
            .get_mut(from)
            .ok_or(RuntimeError::InvalidState)?;

        if from_account.balance < amount {
            return Err(RuntimeError::InvalidState);
        }

        from_account.balance -= amount;

        let to_account = self
            .accounts
            .entry(*to)
            .or_insert_with(|| AccountData::new(0));
        to_account.balance += amount;

        Ok(())
    }

    pub fn increment_nonce(&mut self, address: &Address) -> Result<(), RuntimeError> {
        let account = self
            .accounts
            .get_mut(address)
            .ok_or(RuntimeError::InvalidState)?;
        account.nonce += 1;
        Ok(())
    }

    pub fn set_contract_code(
        &mut self,
        address: &Address,
        code_hash: Hash32,
    ) -> Result<(), RuntimeError> {
        let account = self
            .accounts
            .get_mut(address)
            .ok_or(RuntimeError::InvalidState)?;
        account.contract_code_ref = Some(code_hash);
        Ok(())
    }

    pub fn set_contract_storage_root(
        &mut self,
        address: &Address,
        storage_root: Hash32,
    ) -> Result<(), RuntimeError> {
        let account = self
            .accounts
            .get_mut(address)
            .ok_or(RuntimeError::InvalidState)?;
        account.contract_storage_root = Some(storage_root);
        Ok(())
    }

    pub fn all_accounts(&self) -> &HashMap<Address, AccountData> {
        &self.accounts
    }
}

impl Default for AccountsModule {
    fn default() -> Self {
        Self::new()
    }
}
