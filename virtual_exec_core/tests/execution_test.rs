#![cfg(feature = "parse")]

use async_lock::RwLock;
use std::sync::Arc;
use virtual_exec_core::sequential::compile::compile;
use virtual_exec_core::sequential::exec::{FnStackFrame, InstStateMachine, State};
use virtual_exec_parser::parser::parse;
use virtual_exec_type::HashMap;
use virtual_exec_type::mem::{MemoryAllocator, MemoryAllocatorConstructor, Value, ValuePtr};

#[test]
fn test_execution_compiled_code() {
    let code = "a = 1; b = 2; c = 3; if a != b {d = 2;} d = d + d; d;";
    let parsed = parse(code).unwrap();
    let compiled = compile(&parsed);

    let alloc = MemoryAllocator::construct(600);

    let global_mapping: Arc<RwLock<HashMap<String, ValuePtr<'_>>>> =
        Arc::new(RwLock::new(HashMap::new()));

    let mut state_machine = InstStateMachine {
        lim: 1000,
        fn_stack_frame: vec![FnStackFrame {
            ptr: 0,
            mapping: global_mapping.clone(),
            _acct: None,
        }],
        alloc,
        instructions: compiled,
        state: Ok(State::Ok),
        stack: vec![],
    };

    while let Ok(State::Ok) = state_machine.state {
        if let Some(frame) = state_machine.fn_stack_frame.last() {
            if (frame.ptr as usize) >= state_machine.instructions.len() {
                break;
            }
        } else {
            break;
        }
        let _ = state_machine.run_once();
    }

    let d_value = global_mapping
        .read_arc_blocking()
        .get("d")
        .unwrap()
        .read_arc_blocking()
        .inner
        .clone();

    match d_value {
        Value::Int(i) => assert_eq!(i, 4),
        _ => panic!("Expected d to be Int(4) but got {:?}", d_value),
    }
}

#[test]
fn test_execution_compiled_code_if_false() {
    let code = "a = 1; b = 1; c = 3; if a != b {d = 2;} else {d = 5;} d = d + d; d;";
    let parsed = parse(code).unwrap();
    let compiled = compile(&parsed);

    let alloc = MemoryAllocator::construct(600);

    let global_mapping: Arc<RwLock<HashMap<String, ValuePtr<'_>>>> =
        Arc::new(RwLock::new(HashMap::new()));

    let mut state_machine = InstStateMachine {
        lim: 1000,
        fn_stack_frame: vec![FnStackFrame {
            ptr: 0,
            mapping: global_mapping.clone(),
            _acct: None,
        }],
        alloc,
        instructions: compiled,
        state: Ok(State::Ok),
        stack: vec![],
    };

    while let Ok(State::Ok) = state_machine.state {
        if let Some(frame) = state_machine.fn_stack_frame.last() {
            if (frame.ptr as usize) >= state_machine.instructions.len() {
                break;
            }
        } else {
            break;
        }
        let _ = state_machine.run_once();
    }

    let d_value = global_mapping
        .read_arc_blocking()
        .get("d")
        .unwrap()
        .read_arc_blocking()
        .inner
        .clone();

    match d_value {
        Value::Int(i) => assert_eq!(i, 10),
        _ => panic!("Expected d to be Int(10) but got {:?}", d_value),
    }
}

#[test]
fn test_execution_compiled_code_math_operations() {
    let code = "a = 10; b = 3; c = a - b; d = a * b; e = a / b; f = a % b;";
    let parsed = parse(code).unwrap();
    let compiled = compile(&parsed);

    let alloc = MemoryAllocator::construct(600);

    let global_mapping: Arc<RwLock<HashMap<String, ValuePtr<'_>>>> =
        Arc::new(RwLock::new(HashMap::new()));

    let mut state_machine = InstStateMachine {
        lim: 1000,
        fn_stack_frame: vec![FnStackFrame {
            ptr: 0,
            mapping: global_mapping.clone(),
            _acct: None,
        }],
        alloc,
        instructions: compiled,
        state: Ok(State::Ok),
        stack: vec![],
    };

    while let Ok(State::Ok) = state_machine.state {
        if let Some(frame) = state_machine.fn_stack_frame.last() {
            if (frame.ptr as usize) >= state_machine.instructions.len() {
                break;
            }
        } else {
            break;
        }
        let _ = state_machine.run_once();
    }

    let c_value = global_mapping
        .read_arc_blocking()
        .get("c")
        .unwrap()
        .read_arc_blocking()
        .inner
        .clone();
    match c_value {
        Value::Int(i) => assert_eq!(i, 7),
        _ => panic!("Expected c to be Int(7) but got {:?}", c_value),
    }

    let d_value = global_mapping
        .read_arc_blocking()
        .get("d")
        .unwrap()
        .read_arc_blocking()
        .inner
        .clone();
    match d_value {
        Value::Int(i) => assert_eq!(i, 30),
        _ => panic!("Expected d to be Int(30) but got {:?}", d_value),
    }

    let e_value = global_mapping
        .read_arc_blocking()
        .get("e")
        .unwrap()
        .read_arc_blocking()
        .inner
        .clone();
    match e_value {
        Value::Float(f) => assert_eq!(f, 10.0 / 3.0),
        _ => panic!("Expected e to be Float but got {:?}", e_value),
    }

    let f_value = global_mapping
        .read_arc_blocking()
        .get("f")
        .unwrap()
        .read_arc_blocking()
        .inner
        .clone();
    match f_value {
        Value::Int(i) => assert_eq!(i, 1),
        _ => panic!("Expected f to be Int(1) but got {:?}", f_value),
    }
}

#[test]
fn test_execution_compiled_code_bitwise_operations() {
    let code = "a = 5; b = 3; c = a & b; d = a | b; f = a << 1; g = a >> 1;";
    let parsed = parse(code).unwrap();
    let compiled = compile(&parsed);

    let alloc = MemoryAllocator::construct(600);

    let global_mapping: Arc<RwLock<HashMap<String, ValuePtr<'_>>>> =
        Arc::new(RwLock::new(HashMap::new()));

    let mut state_machine = InstStateMachine {
        lim: 1000,
        fn_stack_frame: vec![FnStackFrame {
            ptr: 0,
            mapping: global_mapping.clone(),
            _acct: None,
        }],
        alloc,
        instructions: compiled,
        state: Ok(State::Ok),
        stack: vec![],
    };

    while let Ok(State::Ok) = state_machine.state {
        if let Some(frame) = state_machine.fn_stack_frame.last() {
            if (frame.ptr as usize) >= state_machine.instructions.len() {
                break;
            }
        } else {
            break;
        }
        let _ = state_machine.run_once();
    }

    let c_value = global_mapping
        .read_arc_blocking()
        .get("c")
        .unwrap()
        .read_arc_blocking()
        .inner
        .clone();
    match c_value {
        Value::Int(i) => assert_eq!(i, 5 & 3),
        _ => panic!("Expected c to be Int(1) but got {:?}", c_value),
    }

    let d_value = global_mapping
        .read_arc_blocking()
        .get("d")
        .unwrap()
        .read_arc_blocking()
        .inner
        .clone();
    match d_value {
        Value::Int(i) => assert_eq!(i, 5 | 3),
        _ => panic!("Expected d to be Int(7) but got {:?}", d_value),
    }
    let f_value = global_mapping
        .read_arc_blocking()
        .get("f")
        .unwrap()
        .read_arc_blocking()
        .inner
        .clone();
    match f_value {
        Value::Int(i) => assert_eq!(i, 5 << 1),
        _ => panic!("Expected f to be Int(10) but got {:?}", f_value),
    }

    let g_value = global_mapping
        .read_arc_blocking()
        .get("g")
        .unwrap()
        .read_arc_blocking()
        .inner
        .clone();
    match g_value {
        Value::Int(i) => assert_eq!(i, 5 >> 1),
        _ => panic!("Expected g to be Int(2) but got {:?}", g_value),
    }
}
