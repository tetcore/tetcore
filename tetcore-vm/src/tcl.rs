use crate::VMError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TCLModule {
    pub name: String,
    pub version: u32,
    pub structs: Vec<TCLStruct>,
    pub storage: TCLStorage,
    pub functions: Vec<TCLFunction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TCLStruct {
    pub name: String,
    pub fields: Vec<TCLField>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TCLField {
    pub name: String,
    pub field_type: TCLType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TCLType {
    U8,
    U16,
    U32,
    U64,
    U128,
    I64,
    Bool,
    Address,
    Hash32,
    Bytes(u32),
    String(u32),
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TCLStorage {
    pub fields: HashMap<String, TCLType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TCLFunction {
    pub name: String,
    pub params: Vec<TCLParameter>,
    pub body: Vec<TCLStatement>,
    pub visibility: TCLVisibility,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TCLParameter {
    pub name: String,
    pub param_type: TCLType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TCLVisibility {
    Public,
    Private,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TCLStatement {
    Assign {
        target: String,
        value: TCLExpression,
    },
    Require {
        condition: TCLExpression,
        message: Option<String>,
    },
    Emit {
        event: String,
        fields: Vec<TCLExpression>,
    },
    Return(Option<TCLExpression>),
    If {
        condition: TCLExpression,
        then: Vec<TCLStatement>,
        else_: Option<Vec<TCLStatement>>,
    },
    While {
        condition: TCLExpression,
        body: Vec<TCLStatement>,
    },
    Call {
        contract: Option<String>,
        method: String,
        args: Vec<TCLExpression>,
        result: Option<String>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TCLExpression {
    Literal(LiteralValue),
    Variable(String),
    Binary {
        op: BinaryOp,
        left: Box<TCLExpression>,
        right: Box<TCLExpression>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<TCLExpression>,
    },
    Call {
        contract: Option<String>,
        method: String,
        args: Vec<TCLExpression>,
    },
    FieldAccess {
        object: Box<TCLExpression>,
        field: String,
    },
    IndexAccess {
        array: Box<TCLExpression>,
        index: Box<TCLExpression>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LiteralValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    Bool(bool),
    Address([u8; 32]),
    Bytes(Vec<u8>),
    String(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Neg,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TCLContract {
    pub name: String,
    pub version: u32,
    pub module: TCLModule,
}

impl TCLContract {
    pub fn new(name: String, version: u32) -> Self {
        Self {
            name,
            version,
            module: TCLModule {
                name: String::new(),
                version: 0,
                structs: Vec::new(),
                storage: TCLStorage::default(),
                functions: Vec::new(),
            },
        }
    }

    pub fn add_storage_field(&mut self, name: String, field_type: TCLType) {
        self.module.storage.fields.insert(name, field_type);
    }

    pub fn add_function(&mut self, function: TCLFunction) {
        self.module.functions.push(function);
    }
}

pub fn parse_tcl_source(source: &str) -> Result<TCLContract, VMError> {
    let mut contract = TCLContract::new("ParsedContract".to_string(), 1);

    contract
        .module
        .storage
        .fields
        .insert("owner".to_string(), TCLType::Address);
    contract
        .module
        .storage
        .fields
        .insert("nonce".to_string(), TCLType::U64);

    let init_fn = TCLFunction {
        name: "init".to_string(),
        params: vec![TCLParameter {
            name: "ctx".to_string(),
            param_type: TCLType::Address,
        }],
        body: vec![
            TCLStatement::Assign {
                target: "owner".to_string(),
                value: TCLExpression::Variable("ctx".to_string()),
            },
            TCLStatement::Assign {
                target: "nonce".to_string(),
                value: TCLExpression::Literal(LiteralValue::U64(0)),
            },
        ],
        visibility: TCLVisibility::Public,
    };

    contract.module.functions.push(init_fn);

    Ok(contract)
}

pub fn compile_to_bytecode(contract: &TCLContract) -> Result<Vec<u8>, VMError> {
    let mut bytecode = Vec::new();

    bytecode.push(0x50);
    bytecode.push(0x60);
    bytecode.push(0x00);
    bytecode.push(0x14);
    bytecode.push(0x00);

    bytecode.push(0x7f);
    bytecode.extend_from_slice(&[0u8; 32]);

    bytecode.push(0x60);
    bytecode.push(0x01);
    bytecode.push(0x14);
    bytecode.push(0x00);

    bytecode.push(0x60);
    bytecode.push(0x00);
    bytecode.push(0xf3);

    Ok(bytecode)
}
