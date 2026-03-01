# Tetcore

The Sovereign Execution Framework for Intelligence Infrastructure

## Overview

Tetcore is a modular, deterministic execution framework designed to implement the Intelligence Fabric Protocol (IFP). It provides the protocol kernel, runtime architecture, contract virtual machine, identity system, economic settlement engine, and node blueprint system required to deploy sovereign intelligence infrastructure networks.

Tetcore transforms artificial intelligence from a centralized service into a protocol-native computational substrate. It enables decentralized model ownership, shard-based storage, programmable pricing, deterministic inference settlement, and revenue routing at the infrastructure layer.

## Architecture

Tetcore consists of five canonical layers:

### Layer 0 — Cryptographic Identity Layer
- Ed25519 keypairs
- Address derivation rules (`TETCORE:ADDR:v1` prefix)
- Signature verification
- Replay protection and nonce management

### Layer 1 — Tetcore Kernel
- Global state management
- Deterministic transition function
- Transaction validation
- Fee settlement and revenue routing
- Module dispatch

### Layer 2 — Protocol Modules
- **Accounts**: Balance and nonce management
- **Model Registry**: Model registration, versioning, ownership
- **Governance**: Proposals, voting, execution
- **Inference**: Prompt submission, receipt validation, settlement
- **Vault**: Staking and revenue participation
- **Revenue**: Fee calculation and distribution

### Layer 3 — Contract System
- **TCL** (Tetcore Contract Language): Deterministic, strongly-typed, gas-metered
- **TVM** (Tetcore Virtual Machine): Stack-based execution, bounded memory, capability-based security

### Layer 4 — Off-Chain Execution
- Inference operators
- Model execution engines
- Storage nodes
- Relay nodes

## Project Structure

```
tetcore/                          # Tetcore - Sovereign Intelligence Blockchain
├── Cargo.toml                    # Workspace manifest
├── tetcore-primitives/           # Core cryptographic types
├── tetcore-kernel/               # Protocol kernel
├── tetcore-runtime/              # Runtime modules (IFP, Vaults, Governance)
├── tetcore-vm/                   # TVM and TCL
└── tetcore-node/                 # Full node (produces tetcore.exe)
```

## Running

### Build

```bash
cd tetcore
cargo build
```

The executable is at `target/debug/tetcore.exe` (or `target/release/tetcore.exe`)

### Testing

```bash
cd tetcore/tetcore-kernel
cargo test
```

**93 tests passing** covering all modules:
- Core kernel (state, transactions, storage)
- Consensus (validator set, BFT voting, finality)
- Economics (token supply, staking, treasury)
- Governance (proposals, voting, emergency powers)
- IFP (model registry, prompts, revenue)
- Network (peer management, gossip)
- SDK (genesis, parameters)
- TVM (contract execution, gas)
- Integration tests

### Commands

```bash
# Initialize a new node
tetcore init --mode validator

# Generate a new keypair
tetcore keygen

# Check chain height
tetcore chain height

# Run the full node
tetcore run --validator
```

### Publishing to crates.io

All crates are publishable:

```bash
cargo publish -p tetcore-primitives
cargo publish -p tetcore-kernel
cargo publish -p tetcore-runtime
cargo publish -p tetcore-vm
cargo publish -p tetcore-node
```

## Key Features

### Deterministic Execution
- Integer-only arithmetic (no floating point)
- No system clock access
- Bounded memory and gas
- Reproducible state transitions

### Cryptographic Identity
- Ed25519 signature algorithm
- 32-byte private keys
- 32-byte public keys
- 64-byte signatures
- Address format: `H("TETCORE:ADDR:v1" || PublicKey)`

### Contract System
- Strongly typed
- Capability-based access control
- Gas metering with deterministic costs
- Storage isolation per contract

### Economic Model
- Programmable pricing modes (Owner, Market, Hybrid)
- Revenue routing with basis points splits
- Vault-based staking for model participation
- Automatic fee settlement

### Model Execution
- On-chain model registration with commitments
- Off-chain weight shard storage
- Inference lifecycle: SubmitPrompt → Execute → SubmitReceipt → Settlement
- Deterministic receipt validation

## Node Modes

- **Client**: Lightweight read-only access
- **Validator**: Block production and consensus
- **Inference Operator**: Model execution
- **Relay**: Encrypted prompt delivery

## Related Projects

- **[tetcore-node-template](../tetcore-node-template/)** - A minimal Tetcore node template for rapid blockchain prototyping
- **[tetcore-spec](../tetcore-spec/)** - Tetcore specifications and documentation

## License

DOSL
