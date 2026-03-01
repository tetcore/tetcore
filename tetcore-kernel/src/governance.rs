// File: governance.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Governance module implementing on-chain governance for Tetcore.
// Handles proposal submission, voting, execution, timelocks, delegation,
// and emergency powers.

use crate::{Address, Hash32};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub const DEFAULT_VOTING_PERIOD_BLOCKS: u64 = 10080;
pub const DEFAULT_TIMELOCK_PERIOD_BLOCKS: u64 = 10080;
pub const DEFAULT_PROPOSAL_BOND: u128 = 1000;
pub const DEFAULT_EMERGENCY_DURATION_BLOCKS: u64 = 14400;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalType {
    ParameterChange,
    RuntimeUpgrade,
    ModelGovernance,
    Treasury,
    Emergency,
    Slash,
    Upgrade,
}

impl ProposalType {
    pub fn threshold(&self) -> VotingThreshold {
        match self {
            ProposalType::ParameterChange => VotingThreshold::parameter_change(),
            ProposalType::RuntimeUpgrade => VotingThreshold::runtime_upgrade(),
            ProposalType::ModelGovernance => VotingThreshold::parameter_change(),
            ProposalType::Treasury => VotingThreshold::treasury(),
            ProposalType::Emergency => VotingThreshold::emergency(),
            ProposalType::Slash => VotingThreshold::constitutional(),
            ProposalType::Upgrade => VotingThreshold::runtime_upgrade(),
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalStatus {
    Pending,
    Voting,
    Approved,
    Rejected,
    Executed,
    FailedExecution,
    Cancelled,
    Timelocked,
}

impl ProposalStatus {
    pub fn is_finalized(&self) -> bool {
        matches!(
            self,
            ProposalStatus::Approved
                | ProposalStatus::Rejected
                | ProposalStatus::Executed
                | ProposalStatus::FailedExecution
                | ProposalStatus::Cancelled
        )
    }

    pub fn is_active(&self) -> bool {
        matches!(self, ProposalStatus::Voting | ProposalStatus::Timelocked)
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteChoice {
    Yes,
    No,
    Abstain,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VotingThreshold {
    pub quorum: u32,
    pub approval: u32,
}

impl VotingThreshold {
    pub fn parameter_change() -> Self {
        Self {
            quorum: 20,
            approval: 50,
        }
    }

    pub fn runtime_upgrade() -> Self {
        Self {
            quorum: 40,
            approval: 66,
        }
    }

    pub fn treasury() -> Self {
        Self {
            quorum: 25,
            approval: 50,
        }
    }

    pub fn emergency() -> Self {
        Self {
            quorum: 50,
            approval: 75,
        }
    }

    pub fn constitutional() -> Self {
        Self {
            quorum: 50,
            approval: 75,
        }
    }

    pub fn is_approved(&self, quorum_pct: u32, approval_pct: u32) -> bool {
        quorum_pct >= self.quorum && approval_pct >= self.approval
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proposal {
    pub proposal_id: Hash32,
    pub proposal_type: ProposalType,
    pub proposer: Address,
    pub payload: Vec<u8>,
    pub description: String,
    pub voting_start: u64,
    pub voting_end: u64,
    pub timelock_end: u64,
    pub status: ProposalStatus,
    pub bond_amount: u128,
    pub yes_votes: u128,
    pub no_votes: u128,
    pub abstain_votes: u128,
    pub executed_at: Option<u64>,
}

impl Proposal {
    pub fn new(
        proposer: Address,
        proposal_type: ProposalType,
        payload: Vec<u8>,
        description: String,
        current_block: u64,
        bond_amount: u128,
        voting_period: u64,
        timelock_period: u64,
    ) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(proposer.as_bytes());
        hasher.update(&payload);
        hasher.update(&description.as_bytes());
        hasher.update(&current_block.to_le_bytes());
        let result = hasher.finalize();
        let mut proposal_id = [0u8; 32];
        proposal_id.copy_from_slice(&result);

        let voting_end = current_block.saturating_add(voting_period);
        let timelock_end = voting_end.saturating_add(timelock_period);

        Self {
            proposal_id: Hash32(proposal_id),
            proposal_type,
            proposer,
            payload,
            description,
            voting_start: current_block,
            voting_end,
            timelock_end,
            status: ProposalStatus::Voting,
            bond_amount,
            yes_votes: 0,
            no_votes: 0,
            abstain_votes: 0,
            executed_at: None,
        }
    }

    pub fn total_votes(&self) -> u128 {
        self.yes_votes
            .saturating_add(self.no_votes)
            .saturating_add(self.abstain_votes)
    }

    pub fn approval_percentage(&self) -> Option<u32> {
        let total = self.total_votes();
        if total == 0 {
            return None;
        }
        Some(((self.yes_votes * 100) / total) as u32)
    }

    pub fn quorum_percentage(&self, total_supply: u128) -> Option<u32> {
        let total = self.total_votes();
        if total_supply == 0 {
            return None;
        }
        Some(((total * 100) / total_supply) as u32)
    }

    pub fn is_approved(&self, total_supply: u128) -> bool {
        let quorum_pct = self.quorum_percentage(total_supply).unwrap_or(0);
        let approval_pct = self.approval_percentage().unwrap_or(0);
        self.proposal_type
            .threshold()
            .is_approved(quorum_pct, approval_pct)
    }

    pub fn voting_active(&self, current_block: u64) -> bool {
        self.status == ProposalStatus::Voting
            && current_block >= self.voting_start
            && current_block < self.voting_end
    }

    pub fn timelock_expired(&self, current_block: u64) -> bool {
        self.status == ProposalStatus::Timelocked && current_block >= self.timelock_end
    }

    pub fn add_vote(&mut self, choice: VoteChoice, weight: u128) {
        match choice {
            VoteChoice::Yes => self.yes_votes = self.yes_votes.saturating_add(weight),
            VoteChoice::No => self.no_votes = self.no_votes.saturating_add(weight),
            VoteChoice::Abstain => self.abstain_votes = self.abstain_votes.saturating_add(weight),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vote {
    pub proposal_id: Hash32,
    pub voter: Address,
    pub choice: VoteChoice,
    pub weight: u128,
    pub timestamp: u64,
    pub block: u64,
}

impl Vote {
    pub fn new(
        proposal_id: Hash32,
        voter: Address,
        choice: VoteChoice,
        weight: u128,
        block: u64,
    ) -> Self {
        Self {
            proposal_id,
            voter,
            choice,
            weight,
            timestamp: 0,
            block,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Delegation {
    pub delegator: Address,
    pub delegate: Address,
    pub balance: u128,
    pub created_at: u64,
}

impl Delegation {
    pub fn new(delegator: Address, delegate: Address, balance: u128, block: u64) -> Self {
        Self {
            delegator,
            delegate,
            balance,
            created_at: block,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmergencyPowers {
    pub active: bool,
    pub activated_at: u64,
    pub expires_at: u64,
    pub scope: EmergencyScope,
    pub activator: Address,
}

impl EmergencyPowers {
    pub fn new(
        scope: EmergencyScope,
        duration_blocks: u64,
        activator: Address,
        current_block: u64,
    ) -> Self {
        Self {
            active: true,
            activated_at: current_block,
            expires_at: current_block.saturating_add(duration_blocks),
            scope,
            activator,
        }
    }

    pub fn is_expired(&self, current_block: u64) -> bool {
        !self.active || current_block > self.expires_at
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EmergencyScope {
    PauseInference,
    DisableModel { model_id: Hash32 },
    FreezeOperator { operator: Address },
    HaltShardRegistry,
    FullEmergency,
    Immediate,
}

pub struct GovernanceConfig {
    pub voting_period_blocks: u64,
    pub timelock_period_blocks: u64,
    pub proposal_bond: u128,
    pub emergency_duration_blocks: u64,
    pub min_proposal_bond: u128,
    pub max_proposals: u32,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            voting_period_blocks: DEFAULT_VOTING_PERIOD_BLOCKS,
            timelock_period_blocks: DEFAULT_TIMELOCK_PERIOD_BLOCKS,
            proposal_bond: DEFAULT_PROPOSAL_BOND,
            emergency_duration_blocks: DEFAULT_EMERGENCY_DURATION_BLOCKS,
            min_proposal_bond: 100,
            max_proposals: 100,
        }
    }
}

pub struct GovernanceModule {
    pub config: GovernanceConfig,
    proposals: HashMap<Hash32, Proposal>,
    votes: HashMap<Hash32, HashMap<Address, Vote>>,
    delegations: HashMap<Address, Delegation>,
    pub emergency_powers: Option<EmergencyPowers>,
    pub total_supply: u128,
    pub proposal_count: u64,
    pub executed_count: u64,
}

impl GovernanceModule {
    pub fn new(total_supply: u128) -> Self {
        Self {
            config: GovernanceConfig::default(),
            proposals: HashMap::new(),
            votes: HashMap::new(),
            delegations: HashMap::new(),
            emergency_powers: None,
            total_supply,
            proposal_count: 0,
            executed_count: 0,
        }
    }

    pub fn submit_proposal(
        &mut self,
        proposer: Address,
        proposal_type: ProposalType,
        payload: Vec<u8>,
        description: String,
        current_block: u64,
    ) -> Result<Proposal, GovernanceError> {
        if self.proposals.len() >= self.config.max_proposals as usize {
            return Err(GovernanceError::TooManyProposals);
        }

        if self.config.proposal_bond < self.config.min_proposal_bond {
            return Err(GovernanceError::InsufficientBond);
        }

        let proposal = Proposal::new(
            proposer,
            proposal_type,
            payload,
            description,
            current_block,
            self.config.proposal_bond,
            self.config.voting_period_blocks,
            self.config.timelock_period_blocks,
        );

        let proposal_id = proposal.proposal_id;
        self.proposals.insert(proposal_id, proposal);
        self.votes.insert(proposal_id, HashMap::new());
        self.proposal_count += 1;

        Ok(self.proposals.get(&proposal_id).unwrap().clone())
    }

    pub fn cast_vote(
        &mut self,
        proposal_id: &Hash32,
        voter: Address,
        choice: VoteChoice,
        weight: u128,
        current_block: u64,
    ) -> Result<(), GovernanceError> {
        let voting_power = self.get_voting_power(&voter, weight);

        if voting_power == 0 {
            return Err(GovernanceError::NoVotingPower);
        }

        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;

        if !proposal.voting_active(current_block) {
            return Err(GovernanceError::VotingNotActive);
        }

        let votes = self.votes.get_mut(proposal_id).unwrap();

        if let Some(existing_vote) = votes.get(&voter) {
            proposal.add_vote(existing_vote.choice, 0);
            proposal.add_vote(choice, voting_power);
            votes.insert(
                voter,
                Vote::new(*proposal_id, voter, choice, voting_power, current_block),
            );
        } else {
            proposal.add_vote(choice, voting_power);
            votes.insert(
                voter,
                Vote::new(*proposal_id, voter, choice, voting_power, current_block),
            );
        }

        Ok(())
    }

    fn get_voting_power(&self, voter: &Address, base_weight: u128) -> u128 {
        let mut power = base_weight;

        if let Some(delegation) = self.delegations.get(voter) {
            power = power.saturating_add(delegation.balance);
        }

        power
    }

    pub fn delegate(
        &mut self,
        delegator: Address,
        delegate: Address,
        amount: u128,
        current_block: u64,
    ) -> Result<(), GovernanceError> {
        if delegator == delegate {
            return Err(GovernanceError::SelfDelegation);
        }

        let delegation = Delegation::new(delegator, delegate, amount, current_block);
        self.delegations.insert(delegator, delegation);

        Ok(())
    }

    pub fn undelegate(&mut self, delegator: &Address) -> Result<(), GovernanceError> {
        self.delegations
            .remove(delegator)
            .ok_or(GovernanceError::NoDelegation)?;
        Ok(())
    }

    pub fn end_voting(
        &mut self,
        proposal_id: &Hash32,
        current_block: u64,
    ) -> Result<ProposalStatus, GovernanceError> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;

        if proposal.status != ProposalStatus::Voting {
            return Err(GovernanceError::VotingNotActive);
        }

        if current_block < proposal.voting_end {
            return Err(GovernanceError::VotingStillActive);
        }

        if proposal.is_approved(self.total_supply) {
            proposal.status = ProposalStatus::Timelocked;
            Ok(ProposalStatus::Timelocked)
        } else {
            proposal.status = ProposalStatus::Rejected;
            Ok(ProposalStatus::Rejected)
        }
    }

    pub fn execute_proposal(
        &mut self,
        proposal_id: &Hash32,
        current_block: u64,
    ) -> Result<(), GovernanceError> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;

        if proposal.status != ProposalStatus::Timelocked {
            return Err(GovernanceError::NotTimelocked);
        }

        if !proposal.timelock_expired(current_block) {
            return Err(GovernanceError::TimelockNotExpired);
        }

        proposal.status = ProposalStatus::Executed;
        proposal.executed_at = Some(current_block);
        self.executed_count += 1;

        Ok(())
    }

    pub fn cancel_proposal(
        &mut self,
        proposal_id: &Hash32,
        proposer: &Address,
    ) -> Result<(), GovernanceError> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;

        if proposal.proposer != *proposer {
            return Err(GovernanceError::NotProposer);
        }

        if proposal.status != ProposalStatus::Voting && proposal.status != ProposalStatus::Pending {
            return Err(GovernanceError::CannotCancel);
        }

        proposal.status = ProposalStatus::Cancelled;

        Ok(())
    }

    pub fn activate_emergency(
        &mut self,
        scope: EmergencyScope,
        activator: Address,
        current_block: u64,
    ) -> Result<(), GovernanceError> {
        if !self.emergency_powers.is_none() {
            if let Some(ref powers) = self.emergency_powers {
                if powers.active && !powers.is_expired(current_block) {
                    return Err(GovernanceError::EmergencyAlreadyActive);
                }
            }
        }

        let emergency = EmergencyPowers::new(
            scope,
            self.config.emergency_duration_blocks,
            activator,
            current_block,
        );

        self.emergency_powers = Some(emergency);

        Ok(())
    }

    pub fn deactivate_emergency(&mut self) -> Result<(), GovernanceError> {
        if let Some(ref mut powers) = self.emergency_powers {
            powers.deactivate();
            Ok(())
        } else {
            Err(GovernanceError::NoEmergency)
        }
    }

    pub fn get_proposal(&self, proposal_id: &Hash32) -> Option<&Proposal> {
        self.proposals.get(proposal_id)
    }

    pub fn get_vote(&self, proposal_id: &Hash32, voter: &Address) -> Option<&Vote> {
        self.votes.get(proposal_id).and_then(|v| v.get(voter))
    }

    pub fn get_delegation(&self, delegator: &Address) -> Option<&Delegation> {
        self.delegations.get(delegator)
    }

    pub fn active_proposals(&self, current_block: u64) -> Vec<&Proposal> {
        self.proposals
            .values()
            .filter(|p| p.voting_active(current_block))
            .collect()
    }

    pub fn timelocked_proposals(&self) -> Vec<&Proposal> {
        self.proposals
            .values()
            .filter(|p| p.status == ProposalStatus::Timelocked)
            .collect()
    }

    pub fn proposal_count(&self) -> u64 {
        self.proposal_count
    }

    pub fn update_config(&mut self, config: GovernanceConfig) {
        self.config = config;
    }

    pub fn update_total_supply(&mut self, supply: u128) {
        self.total_supply = supply;
    }
}

impl Default for GovernanceModule {
    fn default() -> Self {
        Self::new(0)
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum GovernanceError {
    ProposalNotFound,
    VotingNotActive,
    VotingStillActive,
    NotTimelocked,
    TimelockNotExpired,
    InsufficientBond,
    TooManyProposals,
    NoVotingPower,
    SelfDelegation,
    NoDelegation,
    NotProposer,
    CannotCancel,
    EmergencyAlreadyActive,
    NoEmergency,
    ProposalExecutionFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_address(i: u8) -> Address {
        let mut bytes = [0u8; 32];
        bytes[31] = i;
        Address(bytes)
    }

    fn create_hash(i: u8) -> Hash32 {
        let mut bytes = [0u8; 32];
        bytes[31] = i;
        Hash32(bytes)
    }

    #[test]
    fn test_proposal_submission() {
        let mut gov = GovernanceModule::new(1_000_000);
        let proposer = create_address(1);

        let result = gov.submit_proposal(
            proposer,
            ProposalType::ParameterChange,
            vec![1, 2, 3],
            "Test proposal".to_string(),
            0,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_vote_casting() {
        let mut gov = GovernanceModule::new(1_000_000);
        let proposer = create_address(1);
        let voter = create_address(2);

        let proposal = gov
            .submit_proposal(
                proposer,
                ProposalType::ParameterChange,
                vec![1, 2, 3],
                "Test proposal".to_string(),
                0,
            )
            .unwrap();

        let result = gov.cast_vote(&proposal.proposal_id, voter, VoteChoice::Yes, 1000, 1);

        assert!(result.is_ok());
    }

    #[test]
    fn test_delegation() {
        let mut gov = GovernanceModule::new(1_000_000);
        let delegator = create_address(1);
        let delegate = create_address(2);

        let result = gov.delegate(delegator, delegate, 500, 0);
        assert!(result.is_ok());

        let delegation = gov.get_delegation(&delegator);
        assert!(delegation.is_some());
    }

    #[test]
    fn test_proposal_approval() {
        let mut gov = GovernanceModule::new(1_000_000);
        let proposer = create_address(1);

        let mut proposal = gov
            .submit_proposal(
                proposer,
                ProposalType::ParameterChange,
                vec![1, 2, 3],
                "Test proposal".to_string(),
                0,
            )
            .unwrap();

        proposal.yes_votes = 700000;
        proposal.no_votes = 100000;

        assert!(proposal.is_approved(1_000_000));
    }

    #[test]
    fn test_emergency_powers() {
        let mut gov = GovernanceModule::new(1_000_000);
        let activator = create_address(1);

        let scope = EmergencyScope::PauseInference;
        let result = gov.activate_emergency(scope, activator, 0);
        assert!(result.is_ok());

        assert!(gov.emergency_powers.is_some());
    }
}
