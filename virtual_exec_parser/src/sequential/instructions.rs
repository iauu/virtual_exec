use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
pub enum Instruction {
    // Binary Operations

    /// Take item from at the top of stack as `b`, then take item from top of stack as `a`
    /// Perform `a + b`
    Add,
    /// Take item from at the top of stack as `b`, then take item from top of stack as `a`
    /// Perform `a - b`
    Sub,
    /// Take item from at the top of stack as `b`, then take item from top of stack as `a`
    /// Perform `a * b`
    Mul,
    /// Take item from at the top of stack as `b`, then take item from top of stack as `a`
    /// Perform `a / b`
    Div,
    /// Take item from at the top of stack as `b`, then take item from top of stack as `a`
    /// Perform `a % b`
    Mod,
    /// Take item from at the top of stack as `b`, then take item from top of stack as `a`
    /// Perform `a & b`
    BitwiseAnd,
    /// Take item from at the top of stack as `b`, then take item from top of stack as `a`
    /// Perform `a | b`
    BitwiseOr,
    /// Take item from at the top of stack as `b`, then take item from top of stack as `a`
    /// Perform `a ^ b`
    BitwiseXor,
    /// Take item from at the top of stack as `b`, then take item from top of stack as `a`
    /// Perform `a << b`
    Shl,
    /// Take item from at the top of stack as `b`, then take item from top of stack as `a`
    /// Perform `a >> b`
    Shr,

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
    Assign,


    // Control Flow
    /// Take item from at the top of stack as `a`
    /// If `a` is not false and not 0, it jumps to the specify location and run as the next instruction
    /// Otherwise it continue to the next instruction
    JmpNz(u64), // Jump when not zero
    /// Take item from at the top of stack as `a`
    /// If `a` is false or 0, it jumps to the specify location and run as the next instruction
    /// Otherwise it continue to the next instruction
    JmpZ(u64), // Jump when zero
    /// Jump unconditionally to the specific location and run as the next instruction
    Jmp(u64),
    Call,
    Ret,

    // Load
    LoadNone,
    LoadLitFloat(f64),
    LoadLitInt(i64),
    LoadLitString(Box<str>),
    LoadLitBool(bool),
    ConstructArr(u64),
    ConstructObj(u64),
    LoadName(Box<str>),
    LoadObjectAttr(Box<str>),
    LoadObjectIndex(i64),

    // External
    Terminate,
    Interrupt,

    // Stack
    Pop
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InstructionBuilder {
    // Binary Operations
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    Shl,
    Shr,


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
    Assign,


    // Control Flow
    JmpNz(u64), // Jump when not zero
    JmpZ(u64), // Jump when zero
    Jmp(u64),
    JmpNzR(u64), // Releative jmp
    JmpZR(u64),
    JmpR(u64),
    Call,
    Ret,

    // Load
    LoadNone,
    LoadLitFloat(f64),
    LoadLitInt(i64),
    LoadLitString(Box<str>),
    LoadLitBool(bool),
    ConstructArr(u64),
    ConstructObj(u64),
    LoadName(Box<str>),
    LoadObjectAttr(Box<str>),
    LoadObjectIndex(i64),

    // External
    Terminate,
    Interrupt,

    // Stack
    Pop
}

pub trait ConvertInstruction {
    type Output;

    fn convert(self, curr: &InstructionBuilder, relative: u64) -> Self::Output;
}

impl ConvertInstruction for InstructionBuilder {
    type Output = Instruction;

    fn convert(self, _curr: &InstructionBuilder, relative: u64) -> Self::Output {
        match self {
            InstructionBuilder::Add => Instruction::Add,
            InstructionBuilder::Sub => Instruction::Sub,
            InstructionBuilder::Mul => Instruction::Mul,
            InstructionBuilder::Div => Instruction::Div,
            InstructionBuilder::Mod => Instruction::Mod,
            InstructionBuilder::BitwiseAnd => Instruction::BitwiseAnd,
            InstructionBuilder::BitwiseOr => Instruction::BitwiseOr,
            InstructionBuilder::BitwiseXor => Instruction::BitwiseXor,
            InstructionBuilder::Shl => Instruction::Shl,
            InstructionBuilder::Shr => Instruction::Shr,
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
            InstructionBuilder::Assign => Instruction::Assign,
            InstructionBuilder::JmpNz(offset) => Instruction::JmpNz(offset),
            InstructionBuilder::JmpZ(offset) => Instruction::JmpZ(offset),
            InstructionBuilder::Jmp(offset) => Instruction::Jmp(offset),
            InstructionBuilder::JmpNzR(offset) => Instruction::JmpNz(relative + offset),
            InstructionBuilder::JmpZR(offset) => Instruction::JmpZ(relative + offset),
            InstructionBuilder::JmpR(offset) => Instruction::Jmp(relative + offset),
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
            InstructionBuilder::Pop => Instruction::Pop,
        }
    }
}

impl Into<InstructionBuilder> for Instruction {
    fn into(self) -> InstructionBuilder {
        match self {
            Instruction::Add => InstructionBuilder::Add,
            Instruction::Sub => InstructionBuilder::Sub,
            Instruction::Mul => InstructionBuilder::Mul,
            Instruction::Div => InstructionBuilder::Div,
            Instruction::Mod => InstructionBuilder::Mod,
            Instruction::BitwiseAnd => InstructionBuilder::BitwiseAnd,
            Instruction::BitwiseOr => InstructionBuilder::BitwiseOr,
            Instruction::BitwiseXor => InstructionBuilder::BitwiseXor,
            Instruction::Shl => InstructionBuilder::Shl,
            Instruction::Shr => InstructionBuilder::Shr,
            Instruction::UnaryPlus => InstructionBuilder::UnaryPlus,
            Instruction::UnaryMinus => InstructionBuilder::UnaryMinus,
            Instruction::Not => InstructionBuilder::Not,
            Instruction::BitwiseNot => InstructionBuilder::BitwiseNot,
            Instruction::Eq => InstructionBuilder::Eq,
            Instruction::NotEq => InstructionBuilder::NotEq,
            Instruction::Lt => InstructionBuilder::Lt,
            Instruction::Lte => InstructionBuilder::Lte,
            Instruction::Gt => InstructionBuilder::Gt,
            Instruction::Gte => InstructionBuilder::Gte,
            Instruction::Assign => InstructionBuilder::Assign,
            Instruction::JmpNz(offset) => InstructionBuilder::JmpNz(offset),
            Instruction::JmpZ(offset) => InstructionBuilder::JmpZ(offset),
            Instruction::Jmp(offset) => InstructionBuilder::Jmp(offset),
            Instruction::Call => InstructionBuilder::Call,
            Instruction::Ret => InstructionBuilder::Ret,
            Instruction::LoadNone => InstructionBuilder::LoadNone,
            Instruction::LoadLitFloat(value) => InstructionBuilder::LoadLitFloat(value),
            Instruction::LoadLitInt(value) => InstructionBuilder::LoadLitInt(value),
            Instruction::LoadLitString(value) => InstructionBuilder::LoadLitString(value),
            Instruction::LoadLitBool(value) => InstructionBuilder::LoadLitBool(value),
            Instruction::ConstructArr(length) => InstructionBuilder::ConstructArr(length),
            Instruction::ConstructObj(length) => InstructionBuilder::ConstructObj(length),
            Instruction::LoadName(name) => InstructionBuilder::LoadName(name),
            Instruction::LoadObjectAttr(name) => InstructionBuilder::LoadObjectAttr(name),
            Instruction::LoadObjectIndex(index) => InstructionBuilder::LoadObjectIndex(index),
            Instruction::Terminate => InstructionBuilder::Terminate,
            Instruction::Interrupt => InstructionBuilder::Interrupt,
            Instruction::Pop => InstructionBuilder::Pop,
        }
    }
}