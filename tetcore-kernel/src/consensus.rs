// File: consensus.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Consensus engine implementing BFT (Byzantine Fault Tolerant) consensus.
// Provides round-based consensus with proposal, prevote, and precommit phases,
// validator set management, finality signatures, and fork choice rules.

use crate::{Address, Hash32, KernelError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const BYZANTINE_THRESHOLD: u32 = 3;
pub const DEFAULT_ROUND_TIMEOUT_MS: u64 = 5000;
pub const DEFAULT_PROPOSE_TIMEOUT_MS: u64 = 3000;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoundState {
    Propose,
    Prevote,
    Precommit,
    Finalized,
}

impl RoundState {
    pub fn next(&self) -> Self {
        match self {
            RoundState::Propose => RoundState::Prevote,
            RoundState::Prevote => RoundState::Precommit,
            RoundState::Precommit => RoundState::Finalized,
            RoundState::Finalized => RoundState::Finalized,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub address: Address,
    pub stake: u128,
    pub weight: u64,
    pub is_active: bool,
}

impl ValidatorInfo {
    pub fn new(address: Address, stake: u128) -> Self {
        Self {
            address,
            stake,
            weight: 1,
            is_active: true,
        }
    }
}

pub struct ValidatorSet {
    validators: Vec<ValidatorInfo>,
    set_id: u64,
}

impl ValidatorSet {
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
            set_id: 0,
        }
    }

    pub fn with_validators(validators: Vec<ValidatorInfo>) -> Self {
        Self {
            validators,
            set_id: 0,
        }
    }

    pub fn add(&mut self, validator: ValidatorInfo) {
        self.validators.push(validator);
    }

    pub fn remove(&mut self, address: &Address) -> Option<ValidatorInfo> {
        if let Some(pos) = self.validators.iter().position(|v| &v.address == address) {
            Some(self.validators.remove(pos))
        } else {
            None
        }
    }

    pub fn get(&self, address: &Address) -> Option<&ValidatorInfo> {
        self.validators.iter().find(|v| &v.address == address)
    }

    pub fn get_mut(&mut self, address: &Address) -> Option<&mut ValidatorInfo> {
        self.validators.iter_mut().find(|v| &v.address == address)
    }

    pub fn len(&self) -> usize {
        self.validators.len()
    }

    pub fn is_empty(&self) -> bool {
        self.validators.is_empty()
    }

    pub fn total_stake(&self) -> u128 {
        self.validators.iter().map(|v| v.stake).sum()
    }

    pub fn total_weight(&self) -> u64 {
        self.validators.iter().map(|v| v.weight).sum()
    }

    pub fn set_id(&self) -> u64 {
        self.set_id
    }

    pub fn increment_set_id(&mut self) {
        self.set_id += 1;
    }

    pub fn get_proposer(&self, height: u64, round: u64) -> Option<Address> {
        if self.validators.is_empty() {
            return None;
        }
        let proposer_index = (height + round) as usize % self.validators.len();
        self.validators.get(proposer_index).map(|v| v.address)
    }

    pub fn quorum_size(&self) -> u32 {
        ((self.validators.len() * 2) / 3 + 1) as u32
    }

    pub fn sort_by_stake(&mut self) {
        self.validators.sort_by(|a, b| b.stake.cmp(&a.stake));
    }

    pub fn iter(&self) -> impl Iterator<Item = &ValidatorInfo> {
        self.validators.iter()
    }
}

impl Default for ValidatorSet {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proposal {
    pub height: u64,
    pub round: u64,
    pub block_hash: Hash32,
    pub proposer: Address,
    pub timestamp: u64,
    pub valid_round: Option<u64>,
}

impl Proposal {
    pub fn new(height: u64, round: u64, block_hash: Hash32, proposer: Address) -> Self {
        Self {
            height,
            round,
            block_hash,
            proposer,
            timestamp: 0,
            valid_round: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prevote {
    pub height: u64,
    pub round: u64,
    pub block_hash: Option<Hash32>,
    pub voter: Address,
    pub timestamp: u64,
}

impl Prevote {
    pub fn new(height: u64, round: u64, block_hash: Option<Hash32>, voter: Address) -> Self {
        Self {
            height,
            round,
            block_hash,
            voter,
            timestamp: 0,
        }
    }

    pub fn is_nil(&self) -> bool {
        self.block_hash.is_none()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Precommit {
    pub height: u64,
    pub round: u64,
    pub block_hash: Option<Hash32>,
    pub voter: Address,
    pub timestamp: u64,
}

impl Precommit {
    pub fn new(height: u64, round: u64, block_hash: Option<Hash32>, voter: Address) -> Self {
        Self {
            height,
            round,
            block_hash,
            voter,
            timestamp: 0,
        }
    }

    pub fn is_nil(&self) -> bool {
        self.block_hash.is_none()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoteMessage {
    pub vote_type: VoteType,
    pub height: u64,
    pub round: u64,
    pub block_hash: Option<Hash32>,
    pub sender: Address,
    pub signature: Vec<u8>,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteType {
    Prevote,
    Precommit,
}

impl VoteMessage {
    pub fn new_prevote(
        height: u64,
        round: u64,
        block_hash: Option<Hash32>,
        sender: Address,
    ) -> Self {
        Self {
            vote_type: VoteType::Prevote,
            height,
            round,
            block_hash,
            sender,
            signature: Vec::new(),
            timestamp: 0,
        }
    }

    pub fn new_precommit(
        height: u64,
        round: u64,
        block_hash: Option<Hash32>,
        sender: Address,
    ) -> Self {
        Self {
            vote_type: VoteType::Precommit,
            height,
            round,
            block_hash,
            sender,
            signature: Vec::new(),
            timestamp: 0,
        }
    }

    pub fn is_same_round(&self, other: &VoteMessage) -> bool {
        self.height == other.height && self.round == other.round
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoundVotes {
    pub height: u64,
    pub round: u64,
    pub prevotes: HashMap<Address, Prevote>,
    pub precommits: HashMap<Address, Precommit>,
    pub proposal: Option<Proposal>,
}

impl RoundVotes {
    pub fn new(height: u64, round: u64) -> Self {
        Self {
            height,
            round,
            prevotes: HashMap::new(),
            precommits: HashMap::new(),
            proposal: None,
        }
    }

    pub fn set_proposal(&mut self, proposal: Proposal) {
        self.proposal = Some(proposal);
    }

    pub fn add_prevote(&mut self, prevote: Prevote) -> bool {
        if prevote.height != self.height || prevote.round != self.round {
            return false;
        }
        self.prevotes.insert(prevote.voter, prevote).is_none()
    }

    pub fn add_precommit(&mut self, precommit: Precommit) -> bool {
        if precommit.height != self.height || precommit.round != self.round {
            return false;
        }
        self.precommits.insert(precommit.voter, precommit).is_none()
    }

    pub fn prevote_count(&self, block_hash: &Hash32) -> u32 {
        self.prevotes
            .values()
            .filter(|p| p.block_hash.as_ref() == Some(block_hash))
            .count() as u32
    }

    pub fn precommit_count(&self, block_hash: &Hash32) -> u32 {
        self.precommits
            .values()
            .filter(|p| p.block_hash.as_ref() == Some(block_hash))
            .count() as u32
    }

    pub fn has_quorum_prevotes(&self, quorum_size: u32, block_hash: &Hash32) -> bool {
        self.prevote_count(block_hash) >= quorum_size
    }

    pub fn has_quorum_precommits(&self, quorum_size: u32, block_hash: &Hash32) -> bool {
        self.precommit_count(block_hash) >= quorum_size
    }

    pub fn prevote_power(&self, validator_set: &ValidatorSet) -> u64 {
        self.prevotes
            .values()
            .filter_map(|p| {
                p.block_hash
                    .as_ref()
                    .and_then(|h| validator_set.get(&p.voter).map(|v| v.weight))
            })
            .sum()
    }

    pub fn precommit_power(&self, validator_set: &ValidatorSet) -> u64 {
        self.precommits
            .values()
            .filter_map(|p| {
                p.block_hash
                    .as_ref()
                    .and_then(|h| validator_set.get(&p.voter).map(|v| v.weight))
            })
            .sum()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsensusRound {
    pub height: u64,
    pub round: u64,
    pub state: RoundState,
    pub proposer: Option<Address>,
    pub votes: RoundVotes,
    pub locked_block: Option<Hash32>,
    pub locked_round: Option<u64>,
    pub valid_block: Option<Hash32>,
    pub valid_round: Option<u64>,
}

impl ConsensusRound {
    pub fn new(height: u64, round: u64, proposer: Option<Address>) -> Self {
        Self {
            height,
            round,
            state: RoundState::Propose,
            proposer,
            votes: RoundVotes::new(height, round),
            locked_block: None,
            locked_round: None,
            valid_block: None,
            valid_round: None,
        }
    }

    pub fn set_state(&mut self, state: RoundState) {
        self.state = state;
    }

    pub fn set_proposal(&mut self, proposal: Proposal) {
        self.votes.set_proposal(proposal);
    }

    pub fn add_prevote(&mut self, prevote: Prevote) -> bool {
        self.votes.add_prevote(prevote)
    }

    pub fn add_precommit(&mut self, precommit: Precommit) -> bool {
        self.votes.add_precommit(precommit)
    }

    pub fn lock(&mut self, block_hash: Hash32) {
        self.locked_block = Some(block_hash);
        self.locked_round = Some(self.round);
    }

    pub fn unlock(&mut self) {
        self.locked_block = None;
        self.locked_round = None;
    }

    pub fn set_valid(&mut self, block_hash: Hash32) {
        self.valid_block = Some(block_hash);
        self.valid_round = Some(self.round);
    }
}

pub struct ConsensusEngine {
    pub height: u64,
    pub round: u64,
    pub validator_set: ValidatorSet,
    pub current_round: Option<ConsensusRound>,
    pub locked_block: Option<Hash32>,
    pub locked_round: Option<u64>,
    pub valid_block: Option<Hash32>,
    pub valid_round: Option<u64>,
    pub last_commit: Option<FinalitySignature>,
    pub last_commit_round: Option<u64>,
    history: Vec<ConsensusRound>,
}

impl ConsensusEngine {
    pub fn new() -> Self {
        Self {
            height: 0,
            round: 0,
            validator_set: ValidatorSet::new(),
            current_round: None,
            locked_block: None,
            locked_round: None,
            valid_block: None,
            valid_round: None,
            last_commit: None,
            last_commit_round: None,
            history: Vec::new(),
        }
    }

    pub fn with_validators(validators: Vec<ValidatorInfo>) -> Self {
        Self {
            height: 0,
            round: 0,
            validator_set: ValidatorSet::with_validators(validators),
            current_round: None,
            locked_block: None,
            locked_round: None,
            valid_block: None,
            valid_round: None,
            last_commit: None,
            last_commit_round: None,
            history: Vec::new(),
        }
    }

    pub fn start_new_round(&mut self, height: u64, round: u64) {
        if height > self.height {
            self.height = height;
            self.round = 0;
            self.locked_block = None;
            self.locked_round = None;
            self.valid_block = None;
            self.valid_round = None;
        } else {
            self.round = round;
        }

        let proposer = self.validator_set.get_proposer(self.height, self.round);
        let mut consensus_round = ConsensusRound::new(self.height, self.round, proposer);
        consensus_round.locked_block = self.locked_block;
        consensus_round.locked_round = self.locked_round;
        consensus_round.valid_block = self.valid_block;
        consensus_round.valid_round = self.valid_round;

        self.current_round = Some(consensus_round);
    }

    pub fn get_proposer(&self) -> Option<Address> {
        self.validator_set.get_proposer(self.height, self.round)
    }

    pub fn is_proposer(&self, address: &Address) -> bool {
        self.get_proposer().as_ref() == Some(address)
    }

    pub fn receive_proposal(&mut self, proposal: Proposal) -> bool {
        if let Some(ref mut round) = self.current_round {
            if round.height == proposal.height && round.round == proposal.round {
                round.set_proposal(proposal);
                return true;
            }
        }
        false
    }

    pub fn receive_prevote(&mut self, prevote: Prevote) -> bool {
        if let Some(ref mut round) = self.current_round {
            if round.height == prevote.height && round.round == prevote.round {
                if round.add_prevote(prevote) {
                    self.check_prevote_quorum();
                    return true;
                }
            }
        }
        false
    }

    pub fn receive_precommit(&mut self, precommit: Precommit) -> bool {
        if let Some(ref mut round) = self.current_round {
            if round.height == precommit.height && precommit.round == round.round {
                if round.add_precommit(precommit) {
                    self.check_precommit_quorum();
                    return true;
                }
            }
        }
        false
    }

    fn check_prevote_quorum(&mut self) {
        let proposal_block_hash = match &mut self.current_round {
            Some(r) => {
                let proposal = match &r.votes.proposal {
                    Some(p) => p.block_hash,
                    None => return,
                };
                let quorum_size = self.validator_set.quorum_size();
                if r.votes.has_quorum_prevotes(quorum_size, &proposal) {
                    r.set_state(RoundState::Precommit);
                    proposal
                } else {
                    return;
                }
            }
            None => return,
        };

        if self.locked_block.is_none() {
            self.locked_block = Some(proposal_block_hash);
            self.locked_round = Some(self.round);
        }

        self.valid_block = Some(proposal_block_hash);
        self.valid_round = Some(self.round);
    }

    fn check_precommit_quorum(&mut self) {
        let round = match &self.current_round {
            Some(r) => r,
            None => return,
        };

        let proposal_block_hash = match &round.votes.proposal {
            Some(p) => &p.block_hash,
            None => return,
        };

        let quorum_size = self.validator_set.quorum_size();

        if round
            .votes
            .has_quorum_precommits(quorum_size, proposal_block_hash)
        {
            if let Some(ref mut r) = self.current_round {
                r.set_state(RoundState::Finalized);
            }
        }
    }

    pub fn has_finality(&self) -> bool {
        self.current_round
            .as_ref()
            .map(|r| r.state == RoundState::Finalized)
            .unwrap_or(false)
    }

    pub fn get_finalized_block(&self) -> Option<Hash32> {
        if self.has_finality() {
            self.current_round
                .as_ref()
                .and_then(|r| r.votes.proposal.as_ref())
                .map(|p| p.block_hash)
        } else {
            None
        }
    }

    pub fn add_validator(&mut self, validator: ValidatorInfo) {
        self.validator_set.add(validator);
    }

    pub fn remove_validator(&mut self, address: &Address) -> Option<ValidatorInfo> {
        self.validator_set.remove(address)
    }

    pub fn update_validator_stake(&mut self, address: &Address, new_stake: u128) {
        if let Some(validator) = self.validator_set.get_mut(address) {
            validator.stake = new_stake;
        }
    }

    pub fn quorum_size(&self) -> u32 {
        self.validator_set.quorum_size()
    }

    pub fn validator_count(&self) -> usize {
        self.validator_set.len()
    }

    pub fn start_next_height(&mut self) {
        self.height += 1;
        self.round = 0;
        self.locked_block = None;
        self.locked_round = None;
        self.valid_block = None;
        self.valid_round = None;
        self.start_new_round(self.height, self.round);
    }
}

impl Default for ConsensusEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorSignature {
    pub validator: Address,
    pub signature: Vec<u8>,
    pub block_hash: Hash32,
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

    pub fn add_signature(&mut self, validator: Address, signature: Vec<u8>) {
        self.signatures.push(ValidatorSignature {
            validator,
            signature,
            block_hash: self.block_hash,
        });
    }

    pub fn quorum_reached(&self, total_validators: u32) -> bool {
        let quorum = (total_validators * 2) / 3 + 1;
        self.signatures.len() as u32 >= quorum
    }

    pub fn has_signature_from(&self, validator: &Address) -> bool {
        self.signatures.iter().any(|s| &s.validator == validator)
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForkChoice {
    Left,
    Right,
    None,
}

pub struct ForkChoiceRule;

impl ForkChoiceRule {
    pub fn choose(
        current_head: Hash32,
        current_height: u64,
        left_hash: Hash32,
        left_height: u64,
        right_hash: Hash32,
        right_height: u64,
    ) -> ForkChoice {
        if left_height > right_height && left_height > current_height {
            ForkChoice::Left
        } else if right_height > left_height && right_height > current_height {
            ForkChoice::Right
        } else if left_height == right_height {
            if left_hash.0 > right_hash.0 {
                ForkChoice::Left
            } else {
                ForkChoice::Right
            }
        } else {
            ForkChoice::None
        }
    }

    pub fn validate_block(
        block_hash: Hash32,
        parent_hash: Hash32,
        validator_set: &ValidatorSet,
        finality_signatures: &[FinalitySignature],
    ) -> bool {
        if validator_set.is_empty() {
            return false;
        }

        let quorum = validator_set.quorum_size() as usize;
        let mut signature_count = 0;

        for sig in finality_signatures {
            if sig.block_hash == block_hash && sig.quorum_reached(validator_set.len() as u32) {
                signature_count += sig.signatures.len();
            }
        }

        signature_count >= quorum
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsensusMetrics {
    pub height: u64,
    pub round: u64,
    pub prevotes_count: u32,
    pub precommits_count: u32,
    pub validators_online: u32,
    pub last_block_time_ms: u64,
    pub block_time_avg_ms: u64,
}

impl ConsensusMetrics {
    pub fn new() -> Self {
        Self {
            height: 0,
            round: 0,
            prevotes_count: 0,
            precommits_count: 0,
            validators_online: 0,
            last_block_time_ms: 0,
            block_time_avg_ms: 0,
        }
    }
}

impl Default for ConsensusMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_validator(i: u8) -> ValidatorInfo {
        let mut bytes = [0u8; 32];
        bytes[31] = i;
        ValidatorInfo::new(Address(bytes), (i as u128) * 1000)
    }

    #[test]
    fn test_validator_set_proposer_selection() {
        let mut validators = Vec::new();
        for i in 1..=4 {
            validators.push(create_validator(i));
        }
        let mut validator_set = ValidatorSet::with_validators(validators);

        let proposer0 = validator_set.get_proposer(0, 0);
        let proposer1 = validator_set.get_proposer(0, 1);
        let proposer2 = validator_set.get_proposer(0, 2);
        let proposer3 = validator_set.get_proposer(0, 3);

        assert!(proposer0.is_some());
        assert!(proposer1.is_some());
        assert!(proposer2.is_some());
        assert!(proposer3.is_some());
    }

    #[test]
    fn test_quorum_size() {
        let validators = vec![
            create_validator(1),
            create_validator(2),
            create_validator(3),
        ];
        let validator_set = ValidatorSet::with_validators(validators);
        assert_eq!(validator_set.quorum_size(), 3);

        let validators = vec![
            create_validator(1),
            create_validator(2),
            create_validator(3),
            create_validator(4),
            create_validator(5),
            create_validator(6),
        ];
        let validator_set = ValidatorSet::with_validators(validators);
        assert_eq!(validator_set.quorum_size(), 5);
    }

    #[test]
    fn test_consensus_engine_proposer() {
        let validators = vec![
            create_validator(1),
            create_validator(2),
            create_validator(3),
        ];
        let mut engine = ConsensusEngine::with_validators(validators);

        engine.start_new_round(1, 0);
        let proposer = engine.get_proposer();

        assert!(proposer.is_some());
    }

    #[test]
    fn test_fork_choice_rule() {
        let current_head = Hash32::from_slice(&[1u8; 32]);
        let left_hash = Hash32::from_slice(&[2u8; 32]);
        let right_hash = Hash32::from_slice(&[3u8; 32]);

        let choice = ForkChoiceRule::choose(current_head, 10, left_hash, 12, right_hash, 11);
        assert_eq!(choice, ForkChoice::Left);
    }
}
