#![cfg(feature = "parse")]

use virtual_exec_core::{Machine, parse, compile};
use virtual_exec_core::sequential::exec::State;
use virtual_exec_type::error::ExecutionError;
use virtual_exec_type::error::NonRecoverableError;
use virtual_exec_type::mem::OwnedValue;

#[test]
fn test_simple_assignment() {
    let code = "a = 1; b = 2; c = 3; if a != b {d = 2;} d += d; d;";
    let compiled = compile(&parse(code).unwrap());
    println!("{:?}", compiled);
    let mut machine = Machine::new(compiled, 600, 100, vec![]).unwrap();
    match machine.sync_run_all() {
        Ok(State::Ok) => {},
        Ok(reason) => {
            println!("Machine: {:?}, state: {:?}", machine, reason);
        },
        Err(e) => {
            println!("Machine: {:?}, err: {:?}", machine, e);
        }
    }
    assert_eq!(machine.get("a"), Some(OwnedValue::Int(1)));
    assert_eq!(machine.get("d"), Some(OwnedValue::Int(4)));
}


#[test]
fn test_fn() {
    let code = "a = 10;
        fn add(a, b) {
            return a + b;
        }
        while a > 0 {
            a = add(a, -1);
        }";
    let compiled = compile(&parse(code).unwrap());
    println!("{:?}", compiled);
    let mut machine = Machine::new(compiled, 600, 1000, vec![]).unwrap();
    match machine.sync_run_all() {
        Ok(State::Ok) => {},
        Ok(reason) => {
            println!("Machine: {:?}, state: {:?}", machine, reason);
        },
        Err(e) => {
            println!("Machine: {:?}, err: {:?}", machine, e);
        }
    }
    assert_eq!(machine.get("a"), Some(OwnedValue::Int(0)));
}


#[test]
fn test_incorrect_argument_count() {
    let code = "a = 10;
        fn add(a, b) {
            return a + b;
        }
        while a > 0 {
            a = add(a, -1, -2);
        }";
    let compiled = compile(&parse(code).unwrap());
    println!("{:?}", compiled);
    let mut machine = Machine::new(compiled, 1000, 1000, vec![]).unwrap();
    match machine.sync_run_all() {
        Ok(State::Ok) => {},
        Ok(reason) => {
            println!("Machine: {:?}, state: {:?}", machine, reason);
        },
        Err(e) => {
            println!("Machine: {:?}, err: {:?}", machine, e);
        }
    }
    assert_eq!(machine.machine.state.is_err(), true, "Should be error with incorrect argument count");
    assert_eq!(machine.machine.state.err(), Some(ExecutionError::NonRecoverable(NonRecoverableError::IncorrectArgumentCountError)), "Should be incorrect amount of argument");
}


#[test]
fn test_extern_timeout_replay() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};
    use virtual_exec_core::fn_extern::{FnExtern, FnExternConstruct, MethodResolver};
    use virtual_exec_type::HashMap;
    use virtual_exec_type::error::RecoverableError;
    use virtual_exec_type::mem::{Allocator, Value, ValuePtr};

    struct CostlyFn(Arc<AtomicU64>);

    impl FnExternConstruct for CostlyFn {
        fn new() -> Self {
            CostlyFn(Arc::new(AtomicU64::new(0)))
        }
    }

    impl FnExtern for CostlyFn {
        fn fn_extern_sync<'a, 'b>(&self, machine: &'b mut virtual_exec_core::Machine<'a>, _values: Vec<ValuePtr<'a>>) -> Result<ValuePtr<'a>, ExecutionError> {
            self.0.fetch_add(1, Ordering::SeqCst);
            machine.machine.reduce_lim(50).map_err(ExecutionError::Recoverable)?;
            machine.alloc.alloc(Value::Int(42)).map_err(ExecutionError::from)
        }

        fn get_size(&self) -> usize {
            2
        }
    }

    let calls = Arc::new(AtomicU64::new(0));
    let mut funcs: HashMap<String, Arc<dyn FnExtern + Send + Sync>> = HashMap::new();
    funcs.insert("costly".to_string(), Arc::new(CostlyFn(Arc::clone(&calls))));
    let resolver = MethodResolver::new(funcs);

    let code = "a = costly(1, 2);";
    let compiled = compile(&parse(code).unwrap());
    let mut machine = Machine::new(compiled, 1000, 10, vec![resolver]).unwrap();

    let result = machine.sync_run_all();
    assert!(
        matches!(result, Err(ExecutionError::Recoverable(RecoverableError::TimeoutError(50)))),
        "Extern function requiring more budget than remaining should report a recoverable timeout: {:?}",
        result
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    machine.machine.grant_lim(100);
    let result = machine.sync_run_all();
    assert_eq!(result.is_ok(), true, "Execution should complete after granting more budget: {:?}", result);
    assert_eq!(calls.load(Ordering::SeqCst), 2, "Extern function should be replayed after resume");
    assert_eq!(machine.get("a"), Some(OwnedValue::Int(42)));
}
