// File: lib.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Tetcore primitives library providing foundational types for the
// deterministic state machine. Exports modules for account, api, arithmetic,
// block building, consensus, contracts, crypto, economics, governance,
// hash, inference, runtime, storage, traits, transactions, and
// validator discovery.

pub mod account;
pub mod api;
pub mod arithmetic;
pub mod block;
pub mod blockchain;
pub mod consensus;
pub mod contracts;
pub mod core;
pub mod crypto;
pub mod economics;
pub mod governance;
pub mod hash;
pub mod inference;
pub mod runtime;
pub mod storage;
pub mod traits;
pub mod transactions;
pub mod validator_discovery;

pub use account::{AccountData, AccountId};
pub use api::{
    ApiEndpoint, ApiMethod, ApiRegistry, ApiRequest, ApiResponse, ApiResponseStatus, ApiRoute,
    AuthLevel, AvailabilityChallenge, ChallengeStatus, PromptCommitment, PromptEntry, PromptStatus,
    RateLimit, RelayConfig, ShardProof, ShardStorageEntry, ShardStorageLedger,
};
pub use arithmetic::{FixedU128, Gas, PerU16, Perbill, Ratio, SafeAdd, SafeDiv, SafeMul, SafeSub};
pub use block::{
    BlockBuilder, BlockBuilderConfig, BlockBuilderError, BlockBuildingStrategy, BlockExport,
    BlockImportContext, BlockMetadata, BlockStats, BuildableBlock, EmptyBlockStrategy,
    Justification, SealedBlock, TimedBlockStrategy,
};
pub use blockchain::{
    BlockAvailability, BlockFields, BlockRequest, BlockRequestDirection, BlockRequestStart,
    BlockResponse, BlockResponseItem, BlockStatus, Blockchain, BlockchainImportContext, ChainBlock,
    ChainConfig, ChainProperties, ChainSegment, ChainStorage, ChainType, Fork, ForkChoice,
    ForkConfig, ForkMigration, HeaderMetadata, ImportError, ImportErrorKind, ImportResult,
    SyncState, SyncType,
};
pub use consensus::{
    Authority, AuthoritySet, BFTMessage, BFTMessageType, BFTState, ConsensusCommit,
    ConsensusParams, ConsensusProposal, ConsensusRound, ConsensusRoundState, ConsensusState,
    ConsensusTimestamps, ConsensusVote, EquivocationProof, FinalitySignature, RoundState,
    SlashingInfo, SlashingOffense, Validator, ValidatorMetadata, ValidatorPerf, ValidatorRanking,
    ValidatorSet, ValidatorSetChange, ValidatorSignature, ValidatorStatus, VoteMessage,
};
pub use contracts::{
    Contract, ContractCall, ContractCode, ContractEvent, ContractLog, ContractMetadata,
    ContractMethod, ContractResult, ContractStorage, ContractType,
};
pub use core::{
    BlockNumber, ChainId, Constants, CoreChainType, EventEmitter, EventFilter, EventId, EventPhase,
    Nonce, RuntimeVersion, SystemError, SystemErrorKind, SystemEvent, SystemEventType, SystemInfo,
    SystemProperties, SystemVersion, Timestamp,
};
pub use crypto::{Address, PrivateKey, PublicKey, Signature};
pub use economics::{
    Escrow, EscrowStatus, FeeParameters, GasSchedule, TokenBalance, TokenSupply, Transfer, Vault,
    VaultPosition, DECIMALS,
};
pub use governance::{
    Delegation, EmergencyPowers, EmergencyScope, GovernanceParameters, GovernanceThresholds,
    Proposal, ProposalStatus, ProposalType, Vote, VoteChoice, VotingThreshold,
};
pub use hash::Hash32;
pub use inference::{
    InferenceRequest, InferenceResponse, Model, ModelState, PricingMode, PricingPolicy, Prompt,
    Receipt, RevenueSplit, ShardCommitment, ShardMetadata,
};
pub use runtime::{
    Block, Digest, DigestItem, DispatchError, Header, Log, Receipt as RuntimeReceipt, Transaction,
    TransactionAction, TransactionOutcome,
};
pub use storage::{ChildInfo, ChildStorage, Storage, StorageProof};
pub use traits::{
    ApplyTransaction, BlockTrait, Commit, DispatchResult, Dispatchable, HeaderTrait, MerkleTrie,
    Module, OnFinalize, OnInitialize, OnRuntimeUpgrade, Store, TransactionValidity,
    TransactionValidityError, ValidateTransaction,
};
pub use transactions::{
    CheckedExtrinsic, Event, EventRecord, Phase, SignedExtra, TransactionMetadata,
    TransactionStatus, TransactionV1, UncheckedExtrinsic,
};
pub use validator_discovery::{
    ApiVersion, AuthoritySetTransition, DiscoveryAnnounce, DiscoveryFilter, DiscoveryRequest,
    DiscoveryRequestType, DiscoveryResponse, NetworkTopology, PeerInfo, ValidatorDiscoveryConfig,
    ValidatorEndpoint, ValidatorInfo, ValidatorRegistry, ValidatorSessionInfo,
    ValidatorSetSnapshot,
};
