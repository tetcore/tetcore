// File: transactions.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Extended transaction types for Tetcore including TransactionV1,
// TransactionAction variants (Call, Create, Create2, Invoke, SubmitPrompt,
// SubmitReceipt, RegisterModel, etc.), UncheckedExtrinsic, CheckedExtrinsic,
// SignedExtra, Event, EventRecord, and event types for all modules.

use crate::crypto::{Address, Signature};
use crate::hash::Hash32;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionV1 {
    pub sender: Address,
    pub nonce: u64,
    pub action: TransactionAction,
    pub payload: Vec<u8>,
    pub gas_limit: u64,
    pub gas_price: u128,
    pub value: u128,
    pub signature: Option<Signature>,
}

impl TransactionV1 {
    pub fn new(sender: Address) -> Self {
        Self {
            sender,
            nonce: 0,
            action: TransactionAction::Call(Address::from_bytes([0u8; 32])),
            payload: Vec::new(),
            gas_limit: 0,
            gas_price: 0,
            value: 0,
            signature: None,
        }
    }

    pub fn sign(&mut self, signature: Signature) {
        self.signature = Some(signature);
    }

    pub fn is_signed(&self) -> bool {
        self.signature.is_some()
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.sender.as_bytes());
        bytes.extend_from_slice(&self.nonce.to_le_bytes());
        bytes.extend_from_slice(&self.action.encode());
        bytes.extend_from_slice(&self.gas_limit.to_le_bytes());
        bytes.extend_from_slice(&self.gas_price.to_le_bytes());
        bytes.extend_from_slice(&self.value.to_le_bytes());
        bytes.extend_from_slice(&self.payload.len().to_le_bytes());
        bytes.extend_from_slice(&self.payload);
        bytes
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TransactionAction {
    Call(Address),
    CallReadOnly(Address),
    Create,
    Create2 {
        code_hash: Hash32,
        salt: Vec<u8>,
    },
    Invoke {
        module: String,
        method: String,
    },
    InvokeReadOnly {
        module: String,
        method: String,
    },
    SubmitPrompt {
        model_id: Hash32,
        version: u32,
        prompt_commitment: Hash32,
    },
    SubmitReceipt {
        prompt_id: Hash32,
        inference_output: Vec<u8>,
    },
    RegisterModel {
        shard_root: Hash32,
        shard_count: u32,
    },
    UpdateModel {
        model_id: Hash32,
        new_shard_root: Hash32,
    },
    CreateVault {
        model_id: Hash32,
    },
    StakeVault {
        vault_id: Hash32,
        amount: u128,
    },
    UnstakeVault {
        vault_id: Hash32,
        shares: u128,
    },
    SubmitProposal {
        proposal_type: u8,
        payload: Vec<u8>,
    },
    Vote {
        proposal_id: Hash32,
        vote: u8,
    },
    Transfer(Address),
    Lock {
        amount: u128,
        duration: u64,
    },
    Unlock {
        amount: u128,
    },
    Batch(Vec<TransactionAction>),
}

impl TransactionAction {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            TransactionAction::Call(addr) => {
                let mut bytes = vec![0x00];
                bytes.extend_from_slice(addr.as_bytes());
                bytes
            }
            TransactionAction::CallReadOnly(addr) => {
                let mut bytes = vec![0x01];
                bytes.extend_from_slice(addr.as_bytes());
                bytes
            }
            TransactionAction::Create => vec![0x10],
            TransactionAction::Create2 { code_hash, salt } => {
                let mut bytes = vec![0x11];
                bytes.extend_from_slice(code_hash.as_bytes());
                bytes.extend_from_slice(&salt.len().to_le_bytes());
                bytes.extend_from_slice(salt);
                bytes
            }
            TransactionAction::Invoke { module, method } => {
                let mut bytes = vec![0x20];
                bytes.extend_from_slice(&(module.len() as u32).to_le_bytes());
                bytes.extend_from_slice(module.as_bytes());
                bytes.extend_from_slice(&(method.len() as u32).to_le_bytes());
                bytes.extend_from_slice(method.as_bytes());
                bytes
            }
            _ => Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UncheckedExtrinsic {
    pub address: Address,
    pub call: TransactionAction,
    pub signature: Option<(Signature, Address)>,
    pub nonce: u64,
    pub gas_limit: u64,
    pub gas_price: u128,
    pub value: u128,
}

impl UncheckedExtrinsic {
    pub fn new(address: Address, call: TransactionAction) -> Self {
        Self {
            address,
            call,
            signature: None,
            nonce: 0,
            gas_limit: 0,
            gas_price: 0,
            value: 0,
        }
    }

    pub fn sign(&mut self, signature: Signature, signer: Address) {
        self.signature = Some((signature, signer));
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckedExtrinsic {
    pub signer: Address,
    pub call: TransactionAction,
    pub nonce: u64,
    pub gas_limit: u64,
    pub gas_price: u128,
    pub value: u128,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedExtra {
    pub nonce: u64,
    pub fee: u128,
    pub tip: u128,
}

impl SignedExtra {
    pub fn new(nonce: u64) -> Self {
        Self {
            nonce,
            fee: 0,
            tip: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionMetadata {
    pub transaction_hash: Hash32,
    pub block_number: u64,
    pub block_hash: Hash32,
    pub transaction_index: u32,
    pub from: Address,
    pub to: Option<Address>,
    pub value: u128,
    pub fee: u128,
    pub gas_used: u64,
    pub status: TransactionStatus,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    InBlock,
    Finalized,
    Replaced,
    Dropped,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventRecord {
    pub phase: Phase,
    pub event: Event,
    pub topics: Vec<Hash32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Phase {
    ApplyExtrinsic(u32),
    Finalization,
    Initialization,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    System(SystemEvent),
    Balances(BalanceEvent),
    Inference(InferenceEvent),
    Contracts(ContractEvent),
    Governance(GovernanceEvent),
    Vault(VaultEvent),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SystemEvent {
    ExtrinsicSuccess,
    ExtrinsicFailed { error: String },
    CodeUpdated,
    RuntimeUpdated,
    Stalled { block_number: u64 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BalanceEvent {
    Transfer {
        from: Address,
        to: Address,
        amount: u128,
    },
    Deposit {
        who: Address,
        amount: u128,
    },
    Withdraw {
        who: Address,
        amount: u128,
    },
    Locked {
        who: Address,
        amount: u128,
    },
    Unlocked {
        who: Address,
        amount: u128,
    },
    Reserved {
        who: Address,
        amount: u128,
    },
    Unreserved {
        who: Address,
        amount: u128,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InferenceEvent {
    PromptSubmitted {
        prompt_id: Hash32,
        model_id: Hash32,
    },
    ReceiptSubmitted {
        receipt_id: Hash32,
        prompt_id: Hash32,
    },
    ModelRegistered {
        model_id: Hash32,
        owner: Address,
    },
    ModelUpdated {
        model_id: Hash32,
    },
    RevenueDistributed {
        prompt_id: Hash32,
        amount: u128,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ContractEvent {
    CodeStored {
        code_hash: Hash32,
    },
    ContractCreated {
        contract_id: Hash32,
        creator: Address,
    },
    ContractCalled {
        contract_id: Hash32,
        caller: Address,
    },
    ContractEmitted {
        contract_id: Hash32,
        data: Vec<u8>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GovernanceEvent {
    ProposalSubmitted {
        proposal_id: Hash32,
    },
    VoteCast {
        proposal_id: Hash32,
        voter: Address,
        vote: u8,
    },
    ProposalApproved {
        proposal_id: Hash32,
    },
    ProposalRejected {
        proposal_id: Hash32,
    },
    ProposalExecuted {
        proposal_id: Hash32,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum VaultEvent {
    VaultCreated {
        vault_id: Hash32,
        model_id: Hash32,
    },
    Staked {
        vault_id: Hash32,
        staker: Address,
        amount: u128,
    },
    Unstaked {
        vault_id: Hash32,
        staker: Address,
        amount: u128,
    },
    RewardPaid {
        vault_id: Hash32,
        staker: Address,
        amount: u128,
    },
}
