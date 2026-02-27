// File: governance.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Governance primitives for Tetcore including Proposal, ProposalType,
// ProposalStatus, Vote, VoteChoice, VotingThreshold, Delegation,
// GovernanceParameters, EmergencyPowers, and GovernanceThresholds.
// Supports on-chain governance with token-weighted voting and timelocks.

use crate::crypto::Address;
use crate::hash::Hash32;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
}

impl Proposal {
    pub fn new(
        proposer: Address,
        proposal_type: ProposalType,
        payload: Vec<u8>,
        description: String,
        voting_period: u64,
        bond_amount: u128,
    ) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(proposer.as_bytes());
        hasher.update(&payload);
        hasher.update(&description.as_bytes());
        let result = hasher.finalize();
        let mut proposal_id = [0u8; 32];
        proposal_id.copy_from_slice(&result);

        Self {
            proposal_id: Hash32(proposal_id),
            proposal_type,
            proposer,
            payload,
            description,
            voting_start: 0,
            voting_end: 0,
            timelock_end: 0,
            status: ProposalStatus::Pending,
            bond_amount,
            yes_votes: 0,
            no_votes: 0,
            abstain_votes: 0,
        }
    }

    pub fn total_votes(&self) -> u128 {
        self.yes_votes + self.no_votes + self.abstain_votes
    }

    pub fn approval_percentage(&self, total_supply: u128) -> Option<u32> {
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
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalType {
    ParameterChange,
    RuntimeUpgrade,
    ModelGovernance,
    Treasury,
    Emergency,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProposalStatus {
    #[default]
    Pending,
    Voting,
    Approved,
    Rejected,
    Executed,
    FailedExecution,
    Cancelled,
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vote {
    pub proposal_id: Hash32,
    pub voter: Address,
    pub vote: VoteChoice,
    pub weight: u128,
    pub timestamp: u64,
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
pub struct Delegation {
    pub delegator: Address,
    pub delegate: Address,
    pub proposal_type: Option<ProposalType>,
    pub balance: u128,
    pub created_at: u64,
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
    pub fn new(scope: EmergencyScope, duration_blocks: u64) -> Self {
        Self {
            active: true,
            activated_at: 0,
            expires_at: duration_blocks,
            scope,
            activator: Address::from_bytes([0u8; 32]),
        }
    }

    pub fn is_expired(&self, current_block: u64) -> bool {
        !self.active || current_block > self.expires_at
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EmergencyScope {
    PauseInference,
    DisableModel { model_id: Hash32 },
    FreezeOperator { operator: Address },
    HaltShardRegistry,
    FullEmergency,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GovernanceParameters {
    pub voting_period_blocks: u64,
    pub timelock_period_blocks: u64,
    pub proposal_bond: u128,
    pub emergency_duration_blocks: u64,
    pub thresholds: GovernanceThresholds,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GovernanceThresholds {
    pub parameter_change: VotingThreshold,
    pub runtime_upgrade: VotingThreshold,
    pub treasury: VotingThreshold,
    pub emergency: VotingThreshold,
    pub constitutional: VotingThreshold,
}

impl Default for GovernanceParameters {
    fn default() -> Self {
        Self {
            voting_period_blocks: 10080,
            timelock_period_blocks: 10080,
            proposal_bond: 1000,
            emergency_duration_blocks: 14400,
            thresholds: GovernanceThresholds {
                parameter_change: VotingThreshold::parameter_change(),
                runtime_upgrade: VotingThreshold::runtime_upgrade(),
                treasury: VotingThreshold::treasury(),
                emergency: VotingThreshold::emergency(),
                constitutional: VotingThreshold::constitutional(),
            },
        }
    }
}
