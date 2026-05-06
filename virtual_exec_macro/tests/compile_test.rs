use std::collections::HashMap;
use std::sync::{Arc};
use async_lock::RwLock;
use virtual_exec_macro::compile;
use virtual_exec_core::sequential::exec::{FnStackFrame, InstStateMachine, State};
use virtual_exec_type::mem::{MemoryAllocator, MemoryAllocatorConstructor, Value, ValuePtr};

#[test]
fn test_simple_assignment_and_expr() {
    let insts = compile! {
        a = 10;
        a = a + 5;
        a;
    };
    let alloc = MemoryAllocator::construct(100);

    let global_mapping: Arc<RwLock<HashMap<String, ValuePtr<'_>>>> = Arc::new(RwLock::new(HashMap::new()));

    let mut state_machine = InstStateMachine {
        lim: 1000,
        fn_stack_frame: vec![FnStackFrame {
            ptr: 0,
            mapping: global_mapping.clone()
        }],
        alloc,
        instructions: insts,
        state: Ok(State::Ok),
        stack: vec![]
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
    
    assert!(state_machine.state.is_ok(), "Evaluation failed: {:?}", state_machine.state.err());


    let value = global_mapping.read_arc_blocking().get("a").unwrap().lock_arc_blocking().inner.clone();

    match value {
        Value::Int(i) => assert_eq!(i, 15),
        _ => panic!("Expected an integer result, but got {:?}", value),
    }
}

#[test]
fn test_more() {
    let insts = compile! {
        a = 10;
        a = a + 5;
        if a == 15 {
            a = 2;
        }
        a;
    };
    let alloc = MemoryAllocator::construct(100);

    let global_mapping: Arc<RwLock<HashMap<String, ValuePtr<'_>>>> = Arc::new(RwLock::new(HashMap::new()));

    let mut state_machine = InstStateMachine {
        lim: 1000,
        fn_stack_frame: vec![FnStackFrame {
            ptr: 0,
            mapping: global_mapping.clone()
        }],
        alloc,
        instructions: insts,
        state: Ok(State::Ok),
        stack: vec![]
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

    assert!(state_machine.state.is_ok(), "Evaluation failed: {:?}", state_machine.state.err());

    let value = global_mapping.read_arc_blocking().get("a").unwrap().lock_arc_blocking().inner.clone();

    match value {
        Value::Int(i) => assert_eq!(i, 2),
        _ => panic!("Expected an integer result, but got {:?}", value),
    }
}

#[test]
fn test_timeout() {
    let insts = compile! {
        a = 10;
        a = a + 5;
        if a == 15 {
            a = 2;
        }
        a;
    };
    let alloc = MemoryAllocator::construct(100);

    let global_mapping: Arc<RwLock<HashMap<String, ValuePtr<'_>>>> = Arc::new(RwLock::new(HashMap::new()));

    let mut state_machine = InstStateMachine {
        lim: 15,
        fn_stack_frame: vec![FnStackFrame {
            ptr: 0,
            mapping: global_mapping.clone()
        }],
        alloc,
        instructions: insts,
        state: Ok(State::Ok),
        stack: vec![]
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

    assert!(match &state_machine.state {
        Ok(State::Timeout(_)) => true,
        _ => false
    }, "Expected TimeoutError, but got {:?}", state_machine.state);
}

#[test]
fn test_if_fail_path() {
    let insts = compile! {
        a = 10;
        a = a + 5;
        if a == 14 {
            a = 2;
        }
        a;
    };
    let alloc = MemoryAllocator::construct(100);

    let global_mapping: Arc<RwLock<HashMap<String, ValuePtr<'_>>>> = Arc::new(RwLock::new(HashMap::new()));


    let mut state_machine = InstStateMachine {
        lim: 1000,
        fn_stack_frame: vec![FnStackFrame {
            ptr: 0,
            mapping: global_mapping.clone()
        }],
        alloc,
        instructions: insts,
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

    assert!(state_machine.state.is_ok(), "Evaluation failed: {:?}", state_machine.state.err());

    let value = global_mapping.read_arc_blocking().get("a").unwrap().lock_arc_blocking().inner.clone();

    match value {
        Value::Int(i) => assert_eq!(i, 15),
        _ => panic!("Expected an integer result, but got {:?}", value),
    }
}


#[test]
fn test_while_loop() {
    let insts = compile! {
        a = 10;
        while a > 0 {
            a -= 1;
        }
    };
    let alloc = MemoryAllocator::construct(100);

    let global_mapping: Arc<RwLock<HashMap<String, ValuePtr<'_>>>> = Arc::new(RwLock::new(HashMap::new()));


    let mut state_machine = InstStateMachine {
        lim: 1000,
        fn_stack_frame: vec![FnStackFrame {
            ptr: 0,
            mapping: global_mapping.clone()
        }],
        alloc,
        instructions: insts,
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

    assert!(state_machine.state.is_ok(), "Evaluation failed: {:?}", state_machine.state.err());

    let value = global_mapping.read_arc_blocking().get("a").unwrap().lock_arc_blocking().inner.clone();

    match value {
        Value::Int(i) => assert_eq!(i, 0),
        _ => panic!("Expected an integer result, but got {:?}", value),
    }
}


#[test]
fn test_function() {
    let insts = compile! {
        a = 10;
        fn add(a, b) {
            return a + b;
        }
        while a > 0 {
            a = add(a, -1);
        }
    };
    let alloc = MemoryAllocator::construct(100);

    let global_mapping: Arc<RwLock<HashMap<String, ValuePtr<'_>>>> = Arc::new(RwLock::new(HashMap::new()));


    let mut state_machine = InstStateMachine {
        lim: 1000,
        fn_stack_frame: vec![FnStackFrame {
            ptr: 0,
            mapping: global_mapping.clone()
        }],
        alloc,
        instructions: insts,
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
        println!("{}", state_machine.instructions[state_machine.fn_stack_frame.last().unwrap().ptr as usize]);
        let _ = state_machine.run_once();
    }

    assert!(state_machine.state.is_ok(), "Evaluation failed: {:?}", state_machine.state.err());

    let value = global_mapping.read_arc_blocking().get("a").unwrap().lock_arc_blocking().inner.clone();

    println!("{:?}", state_machine);

    match value {
        Value::Int(i) => assert_eq!(i, 0),
        _ => panic!("Expected an integer result, but got {:?}", value),
    }
}
