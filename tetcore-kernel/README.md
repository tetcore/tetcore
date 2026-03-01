# Tetcore Kernel

The protocol kernel providing core state machine logic for the Tetcore deterministic execution framework.

## Overview

The Tetcore Kernel (`tetcore-kernel`) is the heart of the deterministic state machine. It provides:

- Block construction and transaction validation
- State root computation using Merkle tries
- Receipt generation for executed transactions
- Consensus integration points

## Modules

### Core Types (`lib.rs`)

- `Kernel` - Main state machine implementation
- `State` - Account and storage state management
- `Transaction` / `SignedTransaction` - Transaction types
- `Address` - 32-byte account addresses
- `Hash32` - 32-byte cryptographic hashes
- `AccountData` - Balance, nonce, storage per account
- `Storage` - Key-value storage with Merkle root computation
- `GasSchedule` - Configurable gas costs

### Consensus (`consensus.rs`)

BFT consensus engine implementing a round-based voting protocol:

- `ValidatorSet` - Validator registry with stake-weighted selection
- `ConsensusEngine` - Round management, proposer selection
- `Proposal` - Block proposals with justification
- `Prevote` / `Precommit` - Vote messages
- `FinalitySignature` - Multi-signature finality
- `ForkChoiceRule` - Fork resolution algorithm

### Economics (`economics.rs`)

Economic settlement engine:

- `TokenSupply` - Total and circulating supply tracking
- `InflationConfig` - Configurable inflation with BPS rates
- `StakingModule` - Validator registration, staking, slashing
- `FeeModule` - Dynamic fee computation and distribution
- `Treasury` - On-chain treasury management

### Governance (`governance.rs`)

On-chain governance system:

- `GovernanceModule` - Proposal lifecycle management
- `Proposal` - Proposal types (ParameterChange, CodeUpgrade, etc.)
- `VoteChoice` - Voting options (Yes, No, Abstain)
- `VotingThreshold` - Approval threshold calculations
- `EmergencyScope` - Emergency power types (PauseInference, etc.)

### IFP (`ifp.rs`)

Intelligence Fabric Protocol - AI inference coordination:

- `InferenceModule` - Model registry, prompt submission
- `Model` - AI model with versioning and ownership
- `PromptCommitment` - Cryptographic prompt commitments
- `Receipt` - Inference result validation
- `RevenueDistribution` - Automatic revenue routing
- `PricingPolicy` - Programmable pricing modes

### Network (`network.rs`)

P2P networking primitives:

- `PeerId` - 32-byte peer identifiers
- `PeerInfo` - Peer metadata and reputation
- `PeerSet` - Connected peer management
- `NetworkMessage` - Block announce, transactions, consensus
- `GossipProtocol` - Message propagation with caching

### SDK (`sdk.rs`)

Development framework and tooling:

- `GenesisConfig` - Chain genesis configuration
- `ModuleState` - Isolated module state storage
- `RuntimeParametersBuilder` - Runtime parameter management
- `NetworkIdentity` - Chain identity and capabilities

### TVM (`tvm.rs`)

Tetcore Virtual Machine - smart contract execution:

- `VmContext` - Execution context with gas limits
- `GasSchedule` - Instruction-level gas costs
- `ContractModule` - Contract deployment and execution
- `ContractInstance` - Deployed contract state

### Runtime (`runtime.rs`)

Integrated runtime combining all modules:

- `TetcoreRuntime` - Full runtime with all modules
- `RuntimeBuilder` - Fluent runtime construction
- `TransactionPool` - Pending transaction management
- `BlockHeader` - Block metadata and state root

## Building

```bash
cd tetcore-kernel
cargo build
```

## Testing

```bash
cargo test
```

93 tests covering all modules including:
- Unit tests for each module
- Integration tests for cross-module workflows
- Determinism verification

## Deterministic Execution

The kernel ensures deterministic state transitions:

- Integer-only arithmetic (no floating point)
- No system clock dependencies
- Bounded memory and gas limits
- Reproducible Merkle root computation
- Deterministic transaction ordering

## Gas Model

The kernel uses a gas-based fee model:

- `tx_base_gas` - 21000 (transaction overhead)
- `transfer_gas` - 21000
- `contract_deploy_gas` - 100000
- `contract_call_gas` - 50000
- `storage_write_gas` - 50000
- `storage_read_gas` - 5000

## License

DOSL
