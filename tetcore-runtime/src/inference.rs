use crate::RuntimeError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tetcore_primitives::{Address, Hash32};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PromptState {
    Submitted,
    Locked,
    Executing,
    Completed,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prompt {
    pub prompt_id: Hash32,
    pub model_id: Hash32,
    pub version: u32,
    pub sender: Address,
    pub prompt_data: Vec<u8>,
    pub state: PromptState,
    pub escrow_amount: u128,
    pub fee: u64,
    pub submitted_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub receipt_id: Hash32,
    pub prompt_id: Hash32,
    pub operator: Address,
    pub inference_output: Vec<u8>,
    pub execution_proof: Vec<u8>,
    pub submitted_at: u64,
    pub verified: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InferenceSettlement {
    pub receipt_id: Hash32,
    pub prompt_id: Hash32,
    pub model_id: Hash32,
    pub total_fee: u128,
    pub model_owner_amount: u128,
    pub operator_amount: u128,
    pub shard_provider_amount: u128,
    pub validator_amount: u128,
    pub vault_amount: u128,
    pub settled: bool,
}

pub struct InferenceModule {
    prompts: HashMap<Hash32, Prompt>,
    receipts: HashMap<Hash32, Receipt>,
    settlements: HashMap<Hash32, InferenceSettlement>,
    prompt_counter: u64,
    receipt_counter: u64,
}

impl InferenceModule {
    pub fn new() -> Self {
        Self {
            prompts: HashMap::new(),
            receipts: HashMap::new(),
            settlements: HashMap::new(),
            prompt_counter: 0,
            receipt_counter: 0,
        }
    }

    pub fn submit_prompt(
        &mut self,
        model_id: Hash32,
        version: u32,
        sender: Address,
        prompt_data: Vec<u8>,
        fee: u64,
        current_height: u64,
    ) -> Result<Hash32, RuntimeError> {
        self.prompt_counter += 1;

        let mut data = Vec::new();
        data.extend_from_slice(&self.prompt_counter.to_le_bytes());
        data.extend_from_slice(model_id.as_bytes());
        data.extend_from_slice(&prompt_data);

        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(&data);
        let mut id = [0u8; 32];
        id.copy_from_slice(&hash[..32]);

        let prompt = Prompt {
            prompt_id: Hash32(id),
            model_id,
            version,
            sender,
            prompt_data,
            state: PromptState::Submitted,
            escrow_amount: 0,
            fee,
            submitted_at: current_height,
        };

        self.prompts.insert(Hash32(id), prompt);

        Ok(Hash32(id))
    }

    pub fn lock_prompt(
        &mut self,
        prompt_id: &Hash32,
        escrow_amount: u128,
    ) -> Result<(), RuntimeError> {
        let prompt = self
            .prompts
            .get_mut(prompt_id)
            .ok_or(RuntimeError::InvalidState)?;

        if prompt.state != PromptState::Submitted {
            return Err(RuntimeError::InvalidState);
        }

        prompt.state = PromptState::Locked;
        prompt.escrow_amount = escrow_amount;

        Ok(())
    }

    pub fn execute_prompt(&mut self, prompt_id: &Hash32) -> Result<(), RuntimeError> {
        let prompt = self
            .prompts
            .get_mut(prompt_id)
            .ok_or(RuntimeError::InvalidState)?;

        if prompt.state != PromptState::Locked {
            return Err(RuntimeError::InvalidState);
        }

        prompt.state = PromptState::Executing;

        Ok(())
    }

    pub fn submit_receipt(
        &mut self,
        prompt_id: &Hash32,
        operator: Address,
        inference_output: Vec<u8>,
        execution_proof: Vec<u8>,
        current_height: u64,
    ) -> Result<Hash32, RuntimeError> {
        let prompt = self
            .prompts
            .get(prompt_id)
            .ok_or(RuntimeError::InvalidState)?;

        if prompt.state != PromptState::Executing {
            return Err(RuntimeError::InvalidState);
        }

        self.receipt_counter += 1;

        let mut data = Vec::new();
        data.extend_from_slice(&self.receipt_counter.to_le_bytes());
        data.extend_from_slice(prompt_id.as_bytes());
        data.extend_from_slice(&inference_output);

        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(&data);
        let mut id = [0u8; 32];
        id.copy_from_slice(&hash[..32]);

        let receipt = Receipt {
            receipt_id: Hash32(id),
            prompt_id: *prompt_id,
            operator,
            inference_output,
            execution_proof,
            submitted_at: current_height,
            verified: false,
        };

        self.receipts.insert(Hash32(id), receipt);

        let mut prompt = self.prompts.get_mut(prompt_id).unwrap();
        prompt.state = PromptState::Completed;

        Ok(Hash32(id))
    }

    pub fn verify_receipt(&mut self, receipt_id: &Hash32) -> Result<(), RuntimeError> {
        let receipt = self
            .receipts
            .get_mut(receipt_id)
            .ok_or(RuntimeError::InvalidState)?;

        receipt.verified = true;

        Ok(())
    }

    pub fn create_settlement(
        &mut self,
        receipt_id: &Hash32,
        model_id: &Hash32,
        revenue_split: &crate::model_registry::RevenueSplit,
    ) -> Result<InferenceSettlement, RuntimeError> {
        let receipt = self
            .receipts
            .get(receipt_id)
            .ok_or(RuntimeError::InvalidState)?;
        let prompt = self
            .prompts
            .get(&receipt.prompt_id)
            .ok_or(RuntimeError::InvalidState)?;

        let total_fee = prompt.fee as u128;

        let settlement = InferenceSettlement {
            receipt_id: *receipt_id,
            prompt_id: receipt.prompt_id,
            model_id: *model_id,
            total_fee,
            model_owner_amount: (total_fee * revenue_split.model_owner_bps as u128) / 10000,
            operator_amount: (total_fee * revenue_split.operator_bps as u128) / 10000,
            shard_provider_amount: (total_fee * revenue_split.shard_provider_bps as u128) / 10000,
            validator_amount: (total_fee * revenue_split.validator_bps as u128) / 10000,
            vault_amount: (total_fee * revenue_split.vault_bps as u128) / 10000,
            settled: false,
        };

        self.settlements.insert(*receipt_id, settlement.clone());

        Ok(settlement)
    }

    pub fn mark_settled(&mut self, receipt_id: &Hash32) -> Result<(), RuntimeError> {
        let settlement = self
            .settlements
            .get_mut(receipt_id)
            .ok_or(RuntimeError::InvalidState)?;
        settlement.settled = true;
        Ok(())
    }

    pub fn get_prompt(&self, prompt_id: &Hash32) -> Option<&Prompt> {
        self.prompts.get(prompt_id)
    }

    pub fn get_receipt(&self, receipt_id: &Hash32) -> Option<&Receipt> {
        self.receipts.get(receipt_id)
    }

    pub fn get_settlement(&self, receipt_id: &Hash32) -> Option<&InferenceSettlement> {
        self.settlements.get(receipt_id)
    }
}

impl Default for InferenceModule {
    fn default() -> Self {
        Self::new()
    }
}
