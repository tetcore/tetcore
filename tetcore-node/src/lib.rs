use std::collections::HashMap;
use tetcore_kernel::{compute_state_root, BlockHeader, Kernel, Receipt, Transaction};
use tetcore_primitives::{account::AccountData, Address, Hash32};
use tetcore_runtime::Runtime;
use tetcore_vm::TVM;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum NodeError {
    #[error("Block validation failed")]
    BlockValidationFailed,
    #[error("Transaction pool error")]
    TransactionPoolError,
    #[error("Consensus error")]
    ConsensusError,
    #[error("Storage error")]
    StorageError,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub receipts: Vec<Receipt>,
}

impl Block {
    pub fn new(header: BlockHeader, transactions: Vec<Transaction>) -> Self {
        Self {
            header,
            transactions,
            receipts: Vec::new(),
        }
    }

    pub fn validate(&self) -> Result<(), NodeError> {
        if self.transactions.is_empty() {
            return Err(NodeError::BlockValidationFailed);
        }
        Ok(())
    }
}

pub struct TransactionPool {
    pending: Vec<Transaction>,
}

impl TransactionPool {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.pending.push(tx);
    }

    pub fn get_transactions(&self) -> &Vec<Transaction> {
        &self.pending
    }

    pub fn clear(&mut self) {
        self.pending.clear();
    }

    pub fn remove_executed(&mut self, _executed_count: usize) {}
}

impl Default for TransactionPool {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ChainStorage {
    blocks: HashMap<u64, Block>,
    state: HashMap<Address, AccountData>,
    current_height: u64,
}

impl ChainStorage {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            state: HashMap::new(),
            current_height: 0,
        }
    }

    pub fn get_block(&self, height: u64) -> Option<&Block> {
        self.blocks.get(&height)
    }

    pub fn insert_block(&mut self, block: Block) {
        let height = block.header.height;
        self.blocks.insert(height, block);
        if height > self.current_height {
            self.current_height = height;
        }
    }

    pub fn get_state(&self, address: &Address) -> Option<&AccountData> {
        self.state.get(address)
    }

    pub fn set_state(&mut self, address: Address, account: AccountData) {
        self.state.insert(address, account);
    }

    pub fn get_all_state(&self) -> &HashMap<Address, AccountData> {
        &self.state
    }

    pub fn current_height(&self) -> u64 {
        self.current_height
    }

    pub fn get_genesis_hash(&self) -> Option<Hash32> {
        self.blocks.get(&0).map(|b| b.header.state_root.clone())
    }
}

impl Default for ChainStorage {
    fn default() -> Self {
        Self::new()
    }
}

pub enum NodeMode {
    Client,
    Validator,
    InferenceOperator,
    Relay,
}

pub struct TetcoreNode {
    kernel: Kernel,
    runtime: Runtime,
    tvm: TVM,
    tx_pool: TransactionPool,
    chain: ChainStorage,
    mode: NodeMode,
    my_address: Option<Address>,
}

impl TetcoreNode {
    pub fn new(mode: NodeMode) -> Self {
        Self {
            kernel: Kernel::new(),
            runtime: Runtime::new(),
            tvm: TVM::new(),
            tx_pool: TransactionPool::new(),
            chain: ChainStorage::new(),
            mode,
            my_address: None,
        }
    }

    pub fn set_my_address(&mut self, address: Address) {
        self.my_address = Some(address);
    }

    pub fn create_genesis_block(&mut self) -> Block {
        let header = BlockHeader::new(0, Hash32::empty());
        Block::new(header, Vec::new())
    }

    pub fn initialize_genesis(&mut self, accounts: Vec<(Address, u128)>) {
        for (address, balance) in accounts {
            let account = AccountData::new(balance);
            self.chain.set_state(address, account.clone());
            self.kernel.create_account(address, account);
        }

        let genesis = self.create_genesis_block();
        self.chain.insert_block(genesis);
    }

    pub fn submit_transaction(&mut self, tx: Transaction) -> Result<(), NodeError> {
        self.kernel
            .validate_transaction(&tx)
            .map_err(|_| NodeError::TransactionPoolError)?;
        self.tx_pool.add_transaction(tx);
        Ok(())
    }

    pub fn produce_block(&mut self) -> Result<Block, NodeError> {
        let parent_height = self.chain.current_height();
        let parent_hash = self
            .chain
            .get_block(parent_height)
            .map(|b| b.header.state_root.clone())
            .unwrap_or_else(Hash32::empty);

        let mut header = BlockHeader::new(parent_height + 1, parent_hash);

        let transactions: Vec<Transaction> = self.tx_pool.get_transactions().clone();

        for tx in &transactions {
            if self.kernel.apply_transaction(tx).is_ok() {
                if let Some(account) = self.chain.get_state(&tx.sender).cloned() {
                    let mut updated = account.clone();
                    updated.balance = updated.balance.saturating_sub(21000);
                    self.chain.set_state(tx.sender, updated);
                }
            }
        }

        let accounts = self.chain.get_all_state().clone();
        header.state_root = compute_state_root(&accounts);

        self.tx_pool.clear();

        let mut block = Block::new(header, transactions);
        block
            .validate()
            .map_err(|_| NodeError::BlockValidationFailed)?;

        Ok(block)
    }

    pub fn import_block(&mut self, block: Block) -> Result<(), NodeError> {
        block
            .validate()
            .map_err(|_| NodeError::BlockValidationFailed)?;

        for tx in &block.transactions {
            self.kernel.apply_transaction(tx).ok();
        }

        self.chain.insert_block(block);

        Ok(())
    }

    pub fn get_account(&self, address: &Address) -> Option<AccountData> {
        self.chain.get_state(address).cloned()
    }

    pub fn get_runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn get_runtime_mut(&mut self) -> &mut Runtime {
        &mut self.runtime
    }

    pub fn get_tvm(&self) -> &TVM {
        &self.tvm
    }

    pub fn get_tvm_mut(&mut self) -> &mut TVM {
        &mut self.tvm
    }

    pub fn mode(&self) -> &NodeMode {
        &self.mode
    }

    pub fn chain(&self) -> &ChainStorage {
        &self.chain
    }

    pub fn chain_mut(&mut self) -> &mut ChainStorage {
        &mut self.chain
    }
}
