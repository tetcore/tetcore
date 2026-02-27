pub mod tcl;
pub mod tvm;

pub use tcl::{parse_tcl_source, TCLContract, TCLModule};
use thiserror::Error;
pub use tvm::{GasSchedule as VMGasSchedule, OpCode, TVMExecutionResult, TVM};

#[derive(Error, Debug, Clone)]
pub enum VMError {
    #[error("Out of gas")]
    OutOfGas,
    #[error("Invalid opcode")]
    InvalidOpcode,
    #[error("Stack overflow")]
    StackOverflow,
    #[error("Stack underflow")]
    StackUnderflow,
    #[error("Memory out of bounds")]
    MemoryOutOfBounds,
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Invalid jump destination")]
    InvalidJump,
    #[error("Revert called")]
    Reverted,
    #[error("Contract not found")]
    ContractNotFound,
    #[error("Storage error")]
    StorageError,
}
