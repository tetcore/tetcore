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
pub enum ConsensusVote {
    Yes,
    No,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsensusCommit {
    pub height: u64,
    pub round: u64,
    pub block_hash: Hash32,
    pub validator_addresses: Vec<Address>,
    pub signature_threshold: u32,
}

impl ConsensusCommit {
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
    pub vote: ConsensusVote,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BFTState {
    Propose,
    Prevote,
    Precommit,
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
pub struct ConsensusProposal {
    pub height: u64,
    pub round: u64,
    pub block_hash: Hash32,
    pub proposer: Address,
    pub timestamp: u64,
}

impl ConsensusProposal {
    pub fn new(height: u64, round: u64, block_hash: Hash32, proposer: Address) -> Self {
        Self {
            height,
            round,
            block_hash,
            proposer,
            timestamp: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prevote {
    pub height: u64,
    pub round: u64,
    pub block_id: Option<Hash32>,
    pub voter: Address,
    pub timestamp: u64,
}

impl Prevote {
    pub fn new(height: u64, round: u64, block_id: Option<Hash32>, voter: Address) -> Self {
        Self {
            height,
            round,
            block_id,
            voter,
            timestamp: 0,
        }
    }

    pub fn is_nil(&self) -> bool {
        self.block_id.is_none()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Precommit {
    pub height: u64,
    pub round: u64,
    pub block_id: Option<Hash32>,
    pub voter: Address,
    pub timestamp: u64,
}

impl Precommit {
    pub fn new(height: u64, round: u64, block_id: Option<Hash32>, voter: Address) -> Self {
        Self {
            height,
            round,
            block_id,
            voter,
            timestamp: 0,
        }
    }

    pub fn is_nil(&self) -> bool {
        self.block_id.is_none()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BFTMessage {
    pub message_type: BFTMessageType,
    pub height: u64,
    pub round: u64,
    pub block_id: Option<Hash32>,
    pub sender: Address,
    pub signature: Vec<u8>,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BFTMessageType {
    Proposal,
    Prevote,
    Precommit,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EquivocationProof {
    pub round: u64,
    pub height: u64,
    pub equivocator: Address,
    pub first_message: BFTMessage,
    pub second_message: BFTMessage,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorSignature {
    pub validator: Address,
    pub signature: Vec<u8>,
    pub block_hash: Hash32,
}

impl ValidatorSignature {
    pub fn new(validator: Address, signature: Vec<u8>, block_hash: Hash32) -> Self {
        Self {
            validator,
            signature,
            block_hash,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FinalitySignature {
    pub block_hash: Hash32,
    pub block_number: u64,
    pub validator_set_id: u64,
    pub signatures: Vec<ValidatorSignature>,
    pub is_commit: bool,
}

impl FinalitySignature {
    pub fn new(block_hash: Hash32, block_number: u64, validator_set_id: u64) -> Self {
        Self {
            block_hash,
            block_number,
            validator_set_id,
            signatures: Vec::new(),
            is_commit: false,
        }
    }

    pub fn add_signature(&mut self, signature: ValidatorSignature) {
        if signature.block_hash == self.block_hash {
            self.signatures.push(signature);
        }
    }

    pub fn quorum_reached(&self, total_validators: u32) -> bool {
        let quorum = (total_validators * 2) / 3 + 1;
        self.signatures.len() as u32 >= quorum
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorSetChange {
    pub set_id: u64,
    pub added: Vec<Address>,
    pub removed: Vec<Address>,
    pub updated_stake: Vec<(Address, u128)>,
    pub activation_block: u64,
}

impl ValidatorSetChange {
    pub fn new(set_id: u64, activation_block: u64) -> Self {
        Self {
            set_id,
            added: Vec::new(),
            removed: Vec::new(),
            updated_stake: Vec::new(),
            activation_block,
        }
    }

    pub fn add_validator(&mut self, validator: Address) {
        if !self.added.contains(&validator) {
            self.added.push(validator);
        }
    }

    pub fn remove_validator(&mut self, validator: Address) {
        if !self.removed.contains(&validator) {
            self.removed.push(validator);
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsensusRound {
    pub height: u64,
    pub round: u64,
    pub proposer: Address,
    pub state: ConsensusRoundState,
    pub locked_value: Option<Hash32>,
    pub locked_round: Option<u64>,
    pub valid_value: Option<Hash32>,
    pub valid_round: Option<u64>,
    pub prevotes: Vec<Prevote>,
    pub precommits: Vec<Precommit>,
    pub proposal: Option<ConsensusProposal>,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsensusRoundState {
    Propose,
    Prevote,
    Precommit,
    Finalized,
}

impl ConsensusRound {
    pub fn new(height: u64, round: u64, proposer: Address) -> Self {
        Self {
            height,
            round,
            proposer,
            state: ConsensusRoundState::Propose,
            locked_value: None,
            locked_round: None,
            valid_value: None,
            valid_round: None,
            prevotes: Vec::new(),
            precommits: Vec::new(),
            proposal: None,
        }
    }

    pub fn add_prevote(&mut self, prevote: Prevote) {
        if !self.prevotes.iter().any(|p| p.voter == prevote.voter) {
            self.prevotes.push(prevote);
        }
    }

    pub fn add_precommit(&mut self, precommit: Precommit) {
        if !self.precommits.iter().any(|p| p.voter == precommit.voter) {
            self.precommits.push(precommit);
        }
    }

    pub fn prevote_count(&self, block_hash: &Hash32) -> u32 {
        self.prevotes
            .iter()
            .filter(|p| p.block_id.as_ref() == Some(block_hash))
            .count() as u32
    }

    pub fn precommit_count(&self, block_hash: &Hash32) -> u32 {
        self.precommits
            .iter()
            .filter(|p| p.block_id.as_ref() == Some(block_hash))
            .count() as u32
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsensusTimestamps {
    pub propose_start: u64,
    pub propose_end: u64,
    pub prevote_end: u64,
    pub precommit_end: u64,
}

impl ConsensusTimestamps {
    pub fn new(
        propose_timeout_ms: u64,
        prevote_timeout_ms: u64,
        precommit_timeout_ms: u64,
    ) -> Self {
        Self {
            propose_start: 0,
            propose_end: propose_timeout_ms,
            prevote_end: propose_timeout_ms + prevote_timeout_ms,
            precommit_end: propose_timeout_ms + prevote_timeout_ms + precommit_timeout_ms,
        }
    }

    pub fn is_expired(&self, current_time: u64, stage: ConsensusRoundState) -> bool {
        match stage {
            ConsensusRoundState::Propose => current_time >= self.propose_end,
            ConsensusRoundState::Prevote => current_time >= self.prevote_end,
            ConsensusRoundState::Precommit => current_time >= self.precommit_end,
            ConsensusRoundState::Finalized => true,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidatorStatus {
    Active,
    Inactive,
    Pending,
    Jailed,
    Slashed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorRanking {
    pub rank: u32,
    pub account_id: Address,
    pub stake: u128,
    pub commission: u8,
    pub active: bool,
}

impl ValidatorRanking {
    pub fn rank(validators: &[Validator]) -> Vec<Self> {
        let mut ranked: Vec<ValidatorRanking> = validators
            .iter()
            .filter(|v| v.active && !v.jailed)
            .enumerate()
            .map(|(i, v)| Self {
                rank: i as u32 + 1,
                account_id: v.account_id,
                stake: v.stake,
                commission: v.commission,
                active: v.active,
            })
            .collect();
        ranked.sort_by(|a, b| b.stake.cmp(&a.stake));
        for (i, r) in ranked.iter_mut().enumerate() {
            r.rank = i as u32 + 1;
        }
        ranked
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SlashingInfo {
    pub validator: Address,
    pub offense_type: SlashingOffense,
    pub slash_amount: u128,
    pub evidence: Vec<u8>,
    pub block_number: u64,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlashingOffense {
    DoubleVote,
    Equivocation,
    Unresponsive,
    InvalidBlock,
    InvalidVote,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorPerf {
    pub account_id: Address,
    pub blocks_proposed: u64,
    pub blocks_authored: u64,
    pub prevotes_cast: u64,
    pub precommits_cast: u64,
    pub missed_votes: u64,
    pub offline_events: u64,
    pub slashed: bool,
}

impl ValidatorPerf {
    pub fn new(account_id: Address) -> Self {
        Self {
            account_id,
            blocks_proposed: 0,
            blocks_authored: 0,
            prevotes_cast: 0,
            precommits_cast: 0,
            missed_votes: 0,
            offline_events: 0,
            slashed: false,
        }
    }

    pub fn uptime_percentage(&self) -> u32 {
        let total = self.prevotes_cast + self.precommits_cast + self.missed_votes;
        if total == 0 {
            return 100;
        }
        ((self.prevotes_cast + self.precommits_cast) as u64 * 100 / total) as u32
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsensusState {
    pub height: u64,
    pub round: u64,
    pub locked_block: Option<Hash32>,
    pub valid_block: Option<Hash32>,
    pub last_commit: Option<FinalitySignature>,
    pub proposer: Option<Address>,
}

impl ConsensusState {
    pub fn new() -> Self {
        Self {
            height: 0,
            round: 0,
            locked_block: None,
            valid_block: None,
            last_commit: None,
            proposer: None,
        }
    }
}

impl Default for ConsensusState {
    fn default() -> Self {
        Self::new()
    }
}
