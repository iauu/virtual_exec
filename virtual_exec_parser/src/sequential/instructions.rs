use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Instruction {
    // Binary Operations
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    Shl,
    Shr,

    // Special Binary (Shortcut)
    LogicalAnd,
    LogicalOr,

    // Unary Operations
    UnaryPlus,
    UnaryMinus,
    Not,
    BitwiseNot,

    // Comparison Operations (Result push back to stack)
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,

    // Assignment Operations
    Assign(String),
    AddAssign(String),
    SubAssign(String),
    MulAssign(String),
    DivAssign(String),
    ModAssign(String),
    AndAssign(String),
    OrAssign(String),
    XorAssign(String),
    ShlAssign(String),
    ShrAssign(String),


    // Control Flow
    JmpNz(u64), // Jump when not zero
    JmpZ(u64), // Jump when zero
    Call,
    Ret,

    // Load
    LoadNone,
    LoadLitFloat(f64),
    LoadLitInt(i64),
    LoadLitString(String),
    LoadLitBool(bool),
    ConstructArr(u64),
    ConstructObj(u64),
    LoadName(String),
    LoadObjectAttr(String),
    LoadObjectIndex(i64),

    // External
    Terminate,
    Interrupt
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InstructionBuilder {
    // Binary Operations
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    Shl,
    Shr,

    // Special Binary (Shortcut)
    LogicalAnd,
    LogicalOr,

    // Unary Operations
    UnaryPlus,
    UnaryMinus,
    Not,
    BitwiseNot,

    // Comparison Operations (Result push back to stack)
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,

    // Assignment Operations
    Assign(String),
    AddAssign(String),
    SubAssign(String),
    MulAssign(String),
    DivAssign(String),
    ModAssign(String),
    AndAssign(String),
    OrAssign(String),
    XorAssign(String),
    ShlAssign(String),
    ShrAssign(String),


    // Control Flow
    JmpNz(u64), // Jump when not zero
    JmpZ(u64), // Jump when zero
    JmpNzR(u64), // Releative jmp
    JmpZR(u64),
    Call,
    Ret,

    // Load
    LoadNone,
    LoadLitFloat(f64),
    LoadLitInt(i64),
    LoadLitString(String),
    LoadLitBool(bool),
    ConstructArr(u64),
    ConstructObj(u64),
    LoadName(String),
    LoadObjectAttr(String),
    LoadObjectIndex(i64),

    // External
    Terminate,
    Interrupt
}

pub trait ConvertInstruction {
    type Output;

    fn convert(self, curr: &InstructionBuilder, relative: u64) -> Self::Output;
}

impl ConvertInstruction for InstructionBuilder {
    type Output = Instruction;

    fn convert(self, curr: &InstructionBuilder, relative: u64) -> Self::Output {
        match self {
            InstructionBuilder::Add => Instruction::Add,
            InstructionBuilder::Sub => Instruction::Sub,
            InstructionBuilder::Mul => Instruction::Mul,
            InstructionBuilder::Div => Instruction::Div,
            InstructionBuilder::Mod => Instruction::Mod,
            InstructionBuilder::Pow => Instruction::Pow,
            InstructionBuilder::BitwiseAnd => Instruction::BitwiseAnd,
            InstructionBuilder::BitwiseOr => Instruction::BitwiseOr,
            InstructionBuilder::BitwiseXor => Instruction::BitwiseXor,
            InstructionBuilder::Shl => Instruction::Shl,
            InstructionBuilder::Shr => Instruction::Shr,
            InstructionBuilder::LogicalAnd => Instruction::LogicalAnd,
            InstructionBuilder::LogicalOr => Instruction::LogicalOr,
            InstructionBuilder::UnaryPlus => Instruction::UnaryPlus,
            InstructionBuilder::UnaryMinus => Instruction::UnaryMinus,
            InstructionBuilder::Not => Instruction::Not,
            InstructionBuilder::BitwiseNot => Instruction::BitwiseNot,
            InstructionBuilder::Eq => Instruction::Eq,
            InstructionBuilder::NotEq => Instruction::NotEq,
            InstructionBuilder::Lt => Instruction::Lt,
            InstructionBuilder::Lte => Instruction::Lte,
            InstructionBuilder::Gt => Instruction::Gt,
            InstructionBuilder::Gte => Instruction::Gte,
            InstructionBuilder::Assign(name) => Instruction::Assign(name),
            InstructionBuilder::AddAssign(name) => Instruction::AddAssign(name),
            InstructionBuilder::SubAssign(name) => Instruction::SubAssign(name),
            InstructionBuilder::MulAssign(name) => Instruction::MulAssign(name),
            InstructionBuilder::DivAssign(name) => Instruction::DivAssign(name),
            InstructionBuilder::ModAssign(name) => Instruction::ModAssign(name),
            InstructionBuilder::AndAssign(name) => Instruction::AndAssign(name),
            InstructionBuilder::OrAssign(name) => Instruction::OrAssign(name),
            InstructionBuilder::XorAssign(name) => Instruction::XorAssign(name),
            InstructionBuilder::ShlAssign(name) => Instruction::ShlAssign(name),
            InstructionBuilder::ShrAssign(name) => Instruction::ShrAssign(name),
            InstructionBuilder::JmpNz(offset) => Instruction::JmpNz(offset),
            InstructionBuilder::JmpZ(offset) => Instruction::JmpZ(offset),
            InstructionBuilder::JmpNzR(offset) => Instruction::JmpNz(relative + offset),
            InstructionBuilder::JmpZR(offset) => Instruction::JmpZ(relative + offset),
            InstructionBuilder::Call => Instruction::Call,
            InstructionBuilder::Ret => Instruction::Ret,
            InstructionBuilder::LoadNone => Instruction::LoadNone,
            InstructionBuilder::LoadLitFloat(value) => Instruction::LoadLitFloat(value),
            InstructionBuilder::LoadLitInt(value) => Instruction::LoadLitInt(value),
            InstructionBuilder::LoadLitString(value) => Instruction::LoadLitString(value),
            InstructionBuilder::LoadLitBool(value) => Instruction::LoadLitBool(value),
            InstructionBuilder::ConstructArr(length) => Instruction::ConstructArr(length),
            InstructionBuilder::ConstructObj(length) => Instruction::ConstructObj(length),
            InstructionBuilder::LoadName(name) => Instruction::LoadName(name),
            InstructionBuilder::LoadObjectAttr(name) => Instruction::LoadObjectAttr(name),
            InstructionBuilder::LoadObjectIndex(index) => Instruction::LoadObjectIndex(index),
            InstructionBuilder::Terminate => Instruction::Terminate,
            InstructionBuilder::Interrupt => Instruction::Interrupt,
        }
    }
}