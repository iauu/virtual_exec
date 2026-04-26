use crate::mem::ValuePtr;

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub row: u64,
    pub col: u64,
    pub length: u64,
}

pub trait ASTNode {
    type Output<'ctx>; // = ValueKind<'ctx>; // Oh cool that this is a unstable feature?
    // fn eval<'ctx>(&self, ctx: Rc<RefCell<ExecutionContext<'ctx>>>) -> Result<Self::Output<'ctx>>;

    fn get_callsite(&self) -> Option<Span>;
}

// fn with_arena<'ctx, F, R>(ctx: &Rc<RefCell<ExecutionContext<'ctx>>>, f: F) -> R
// where
//     F: FnOnce(&'ctx bumpalo::Bump) -> R,
// {
//     let arena_rc = { ctx.borrow().arena.clone() };
//     let arena_borrow: Ref<bumpalo::Bump> = arena_rc.borrow();
//     let arena_ref: &bumpalo::Bump = &arena_borrow;
//     let arena_ref_ctx: &'ctx bumpalo::Bump = unsafe { std::mem::transmute(arena_ref) };
//
//     f(arena_ref_ctx)
// }

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
    // fn eval<'ctx>(&self, ctx: Rc<RefCell<ExecutionContext<'ctx>>>) -> Result<ValueKind<'ctx>> {
    //     let result = catch_unwind(std::panic::AssertUnwindSafe(|| {
    //         for stmt in self.body.clone() {
    //             stmt.kind.eval(ctx.clone())?;
    //             ctx.borrow_mut().consume_one()?;
    //         }
    //         Ok(ValueKind::None)
    //     }));
    // 
    //     match result {
    //         Ok(Ok(value)) => Ok(value),
    //         Ok(Err(e)) => Err(e),
    //         Err(_) => Err(SandboxExecutionError::GenericPanicRewindError),
    //     }
    // }

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
    // Call {
    //     function: Box<Node<Expr>>,
    //     args: Vec<Node<Expr>>,
    // },
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

impl ASTNode for Expr {
    type Output<'ctx> = ValuePtr<'ctx>;

    // fn eval<'ctx>(&self, ctx: Rc<RefCell<ExecutionContext<'ctx>>>) -> Result<Self::Output<'ctx>> {
    //     ctx.borrow_mut().consume_one()?;
    //     match self {
    //         Expr::Literal(l) => l.eval(ctx),
    //         Expr::Variable(v) => Ok(ctx.borrow().get(v)?.borrow().kind.clone()),
    //         Expr::UnaryOp { op, operand } => {
    //             let rhs_kind = operand.kind.eval(ctx.clone())?;
    //             let arena = ctx.borrow().arena.clone();
    //             let rhs = ctx.borrow().arena.allocate(rhs_kind);
    //             match op {
    //                 UnaryOperator::Negative => Ok(err_op_neg(rhs, &arena)?.kind.clone()),
    //                 UnaryOperator::Positive => Ok(err_op_pos(rhs, &arena)?.kind.clone()),
    //                 UnaryOperator::Not => Ok(err_op_not(rhs, &arena)?.kind.clone()),
    //             }
    //         }
    //         Expr::BinaryOp { left, op, right } => {
    //             let lhs_kind = left.kind.eval(ctx.clone())?;
    //             // Special Case:
    //             match (op, &lhs_kind) {
    //                 (BinaryOperator::And, ValueKind::Bool(false) | ValueKind::None) => {
    //                     return Ok(ValueKind::Bool(false));
    //                 }
    //                 (BinaryOperator::Or, ValueKind::Bool(true)) => {
    //                     return Ok(ValueKind::Bool(true));
    //                 }
    //                 (BinaryOperator::Or, ValueKind::None | ValueKind::Bool(false))
    //                 | (BinaryOperator::And, ValueKind::Bool(true)) => {
    //                     return right.kind.eval(ctx.clone());
    //                 }
    //                 _ => {}
    //             }
    //             let rhs_kind = right.kind.eval(ctx.clone())?;
    //             let arena = ctx.borrow().arena.clone();
    //             let lhs = arena.allocate(lhs_kind);
    //             let rhs = arena.allocate(rhs_kind);
    //             match op {
    //                 BinaryOperator::Add => Ok(err_op_add(lhs, rhs, &arena)?.kind.clone()),
    //                 BinaryOperator::Subtract => Ok(err_op_sub(lhs, rhs, &arena)?.kind.clone()),
    //                 BinaryOperator::Multiply => Ok(err_op_mul(lhs, rhs, &arena)?.kind.clone()),
    //                 BinaryOperator::Divide => Ok(err_op_div(lhs, rhs, &arena)?.kind.clone()),
    //                 BinaryOperator::And => Ok(err_op_and(lhs, rhs, &arena)?.kind.clone()),
    //                 BinaryOperator::Or => Ok(err_op_or(lhs, rhs, &arena)?.kind.clone()),
    //                 BinaryOperator::Xor => Ok(err_op_bxor(lhs, rhs, &arena)?.kind.clone()),
    //                 BinaryOperator::Modulo => Ok(err_op_moduls(lhs, rhs, &arena)?.kind.clone()),
    //                 BinaryOperator::BitwiseAnd => {
    //                     Ok(err_op_band(lhs, rhs, &arena)?.kind.clone())
    //                 }
    //                 BinaryOperator::BitwiseOr => Ok(err_op_bor(lhs, rhs, &arena)?.kind.clone()),
    //                 BinaryOperator::Eq => Ok(err_op_eq(lhs, rhs, &arena)?.kind.clone()),
    //                 BinaryOperator::NotEq => Ok(err_op_ne(lhs, rhs, &arena)?.kind.clone()),
    //                 BinaryOperator::Lt => Ok(err_op_lt(lhs, rhs, &arena)?.kind.clone()),
    //                 BinaryOperator::Lte => Ok(err_op_le(lhs, rhs, &arena)?.kind.clone()),
    //                 BinaryOperator::Gt => Ok(err_op_gt(lhs, rhs, &arena)?.kind.clone()),
    //                 BinaryOperator::Gte => Ok(err_op_ge(lhs, rhs, &arena)?.kind.clone()),
    //                 BinaryOperator::LeftShift => Ok(err_op_bsl(lhs, rhs, &arena)?.kind.clone()),
    //                 BinaryOperator::RightShift => Ok(err_op_bsr(lhs, rhs, &arena)?.kind.clone()),
    //             }
    //         }
    //         Expr::Wrapped(expr) => expr.kind.eval(ctx.clone()),
    //     }
    // }

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

    // fn eval<'ctx>(&self, ctx: Rc<RefCell<ExecutionContext<'ctx>>>) -> Result<Self::Output<'ctx>> {
    //     ctx.borrow_mut().consume_one()?;
    //     match self {
    //         AssignExpr::Variable(v) => Ok(ctx.borrow().get(v)?.borrow().kind.clone()),
    //         AssignExpr::Wrapped(expr) => expr.kind.eval(ctx.clone()),
    //     }
    // }

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

    // fn eval<'ctx>(&self, ctx: Rc<RefCell<ExecutionContext<'ctx>>>) -> Result<Self::Output<'ctx>> {
    //     ctx.borrow_mut().consume_one()?;
    //     match self {
    //         Literal::Bool(v) => Ok(ValueKind::Bool(*v)),
    //         Literal::None => Ok(ValueKind::None),
    //         Literal::Int(v) => Ok(ValueKind::Int(VirInt::new(*v))),
    //         Literal::Float(v) => Ok(ValueKind::Float(VirFloat::new(*v))),
    //         Literal::String(v) => Ok(ValueKind::String(v.clone())),
    //     }
    // }

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
    RightShift
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
    }
    // FunctionDef {
    //     name: String,
    //     args: Vec<String>,
    //     body: Vec<Node<Stmt>>,
    // },
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
    // Return(Option<Node<Expr>>),
    // Break,
    // Continue,
}

impl ASTNode for Stmt {
    type Output<'ctx> = ValuePtr<'ctx>;

    // fn eval<'ctx>(&self, ctx: Rc<RefCell<ExecutionContext<'ctx>>>) -> Result<Self::Output<'ctx>> {
    //     ctx.borrow_mut().consume_one()?;
    //     match self {
    //         Stmt::Expression(expr) => {
    //             expr.kind.eval(ctx.clone())?;
    //         }
    //         Stmt::Assign { target, value } => {
    //             let value_kind = value.kind.eval(ctx.clone())?;
    //             match &target.kind {
    //                 AssignExpr::Variable(name) => {
    //                     let value_container = ctx.borrow().arena.allocate(value_kind);
    //                     ctx.borrow_mut()
    //                         .get_ignore_missing(&name, value_container)?;
    //                     Ok::<(), SandboxExecutionError>(())
    //                 }
    //                 _ => {
    //                     Err(SandboxExecutionError::InvalidSyntaxError)?
    //                 }
    //             }?
    //         }
    //         Stmt::If {
    //             test,
    //             body,
    //             otherwise,
    //         } => {
    //             let value_kind = test.kind.eval(ctx.clone())?;
    //             match value_kind {
    //                 ValueKind::Bool(true) => {
    //                     for stmt in body.clone() {
    //                         stmt.kind.eval(ctx.clone())?;
    //                         ctx.borrow_mut().consume_one()?;
    //                     }
    //                 }
    //                 ValueKind::Bool(false) | ValueKind::None => {
    //                     if let Some(otherwise) = otherwise {
    //                         for stmt in otherwise.clone() {
    //                             stmt.kind.eval(ctx.clone())?;
    //                             ctx.borrow_mut().consume_one()?;
    //                         }
    //                     }
    //                 }
    //                 _ => return Err(SandboxExecutionError::InvalidTypeError),
    //             }
    //         },
    //         Stmt::Scoped(scoped) => {
    //             for stmt in scoped.clone() {
    //                 stmt.kind.eval(ctx.clone())?;
    //             }
    //         }
    //     };
    //     Ok(ValueKind::None)
    // }

    fn get_callsite(&self) -> Option<Span> {
        todo!()
    }
}
