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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proposal {
    pub id: Hash32,
    pub proposer: Address,
    pub title: String,
    pub description: String,
    pub payload: Vec<u8>,
    pub state: ProposalState,
    pub votes_for: u128,
    pub votes_against: u128,
    pub voting_start: u64,
    pub voting_end: u64,
    pub executed: bool,
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
}

impl GovernanceModule {
    pub fn new() -> Self {
        Self {
            proposals: HashMap::new(),
            votes: HashMap::new(),
            proposal_counter: 0,
        }
    }

    pub fn create_proposal(
        &mut self,
        proposer: Address,
        title: String,
        description: String,
        payload: Vec<u8>,
        voting_end: u64,
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

        let proposal = Proposal {
            id: Hash32(id),
            proposer,
            title,
            description,
            payload,
            state: ProposalState::Pending,
            votes_for: 0,
            votes_against: 0,
            voting_start: 0,
            voting_end,
            executed: false,
        };

        self.proposals.insert(Hash32(id), proposal);

        Ok(Hash32(id))
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
    ) -> Result<ProposalState, RuntimeError> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or(RuntimeError::InvalidState)?;

        if proposal.votes_for > proposal.votes_against {
            proposal.state = ProposalState::Passed;
        } else {
            proposal.state = ProposalState::Rejected;
        }

        Ok(proposal.state.clone())
    }

    pub fn execute_proposal(&mut self, proposal_id: &Hash32) -> Result<(), RuntimeError> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or(RuntimeError::InvalidState)?;

        if proposal.state != ProposalState::Passed {
            return Err(RuntimeError::InvalidState);
        }

        if proposal.executed {
            return Err(RuntimeError::InvalidState);
        }

        proposal.state = ProposalState::Executed;
        proposal.executed = true;

        Ok(())
    }

    pub fn get_proposal(&self, proposal_id: &Hash32) -> Option<&Proposal> {
        self.proposals.get(proposal_id)
    }

    pub fn all_proposals(&self) -> &HashMap<Hash32, Proposal> {
        &self.proposals
    }
}

impl Default for GovernanceModule {
    fn default() -> Self {
        Self::new()
    }
}
