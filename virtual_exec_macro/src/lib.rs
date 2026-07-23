use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{ToTokens, quote};
use std::ops::Deref;
use syn::spanned::Spanned;
use syn::{FnArg, ItemFn, Pat, Type, parse_macro_input, parse_quote};
use virtual_exec_core::sequential::instructions::{Instruction, SubscriptLoad};
use virtual_exec_parser::parser::convert_stmt;
use virtual_exec_parser::tokenizer::{AssignExpr, Atom, Expr, Stmt, TopLevelBlock};
use virtual_exec_type::ast::core::{BinaryOperator, Literal, Module, UnaryOperator};

fn literal_to_token(lit: Literal) -> impl ToTokens {
    match lit {
        Literal::Int(v) => quote! { ::virtual_exec_type::ast::core::Literal::Int(#v) },
        Literal::Float(v) => quote! { ::virtual_exec_type::ast::core::Literal::Float(#v) },
        Literal::String(v) => {
            quote! { ::virtual_exec_type::ast::core::Literal::String(#v.to_string()) }
        }
        Literal::Bool(v) => quote! { ::virtual_exec_type::ast::core::Literal::Bool(#v) },
        Literal::None => quote! { ::virtual_exec_type::ast::core::Literal::None },
    }
}

fn binary_op_to_token(op: BinaryOperator) -> impl ToTokens {
    match op {
        BinaryOperator::Add => quote! { ::virtual_exec_type::ast::core::BinaryOperator::Add },
        BinaryOperator::Subtract => {
            quote! { ::virtual_exec_type::ast::core::BinaryOperator::Subtract }
        }
        BinaryOperator::Multiply => {
            quote! { ::virtual_exec_type::ast::core::BinaryOperator::Multiply }
        }
        BinaryOperator::Divide => quote! { ::virtual_exec_type::ast::core::BinaryOperator::Divide },
        BinaryOperator::And => quote! { ::virtual_exec_type::ast::core::BinaryOperator::And },
        BinaryOperator::Or => quote! { ::virtual_exec_type::ast::core::BinaryOperator::Or },
        BinaryOperator::Xor => quote! { ::virtual_exec_type::ast::core::BinaryOperator::Xor },
        BinaryOperator::Modulo => quote! { ::virtual_exec_type::ast::core::BinaryOperator::Modulo },
        BinaryOperator::BitwiseAnd => {
            quote! { ::virtual_exec_type::ast::core::BinaryOperator::BitwiseAnd }
        }
        BinaryOperator::BitwiseOr => {
            quote! { ::virtual_exec_type::ast::core::BinaryOperator::BitwiseOr }
        }
        BinaryOperator::Eq => quote! { ::virtual_exec_type::ast::core::BinaryOperator::Eq },
        BinaryOperator::NotEq => quote! { ::virtual_exec_type::ast::core::BinaryOperator::NotEq },
        BinaryOperator::Lt => quote! { ::virtual_exec_type::ast::core::BinaryOperator::Lt },
        BinaryOperator::Lte => quote! { ::virtual_exec_type::ast::core::BinaryOperator::Lte },
        BinaryOperator::Gt => quote! { ::virtual_exec_type::ast::core::BinaryOperator::Gt },
        BinaryOperator::Gte => quote! { ::virtual_exec_type::ast::core::BinaryOperator::Gte },
        BinaryOperator::LeftShift => {
            quote! { ::virtual_exec_type::ast::core::BinaryOperator::LeftShift }
        }
        BinaryOperator::RightShift => {
            quote! { ::virtual_exec_type::ast::core::BinaryOperator::RightShift }
        }
    }
}

fn unary_op_to_token(op: UnaryOperator) -> impl ToTokens {
    match op {
        UnaryOperator::Positive => {
            quote! { ::virtual_exec_type::ast::core::UnaryOperator::Positive }
        }
        UnaryOperator::Negative => {
            quote! { ::virtual_exec_type::ast::core::UnaryOperator::Negative }
        }
        UnaryOperator::Not => quote! { ::virtual_exec_type::ast::core::UnaryOperator::Not },
    }
}

fn atom_to_token(atom: Atom) -> TokenStream2 {
    match atom {
        Atom::Literal(l) => {
            let lit_token = literal_to_token(l);
            quote! { ::virtual_exec_type::ast::core::Expr::Literal(#lit_token) }
        }
        Atom::Variable(v) => {
            quote! { ::virtual_exec_type::ast::core::Expr::Variable(#v.to_string()) }
        }
        Atom::Paren(expr) => {
            let expr_token = expr_to_token(*expr);
            quote! {
                ::virtual_exec_type::ast::core::Expr::Wrapped(
                    Box::new(#expr_token)
                )
            }
        }
    }
}

fn expr_to_token(expr: Expr) -> impl ToTokens {
    let kind = match expr {
        Expr::Atom(atom) => atom_to_token(atom),
        Expr::Binary(left, op, right) => {
            let left_token = expr_to_token(*left);
            let op_token = binary_op_to_token(op);
            let right_token = expr_to_token(*right);
            quote! {
                ::virtual_exec_type::ast::core::Expr::BinaryOp {
                    left: Box::new(#left_token),
                    op: #op_token,
                    right: Box::new(#right_token),
                }
            }
        }
        Expr::Unary(op, operand) => {
            let op_token = unary_op_to_token(op);
            let operand_token = expr_to_token(*operand);
            quote! {
                ::virtual_exec_type::ast::core::Expr::UnaryOp {
                    op: #op_token,
                    operand: Box::new(#operand_token),
                }
            }
        }
        Expr::Call(func, args) => {
            let func = expr_to_token(*func);
            let args = args
                .iter()
                .map(|x| expr_to_token(x.clone()))
                .collect::<Vec<_>>();
            quote! {
                ::virtual_exec_type::ast::core::Expr::Call {
                    function: Box::new(#func),
                    args: vec![ #(#args),* ],
                }
            }
        },
        Expr::Subscript(value, slice) => {
            let value = expr_to_token(*value);
            let slice = expr_to_token(*slice);
            quote! {
                ::virtual_exec_type::ast::core::Expr::Subscript {
                    value: Box::new(#value),
                    slice: Box::new(#slice),
                }
            }
        },
        Expr::Attr(value, attr) => {
            let value = expr_to_token(*value);
            quote! {
                ::virtual_exec_type::ast::core::Expr::Attr {
                    value: Box::new(#value),
                    #attr
                }
            }
        }
    };
    quote! {
        ::virtual_exec_type::ast::core::Node {
            kind: #kind,
            span: None,
        }
    }
}

fn assign_expr_to_token(expr: AssignExpr) -> impl ToTokens {
    let kind = match expr {
        AssignExpr::Variable(v) => {
            quote! { ::virtual_exec_type::ast::core::AssignExpr::Variable(#v.to_string()) }
        }
        AssignExpr::Paren(inner) => {
            let inner_token = assign_expr_to_token(*inner);
            quote! {
                ::virtual_exec_type::ast::core::AssignExpr::Wrapped(
                    Box::new(#inner_token)
                )
            }
        }
        AssignExpr::Subscript(value, slice) => {
            let value = expr_to_token(*value);
            let slice = expr_to_token(*slice);
            quote! {
                ::virtual_exec_type::ast::core::AssignExpr::Subscript {
                    value: Box::new(#value),
                    slice: Box::new(#slice),
                }
            }
        }
        AssignExpr::Attr(value, name) => {
            let value = expr_to_token(*value);
            quote! {
                ::virtual_exec_type::ast::core::AssignExpr::Attr {
                    value: Box::new(value),
                    name: (#name).to_string()
                }
            }
        }
    };
    quote! {
        ::virtual_exec_type::ast::core::Node {
            kind: #kind,
            span: None,
        }
    }
}

fn stmts_to_token(stmts: Vec<Stmt>) -> impl ToTokens {
    let mut e = Vec::new();
    for stmt in stmts {
        let tokens = stmt_to_token(stmt);
        e.push(tokens);
    }
    quote! {
        vec![
            #(#e),*
        ]
    }
}

fn fn_stmts_to_token(stmts: Vec<virtual_exec_parser::tokenizer::FnStmt>) -> impl ToTokens {
    let mut e = Vec::new();
    for fn_stmt in stmts {
        e.push(fn_stmt_to_token(fn_stmt));
    }
    quote! {
        vec![
            #(#e),*
        ]
    }
}

fn fn_stmt_to_token(stmt: virtual_exec_parser::tokenizer::FnStmt) -> impl ToTokens {
    match stmt {
        virtual_exec_parser::tokenizer::FnStmt::Expr(expr) => {
            let expr_token = expr_to_token(expr);
            quote! {
                ::virtual_exec_type::ast::core::Node {
                    kind: ::virtual_exec_type::ast::core::Stmt::Expression( #expr_token ),
                    span: None,
                }
            }
        }
        virtual_exec_parser::tokenizer::FnStmt::Assign { target, value } => {
            let target_token = assign_expr_to_token(target);
            let value_token = expr_to_token(value);
            quote! {
                ::virtual_exec_type::ast::core::Node {
                    kind: ::virtual_exec_type::ast::core::Stmt::Assign {
                        target: #target_token,
                        value: #value_token,
                    },
                    span: None,
                }
            }
        }
        virtual_exec_parser::tokenizer::FnStmt::If {
            test,
            body,
            otherwise,
        } => {
            let test_token = expr_to_token(test);
            let body_token = fn_stmts_to_token(body.stmts);
            let otherwise_token = match otherwise {
                Some(b) => {
                    let stmts = fn_stmts_to_token(b.stmts);
                    quote! { Some(#stmts) }
                }
                None => quote! { None },
            };

            quote! {
                ::virtual_exec_type::ast::core::Node {
                    kind: ::virtual_exec_type::ast::core::Stmt::If {
                        test: #test_token,
                        body: #body_token,
                        otherwise: #otherwise_token,
                    },
                    span: None,
                }
            }
        }
        virtual_exec_parser::tokenizer::FnStmt::Scoped(block) => {
            let stmts = stmts_to_token(block.stmts);
            quote! {
                ::virtual_exec_type::ast::core::Node {
                    kind: ::virtual_exec_type::ast::core::Stmt::Scoped(#stmts),
                    span: None,
                }
            }
        }
        virtual_exec_parser::tokenizer::FnStmt::Loop { test, body } => {
            let test_token = expr_to_token(test);
            let body_token = fn_stmts_to_token(body.stmts);
            quote! {
                ::virtual_exec_type::ast::core::Node {
                    kind: ::virtual_exec_type::ast::core::Stmt::Loop {
                        test: #test_token,
                        body: #body_token,
                    },
                    span: None,
                }
            }
        }
        virtual_exec_parser::tokenizer::FnStmt::Return(expr) => {
            let expr_token = match expr {
                Some(e) => {
                    let et = expr_to_token(e);
                    quote! { Some(#et) }
                }
                None => quote! { None },
            };
            quote! {
                ::virtual_exec_type::ast::core::Node {
                    kind: ::virtual_exec_type::ast::core::Stmt::Return( #expr_token ),
                    span: None,
                }
            }
        }
        virtual_exec_parser::tokenizer::FnStmt::Fn { name, args, body } => {
            let body_token = fn_stmts_to_token(body.stmts);

            quote! {
                ::virtual_exec_type::ast::core::Node {
                    kind: ::virtual_exec_type::ast::core::Stmt::FunctionDef {
                        name: #name.to_string(),
                        args: vec![ #(#args.to_string()),* ],
                        body: #body_token,
                    },
                    span: None,
                }
            }
        }
    }
}

fn stmt_to_token(stmt: Stmt) -> impl ToTokens {
    match stmt {
        Stmt::Expr(expr) => {
            let expr_token = expr_to_token(expr);
            quote! {
                ::virtual_exec_type::ast::core::Node {
                    kind: ::virtual_exec_type::ast::core::Stmt::Expression( #expr_token ),
                    span: None,
                }
            }
        }
        Stmt::Assign { target, value } => {
            let target_token = assign_expr_to_token(target);
            let value_token = expr_to_token(value);
            quote! {
                ::virtual_exec_type::ast::core::Node {
                    kind: ::virtual_exec_type::ast::core::Stmt::Assign {
                        target: #target_token,
                        value: #value_token,
                    },
                    span: None,
                }
            }
        }
        Stmt::If {
            test,
            body,
            otherwise,
        } => {
            let test_token = expr_to_token(test);
            let body_token = stmts_to_token(body.stmts);
            let otherwise_token = match otherwise {
                Some(b) => {
                    let stmts = stmts_to_token(b.stmts);
                    quote! { Some(#stmts) }
                }
                None => quote! { None },
            };

            quote! {
                ::virtual_exec_type::ast::core::Node {
                    kind: ::virtual_exec_type::ast::core::Stmt::If {
                        test: #test_token,
                        body: #body_token,
                        otherwise: #otherwise_token,
                    },
                    span: None,
                }
            }
        }
        Stmt::Scoped(block) => {
            let stmts = stmts_to_token(block.stmts);
            quote! {
                ::virtual_exec_type::ast::core::Node {
                    kind: ::virtual_exec_type::ast::core::Stmt::Scoped(#stmts),
                    span: None,
                }
            }
        }
        Stmt::Loop { test, body } => {
            let test_token = expr_to_token(test);
            let body_token = stmts_to_token(body.stmts);
            quote! {
                ::virtual_exec_type::ast::core::Node {
                    kind: ::virtual_exec_type::ast::core::Stmt::Loop {
                        test: #test_token,
                        body: #body_token,
                    },
                    span: None,
                }
            }
        }
        Stmt::Fn { name, args, body } => {
            let body_token = fn_stmts_to_token(body.stmts);

            quote! {
                ::virtual_exec_type::ast::core::Node {
                    kind: ::virtual_exec_type::ast::core::Stmt::FunctionDef {
                        name: #name.to_string(),
                        args: vec![ #(#args.to_string()),* ],
                        body: #body_token,
                    },
                    span: None,
                }
            }
        }
    }
}

fn block_to_token(v: TopLevelBlock) -> impl ToTokens {
    let body = stmts_to_token(v.stmts);
    quote! {
        ::virtual_exec_type::ast::core::Module {
            body: #body,
            span: None,
        }
    }
}

#[proc_macro]
pub fn parse(input: TokenStream) -> TokenStream {
    let output = parse_macro_input!(input as TopLevelBlock);
    let token_content = block_to_token(output);
    quote! { #token_content }.into()
}

fn subscript_to_token(sub: &SubscriptLoad) -> impl ToTokens {
    match sub {
        SubscriptLoad::Idx(idx) => {
            quote! { ::virtual_exec_core::sequential::instruction::SubscriptLoad::Idx(#idx) }
        }
        SubscriptLoad::String(s) => {
            quote! { ::virtual_exec_core::sequential::instruction::SubscriptLoad::String(::std::boxed::Box::from(#s))}
        }
    }
}

fn inst_to_token(inst: Instruction) -> impl ToTokens {
    match inst {
        Instruction::Add => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::Add }
        }
        Instruction::Sub => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::Sub }
        }
        Instruction::Mul => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::Mul }
        }
        Instruction::Div => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::Div }
        }
        Instruction::Mod => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::Mod }
        }
        Instruction::BitwiseAnd => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::BitwiseAnd }
        }
        Instruction::BitwiseOr => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::BitwiseOr }
        }
        Instruction::BitwiseXor => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::BitwiseXor }
        }
        Instruction::Shl => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::Shl }
        }
        Instruction::Shr => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::Shr }
        }
        Instruction::UnaryPlus => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::UnaryPlus }
        }
        Instruction::UnaryMinus => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::UnaryMinus }
        }
        Instruction::Not => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::Not }
        }
        Instruction::BitwiseNot => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::BitwiseNot }
        }
        Instruction::Eq => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::Eq }
        }
        Instruction::NotEq => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::NotEq }
        }
        Instruction::Lt => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::Lt }
        }
        Instruction::Lte => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::Lte }
        }
        Instruction::Gt => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::Gt }
        }
        Instruction::Gte => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::Gte }
        }
        Instruction::Assign => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::Assign }
        }
        Instruction::JmpNz(loc) => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::JmpNz(#loc) }
        }
        Instruction::JmpZ(loc) => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::JmpZ(#loc) }
        }
        Instruction::Jmp(loc) => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::Jmp(#loc) }
        }
        Instruction::Call => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::Call }
        }
        Instruction::Ret => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::Ret }
        }
        Instruction::LoadNone => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::LoadNone }
        }
        Instruction::LoadLitFloat(val) => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::LoadLitFloat(#val) }
        }
        Instruction::LoadLitInt(val) => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::LoadLitInt(#val) }
        }
        Instruction::LoadLitString(val) => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::LoadLitString(::std::boxed::Box::from(#val)) }
        }
        Instruction::LoadLitBool(val) => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::LoadLitBool(#val) }
        }
        Instruction::ConstructArr(len) => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::ConstructArr(#len) }
        }
        Instruction::ConstructObj(len2) => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::ConstructObj(#len2) }
        }
        Instruction::LoadName(name) => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::LoadName(::std::boxed::Box::from(#name)) }
        }
        Instruction::LoadObjectAttr(attr) => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::LoadObjectAttr(::std::boxed::Box::from(#attr)) }
        }
        Instruction::ResolveObject => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::ResolveObject }
        }
        Instruction::Terminate => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::Terminate }
        }
        Instruction::Interrupt => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::Interrupt }
        }
        Instruction::Pop => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::Pop }
        }
        Instruction::LoadDPtr(ptr, size) => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::LoadDPtr(#ptr, #size) }
        }
        Instruction::Swap => {
            quote! { ::virtual_exec_core::sequential::instructions::Instruction::Swap }
        }
    }
}

fn insts_to_token(stmts: Vec<Instruction>) -> impl ToTokens {
    let mut e = Vec::new();
    for inst in stmts {
        let tokens = inst_to_token(inst);
        e.push(tokens);
    }
    quote! {
        vec![
            #(#e),*
        ]
    }
}

#[proc_macro]
pub fn compile(input: TokenStream) -> TokenStream {
    let output = parse_macro_input!(input as TopLevelBlock);
    let body = output
        .stmts
        .into_iter()
        .map(convert_stmt)
        .collect::<Result<_, _>>()
        .unwrap();
    let module = Module { body, span: None };
    let compiled = virtual_exec_core::compile(&module);
    let token_content = insts_to_token(compiled);
    quote! { #token_content }.into()
}

macro_rules! gen_op {
    ($name:ident, $path:ident) => {
        ::paste::paste! {
            quote! {
                {
                    #[allow(path_statements)]
                    if false {
                        #$path;
                    };
                    mapping.get(::virtual_exec_core::fn_extern::fn_args::FnExternArgType::$name).[<unwrap_ $name>]()
                }
            }
        }
    };
}

macro_rules! gen_branch {
    ($opt:ident,$path:ident, $(($name:ident, $t:path)),*) => {
        match $opt {
            $(
                ::virtual_exec_core::fn_extern::fn_args::FnExternArgType::$name => (gen_op!($name, $path), Some(parse_quote!( $t ))),
            )*
        }
    };
}

fn arg_to_token(arg: FnArg, idx: usize) -> (impl ToTokens, Option<syn::Type>) {
    let span = arg.span();
    if let FnArg::Typed(pat_type) = arg {
        if let Pat::TupleStruct(pat_tuple) = &*pat_type.pat {
            if let Some(opt) = pat_tuple
                .path
                .get_ident()
                .map(virtual_exec_core::fn_extern::fn_args::FnExternArgType::from_ident)
                .flatten()
            {
                let path = pat_tuple.path.to_token_stream();
                // match opt {
                //     virtual_exec_core::fn_extern::fn_args::FnExternArgType::Alloc => (gen_op!(Alloc, path), Some(parse_quote!( ::virtual_exec_type::mem::MemoryAllocator ))),
                //     virtual_exec_core::fn_extern::fn_args::FnExternArgType::Machine => (gen_op!(Machine, path), Some(parse_quote!( ::virtual_exec_type::__private::Arc<::async_lock::Mutex<&mut ::virtual_exec_core::machine::Machine>>) )),
                // }
                gen_branch!(
                    opt,
                    path,
                    (Alloc, ::virtual_exec_type::mem::MemoryAllocator<'a>),
                    (
                        Machine,
                        ::virtual_exec_type::__private::Arc<
                            ::async_lock::Mutex<
                                &'__wrap_internal2 mut ::virtual_exec_core::machine::Machine<'a>,
                            >,
                        >
                    ),
                    (
                        Recurse,
                        ::virtual_exec_type::config::recurse::RecurseRestricter<'a>
                    )
                )
            } else {
                (
                    syn::Error::new(
                        span,
                        virtual_exec_core::fn_extern::fn_args::FnExternArgType::err_string(),
                    )
                    .to_compile_error(),
                    None,
                )
            }
        } else {
            (
                quote! {
                    ::virtual_exec_type::base::Downcast::from_value(values[#idx].clone()).ok_or(::virtual_exec_type::error::ExecutionError::NonRecoverable(::virtual_exec_type::error::NonRecoverableError::InvalidTypeError))?
                },
                None,
            )
        }
    } else {
        (
            syn::Error::new(span, "Methods taking 'self' are not supported here.")
                .to_compile_error(),
            None,
        )
    }
}

#[proc_macro_attribute]
pub fn fn_extern_wrap(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as ItemFn);
    let ident = input.sig.ident;
    let tokens = input
        .sig
        .inputs
        .clone()
        .into_iter()
        .enumerate()
        .map(|(idx, arg)| arg_to_token(arg, idx))
        .collect::<Vec<_>>();
    input.sig.ident = Ident::new("__fn_wrap", ident.span());
    let expected_length = tokens.iter().filter(|x| x.1.is_none()).count();
    for (arg_token, replacement) in input.sig.inputs.iter_mut().zip(tokens.iter().as_ref()) {
        match (arg_token, &replacement.1) {
            (FnArg::Typed(t), Some(ty)) => {
                if let Pat::TupleStruct(pat_tuple) = t.pat.deref()
                    && pat_tuple.elems.len() == 1
                {
                    if let Type::Infer(_) = t.ty.deref() {
                        t.ty = Box::new(ty.clone());
                    }
                    t.pat = Box::new(pat_tuple.elems[0].clone());
                }
            }
            _ => {}
        }
    }
    let reduced: Vec<_> = tokens.into_iter().map(|x| x.0).collect();
    quote! {
        fn #ident<'__wrap_internal, '__wrap_internal2>(
            machine: &'__wrap_internal2 mut ::virtual_exec_core::Machine<'__wrap_internal>,
            values: ::std::vec::Vec<::virtual_exec_type::mem::ValuePtr<'__wrap_internal>>
        ) -> ::core::result::Result<::virtual_exec_type::mem::ValuePtr<'__wrap_internal>, ::virtual_exec_type::error::ExecutionError> {
            use virtual_exec_type::mem::Allocator;
            if values.len() != #expected_length {
                return Err(::virtual_exec_type::error::ExecutionError::NonRecoverable(::virtual_exec_type::error::NonRecoverableError::IncorrectArgumentCountError))
            }
            let alloc = machine.alloc.clone();
            let mut mapping = ::virtual_exec_core::fn_extern::fn_args::LazyMapping::new(machine);
            #input
            let result = __fn_wrap(#(#reduced),*).map(|x| ::virtual_exec_type::base::Upcast::from_value(&x, &alloc))??;
            for mut item in values {
                alloc.change_alloc(&mut item)?;
            }
            mapping.settle_recurse_cost();
            Ok(result)
        }
    }.into()
}

#[proc_macro_attribute]
pub fn fn_extern_wrap_async(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as ItemFn);
    let ident = input.sig.ident;
    let tokens = input
        .sig
        .inputs
        .clone()
        .into_iter()
        .enumerate()
        .map(|(idx, arg)| arg_to_token(arg, idx))
        .collect::<Vec<_>>();
    input.sig.ident = Ident::new("__fn_wrap", ident.span());
    let expected_length = tokens.iter().filter(|x| x.1.is_none()).count();
    for (arg_token, replacement) in input.sig.inputs.iter_mut().zip(tokens.iter().as_ref()) {
        match (arg_token, &replacement.1) {
            (FnArg::Typed(t), Some(ty)) => {
                if let Pat::TupleStruct(pat_tuple) = t.pat.deref()
                    && pat_tuple.elems.len() == 1
                {
                    if let Type::Infer(_) = t.ty.deref() {
                        t.ty = Box::new(ty.clone());
                    }
                    t.pat = Box::new(pat_tuple.elems[0].clone());
                }
            }
            _ => {}
        }
    }
    let reduced: Vec<_> = tokens.into_iter().map(|x| x.0).collect();
    quote! {
        async fn #ident<'__wrap_internal, '__wrap_internal2>(
            machine: &'__wrap_internal2 mut ::virtual_exec_core::Machine<'__wrap_internal>,
            values: ::std::vec::Vec<::virtual_exec_type::mem::ValuePtr<'__wrap_internal>>
        ) -> ::core::result::Result<::virtual_exec_type::mem::ValuePtr<'__wrap_internal>, ::virtual_exec_type::error::ExecutionError> {
            use virtual_exec_type::mem::Allocator;
            if values.len() != #expected_length {
                return Err(::virtual_exec_type::error::ExecutionError::NonRecoverable(::virtual_exec_type::error::NonRecoverableError::IncorrectArgumentCountError))
            }
            let alloc = machine.alloc.clone();
            let mut mapping = ::virtual_exec_core::fn_extern::fn_args::LazyMapping::new(machine);
            #input
            let result = __fn_wrap(#(#reduced),*).await.map(|x| ::virtual_exec_type::base::Upcast::from_value(&x, &alloc))??;
            for mut item in values {
                alloc.change_alloc(&mut item)?;
            }
            mapping.settle_recurse_cost();
            Ok(result)
        }
    }.into()
}
