# Changelog

All notable changes to the Tetcore Kernel will be documented in this file.

## [0.1.1] - 2026-03-01

### Added
- Comprehensive test suite (93 tests)
- Module-level tests for consensus, economics, governance, IFP, network, SDK, TVM
- Integration tests for full runtime lifecycle
- Determinism verification tests

### Fixed
- Duplicate module definition in lib.rs
- Missing imports in test modules
- Private field access (peer_set in GossipProtocol)
- Test logic errors in IFP, governance, and TVM tests

### Modules
- `lib.rs`: Core kernel types, state management, transaction validation
- `consensus.rs`: BFT consensus engine with validator sets
- `economics.rs`: Token supply, staking, treasury, fee distribution
- `governance.rs`: Proposals, voting, emergency powers
- `ifp.rs`: AI model registry, prompts, revenue distribution
- `network.rs`: P2P networking primitives
- `sdk.rs`: Genesis config, module traits, runtime builder
- `tvm.rs`: Virtual machine with gas metering
- `runtime.rs`: Integrated runtime combining all modules

## [0.1.0] - 2026-02-15

### Added
- Initial release of Tetcore Kernel
- Core primitives (Address, Hash32, Transaction)
- Deterministic state machine implementation
- Merkle trie storage with root computation
- Module system with trait-based design
- All 8 protocol phases implemented

### Architecture
- Layer 0: Cryptographic identity (Ed25519)
- Layer 1: Kernel (state management, validation)
- Layer 2: Protocol modules (Accounts, Governance, IFP)
- Layer 3: Contract system (TVM, gas metering)
- Layer 4: Off-chain execution coordination
