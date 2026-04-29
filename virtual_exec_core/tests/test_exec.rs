use virtual_exec_core::{Machine, parse, compile};
use virtual_exec_core::sequential::exec::State;
use virtual_exec_type::error::ExecutionError;
use virtual_exec_type::mem::OwnedValue;

#[test]
fn test_simple_assignment() {
    let code = "a = 1; b = 2; c = 3; if a != b {d = 2;} d += d; d;";
    let compiled = compile(&parse(code).unwrap());
    println!("{:?}", compiled);
    let mut machine = Machine::new(compiled, 100, 100);
    match machine.run_all() {
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
    let mut machine = Machine::new(compiled, 100, 1000);
    match machine.run_all() {
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
    let mut machine = Machine::new(compiled, 100, 1000);
    match machine.run_all() {
        Ok(State::Ok) => {},
        Ok(reason) => {
            println!("Machine: {:?}, state: {:?}", machine, reason);
        },
        Err(e) => {
            println!("Machine: {:?}, err: {:?}", machine, e);
        }
    }
    assert_eq!(machine.machine.state.is_err(), true, "Should be error with incorrect argument count");
    assert_eq!(machine.machine.state.err(), Some(ExecutionError::IncorrectArgumentCountError), "Should be incorrect amount of argument");
}
