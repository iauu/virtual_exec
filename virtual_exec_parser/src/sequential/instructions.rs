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
    LoadObjectIndex(i64)
}