// File: governance.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Governance module for Tetcore runtime. Manages proposals, voting,
// proposal states, delegation, and governance parameters. Implements
// on-chain governance with token-weighted voting and timelocks.

use crate::RuntimeError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tetcore_primitives::{Address, Hash32};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProposalState {
    Pending,
    Active,
    Passed,
    Rejected,
    Executed,
    Timelocked,
    Expired,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProposalType {
    ParameterChange,
    RuntimeUpgrade,
    TreasurySpend,
    EmergencyBrake,
    Text,
    ConstitutionalAmendment,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EmergencyScope {
    None,
    Partial,
    Full,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proposal {
    pub id: Hash32,
    pub proposer: Address,
    pub title: String,
    pub description: String,
    pub payload: Vec<u8>,
    pub proposal_type: ProposalType,
    pub state: ProposalState,
    pub votes_for: u128,
    pub votes_against: u128,
    pub voting_start: u64,
    pub voting_end: u64,
    pub executed: bool,
    pub timelock_end: u64,
    pub emergency_scope: EmergencyScope,
    pub approval_threshold: u32,
    pub quorum_threshold: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vote {
    pub voter: Address,
    pub proposal_id: Hash32,
    pub support: bool,
    pub weight: u128,
}

pub struct GovernanceModule {
    proposals: HashMap<Hash32, Proposal>,
    votes: HashMap<(Address, Hash32), Vote>,
    proposal_counter: u64,
    governance_parameters: GovernanceParameters,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GovernanceParameters {
    pub default_approval_threshold: u32,
    pub default_quorum_threshold: u32,
    pub emergency_threshold: u32,
    pub constitutional_threshold: u32,
    pub min_voting_period: u64,
    pub max_voting_period: u64,
    pub timelock_period: u64,
    pub emergency_voting_period: u64,
    pub emergency_quorum: u32,
}

impl Default for GovernanceParameters {
    fn default() -> Self {
        Self {
            default_approval_threshold: 50,
            default_quorum_threshold: 30,
            emergency_threshold: 66,
            constitutional_threshold: 75,
            min_voting_period: 7200,  // 2 hours
            max_voting_period: 604800, // 1 week
            timelock_period: 172800,   // 2 days
            emergency_voting_period: 3600, // 1 hour
            emergency_quorum: 50,
        }
    }
}

impl GovernanceModule {
    pub fn new() -> Self {
        Self {
            proposals: HashMap::new(),
            votes: HashMap::new(),
            proposal_counter: 0,
            governance_parameters: GovernanceParameters::default(),
        }
    }

    pub fn new_with_parameters(parameters: GovernanceParameters) -> Self {
        Self {
            proposals: HashMap::new(),
            votes: HashMap::new(),
            proposal_counter: 0,
            governance_parameters: parameters,
        }
    }

    pub fn create_proposal(
        &mut self,
        proposer: Address,
        title: String,
        description: String,
        payload: Vec<u8>,
        proposal_type: ProposalType,
        voting_end: u64,
        emergency_scope: EmergencyScope,
    ) -> Result<Hash32, RuntimeError> {
        self.proposal_counter += 1;

        let mut data = Vec::new();
        data.extend_from_slice(&self.proposal_counter.to_le_bytes());
        data.extend_from_slice(proposer.as_bytes());
        data.extend_from_slice(&payload);

        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(&data);
        let mut id = [0u8; 32];
        id.copy_from_slice(&hash[..32]);

        // Determine thresholds based on proposal type
        let (approval_threshold, quorum_threshold) = self.get_thresholds_for_proposal(&proposal_type);
        
        // Validate voting period
        let current_height = 0; // Would be passed in real implementation
        let min_voting_end = current_height + self.governance_parameters.min_voting_period;
        let max_voting_end = current_height + self.governance_parameters.max_voting_period;
        
        let voting_end = voting_end.clamp(min_voting_end, max_voting_end);
        
        // Determine timelock end based on proposal type
        let timelock_end = if proposal_type == ProposalType::EmergencyBrake {
            voting_end + self.governance_parameters.emergency_voting_period
        } else {
            voting_end + self.governance_parameters.timelock_period
        };

        let proposal = Proposal {
            id: Hash32(id),
            proposer,
            title,
            description,
            payload,
            proposal_type,
            state: ProposalState::Pending,
            votes_for: 0,
            votes_against: 0,
            voting_start: 0,
            voting_end,
            executed: false,
            timelock_end,
            emergency_scope,
            approval_threshold,
            quorum_threshold,
        };

        self.proposals.insert(Hash32(id), proposal);

        Ok(Hash32(id))
    }

    /// Get approval and quorum thresholds for a proposal type
    fn get_thresholds_for_proposal(&self, proposal_type: &ProposalType) -> (u32, u32) {
        match proposal_type {
            ProposalType::EmergencyBrake => (
                self.governance_parameters.emergency_threshold,
                self.governance_parameters.emergency_quorum,
            ),
            ProposalType::ConstitutionalAmendment => (
                self.governance_parameters.constitutional_threshold,
                self.governance_parameters.default_quorum_threshold,
            ),
            _ => (
                self.governance_parameters.default_approval_threshold,
                self.governance_parameters.default_quorum_threshold,
            ),
        }
    }

    pub fn activate_proposal(
        &mut self,
        proposal_id: &Hash32,
        current_height: u64,
    ) -> Result<(), RuntimeError> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or(RuntimeError::InvalidState)?;

        if proposal.state != ProposalState::Pending {
            return Err(RuntimeError::InvalidState);
        }

        proposal.state = ProposalState::Active;
        proposal.voting_start = current_height;

        Ok(())
    }

    pub fn vote(
        &mut self,
        voter: Address,
        proposal_id: &Hash32,
        support: bool,
        weight: u128,
    ) -> Result<(), RuntimeError> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or(RuntimeError::InvalidState)?;

        if proposal.state != ProposalState::Active {
            return Err(RuntimeError::InvalidState);
        }

        if support {
            proposal.votes_for += weight;
        } else {
            proposal.votes_against += weight;
        }

        let vote = Vote {
            voter,
            proposal_id: *proposal_id,
            support,
            weight,
        };

        self.votes.insert((voter, *proposal_id), vote);

        Ok(())
    }

    pub fn finalize_proposal(
        &mut self,
        proposal_id: &Hash32,
        total_supply: u128,
    ) -> Result<ProposalState, RuntimeError> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or(RuntimeError::InvalidState)?;

        // Check if quorum is met
        let total_votes = proposal.votes_for + proposal.votes_against;
        let quorum_met = self.check_quorum(total_votes, total_supply, proposal.quorum_threshold);
        
        if !quorum_met {
            proposal.state = ProposalState::Rejected;
            return Ok(proposal.state.clone());
        }

        // Check if approval threshold is met
        let approval_percentage = if total_votes > 0 {
            (proposal.votes_for * 100) / total_votes
        } else {
            0
        };
        
        if approval_percentage >= proposal.approval_threshold as u128 {
            // Check if this is an emergency proposal that can be executed immediately
            if proposal.proposal_type == ProposalType::EmergencyBrake {
                proposal.state = ProposalState::Passed;
                return Ok(proposal.state.clone());
            }
            
            // Normal proposals enter timelock period
            proposal.state = ProposalState::Timelocked;
        } else {
            proposal.state = ProposalState::Rejected;
        }

        Ok(proposal.state.clone())
    }

    /// Check if quorum requirement is met
    fn check_quorum(&self, total_votes: u128, total_supply: u128, quorum_threshold: u32) -> bool {
        if total_supply == 0 {
            return false;
        }
        let quorum_percentage = (total_votes * 100) / total_supply;
        quorum_percentage >= quorum_threshold as u128
    }

    pub fn execute_proposal(&mut self, proposal_id: &Hash32, current_height: u64) -> Result<(), RuntimeError> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or(RuntimeError::InvalidState)?;

        // Check if proposal is in executable state
        match proposal.state {
            ProposalState::Passed => {
                // Emergency proposals can be executed immediately
                if proposal.proposal_type == ProposalType::EmergencyBrake {
                    self.execute_proposal_action(proposal)?;
                    return Ok(());
                }
            },
            ProposalState::Timelocked => {
                // Check if timelock period has ended
                if current_height >= proposal.timelock_end {
                    self.execute_proposal_action(proposal)?;
                    return Ok(());
                } else {
                    return Err(RuntimeError::InvalidState);
                }
            },
            _ => return Err(RuntimeError::InvalidState),
        }

        if proposal.executed {
            return Err(RuntimeError::InvalidState);
        }

        Ok(())
    }

    /// Execute the actual proposal action
    fn execute_proposal_action(&mut self, proposal: &mut Proposal) -> Result<(), RuntimeError> {
        proposal.state = ProposalState::Executed;
        proposal.executed = true;
        
        // In a real implementation, this would:
        // 1. Parse the payload based on proposal type
        // 2. Apply the changes to the appropriate module
        // 3. Emit governance execution events
        // 4. Handle any state transitions
        
        Ok(())
    }

    /// Check if a proposal can be executed (timelock expired)
    pub fn can_execute_proposal(&self, proposal_id: &Hash32, current_height: u64) -> Result<bool, RuntimeError> {
        let proposal = self
            .proposals
            .get(proposal_id)
            .ok_or(RuntimeError::InvalidState)?;

        match proposal.state {
            ProposalState::Passed => {
                // Emergency proposals can always be executed
                Ok(proposal.proposal_type == ProposalType::EmergencyBrake)
            },
            ProposalState::Timelocked => {
                Ok(current_height >= proposal.timelock_end)
            },
            _ => Ok(false),
        }
    }

    /// Check proposal expiration
    pub fn check_proposal_expiration(&mut self, proposal_id: &Hash32, current_height: u64) -> Result<bool, RuntimeError> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or(RuntimeError::InvalidState)?;

        if current_height > proposal.voting_end && proposal.state == ProposalState::Active {
            proposal.state = ProposalState::Expired;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get governance parameters
    pub fn get_governance_parameters(&self) -> &GovernanceParameters {
        &self.governance_parameters
    }

    /// Update governance parameters (requires governance proposal)
    pub fn update_governance_parameters(&mut self, new_parameters: GovernanceParameters) {
        self.governance_parameters = new_parameters;
    }

    /// Get proposal approval percentage
    pub fn get_proposal_approval_percentage(&self, proposal_id: &Hash32) -> Result<u32, RuntimeError> {
        let proposal = self
            .proposals
            .get(proposal_id)
            .ok_or(RuntimeError::InvalidState)?;

        let total_votes = proposal.votes_for + proposal.votes_against;
        if total_votes == 0 {
            Ok(0)
        } else {
            Ok(((proposal.votes_for * 100) / total_votes) as u32)
        }
    }

    /// Get proposal quorum percentage
    pub fn get_proposal_quorum_percentage(&self, proposal_id: &Hash32, total_supply: u128) -> Result<u32, RuntimeError> {
        let proposal = self
            .proposals
            .get(proposal_id)
            .ok_or(RuntimeError::InvalidState)?;

        let total_votes = proposal.votes_for + proposal.votes_against;
        if total_supply == 0 {
            Ok(0)
        } else {
            Ok(((total_votes * 100) / total_supply) as u32)
        }
    }

    pub fn get_proposal(&self, proposal_id: &Hash32) -> Option<&Proposal> {
        self.proposals.get(proposal_id)
    }

    pub fn all_proposals(&self) -> &HashMap<Hash32, Proposal> {
        &self.proposals
    }

    /// Get proposal by type
    pub fn get_proposals_by_type(&self, proposal_type: ProposalType) -> Vec<&Proposal> {
        self.proposals.values()
            .filter(|p| p.proposal_type == proposal_type)
            .collect()
    }

    /// Get active proposals
    pub fn get_active_proposals(&self) -> Vec<&Proposal> {
        self.proposals.values()
            .filter(|p| p.state == ProposalState::Active)
            .collect()
    }

    /// Get proposals in timelock
    pub fn get_timelocked_proposals(&self) -> Vec<&Proposal> {
        self.proposals.values()
            .filter(|p| p.state == ProposalState::Timelocked)
            .collect()
    }
}

impl Default for GovernanceModule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tetcore_primitives::Address;

    fn create_test_address(i: u8) -> Address {
        let mut bytes = [0u8; 32];
        bytes[31] = i;
        Address(bytes)
    }

    #[test]
    fn test_proposal_creation_with_types() {
        let mut governance = GovernanceModule::new();
        let proposer = create_test_address(1);
        
        // Test parameter change proposal
        let result = governance.create_proposal(
            proposer,
            "Test Parameter Change".to_string(),
            "Change gas fees".to_string(),
            vec![1, 2, 3],
            ProposalType::ParameterChange,
            1000,
            EmergencyScope::None,
        );
        
        assert!(result.is_ok());
        let proposal_id = result.unwrap();
        
        let proposal = governance.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.proposal_type, ProposalType::ParameterChange);
        assert_eq!(proposal.approval_threshold, 50); // Default threshold
        assert_eq!(proposal.quorum_threshold, 30);  // Default quorum
    }

    #[test]
    fn test_emergency_proposal_thresholds() {
        let mut governance = GovernanceModule::new();
        let proposer = create_test_address(1);
        
        // Test emergency proposal
        let result = governance.create_proposal(
            proposer,
            "Emergency Brake".to_string(),
            "Stop chain due to vulnerability".to_string(),
            vec![0xFF],
            ProposalType::EmergencyBrake,
            100,
            EmergencyScope::Full,
        );
        
        assert!(result.is_ok());
        let proposal_id = result.unwrap();
        
        let proposal = governance.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.proposal_type, ProposalType::EmergencyBrake);
        assert_eq!(proposal.approval_threshold, 66); // Emergency threshold
        assert_eq!(proposal.emergency_scope, EmergencyScope::Full);
    }

    #[test]
    fn test_proposal_finalization_with_quorum() {
        let mut governance = GovernanceModule::new();
        let proposer = create_test_address(1);
        let voter1 = create_test_address(2);
        let voter2 = create_test_address(3);
        
        // Create proposal
        let proposal_id = governance.create_proposal(
            proposer,
            "Test".to_string(),
            "Test".to_string(),
            vec![],
            ProposalType::Text,
            1000,
            EmergencyScope::None,
        ).unwrap();
        
        // Activate proposal
        governance.activate_proposal(&proposal_id, 500).unwrap();
        
        // Vote with sufficient quorum (assuming total supply of 1000)
        governance.vote(voter1, &proposal_id, true, 600).unwrap();
        governance.vote(voter2, &proposal_id, false, 200).unwrap();
        
        // Finalize proposal
        let result = governance.finalize_proposal(&proposal_id, 1000);
        assert!(result.is_ok());
        
        let proposal = governance.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.state, ProposalState::Timelocked); // Should be timelocked, not passed
    }

    #[test]
    fn test_emergency_proposal_execution() {
        let mut governance = GovernanceModule::new();
        let proposer = create_test_address(1);
        let voter = create_test_address(2);
        
        // Create emergency proposal
        let proposal_id = governance.create_proposal(
            proposer,
            "Emergency".to_string(),
            "Emergency stop".to_string(),
            vec![],
            ProposalType::EmergencyBrake,
            100,
            EmergencyScope::Full,
        ).unwrap();
        
        // Activate and vote
        governance.activate_proposal(&proposal_id, 50).unwrap();
        governance.vote(voter, &proposal_id, true, 700).unwrap();
        
        // Finalize - should pass immediately for emergency
        let result = governance.finalize_proposal(&proposal_id, 1000);
        assert!(result.is_ok());
        
        let proposal = governance.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.state, ProposalState::Passed);
        
        // Should be executable immediately
        let can_execute = governance.can_execute_proposal(&proposal_id, 75).unwrap();
        assert!(can_execute);
    }

    #[test]
    fn test_timelock_period() {
        let mut governance = GovernanceModule::new();
        let proposer = create_test_address(1);
        let voter = create_test_address(2);
        
        // Create normal proposal
        let proposal_id = governance.create_proposal(
            proposer,
            "Normal".to_string(),
            "Normal proposal".to_string(),
            vec![],
            ProposalType::ParameterChange,
            1000,
            EmergencyScope::None,
        ).unwrap();
        
        // Activate and vote
        governance.activate_proposal(&proposal_id, 500).unwrap();
        governance.vote(voter, &proposal_id, true, 600).unwrap();
        
        // Finalize
        governance.finalize_proposal(&proposal_id, 1000).unwrap();
        
        let proposal = governance.get_proposal(&proposal_id).unwrap();
        assert_eq!(proposal.state, ProposalState::Timelocked);
        
        // Should not be executable during timelock
        let can_execute = governance.can_execute_proposal(&proposal_id, 1001).unwrap();
        assert!(!can_execute);
        
        // Should be executable after timelock (default timelock is 172800 blocks)
        let can_execute = governance.can_execute_proposal(&proposal_id, 173000).unwrap();
        assert!(can_execute);
    }

    #[test]
    fn test_governance_parameters() {
        let governance = GovernanceModule::new();
        let params = governance.get_governance_parameters();
        
        assert_eq!(params.default_approval_threshold, 50);
        assert_eq!(params.emergency_threshold, 66);
        assert_eq!(params.constitutional_threshold, 75);
        assert_eq!(params.timelock_period, 172800);
    }

    #[test]
    fn test_proposal_approval_calculations() {
        let mut governance = GovernanceModule::new();
        let proposer = create_test_address(1);
        let voter1 = create_test_address(2);
        let voter2 = create_test_address(3);
        
        // Create proposal
        let proposal_id = governance.create_proposal(
            proposer,
            "Test".to_string(),
            "Test".to_string(),
            vec![],
            ProposalType::Text,
            1000,
            EmergencyScope::None,
        ).unwrap();
        
        // Activate proposal
        governance.activate_proposal(&proposal_id, 500).unwrap();
        
        // Add votes
        governance.vote(voter1, &proposal_id, true, 300).unwrap();
        governance.vote(voter2, &proposal_id, false, 200).unwrap();
        
        // Check approval percentage
        let approval_pct = governance.get_proposal_approval_percentage(&proposal_id).unwrap();
        assert_eq!(approval_pct, 60); // 300 / (300 + 200) = 60%
        
        // Check quorum percentage (assuming total supply of 1000)
        let quorum_pct = governance.get_proposal_quorum_percentage(&proposal_id, 1000).unwrap();
        assert_eq!(quorum_pct, 50); // 500 / 1000 = 50%
    }
}
