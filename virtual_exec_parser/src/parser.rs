use crate::tokenizer;
use virtual_exec_type::ast::core as final_ast;
use crate::error::ParseError;

pub fn convert_stmt(stmt: tokenizer::Stmt) -> Result<final_ast::Node<final_ast::Stmt>, ParseError> {
    let kind = match stmt {
        tokenizer::Stmt::Expr(expr) => final_ast::Stmt::Expression(convert_expr(expr)),
        tokenizer::Stmt::Assign { target, value } => {
            final_ast::Stmt::Assign {
                target: convert_assign_expr(target),
                value: convert_expr(value),
            }
        }
        tokenizer::Stmt::Scoped(block) => {
            let body = block.stmts.into_iter().map(convert_stmt).collect::<Result<_, _>>()?;
            final_ast::Stmt::Scoped(body)
        }
        tokenizer::Stmt::If { test, body, otherwise } => {
            let final_test = convert_expr(test);
            let final_body = body.stmts.into_iter().map(convert_stmt).collect::<Result<_, _>>()?;
            let final_otherwise = otherwise
                .map(|b| b.stmts.into_iter().map(convert_stmt).collect())
                .transpose()?;
            
            final_ast::Stmt::If {
                test: final_test,
                body: final_body,
                otherwise: final_otherwise,
            }
        }
    };
    Ok(final_ast::Node { kind, span: None })
}


fn convert_expr(expr: tokenizer::Expr) -> final_ast::Node<final_ast::Expr> {
    let kind = match expr {
        tokenizer::Expr::Atom(atom) => match atom {
            tokenizer::Atom::Literal(l) => final_ast::Expr::Literal(l),
            tokenizer::Atom::Variable(v) => final_ast::Expr::Variable(v),
            tokenizer::Atom::Paren(expr_in_paren) => final_ast::Expr::Wrapped(Box::new(convert_expr(*expr_in_paren)))
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
        }
    };
    final_ast::Node { kind, span: None }
}

pub fn parse(source: &str) -> std::result::Result<final_ast::Module, ParseError> {
    let block: tokenizer::TopLevelBlock = syn::parse_str(source).map_err(ParseError::SynParseError)?;
    let body = block.stmts.into_iter().map(convert_stmt).collect::<Result<_, _>>()?;
    Ok(final_ast::Module { body, span: None })
}