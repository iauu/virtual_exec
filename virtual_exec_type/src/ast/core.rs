use crate::mem::ValuePtr;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub row: u64,
    pub col: u64,
    pub length: u64,
}

pub trait ASTNode {
    type Output<'ctx>; // = ValueKind<'ctx>; // Oh cool that this is a unstable feature?

    fn get_callsite(&self) -> Option<Span>;
}
#[derive(Debug, Clone)]
pub struct Node<T>
where
    T: ASTNode,
{
    pub kind: T,
    pub span: Option<Span>,
}

pub struct Module {
    pub body: Vec<Node<Stmt>>,
    pub span: Option<Span>,
}

impl ASTNode for Module {
    type Output<'ctx> = ValuePtr<'ctx>;

    fn get_callsite(&self) -> Option<Span> {
        self.span
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Variable(String),
    BinaryOp {
        left: Box<Node<Expr>>,
        op: BinaryOperator,
        right: Box<Node<Expr>>,
    },
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Node<Expr>>,
    },
    Wrapped(Box<Node<Expr>>),
    Call {
        function: Box<Node<Expr>>,
        args: Vec<Node<Expr>>,
    },
    Attribute {
        value: Box<Node<Expr>>,
        attr: String,
    },
    Subscript {
        value: Box<Node<Expr>>,
        slice: Box<Node<Expr>>,
    },
    // Range {
    //     lower: Option<i64>,
    //     upper: Option<i64>,
    //     step: Option<i64>,
    // },
}

impl ASTNode for Expr {
    type Output<'ctx> = ValuePtr<'ctx>;

    fn get_callsite(&self) -> Option<Span> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum AssignExpr {
    Variable(String),
    Wrapped(Box<Node<AssignExpr>>),
    // Attribute {
    //     value: Box<Node<Expr>>,
    //     attr: String,
    // },
    // Subscript {
    //     value: Box<Node<Expr>>,
    //     slice: Box<Node<Expr>>,
    // },
    // Range {
    //     lower: Option<i64>,
    //     upper: Option<i64>,
    //     step: Option<i64>,
    // },
}
impl ASTNode for AssignExpr {
    type Output<'ctx> = ValuePtr<'ctx>;

    fn get_callsite(&self) -> Option<Span> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    None,
}

impl ASTNode for Literal {
    type Output<'ctx> = ValuePtr<'ctx>;

    fn get_callsite(&self) -> Option<Span> {
        todo!()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    And,
    Or,
    Xor,
    Modulo,
    BitwiseAnd,
    BitwiseOr,
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
    LeftShift,
    RightShift,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOperator {
    Positive,
    Negative,
    Not,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Node<Expr>),
    Assign {
        target: Node<AssignExpr>, // temp since no a[b] or a.b right now
        value: Node<Expr>,
    },
    If {
        test: Node<Expr>,
        body: Vec<Node<Stmt>>,
        otherwise: Option<Vec<Node<Stmt>>>,
    },
    Scoped(Vec<Node<Stmt>>),
    Loop {
        test: Node<Expr>,
        body: Vec<Node<Stmt>>,
    },
    FunctionDef {
        name: String,
        args: Vec<String>,
        body: Vec<Node<Stmt>>,
    },
    Return(Option<Node<Expr>>),
    // ClassDef {
    //     name: String,
    //     bases: Vec<Node<Expr>>,
    //     body: Vec<Node<Stmt>>,
    // },
    // ForLoop {
    //     target: Node<Expr>, // Added target
    //     iter_expr: Node<Expr>,
    //     body: Vec<Node<Stmt>>,
    //     not_break: Vec<Node<Stmt>>,
    // },
    // WhileLoop {
    //     test: Node<Expr>,
    //     body: Vec<Node<Stmt>>,
    //     otherwise: Option<Vec<Node<Stmt>>>, // Added otherwise
    // },
    // Break,
    // Continue,
}

impl ASTNode for Stmt {
    type Output<'ctx> = ValuePtr<'ctx>;

    fn get_callsite(&self) -> Option<Span> {
        todo!()
    }
}
