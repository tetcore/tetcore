// File: operator.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Inference Operator implementation for Tetcore nodes.
// Handles prompt execution, receipt generation, and economic compliance
// as specified in SDK-502 (Operator Reference Implementation).

use std::collections::HashMap;
use tetcore_primitives::{Address, Hash32, PromptCommitment};
use tetcore_kernel::ifp::{Prompt, Receipt, ModelRegistry};
use serde::{Serialize, Deserialize};
use thiserror::Error;

/// Operator error types
#[derive(Error, Debug, Clone)]
pub enum OperatorError {
    #[error("Prompt verification failed")]
    PromptVerificationFailed,
    #[error("Model not found or inactive")]
    ModelNotFound,
    #[error("Deadline expired")]
    DeadlineExpired,
    #[error("Execution failed")]
    ExecutionFailed,
    #[error("Receipt submission failed")]
    ReceiptSubmissionFailed,
    #[error("Economic compliance violation")]
    EconomicComplianceViolation,
}

/// Operator configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperatorConfig {
    pub operator_address: Address,
    pub supported_models: Vec<Hash32>,
    pub max_concurrent_executions: usize,
    pub relay_mode: RelayMode,
    pub pricing_policy: PricingPolicy,
    pub compute_limits: ComputeLimits,
}

impl Default for OperatorConfig {
    fn default() -> Self {
        Self {
            operator_address: Address::zero(),
            supported_models: Vec::new(),
            max_concurrent_executions: 4,
            relay_mode: RelayMode::RelayTransport,
            pricing_policy: PricingPolicy::MarketBased,
            compute_limits: ComputeLimits::default(),
        }
    }
}

/// Relay mode for prompt transport
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelayMode {
    RelayTransport,
    DirectConnection,
}

/// Pricing policy for operator services
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PricingPolicy {
    MarketBased,
    FixedRate,
    Dynamic,
}

/// Compute resource limits
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComputeLimits {
    pub max_input_tokens: u32,
    pub max_output_tokens: u32,
    pub max_compute_units: u64,
    pub max_execution_time_ms: u64,
}

impl Default for ComputeLimits {
    fn default() -> Self {
        Self {
            max_input_tokens: 4096,
            max_output_tokens: 2048,
            max_compute_units: 1000000,
            max_execution_time_ms: 30000,
        }
    }
}

/// Execution status of a prompt
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionStatus {
    Pending,
    Executing,
    Completed,
    Failed,
    ReceiptSubmitted,
}

/// Operator execution state for a prompt
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptExecution {
    pub prompt_id: Hash32,
    pub model_id: Hash32,
    pub client_address: Address,
    pub status: ExecutionStatus,
    pub start_time: u64,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub compute_units: u64,
    pub deadline_height: u64,
    pub relay_mode: RelayMode,
}

/// Main Inference Operator structure
pub struct InferenceOperator {
    config: OperatorConfig,
    model_registry: ModelRegistry,
    active_executions: HashMap<Hash32, PromptExecution>,
    execution_history: Vec<PromptExecution>,
    economic_compliance: EconomicCompliance,
}

impl InferenceOperator {
    /// Create a new Inference Operator
    pub fn new(config: OperatorConfig, model_registry: ModelRegistry) -> Self {
        Self {
            config,
            model_registry,
            active_executions: HashMap::new(),
            execution_history: Vec::new(),
            economic_compliance: EconomicCompliance::new(),
        }
    }

    /// Monitor for new SubmitPrompt events
    pub fn monitor_events(&mut self, current_height: u64) -> Vec<Hash32> {
        // In a real implementation, this would monitor the blockchain for SubmitPrompt events
        // For now, return empty vector
        Vec::new()
    }

    /// Accept a new prompt for execution
    pub fn accept_prompt(
        &mut self,
        prompt: &Prompt,
        current_height: u64,
    ) -> Result<(), OperatorError> {
        // Verify model is supported and active
        if !self.model_registry.is_model_active(&prompt.model_id) {
            return Err(OperatorError::ModelNotFound);
        }

        // Check if we support this model
        if !self.config.supported_models.contains(&prompt.model_id) {
            return Err(OperatorError::ModelNotFound);
        }

        // Check deadline
        if current_height >= prompt.deadline_height {
            return Err(OperatorError::DeadlineExpired);
        }

        // Check concurrent execution limit
        if self.active_executions.len() >= self.config.max_concurrent_executions {
            return Err(OperatorError::ExecutionFailed);
        }

        // Create execution record
        let execution = PromptExecution {
            prompt_id: prompt.prompt_id,
            model_id: prompt.model_id,
            client_address: prompt.client,
            status: ExecutionStatus::Pending,
            start_time: current_height,
            input_tokens: prompt.input_tokens,
            output_tokens: 0,
            compute_units: 0,
            deadline_height: prompt.deadline_height,
            relay_mode: self.config.relay_mode.clone(),
        };

        self.active_executions.insert(prompt.prompt_id, execution);
        Ok(())
    }

    /// Execute a prompt and generate output
    pub fn execute_prompt(&mut self, prompt_id: &Hash32) -> Result<Vec<u8>, OperatorError> {
        let execution = self.active_executions.get_mut(prompt_id)
            .ok_or(OperatorError::PromptVerificationFailed)?;

        if execution.status != ExecutionStatus::Pending {
            return Err(OperatorError::ExecutionFailed);
        }

        // Update status to executing
        execution.status = ExecutionStatus::Executing;

        // In a real implementation, this would:
        // 1. Retrieve the encrypted prompt payload
        // 2. Decrypt and verify the prompt commitment
        // 3. Load the correct model version
        // 4. Execute inference with resource limits
        // 5. Generate output and compute output commitment

        // For now, simulate execution
        let output = vec![0u8; 1024]; // Simulated output
        execution.output_tokens = 512;
        execution.compute_units = 10000;
        execution.status = ExecutionStatus::Completed;

        Ok(output)
    }

    /// Generate receipt for completed execution
    pub fn generate_receipt(&self, prompt_id: &Hash32) -> Result<Receipt, OperatorError> {
        let execution = self.active_executions.get(prompt_id)
            .ok_or(OperatorError::PromptVerificationFailed)?;

        if execution.status != ExecutionStatus::Completed {
            return Err(OperatorError::ExecutionFailed);
        }

        // Create receipt with proper economic parameters
        let receipt = Receipt {
            receipt_id: Hash32::empty(), // Would be computed in real implementation
            prompt_id: *prompt_id,
            model_id: execution.model_id,
            operator: self.config.operator_address,
            client: execution.client_address,
            input_tokens: execution.input_tokens,
            output_tokens: execution.output_tokens,
            compute_units: execution.compute_units,
            output_commitment: Hash32::empty(), // Would be computed from output
            timestamp: execution.start_time,
            signature: vec![], // Would be signed in real implementation
        };

        Ok(receipt)
    }

    /// Submit receipt to the blockchain
    pub fn submit_receipt(&mut self, receipt: Receipt, current_height: u64) -> Result<(), OperatorError> {
        let execution = self.active_executions.get_mut(&receipt.prompt_id)
            .ok_or(OperatorError::PromptVerificationFailed)?;

        // Check deadline
        if current_height >= execution.deadline_height {
            return Err(OperatorError::DeadlineExpired);
        }

        // Verify economic compliance
        self.economic_compliance.verify_receipt(&receipt)?;

        // In a real implementation, this would submit the receipt transaction
        // For now, just update the status
        execution.status = ExecutionStatus::ReceiptSubmitted;

        // Move to history
        self.execution_history.push(execution.clone());
        self.active_executions.remove(&receipt.prompt_id);

        Ok(())
    }

    /// Get current active executions
    pub fn get_active_executions(&self) -> Vec<&PromptExecution> {
        self.active_executions.values().collect()
    }

    /// Get execution history
    pub fn get_execution_history(&self) -> &Vec<PromptExecution> {
        &self.execution_history
    }

    /// Get operator configuration
    pub fn get_config(&self) -> &OperatorConfig {
        &self.config
    }
}

/// Economic compliance tracker
pub struct EconomicCompliance {
    pub valid_receipts: u64,
    pub invalid_receipts: u64,
    pub total_revenue: u128,
    pub compliance_violations: u64,
}

impl EconomicCompliance {
    pub fn new() -> Self {
        Self {
            valid_receipts: 0,
            invalid_receipts: 0,
            total_revenue: 0,
            compliance_violations: 0,
        }
    }

    /// Verify receipt meets economic compliance rules
    pub fn verify_receipt(&mut self, receipt: &Receipt) -> Result<(), OperatorError> {
        // Check token counts are reasonable
        if receipt.output_tokens > self.max_allowed_output_tokens(receipt.input_tokens) {
            self.compliance_violations += 1;
            return Err(OperatorError::EconomicComplianceViolation);
        }

        // Check compute units are reasonable
        if receipt.compute_units > self.max_allowed_compute_units(receipt.input_tokens) {
            self.compliance_violations += 1;
            return Err(OperatorError::EconomicComplianceViolation);
        }

        self.valid_receipts += 1;
        Ok(())
    }

    fn max_allowed_output_tokens(&self, input_tokens: u32) -> u32 {
        // Allow up to 2x input tokens as output
        input_tokens * 2
    }

    fn max_allowed_compute_units(&self, input_tokens: u32) -> u64 {
        // Allow up to 10,000 compute units per input token
        input_tokens as u64 * 10_000
    }

    /// Get compliance report
    pub fn get_compliance_report(&self) -> ComplianceReport {
        ComplianceReport {
            valid_receipts: self.valid_receipts,
            invalid_receipts: self.invalid_receipts,
            total_revenue: self.total_revenue,
            compliance_violations: self.compliance_violations,
            compliance_score: self.calculate_compliance_score(),
        }
    }

    fn calculate_compliance_score(&self) -> f64 {
        if self.valid_receipts + self.invalid_receipts == 0 {
            return 1.0;
        }
        self.valid_receipts as f64 / (self.valid_receipts + self.invalid_receipts) as f64
    }
}

/// Compliance report structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub valid_receipts: u64,
    pub invalid_receipts: u64,
    pub total_revenue: u128,
    pub compliance_violations: u64,
    pub compliance_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tetcore_kernel::ifp::ModelRegistry;

    fn create_test_operator() -> InferenceOperator {
        let config = OperatorConfig {
            operator_address: Address::from_bytes([1u8; 32]),
            supported_models: vec![Hash32::empty()],
            ..Default::default()
        };
        let model_registry = ModelRegistry::new();
        InferenceOperator::new(config, model_registry)
    }

    fn create_test_prompt() -> Prompt {
        Prompt {
            prompt_id: Hash32::empty(),
            model_id: Hash32::empty(),
            client: Address::from_bytes([2u8; 32]),
            input_tokens: 256,
            max_output_tokens: 512,
            compute_limit: 50000,
            pricing_mode: 0,
            relay_mode: 0,
            deadline_height: 1000,
            prompt_commitment: PromptCommitment::default(),
            escrow_amount: 1000,
            escrow_release_height: 1001,
        }
    }

    #[test]
    fn test_operator_creation() {
        let operator = create_test_operator();
        assert_eq!(operator.get_config().supported_models.len(), 1);
        assert_eq!(operator.get_active_executions().len(), 0);
    }

    #[test]
    fn test_prompt_acceptance() {
        let mut operator = create_test_operator();
        let prompt = create_test_prompt();
        
        let result = operator.accept_prompt(&prompt, 500);
        assert!(result.is_ok());
        assert_eq!(operator.get_active_executions().len(), 1);
    }

    #[test]
    fn test_prompt_execution() {
        let mut operator = create_test_operator();
        let prompt = create_test_prompt();
        
        operator.accept_prompt(&prompt, 500).unwrap();
        let result = operator.execute_prompt(&prompt.prompt_id);
        assert!(result.is_ok());
        
        let executions = operator.get_active_executions();
        assert_eq!(executions[0].status, ExecutionStatus::Completed);
    }

    #[test]
    fn test_receipt_generation() {
        let mut operator = create_test_operator();
        let prompt = create_test_prompt();
        
        operator.accept_prompt(&prompt, 500).unwrap();
        operator.execute_prompt(&prompt.prompt_id).unwrap();
        
        let receipt = operator.generate_receipt(&prompt.prompt_id);
        assert!(receipt.is_ok());
        assert_eq!(receipt.unwrap().operator, operator.get_config().operator_address);
    }

    #[test]
    fn test_deadline_enforcement() {
        let mut operator = create_test_operator();
        let prompt = create_test_prompt();
        
        // Try to accept prompt after deadline
        let result = operator.accept_prompt(&prompt, 1001);
        assert!(matches!(result, Err(OperatorError::DeadlineExpired)));
    }

    #[test]
    fn test_economic_compliance() {
        let mut operator = create_test_operator();
        let prompt = create_test_prompt();
        
        operator.accept_prompt(&prompt, 500).unwrap();
        operator.execute_prompt(&prompt.prompt_id).unwrap();
        let receipt = operator.generate_receipt(&prompt.prompt_id).unwrap();
        
        let result = operator.submit_receipt(receipt, 999);
        assert!(result.is_ok());
        
        let report = operator.economic_compliance.get_compliance_report();
        assert_eq!(report.valid_receipts, 1);
        assert_eq!(report.compliance_score, 1.0);
    }
}
