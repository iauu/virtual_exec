use crate::token;
use syn::parse::{Parse, ParseStream, Result};
use syn::{braced, parenthesized, Ident, Lit, Token};
use virtual_exec_type::ast::core as final_ast;

#[derive(Clone)]
pub struct TopLevelBlock {
    pub stmts: Vec<Stmt>,
}

impl Parse for TopLevelBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut stmts = Vec::new();
        while !input.is_empty() {
            stmts.push(input.parse()?);
        }
        Ok(TopLevelBlock { stmts })
    }
}

#[derive(Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Clone)]
pub enum AssignExpr {
    Variable(String),
    Paren(Box<AssignExpr>),
}

impl TryFrom<Expr> for AssignExpr {
    type Error = ();
    fn try_from(expr: Expr) -> std::result::Result<Self, Self::Error> {
        match expr {
            Expr::Atom(Atom::Variable(v)) => Ok(AssignExpr::Variable(v)),
            Expr::Atom(Atom::Paren(inner)) => {
                let inner_assign = AssignExpr::try_from(*inner)?;
                Ok(AssignExpr::Paren(Box::new(inner_assign)))
            }
            _ => Err(()),
        }
    }
}

impl Parse for AssignExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        let expr = input.parse::<Expr>()?;
        AssignExpr::try_from(expr).map_err(|_| input.error("expected an assignable expression"))
    }
}

impl Into<Expr> for AssignExpr {
    fn into(self) -> Expr {
        match self {
            AssignExpr::Variable(v) => Expr::Atom(Atom::Variable(v)),
            AssignExpr::Paren(inner) => Expr::Atom(Atom::Paren(Box::new((*inner).into()))),
        }
    }
}

#[derive(Clone)]
pub enum Stmt {
    Expr(Expr),
    Assign {
        target: AssignExpr,
        value: Expr,
    },
    If {
        test: Expr,
        body: Block,
        otherwise: Option<Block>,
    },
    Scoped(Block),
    Loop {
        test: Expr,
        body: Block
    },
}

#[derive(Clone)]
pub enum Expr {
    Atom(Atom),
    Binary(Box<Expr>, final_ast::BinaryOperator, Box<Expr>),
    Unary(final_ast::UnaryOperator, Box<Expr>),
}

#[derive(Clone)]
pub enum Atom {
    Literal(final_ast::Literal),
    Variable(String),
    Paren(Box<Expr>),
}

// --- Parser Implementation ---

impl Parse for Block {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        braced!(content in input);
        let mut stmts = Vec::new();
        while !content.is_empty() {
            stmts.push(content.parse()?);
        }
        Ok(Block { stmts })
    }
}

impl Parse for Stmt {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![if]) {
            return parse_if_statement(input);
        }
        else if input.peek(Token![while]) {
            return parse_while_statement(input);
        }

        let fork = input.fork();
        let _ = fork.parse::<Expr>()?;

        if fork.peek(Token![=])
            || fork.peek(token::PlusAssign) || fork.peek(token::MinusAssign)
            || fork.peek(token::StarAssign) || fork.peek(token::SlashAssign)
            || fork.peek(token::PercentAssign) || fork.peek(token::BitAndAssign)
            || fork.peek(token::BitOrAssign) || fork.peek(token::BitXorAssign)
            || fork.peek(token::LeftShiftAssign) || fork.peek(token::RightShiftAssign)
        {
            let target = input.parse::<AssignExpr>()?;
            let op: AssignOp = input.parse()?;
            let value = input.parse::<Expr>()?;
            input.parse::<Token![;]>()?;

            let final_value = match map_assign_op_to_binary_op(op) {
                None => value,
                Some(binary_op) => Expr::Binary(Box::new(target.clone().into()), binary_op, Box::new(value))
            };
            
            Ok(Stmt::Assign { target, value: final_value })

        }
        else if input.peek(syn::token::Brace) {
            let stmts = input.parse::<Block>()?;
            input.parse::<Token![;]>()?;
            Ok(Stmt::Scoped(stmts))
        }
        else {
            let expr = input.parse::<Expr>()?;
            input.parse::<Token![;]>()?;
            Ok(Stmt::Expr(expr))
        }
    }
}

fn parse_if_statement(input: ParseStream) -> Result<Stmt> {
    input.parse::<Token![if]>()?;
    let test = input.parse::<Expr>()?;
    let body = input.parse::<Block>()?;
    let mut otherwise = None;

    if input.peek(Token![else]) {
        input.parse::<Token![else]>()?;
        if input.peek(Token![if]) {
            let nested_if = parse_if_statement(input)?;
            otherwise = Some(Block { stmts: vec![nested_if] });
        } else {
            otherwise = Some(input.parse::<Block>()?);
        }
    }

    Ok(Stmt::If { test, body, otherwise })
}

fn parse_while_statement(input: ParseStream) -> Result<Stmt> {
    input.parse::<Token![while]>()?;
    let test = input.parse::<Expr>()?;
    let body = input.parse::<Block>()?;
    Ok(Stmt::Loop { test, body })
}

impl Parse for Expr {
    fn parse(input: ParseStream) -> Result<Self> {
        parse_expr_with_precedence(input, 0)
    }
}

fn parse_expr_with_precedence(input: ParseStream, min_bp: u8) -> Result<Expr> {
    let mut lhs = if input.peek(Token![!]) {
        input.parse::<Token![!]>()?;
        let rhs = parse_expr_with_precedence(input, prefix_binding_power(&final_ast::UnaryOperator::Not))?;
        Expr::Unary(final_ast::UnaryOperator::Not, Box::new(rhs))
    } else if input.peek(Token![-]) {
        input.parse::<Token![-]>()?;
        let rhs = parse_expr_with_precedence(input, prefix_binding_power(&final_ast::UnaryOperator::Negative))?;
        Expr::Unary(final_ast::UnaryOperator::Negative, Box::new(rhs))
    } else {
        Expr::Atom(input.parse()?)
    };

    loop {
        let (op, _l_bp, r_bp) = match peek_infix_op(input) {
            Some(op_data) if op_data.1 >= min_bp => op_data,
            _ => break,
        };
        consume_op(input, &op)?;
        let rhs = parse_expr_with_precedence(input, r_bp)?;
        lhs = Expr::Binary(Box::new(lhs), op, Box::new(rhs));
    }
    Ok(lhs)
}

impl Parse for Atom {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(token::None) {
            input.parse::<token::None>()?;
            return Ok(Atom::Literal(final_ast::Literal::None));
        }
        else if input.peek(Lit) {
            let lit: Lit = input.parse()?;
            let final_lit = match lit {
                Lit::Int(i) => final_ast::Literal::Int(i.base10_parse()?),
                Lit::Float(f) => final_ast::Literal::Float(f.base10_parse()?),
                Lit::Str(s) => final_ast::Literal::String(s.value()),
                Lit::Bool(b) => final_ast::Literal::Bool(b.value),
                _ => return Err(input.error("unsupported literal type")),
            };
            Ok(Atom::Literal(final_lit))
        } else if input.peek(Ident) {
            let ident: Ident = input.parse()?;
            Ok(Atom::Variable(ident.to_string()))
        } else if input.peek(syn::token::Paren) {
            let content;
            parenthesized!(content in input);
            Ok(Atom::Paren(Box::new(content.parse()?)))
        } else {
            Err(input.error("expected a literal, an identifier, or a parenthesized expression"))
        }
    }
}


#[derive(Clone, Copy)]
pub enum AssignOp {
    Assign, Add, Sub, Mul, Div, Mod, BitAnd, BitOr, BitXor, Shl, Shr,
}

impl Parse for AssignOp {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![=]) { input.parse::<Token![=]>()?; Ok(AssignOp::Assign) }
        else if input.peek(token::PlusAssign) { input.parse::<token::PlusAssign>()?; Ok(AssignOp::Add) }
        else if input.peek(token::MinusAssign) { input.parse::<token::MinusAssign>()?; Ok(AssignOp::Sub) }
        else if input.peek(token::StarAssign) { input.parse::<token::StarAssign>()?; Ok(AssignOp::Mul) }
        else if input.peek(token::SlashAssign) { input.parse::<token::SlashAssign>()?; Ok(AssignOp::Div) }
        else if input.peek(token::PercentAssign) { input.parse::<token::PercentAssign>()?; Ok(AssignOp::Mod) }
        else if input.peek(token::BitAndAssign) { input.parse::<token::BitAndAssign>()?; Ok(AssignOp::BitAnd) }
        else if input.peek(token::BitOrAssign) { input.parse::<token::BitOrAssign>()?; Ok(AssignOp::BitOr) }
        else if input.peek(token::BitXorAssign) { input.parse::<token::BitXorAssign>()?; Ok(AssignOp::BitXor) }
        else if input.peek(token::LeftShiftAssign) { input.parse::<token::LeftShiftAssign>()?; Ok(AssignOp::Shl) }
        else if input.peek(token::RightShiftAssign) { input.parse::<token::RightShiftAssign>()?; Ok(AssignOp::Shr) }
        else { Err(input.error("unsupported assignment operator")) }
    }
}

fn map_assign_op_to_binary_op(op: AssignOp) -> Option<final_ast::BinaryOperator> {
    match op {
        AssignOp::Assign => None,
        AssignOp::Add => Some(final_ast::BinaryOperator::Add),
        AssignOp::Sub => Some(final_ast::BinaryOperator::Subtract),
        AssignOp::Mul => Some(final_ast::BinaryOperator::Multiply),
        AssignOp::Div => Some(final_ast::BinaryOperator::Divide),
        AssignOp::Mod => Some(final_ast::BinaryOperator::Modulo),
        AssignOp::BitAnd => Some(final_ast::BinaryOperator::BitwiseAnd),
        AssignOp::BitOr => Some(final_ast::BinaryOperator::BitwiseOr),
        AssignOp::BitXor => Some(final_ast::BinaryOperator::Xor),
        AssignOp::Shl => Some(final_ast::BinaryOperator::LeftShift),
        AssignOp::Shr => Some(final_ast::BinaryOperator::RightShift),
    }
}

fn prefix_binding_power(op: &final_ast::UnaryOperator) -> u8 {
    match op {
        final_ast::UnaryOperator::Not => 9,
        final_ast::UnaryOperator::Negative | final_ast::UnaryOperator::Positive => 9,
    }
}

fn infix_binding_power(op: &final_ast::BinaryOperator) -> (u8, u8) {
    match op {
        final_ast::BinaryOperator::Or => (1, 2),
        final_ast::BinaryOperator::And => (3, 4),
        final_ast::BinaryOperator::Eq | final_ast::BinaryOperator::NotEq => (5, 6),
        final_ast::BinaryOperator::Lt | final_ast::BinaryOperator::Lte | final_ast::BinaryOperator::Gt | final_ast::BinaryOperator::Gte => (7, 8),
        final_ast::BinaryOperator::BitwiseOr => (9, 10),
        final_ast::BinaryOperator::Xor => (11, 12),
        final_ast::BinaryOperator::BitwiseAnd => (13, 14),
        final_ast::BinaryOperator::LeftShift | final_ast::BinaryOperator::RightShift => (15, 16),
        final_ast::BinaryOperator::Add | final_ast::BinaryOperator::Subtract => (17, 18),
        final_ast::BinaryOperator::Multiply | final_ast::BinaryOperator::Divide | final_ast::BinaryOperator::Modulo => (19, 20),
    }
}

fn peek_infix_op(input: ParseStream) -> Option<(final_ast::BinaryOperator, u8, u8)> {
    let op = if input.peek(Token![&&]) { final_ast::BinaryOperator::And }
    else if input.peek(Token![||]) { final_ast::BinaryOperator::Or }
    else if input.peek(Token![==]) { final_ast::BinaryOperator::Eq }
    else if input.peek(Token![!=]) { final_ast::BinaryOperator::NotEq }
    else if input.peek(Token![<=]) { final_ast::BinaryOperator::Lte }
    else if input.peek(Token![>=]) { final_ast::BinaryOperator::Gte }
    else if input.peek(token::LeftShiftAssign) { return None }
    else if input.peek(Token![<<]) { final_ast::BinaryOperator::LeftShift }
    else if input.peek(token::RightShiftAssign) { return None }
    else if input.peek(Token![>>]) { final_ast::BinaryOperator::RightShift }
    else if input.peek(Token![<]) { final_ast::BinaryOperator::Lt }
    else if input.peek(Token![>]) { final_ast::BinaryOperator::Gt }
    else if input.peek(token::PlusAssign) { return None }
    else if input.peek(Token![+]) { final_ast::BinaryOperator::Add }
    else if input.peek(token::MinusAssign) { return None }
    else if input.peek(Token![-]) { final_ast::BinaryOperator::Subtract }
    else if input.peek(token::StarAssign) { return None }
    else if input.peek(Token![*]) { final_ast::BinaryOperator::Multiply }
    else if input.peek(token::SlashAssign) { return None }
    else if input.peek(Token![/]) { final_ast::BinaryOperator::Divide }
    else if input.peek(token::PercentAssign) { return None }
    else if input.peek(Token![%]) { final_ast::BinaryOperator::Modulo }
    else if input.peek(token::BitAndAssign) { return None }
    else if input.peek(Token![&]) { final_ast::BinaryOperator::BitwiseAnd }
    else if input.peek(token::BitOrAssign) { return None }
    else if input.peek(Token![|]) { final_ast::BinaryOperator::BitwiseOr }
    else if input.peek(token::BitXorAssign) { return None }
    else if input.peek(Token![^]) { final_ast::BinaryOperator::Xor }
    else { return None; };
    let (l_bp, r_bp) = infix_binding_power(&op);
    Some((op, l_bp, r_bp))
}

fn consume_op(input: ParseStream, op: &final_ast::BinaryOperator) -> Result<()> {
    match op {
        final_ast::BinaryOperator::Add => input.parse::<Token![+]>().map(|_| ()),
        final_ast::BinaryOperator::Subtract => input.parse::<Token![-]>().map(|_| ()),
        final_ast::BinaryOperator::Multiply => input.parse::<Token![*]>().map(|_| ()),
        final_ast::BinaryOperator::Divide => input.parse::<Token![/]>().map(|_| ()),
        final_ast::BinaryOperator::Modulo => input.parse::<Token![%]>().map(|_| ()),
        final_ast::BinaryOperator::And => input.parse::<Token![&&]>().map(|_| ()),
        final_ast::BinaryOperator::Or => input.parse::<Token![||]>().map(|_| ()),
        final_ast::BinaryOperator::Eq => input.parse::<Token![==]>().map(|_| ()),
        final_ast::BinaryOperator::NotEq => input.parse::<Token![!=]>().map(|_| ()),
        final_ast::BinaryOperator::Lt => input.parse::<Token![<]>().map(|_| ()),
        final_ast::BinaryOperator::Lte => input.parse::<Token![<=]>().map(|_| ()),
        final_ast::BinaryOperator::Gt => input.parse::<Token![>]>().map(|_| ()),
        final_ast::BinaryOperator::Gte => input.parse::<Token![>=]>().map(|_| ()),
        final_ast::BinaryOperator::Xor => input.parse::<Token![^]>().map(|_| ()),
        final_ast::BinaryOperator::BitwiseAnd => input.parse::<Token![&]>().map(|_| ()),
        final_ast::BinaryOperator::BitwiseOr => input.parse::<Token![|]>().map(|_| ()),
        final_ast::BinaryOperator::LeftShift => input.parse::<Token![<<]>().map(|_| ()),
        final_ast::BinaryOperator::RightShift => input.parse::<Token![>>]>().map(|_| ())
    }
}
