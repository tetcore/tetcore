// File: consensus.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Consensus primitives for Tetcore including Validator, AuthoritySet,
// ValidatorSet, BFT vote types, Commit messages, RoundState, and
// consensus parameters. Supports BFT consensus with quorum detection
// and validator set management.

use crate::crypto::Address;
use crate::hash::Hash32;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Validator {
    pub account_id: Address,
    pub stake: u128,
    pub commission: u8,
    pub active: bool,
    pub jailed: bool,
    pub metadata: ValidatorMetadata,
}

impl Validator {
    pub fn new(account_id: Address, stake: u128) -> Self {
        Self {
            account_id,
            stake,
            commission: 0,
            active: true,
            jailed: false,
            metadata: ValidatorMetadata::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ValidatorMetadata {
    pub name: String,
    pub website: String,
    pub contact: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Authority {
    pub address: Address,
    pub weight: u64,
    pub enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthoritySet {
    pub authorities: Vec<Authority>,
    pub set_id: u64,
}

impl AuthoritySet {
    pub fn new(authorities: Vec<Authority>) -> Self {
        Self {
            authorities,
            set_id: 0,
        }
    }

    pub fn add_authority(&mut self, authority: Authority) {
        self.authorities.push(authority);
    }

    pub fn remove_authority(&mut self, address: &Address) {
        self.authorities.retain(|a| &a.address != address);
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Vote {
    Yes,
    No,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Commit {
    pub height: u64,
    pub round: u64,
    pub block_hash: Hash32,
    pub validator_addresses: Vec<Address>,
    pub signature_threshold: u32,
}

impl Commit {
    pub fn new(height: u64, round: u64, block_hash: Hash32) -> Self {
        Self {
            height,
            round,
            block_hash,
            validator_addresses: Vec::new(),
            signature_threshold: 0,
        }
    }

    pub fn add_signer(&mut self, address: Address) {
        self.validator_addresses.push(address);
    }

    pub fn has_quorum(&self, total_validators: u32) -> bool {
        let quorum = (total_validators * 2) / 3 + 1;
        self.validator_addresses.len() as u32 >= quorum
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoteMessage {
    pub height: u64,
    pub round: u64,
    pub block_id: Option<Hash32>,
    pub voter: Address,
    pub vote: Vote,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BFTState {
    Propose,
    Vote,
    Commit,
    Finalized,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoundState {
    pub height: u64,
    pub round: u64,
    pub state: BFTState,
    pub proposer: Option<Address>,
    pub votes: Vec<VoteMessage>,
    pub locked_block: Option<Hash32>,
}

impl RoundState {
    pub fn new(height: u64, round: u64) -> Self {
        Self {
            height,
            round,
            state: BFTState::Propose,
            proposer: None,
            votes: Vec::new(),
            locked_block: None,
        }
    }

    pub fn add_vote(&mut self, vote: VoteMessage) {
        if !self.votes.iter().any(|v| v.voter == vote.voter) {
            self.votes.push(vote);
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsensusParams {
    pub max_validators: u32,
    pub min_validators: u32,
    pub block_time_ms: u64,
    pub propose_timeout_ms: u64,
    pub vote_timeout_ms: u64,
    pub finality_threshold: u32,
}

impl Default for ConsensusParams {
    fn default() -> Self {
        Self {
            max_validators: 100,
            min_validators: 4,
            block_time_ms: 5000,
            propose_timeout_ms: 3000,
            vote_timeout_ms: 2000,
            finality_threshold: 2,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorSet {
    pub validators: Vec<Validator>,
    pub total_stake: u128,
}

impl ValidatorSet {
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
            total_stake: 0,
        }
    }

    pub fn add(&mut self, validator: Validator) {
        self.total_stake = self.total_stake.saturating_add(validator.stake);
        self.validators.push(validator);
    }

    pub fn remove(&mut self, address: &Address) -> Option<Validator> {
        if let Some(pos) = self
            .validators
            .iter()
            .position(|v| &v.account_id == address)
        {
            let validator = self.validators.remove(pos);
            self.total_stake = self.total_stake.saturating_sub(validator.stake);
            Some(validator)
        } else {
            None
        }
    }
}

impl Default for ValidatorSet {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockHeader {
    pub parent_hash: Hash32,
    pub number: u64,
    pub state_root: Hash32,
    pub transaction_root: Hash32,
    pub receipts_root: Hash32,
    pub validator_set_id: u64,
    pub timestamp: u64,
    pub extra_data: Vec<u8>,
}

impl BlockHeader {
    pub fn new(parent_hash: Hash32, number: u64) -> Self {
        Self {
            parent_hash,
            number,
            state_root: Hash32::empty(),
            transaction_root: Hash32::empty(),
            receipts_root: Hash32::empty(),
            validator_set_id: 0,
            timestamp: 0,
            extra_data: Vec::new(),
        }
    }

    pub fn hash(&self) -> Hash32 {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(self.parent_hash.as_bytes());
        hasher.update(&self.number.to_le_bytes());
        hasher.update(self.state_root.as_bytes());
        hasher.update(self.transaction_root.as_bytes());
        hasher.update(self.receipts_root.as_bytes());
        hasher.update(&self.validator_set_id.to_le_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        Hash32(hash)
    }
}

use sha2::{Digest, Sha256};
