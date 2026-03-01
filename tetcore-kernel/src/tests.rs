// File: tests.rs - This file is part of Tetcore
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
// Comprehensive tests for the Tetcore kernel covering all modules:
// consensus, economics, governance, ifp, network, runtime, sdk, and tvm.

use crate::consensus::{
    ConsensusEngine, ForkChoice, ForkChoiceRule, Proposal, ValidatorInfo, ValidatorSet,
};
use crate::economics::{
    FeeModule, InflationConfig, InflationState, StakingModule, TokenSupply, Treasury, TOTAL_SUPPLY,
};
use crate::governance::{
    EmergencyScope, GovernanceModule, ProposalStatus, ProposalType, VoteChoice, VotingThreshold,
};
use crate::ifp::{
    InferenceModule, PricingMode, PricingPolicy, PromptCommitment, RelayMode, RevenueDistribution,
    RevenueSplit,
};
use crate::network::{
    BlockRequest, GossipProtocol, NetworkAddress, NetworkMessage, NetworkMessageType, PeerId,
    PeerInfo, PeerSet,
};
use crate::runtime::{
    BlockHeader, PoolTransaction, RuntimeBuilder, TetcoreRuntime as Runtime, TransactionPool,
};
use crate::sdk::{
    GenesisAccount, GenesisConfig, ModuleState, NetworkIdentity, RuntimeParametersBuilder,
    RuntimeValue,
};
use crate::tvm::{ContractModule, GasSchedule, VmContext};
use crate::{Address, ExecutionReceipt, Hash32, Kernel, Storage, Transaction};

#[cfg(test)]
mod consensus_tests {
    use super::*;
    use crate::consensus::*;

    fn create_validator(i: u8) -> ValidatorInfo {
        let mut bytes = [0u8; 32];
        bytes[31] = i;
        ValidatorInfo::new(Address(bytes), (i as u128) * 1000)
    }

    #[test]
    fn test_validator_set_creation() {
        let validators = vec![
            create_validator(1),
            create_validator(2),
            create_validator(3),
        ];
        let set = ValidatorSet::with_validators(validators);
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn test_validator_set_quorum() {
        let validators = vec![
            create_validator(1),
            create_validator(2),
            create_validator(3),
        ];
        let set = ValidatorSet::with_validators(validators);
        assert_eq!(set.quorum_size(), 3);

        let validators = (1..=6).map(create_validator).collect();
        let set = ValidatorSet::with_validators(validators);
        assert_eq!(set.quorum_size(), 5);
    }

    #[test]
    fn test_proposer_selection() {
        let validators = (1..=4).map(create_validator).collect();
        let mut set = ValidatorSet::with_validators(validators);

        let p0 = set.get_proposer(0, 0);
        let p1 = set.get_proposer(0, 1);
        let p2 = set.get_proposer(0, 2);
        let p3 = set.get_proposer(0, 3);
        let p4 = set.get_proposer(0, 4);

        assert!(p0.is_some());
        assert!(p1.is_some());
        assert!(p2.is_some());
        assert!(p3.is_some());
        assert!(p4.is_some());
    }

    #[test]
    fn test_proposal_creation() {
        let proposer = create_validator(1).address;
        let proposal = Proposal::new(1, 0, Hash32::empty(), proposer);

        assert_eq!(proposal.height, 1);
        assert_eq!(proposal.round, 0);
    }

    #[test]
    fn test_prevote_creation() {
        let voter = create_validator(1).address;
        let prevote = Prevote::new(1, 0, Some(Hash32::empty()), voter);

        assert_eq!(prevote.height, 1);
        assert!(!prevote.is_nil());
    }

    #[test]
    fn test_precommit_creation() {
        let voter = create_validator(1).address;
        let precommit = Precommit::new(1, 0, None, voter);

        assert!(precommit.is_nil());
    }

    #[test]
    fn test_consensus_engine_new_round() {
        let validators = (1..=4).map(create_validator).collect();
        let mut engine = ConsensusEngine::with_validators(validators);

        engine.start_new_round(1, 0);

        assert_eq!(engine.height, 1);
        assert_eq!(engine.round, 0);
        assert!(engine.current_round.is_some());
    }

    #[test]
    fn test_finality_signature() {
        let mut sig = FinalitySignature::new(Hash32::empty(), 1, 0);

        let validator = create_validator(1).address;
        sig.add_signature(validator, vec![1, 2, 3]);

        assert_eq!(sig.signatures.len(), 1);
    }

    #[test]
    fn test_fork_choice_rule() {
        let left_hash = Hash32::from_slice(&[2u8; 32]);
        let right_hash = Hash32::from_slice(&[3u8; 32]);

        let choice = ForkChoiceRule::choose(Hash32::empty(), 10, left_hash, 12, right_hash, 11);
        assert_eq!(choice, ForkChoice::Left);
    }
}

#[cfg(test)]
mod economics_tests {
    use super::*;
    use crate::economics::*;

    fn create_address(i: u8) -> Address {
        let mut bytes = [0u8; 32];
        bytes[31] = i;
        Address(bytes)
    }

    #[test]
    fn test_token_supply_creation() {
        let supply = TokenSupply::new();
        assert_eq!(supply.total, TOTAL_SUPPLY);
        assert_eq!(supply.circulating, TOTAL_SUPPLY);
    }

    #[test]
    fn test_token_supply_invariant() {
        let supply = TokenSupply::new();
        assert!(supply.verify_invariant());
    }

    #[test]
    fn test_inflation_config_disabled() {
        let config = InflationConfig::default();
        assert_eq!(config.state, InflationState::Disabled);
        assert_eq!(config.compute_mint(1000), 0);
    }

    #[test]
    fn test_inflation_config_enabled() {
        let config = InflationConfig::new(100);
        assert_eq!(config.state, InflationState::Enabled);
        assert_eq!(config.rate_bps, 100);
    }

    #[test]
    fn test_inflation_distribution() {
        let config = InflationConfig::default();
        let (treasury, validators) = config.distribute(1000);

        assert_eq!(treasury, 200);
        assert_eq!(validators, 800);
    }

    #[test]
    fn test_staking_module_creation() {
        let staking = StakingModule::new();
        assert_eq!(staking.total_staked, 0);
    }

    #[test]
    fn test_validator_registration() {
        let mut staking = StakingModule::new();
        let validator = create_address(1);

        let result = staking.register_validator(validator, 1000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_staking_stake() {
        let mut staking = StakingModule::new();
        let staker = create_address(1);

        staking.token_supply.circulating = 1000;
        let shares = staking.stake(staker, 100, 1).unwrap();

        assert_eq!(shares, 100);
        assert_eq!(staking.total_staked, 100);
    }

    #[test]
    fn test_staking_unstake() {
        let mut staking = StakingModule::new();
        let staker = create_address(1);

        staking.token_supply.circulating = 1000;
        staking.stake(staker, 100, 1).unwrap();

        let result = staking.unstake(staker, 50, 2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fee_module_default() {
        let fee = FeeModule::default();
        assert_eq!(fee.base_fee, 1000);
    }

    #[test]
    fn test_fee_computation() {
        let fee = FeeModule::default();
        let total = fee.compute_fee(21000);

        assert!(total > 0);
    }

    #[test]
    fn test_fee_distribution() {
        let fee = FeeModule::default();
        let (burn, treasury, validators) = fee.distribute_fee(1000);

        assert_eq!(burn, 0);
        assert_eq!(treasury, 100);
        assert_eq!(validators, 900);
    }

    #[test]
    fn test_treasury_deposit_spend() {
        let mut treasury = Treasury::new();

        treasury.deposit(1000);
        assert_eq!(treasury.balance, 1000);

        treasury.spend(100).unwrap();
        assert_eq!(treasury.balance, 900);
    }

    #[test]
    fn test_treasury_spend_limit() {
        let mut treasury = Treasury::new();
        treasury.deposit(TOTAL_SUPPLY);

        let limit = treasury.spend_limit_per_proposal;
        let result = treasury.spend(limit + 1);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod governance_tests {
    use super::*;
    use crate::governance::*;

    fn create_address(i: u8) -> Address {
        let mut bytes = [0u8; 32];
        bytes[31] = i;
        Address(bytes)
    }

    #[test]
    fn test_proposal_submission() {
        let mut gov = GovernanceModule::new(1_000_000);
        let proposer = create_address(1);

        let result = gov.submit_proposal(
            proposer,
            ProposalType::ParameterChange,
            vec![1, 2, 3],
            "Test proposal".to_string(),
            0,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_proposal_approval() {
        let mut gov = GovernanceModule::new(1_000_000);
        let proposer = create_address(1);

        let mut proposal = gov
            .submit_proposal(
                proposer,
                ProposalType::ParameterChange,
                vec![1, 2, 3],
                "Test".to_string(),
                0,
            )
            .unwrap();

        proposal.yes_votes = 700000;
        proposal.no_votes = 100000;

        assert!(proposal.is_approved(1_000_000));
    }

    #[test]
    fn test_vote_casting() {
        let mut gov = GovernanceModule::new(1_000_000);
        let proposer = create_address(1);
        let voter = create_address(2);

        let proposal = gov
            .submit_proposal(
                proposer,
                ProposalType::ParameterChange,
                vec![1, 2, 3],
                "Test".to_string(),
                0,
            )
            .unwrap();

        let result = gov.cast_vote(&proposal.proposal_id, voter, VoteChoice::Yes, 1000, 1);

        assert!(result.is_ok());
    }

    #[test]
    fn test_delegation() {
        let mut gov = GovernanceModule::new(1_000_000);
        let delegator = create_address(1);
        let delegate = create_address(2);

        let result = gov.delegate(delegator, delegate, 500, 0);
        assert!(result.is_ok());

        let delegation = gov.get_delegation(&delegator);
        assert!(delegation.is_some());
    }

    #[test]
    fn test_emergency_powers() {
        let mut gov = GovernanceModule::new(1_000_000);
        let activator = create_address(1);

        let scope = EmergencyScope::PauseInference;
        let result = gov.activate_emergency(scope, activator, 0);
        assert!(result.is_ok());

        assert!(gov.emergency_powers.is_some());
    }

    #[test]
    fn test_voting_threshold() {
        let threshold = VotingThreshold::parameter_change();
        assert!(threshold.is_approved(20, 50));
        assert!(!threshold.is_approved(10, 50));
    }
}

#[cfg(test)]
mod ifp_tests {
    use super::*;
    use crate::ifp::*;

    fn create_address(i: u8) -> Address {
        let mut bytes = [0u8; 32];
        bytes[31] = i;
        Address(bytes)
    }

    fn create_hash(i: u8) -> Hash32 {
        let mut bytes = [0u8; 32];
        bytes[31] = i;
        Hash32(bytes)
    }

    #[test]
    fn test_model_registration() {
        let mut module = InferenceModule::new();
        let owner = create_address(1);
        let shard_root = create_hash(1);

        let result = module.register_model(
            owner,
            shard_root,
            4,
            PricingPolicy::default(),
            RevenueSplit::default_ifp(),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_model_activation() {
        let mut module = InferenceModule::new();
        let owner = create_address(1);
        let shard_root = create_hash(1);

        let model = module
            .register_model(
                owner,
                shard_root,
                4,
                PricingPolicy::default(),
                RevenueSplit::default_ifp(),
            )
            .unwrap();

        let result = module.activate_model(&model.model_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_prompt_submission() {
        let mut module = InferenceModule::new();
        let owner = create_address(1);
        let sender = create_address(2);
        let shard_root = create_hash(1);

        let model = module
            .register_model(
                owner,
                shard_root,
                4,
                PricingPolicy::default(),
                RevenueSplit::default_ifp(),
            )
            .unwrap();

        module.activate_model(&model.model_id).unwrap();

        let commitment = PromptCommitment::new(b"test prompt", vec![1, 2, 3]);

        let result = module.submit_prompt(
            model.model_id,
            1,
            sender,
            commitment,
            100,
            10000,
            RelayMode::Direct,
            PricingMode::Owner,
            1000,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_revenue_distribution() {
        let split = RevenueSplit::default_ifp();
        let distribution = RevenueDistribution::distribute(10000, &split);

        assert_eq!(distribution.operator_amount, 7000);
        assert_eq!(distribution.owner_amount, 2000);
        assert_eq!(distribution.shard_provider_amount, 500);
        assert_eq!(distribution.validator_amount, 400);
        assert_eq!(distribution.treasury_amount, 100);
    }

    #[test]
    fn test_pricing_computation() {
        let policy = PricingPolicy {
            mode: PricingMode::Owner,
            base_price: 1000,
            per_token_price: 10,
            complexity_multiplier: 100,
            latency_multiplier: 100,
        };

        let price = policy.compute_price(100, 200);
        assert!(price >= 1000);
    }

    #[test]
    fn test_prompt_commitment_verification() {
        let commitment = PromptCommitment::new(b"test prompt", vec![1, 2, 3]);
        assert!(commitment.verify(b"test prompt"));
    }
}

#[cfg(test)]
mod network_tests {
    use super::*;
    use crate::network::*;

    #[test]
    fn test_peer_info_creation() {
        let peer_id = PeerId::from_bytes([1u8; 32]);
        let addr = NetworkAddress::new_ip("127.0.0.1".to_string(), 30333);
        let peer = PeerInfo::new(peer_id, addr);

        assert!(!peer.is_validator());
    }

    #[test]
    fn test_peer_set_management() {
        let mut peer_set = PeerSet::new();

        let peer_id = PeerId::from_bytes([1u8; 32]);
        let addr = NetworkAddress::new_ip("127.0.0.1".to_string(), 30333);
        let peer = PeerInfo::new(peer_id, addr);

        peer_set.add_peer(peer);

        assert_eq!(peer_set.len(), 1);
    }

    #[test]
    fn test_network_message_creation() {
        let message =
            NetworkMessage::new_block_announce(Hash32::empty(), 1, PeerId::from_bytes([1u8; 32]));

        assert!(matches!(
            message.message_type,
            NetworkMessageType::BlockAnnounce
        ));
    }

    #[test]
    fn test_block_request() {
        let request = BlockRequest::new_by_hash(vec![Hash32::empty()]);

        assert_eq!(request.block_hashes.len(), 1);
    }

    #[test]
    fn test_gossip_protocol_creation() {
        let gossip = GossipProtocol::new();
        assert!(gossip.peer_set.is_empty());
    }
}

#[cfg(test)]
mod sdk_tests {
    use super::*;
    use crate::sdk::*;

    fn create_address(i: u8) -> Address {
        let mut bytes = [0u8; 32];
        bytes[31] = i;
        Address(bytes)
    }

    #[test]
    fn test_genesis_config() {
        let config = GenesisConfig::new("test".to_string(), "testnet".to_string(), 1)
            .with_validator(Address([1u8; 32]))
            .with_account(GenesisAccount::new(Address([2u8; 32]), 1000));

        assert!(config.verify().is_ok());
    }

    #[test]
    fn test_runtime_parameters() {
        let params = RuntimeParametersBuilder::new()
            .set("max_validators", 100u64)
            .set("block_time", 5000u64)
            .set("enable_inflation", false)
            .build();

        assert_eq!(params.get_u64("max_validators", 0), 100);
        assert_eq!(params.get_bool("enable_inflation", true), false);
    }

    #[test]
    fn test_module_state_root() {
        let mut state = ModuleState::new();
        state.set(b"key1".to_vec(), b"value1".to_vec());
        state.set(b"key2".to_vec(), b"value2".to_vec());

        let root = state.root();
        assert!(!root.0.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_network_identity() {
        let config = GenesisConfig::new("test".to_string(), "testnet".to_string(), 1);
        let identity = NetworkIdentity::new("testnet".to_string(), 1, &config);

        assert_eq!(identity.chain_id, 1);
    }

    #[test]
    fn test_runtime_value() {
        let v1: RuntimeValue = 42u64.into();
        assert_eq!(v1.as_u64(), Some(42));

        let v2: RuntimeValue = true.into();
        assert_eq!(v2.as_bool(), Some(true));
    }
}

#[cfg(test)]
mod tvm_tests {
    use super::*;
    use crate::tvm::*;

    #[test]
    fn test_gas_schedule() {
        let schedule = GasSchedule::default();
        assert_eq!(schedule.step, 1);
        assert_eq!(schedule.storage_read, 50);
    }

    #[test]
    fn test_vm_context() {
        let address = Address([1u8; 32]);
        let caller = Address([2u8; 32]);
        let context = VmContext::new(address, caller, caller, 1000);

        assert_eq!(context.gas_limit, 1000);
    }

    #[test]
    fn test_contract_deployment() {
        let mut module = ContractModule::new();
        let owner = Address([1u8; 32]);
        let code = vec![0x00];

        let result = module.deploy(owner, code, 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_contract_call() {
        let mut module = ContractModule::new();
        let owner = Address([1u8; 32]);

        let contract = module.deploy(owner, vec![0x00], 1000).unwrap();

        let result = module.call(&contract, owner, b"test", vec![], 10000, 0);

        assert!(result.is_ok());
    }

    #[test]
    fn test_contract_balance_transfer() {
        let mut module = ContractModule::new();
        let owner = Address([1u8; 32]);
        let addr2 = Address([2u8; 32]);

        let contract_addr = module.deploy(owner, vec![0x00], 1000).unwrap();

        module.contracts.get_mut(&contract_addr).unwrap().balance = 500;

        let result = module.transfer(&contract_addr, &addr2, 100);
        assert!(result.is_ok());
        assert_eq!(module.get_balance(&addr2), 100);
    }
}

#[cfg(test)]
mod runtime_tests {
    use super::*;
    use crate::runtime::*;

    fn create_address(i: u8) -> Address {
        let mut bytes = [0u8; 32];
        bytes[31] = i;
        Address(bytes)
    }

    #[test]
    fn test_runtime_creation() {
        let runtime = RuntimeBuilder::new(1)
            .with_account(create_address(1), 1000)
            .with_validator(create_address(2), 500)
            .build();

        assert_eq!(runtime.chain_id, 1);
    }

    #[test]
    fn test_account_creation() {
        let mut runtime = TetcoreRuntime::new(1);
        runtime.create_account(create_address(1), 1000);

        assert_eq!(runtime.get_balance(&create_address(1)), 1000);
    }

    #[test]
    fn test_account_transfer() {
        let mut runtime = RuntimeBuilder::new(1)
            .with_account(create_address(1), 1000)
            .with_account(create_address(2), 0)
            .build();

        runtime
            .transfer(&create_address(1), &create_address(2), 500)
            .unwrap();

        assert_eq!(runtime.get_balance(&create_address(1)), 500);
        assert_eq!(runtime.get_balance(&create_address(2)), 500);
    }

    #[test]
    fn test_block_execution() {
        let mut runtime = RuntimeBuilder::new(1)
            .with_account(create_address(1), 10000)
            .build();

        let tx_data = vec![1u8; 48];
        let result = runtime.execute_block(vec![tx_data]);

        assert!(result.is_ok());
        assert_eq!(runtime.block_number, 1);
    }

    #[test]
    fn test_block_header() {
        let header = BlockHeader::new(Hash32::empty(), 1);

        assert_eq!(header.block_number, 1);

        let hash = header.hash();
        assert!(!hash.0.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_transaction_pool() {
        let mut pool = TransactionPool::new();

        pool.add(PoolTransaction {
            tx_hash: Hash32::empty(),
            sender: create_address(1),
            nonce: 0,
            gas_price: 1,
            data: vec![],
        });

        assert_eq!(pool.pending.len(), 1);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::*;

    fn create_address(i: u8) -> Address {
        let mut bytes = [0u8; 32];
        bytes[31] = i;
        Address(bytes)
    }

    #[test]
    fn test_full_runtime_lifecycle() {
        let mut runtime = RuntimeBuilder::new(1)
            .with_account(create_address(1), 100000)
            .with_account(create_address(2), 100000)
            .with_validator(create_address(3), 10000)
            .build();

        runtime
            .transfer(&create_address(1), &create_address(2), 1000)
            .unwrap();
        assert_eq!(runtime.get_balance(&create_address(1)), 99000);

        let tx_data = vec![1u8; 48];
        let block = runtime.execute_block(vec![tx_data]).unwrap();

        assert_eq!(runtime.block_number, 1);
        assert!(!block.header.state_root.0.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_state_determinism() {
        let mut runtime1 = RuntimeBuilder::new(1)
            .with_account(create_address(1), 10000)
            .build();

        let mut runtime2 = RuntimeBuilder::new(1)
            .with_account(create_address(1), 10000)
            .build();

        runtime1
            .transfer(&create_address(1), &create_address(2), 500)
            .unwrap();
        runtime2
            .transfer(&create_address(1), &create_address(2), 500)
            .unwrap();

        assert_eq!(
            runtime1.get_balance(&create_address(1)),
            runtime2.get_balance(&create_address(1))
        );
    }

    #[test]
    fn test_governance_proposal_voting_flow() {
        let mut governance = GovernanceModule::new(1_000_000);

        let proposer = create_address(1);
        let proposal = governance
            .submit_proposal(
                proposer,
                ProposalType::ParameterChange,
                vec![1, 2, 3],
                "Test".to_string(),
                0,
            )
            .unwrap();

        let voter1 = create_address(2);
        governance
            .cast_vote(&proposal.proposal_id, voter1, VoteChoice::Yes, 300000, 1)
            .unwrap();

        let voter2 = create_address(3);
        governance
            .cast_vote(&proposal.proposal_id, voter2, VoteChoice::Yes, 300000, 2)
            .unwrap();

        governance.end_voting(&proposal.proposal_id, 20000).unwrap();

        let final_proposal = governance.get_proposal(&proposal.proposal_id).unwrap();
        assert_eq!(final_proposal.status, ProposalStatus::Timelocked);
    }

    #[test]
    fn test_ifp_model_prompt_flow() {
        let mut ifp = InferenceModule::new();

        let owner = create_address(1);
        let model = ifp
            .register_model(
                owner,
                Hash32::from_slice(&[1u8; 32]),
                4,
                PricingPolicy::default(),
                RevenueSplit::default_ifp(),
            )
            .unwrap();

        ifp.activate_model(&model.model_id).unwrap();

        let sender = create_address(2);
        let commitment = PromptCommitment::new(b"test prompt", vec![1, 2, 3]);

        let prompt = ifp
            .submit_prompt(
                model.model_id,
                1,
                sender,
                commitment,
                100,
                10000,
                RelayMode::Direct,
                PricingMode::Owner,
                1000,
            )
            .unwrap();

        assert_eq!(ifp.pending_prompt_count(), 1);
    }
}
