pub mod accounts;
pub mod governance;
pub mod inference;
pub mod model_registry;
pub mod revenue;
pub mod vault;

pub use accounts::AccountsModule;
pub use governance::GovernanceModule;
pub use inference::InferenceModule;
pub use model_registry::ModelRegistryModule;
pub use revenue::RevenueModule;
pub use vault::VaultModule;

use std::collections::HashMap;
use tetcore_primitives::{Address, Hash32};
use thiserror::Error;

pub trait Module: Send + Sync {
    fn name(&self) -> &str;
    fn process(&mut self, tx_data: &[u8]) -> Result<(), RuntimeError>;
}

#[derive(Error, Debug, Clone)]
pub enum RuntimeError {
    #[error("Module not found")]
    ModuleNotFound,
    #[error("Invalid module call")]
    InvalidModuleCall,
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    #[error("Storage error")]
    StorageError,
    #[error("Invalid state")]
    InvalidState,
}

pub struct Runtime {
    accounts: AccountsModule,
    governance: GovernanceModule,
    model_registry: ModelRegistryModule,
    inference: InferenceModule,
    vault: VaultModule,
    revenue: RevenueModule,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            accounts: AccountsModule::new(),
            governance: GovernanceModule::new(),
            model_registry: ModelRegistryModule::new(),
            inference: InferenceModule::new(),
            vault: VaultModule::new(),
            revenue: RevenueModule::new(),
        }
    }

    pub fn accounts(&self) -> &AccountsModule {
        &self.accounts
    }

    pub fn accounts_mut(&mut self) -> &mut AccountsModule {
        &mut self.accounts
    }

    pub fn governance(&self) -> &GovernanceModule {
        &self.governance
    }

    pub fn governance_mut(&mut self) -> &mut GovernanceModule {
        &mut self.governance
    }

    pub fn model_registry(&self) -> &ModelRegistryModule {
        &self.model_registry
    }

    pub fn model_registry_mut(&mut self) -> &mut ModelRegistryModule {
        &mut self.model_registry
    }

    pub fn inference(&self) -> &InferenceModule {
        &self.inference
    }

    pub fn inference_mut(&mut self) -> &mut InferenceModule {
        &mut self.inference
    }

    pub fn vault(&self) -> &VaultModule {
        &self.vault
    }

    pub fn vault_mut(&mut self) -> &mut VaultModule {
        &mut self.vault
    }

    pub fn revenue(&self) -> &RevenueModule {
        &self.revenue
    }

    pub fn revenue_mut(&mut self) -> &mut RevenueModule {
        &mut self.revenue
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}
