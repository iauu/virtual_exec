use crate::error::ParseError;
use crate::tokenizer;
use virtual_exec_type::ast::core as final_ast;

pub fn convert_stmt(stmt: tokenizer::Stmt) -> Result<final_ast::Node<final_ast::Stmt>, ParseError> {
    let kind = match stmt {
        tokenizer::Stmt::Expr(expr) => final_ast::Stmt::Expression(convert_expr(expr)),
        tokenizer::Stmt::Assign { target, value } => final_ast::Stmt::Assign {
            target: convert_assign_expr(target),
            value: convert_expr(value),
        },
        tokenizer::Stmt::Scoped(block) => {
            let body = block
                .stmts
                .into_iter()
                .map(convert_stmt)
                .collect::<Result<_, _>>()?;
            final_ast::Stmt::Scoped(body)
        }
        tokenizer::Stmt::If {
            test,
            body,
            otherwise,
        } => {
            let final_test = convert_expr(test);
            let final_body = body
                .stmts
                .into_iter()
                .map(convert_stmt)
                .collect::<Result<_, _>>()?;
            let final_otherwise = otherwise
                .map(|b| b.stmts.into_iter().map(convert_stmt).collect())
                .transpose()?;

            final_ast::Stmt::If {
                test: final_test,
                body: final_body,
                otherwise: final_otherwise,
            }
        }
        tokenizer::Stmt::Loop { test, body } => {
            let final_test = convert_expr(test);
            let final_body = body
                .stmts
                .into_iter()
                .map(convert_stmt)
                .collect::<Result<_, _>>()?;
            final_ast::Stmt::Loop {
                test: final_test,
                body: final_body,
            }
        }
        tokenizer::Stmt::Fn { name, args, body } => {
            let final_body = body
                .stmts
                .into_iter()
                .map(convert_fn_stmt)
                .collect::<Result<_, _>>()?;
            final_ast::Stmt::FunctionDef {
                name,
                args,
                body: final_body,
            }
        }
    };
    Ok(final_ast::Node { kind, span: None })
}

fn convert_fn_stmt(
    stmt: tokenizer::FnStmt,
) -> Result<final_ast::Node<final_ast::Stmt>, ParseError> {
    let kind = match stmt {
        tokenizer::FnStmt::Expr(expr) => final_ast::Stmt::Expression(convert_expr(expr)),
        tokenizer::FnStmt::Assign { target, value } => final_ast::Stmt::Assign {
            target: convert_assign_expr(target),
            value: convert_expr(value),
        },
        tokenizer::FnStmt::Scoped(block) => {
            let body = block
                .stmts
                .into_iter()
                .map(convert_stmt)
                .collect::<Result<_, _>>()?;
            final_ast::Stmt::Scoped(body)
        }
        tokenizer::FnStmt::If {
            test,
            body,
            otherwise,
        } => {
            let final_test = convert_expr(test);
            let final_body = body
                .stmts
                .into_iter()
                .map(convert_fn_stmt)
                .collect::<Result<_, _>>()?;
            let final_otherwise = otherwise
                .map(|b| b.stmts.into_iter().map(convert_fn_stmt).collect())
                .transpose()?;

            final_ast::Stmt::If {
                test: final_test,
                body: final_body,
                otherwise: final_otherwise,
            }
        }
        tokenizer::FnStmt::Loop { test, body } => {
            let final_test = convert_expr(test);
            let final_body = body
                .stmts
                .into_iter()
                .map(convert_fn_stmt)
                .collect::<Result<_, _>>()?;
            final_ast::Stmt::Loop {
                test: final_test,
                body: final_body,
            }
        }
        tokenizer::FnStmt::Fn { name, args, body } => {
            let final_body = body
                .stmts
                .into_iter()
                .map(convert_fn_stmt)
                .collect::<Result<_, _>>()?;
            final_ast::Stmt::FunctionDef {
                name,
                args,
                body: final_body,
            }
        }
        tokenizer::FnStmt::Return(expr) => final_ast::Stmt::Return(expr.map(|x| convert_expr(x))),
    };
    Ok(final_ast::Node { kind, span: None })
}

fn convert_expr(expr: tokenizer::Expr) -> final_ast::Node<final_ast::Expr> {
    let kind = match expr {
        tokenizer::Expr::Atom(atom) => match atom {
            tokenizer::Atom::Literal(l) => final_ast::Expr::Literal(l),
            tokenizer::Atom::Variable(v) => final_ast::Expr::Variable(v),
            tokenizer::Atom::Paren(expr_in_paren) => {
                final_ast::Expr::Wrapped(Box::new(convert_expr(*expr_in_paren)))
            },
        },
        tokenizer::Expr::Binary(left, op, right) => final_ast::Expr::BinaryOp {
            left: Box::new(convert_expr(*left)),
            op,
            right: Box::new(convert_expr(*right)),
        },
        tokenizer::Expr::Unary(op, operand) => final_ast::Expr::UnaryOp {
            op,
            operand: Box::new(convert_expr(*operand)),
        },
        tokenizer::Expr::Call(func, args) => final_ast::Expr::Call {
            function: Box::new(convert_expr(*func)),
            args: args.into_iter().map(convert_expr).collect(),
        },
        tokenizer::Expr::Subscript(outer, inner) => final_ast::Expr::Subscript {
            value: Box::new(convert_expr(*outer)),
            slice: Box::new(convert_expr(*inner)),
        },
        tokenizer::Expr::Attr(expr, attr) => final_ast::Expr::Attribute {
            value: Box::new(convert_expr(*expr)),
            attr,
        }
    };
    final_ast::Node { kind, span: None }
}

fn convert_assign_expr(expr: tokenizer::AssignExpr) -> final_ast::Node<final_ast::AssignExpr> {
    let kind = match expr {
        tokenizer::AssignExpr::Variable(v) => final_ast::AssignExpr::Variable(v),
        tokenizer::AssignExpr::Paren(expr_in_paren) => {
            return final_ast::Node {
                kind: final_ast::AssignExpr::Wrapped(Box::new(convert_assign_expr(*expr_in_paren))),
                span: None,
            };
        },
        tokenizer::AssignExpr::Subscript(value, slice) => final_ast::AssignExpr::Subscript {
            value: Box::new(convert_expr(*value)), slice: Box::new(convert_expr(*slice))
        },
        tokenizer::AssignExpr::Attr(value, name) => final_ast::AssignExpr::Attribute {
            value: Box::new(convert_expr(*value)), attr: name
        }
    };
    final_ast::Node { kind, span: None }
}

pub fn parse(source: &str) -> std::result::Result<final_ast::Module, ParseError> {
    let block: tokenizer::TopLevelBlock =
        syn::parse_str(source).map_err(ParseError::SynParseError)?;
    let body = block
        .stmts
        .into_iter()
        .map(convert_stmt)
        .collect::<Result<_, _>>()?;
    Ok(final_ast::Module { body, span: None })
}
