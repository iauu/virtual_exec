use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use bumpalo::Bump;
use virtual_exec_parser::error::ParseError;
use virtual_exec_parser::parser;
use virtual_exec_type::alloc::Allocator;
use virtual_exec_type::ast::core::ASTNode;
use virtual_exec_type::builtin::Mapping;
use virtual_exec_type::exec_ctx::{ExecutionContext, RsValue};
use virtual_exec_type::error::SandboxExecutionError as InterpretedSandboxExecutionError;

/// The interpreted unified error type for the `virtual_exec` library.
#[derive(Debug)]
pub enum InterpretedExecError {
    /// An error that occurred during the parsing phase.
    Parse(ParseError),
    /// An error that occurred during the execution phase.
    Execution(InterpretedSandboxExecutionError),
}

impl From<ParseError> for InterpretedExecError {
    fn from(e: ParseError) -> Self {
        InterpretedExecError::Parse(e)
    }
}

impl From<InterpretedSandboxExecutionError> for InterpretedExecError {
    fn from(e: InterpretedSandboxExecutionError) -> Self {
        InterpretedExecError::Execution(e)
    }
}

/// Executes a string of code in a sandboxed environment.
///
/// # Arguments
///
/// * `code` - A string slice containing the code to execute.
/// * `ttl` - A time-to-live value representing the maximum number of operations allowed.
///
/// # Returns
///
/// A `Result` which is either:
/// * `Ok(HashMap<String, RsValue>)` - A dictionary of the final state of all variables.
/// * `Err(InterpretedExecError)` - An error that occurred during parsing or execution.
pub fn interpreted_exec(code: &str, ttl: i64) -> Result<HashMap<String, RsValue>, InterpretedExecError> {
    // 1. Parse the code into an AST.
    let module = parser::parse(code)?;

    let arena = Bump::new();
    let alloc = Allocator::new(&arena);
    let global_scope = Rc::new(RefCell::new(Mapping { mapping: HashMap::new() }));
    let mapping = vec![global_scope];
    let ctx = Rc::new(RefCell::new(ExecutionContext::new(alloc, ttl, mapping)));

    module.eval(ctx.clone())?;

    let final_state = ctx.borrow().to_hashmap();
    Ok(final_state)
}
