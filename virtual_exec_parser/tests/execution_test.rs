use virtual_exec_parser::parser::parse;
use virtual_exec_parser::sequential::compile::compile;
use virtual_exec_type::base::ValueKind;
use virtual_exec_type::alloc::{create_arena, Allocator};
use virtual_exec_type::builtin::Mapping;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use virtual_exec_parser::sequential::exec::{InstStateMachine, FnStackFrame, State};

#[test]
fn test_execution_compiled_code() {


    let code = "a = 1; b = 2; c = 3; if a != b {d = 2;} d = d + d; d;";
    let parsed = parse(code).unwrap();
    let compiled = compile(&parsed);

    let arena = create_arena(None);
    let alloc = Allocator::new(&arena);

    let global_mapping = Rc::new(RefCell::new(Mapping { mapping: HashMap::new() }));

    let mut state_machine = InstStateMachine {
        lim: 1000,
        fn_stack_frame: vec![FnStackFrame {
            ptr: 0,
            stack: vec![],
            mapping: global_mapping.clone()
        }],
        alloc,
        instructions: compiled,
        state: Ok(State::Ok)
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

    let d_value = global_mapping.borrow().mapping.get("d").unwrap().borrow().kind.clone();

    match d_value {
        ValueKind::Int(i) => assert_eq!(i.value, 4),
        _ => panic!("Expected d to be Int(4) but got {:?}", d_value),
    }
}

#[test]
fn test_execution_compiled_code_if_false() {
    let code = "a = 1; b = 1; c = 3; if a != b {d = 2;} else {d = 5;} d = d + d; d;";
    let parsed = parse(code).unwrap();
    let compiled = compile(&parsed);

    let arena = create_arena(None);
    let alloc = Allocator::new(&arena);

    let global_mapping = Rc::new(RefCell::new(Mapping { mapping: HashMap::new() }));

    let mut state_machine = InstStateMachine {
        lim: 1000,
        fn_stack_frame: vec![FnStackFrame {
            ptr: 0,
            stack: vec![],
            mapping: global_mapping.clone()
        }],
        alloc,
        instructions: compiled,
        state: Ok(State::Ok)
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

    let d_value = global_mapping.borrow().mapping.get("d").unwrap().borrow().kind.clone();

    match d_value {
        ValueKind::Int(i) => assert_eq!(i.value, 10),
        _ => panic!("Expected d to be Int(10) but got {:?}", d_value),
    }
}

#[test]
fn test_execution_compiled_code_math_operations() {
    let code = "a = 10; b = 3; c = a - b; d = a * b; e = a / b; f = a % b;";
    let parsed = parse(code).unwrap();
    let compiled = compile(&parsed);

    let arena = create_arena(None);
    let alloc = Allocator::new(&arena);

    let global_mapping = Rc::new(RefCell::new(Mapping { mapping: HashMap::new() }));

    let mut state_machine = InstStateMachine {
        lim: 1000,
        fn_stack_frame: vec![FnStackFrame {
            ptr: 0,
            stack: vec![],
            mapping: global_mapping.clone()
        }],
        alloc,
        instructions: compiled,
        state: Ok(State::Ok)
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

    let c_value = global_mapping.borrow().mapping.get("c").unwrap().borrow().kind.clone();
    match c_value {
        ValueKind::Int(i) => assert_eq!(i.value, 7),
        _ => panic!("Expected c to be Int(7) but got {:?}", c_value),
    }

    let d_value = global_mapping.borrow().mapping.get("d").unwrap().borrow().kind.clone();
    match d_value {
        ValueKind::Int(i) => assert_eq!(i.value, 30),
        _ => panic!("Expected d to be Int(30) but got {:?}", d_value),
    }

    let e_value = global_mapping.borrow().mapping.get("e").unwrap().borrow().kind.clone();
    match e_value {
        ValueKind::Float(f) => assert_eq!(f.value, 10.0 / 3.0),
        _ => panic!("Expected e to be Float but got {:?}", e_value),
    }

    let f_value = global_mapping.borrow().mapping.get("f").unwrap().borrow().kind.clone();
    match f_value {
        ValueKind::Int(i) => assert_eq!(i.value, 1),
        _ => panic!("Expected f to be Int(1) but got {:?}", f_value),
    }
}

#[test]
fn test_execution_compiled_code_bitwise_operations() {
    let code = "a = 5; b = 3; c = a & b; d = a | b; f = a << 1; g = a >> 1;";
    let parsed = parse(code).unwrap();
    let compiled = compile(&parsed);

    let arena = create_arena(None);
    let alloc = Allocator::new(&arena);

    let global_mapping = Rc::new(RefCell::new(Mapping { mapping: HashMap::new() }));

    let mut state_machine = InstStateMachine {
        lim: 1000,
        fn_stack_frame: vec![FnStackFrame {
            ptr: 0,
            stack: vec![],
            mapping: global_mapping.clone()
        }],
        alloc,
        instructions: compiled,
        state: Ok(State::Ok)
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

    let c_value = global_mapping.borrow().mapping.get("c").unwrap().borrow().kind.clone();
    match c_value {
        ValueKind::Int(i) => assert_eq!(i.value, 5 & 3),
        _ => panic!("Expected c to be Int(1) but got {:?}", c_value),
    }

    let d_value = global_mapping.borrow().mapping.get("d").unwrap().borrow().kind.clone();
    match d_value {
        ValueKind::Int(i) => assert_eq!(i.value, 5 | 3),
        _ => panic!("Expected d to be Int(7) but got {:?}", d_value),
    }
    let f_value = global_mapping.borrow().mapping.get("f").unwrap().borrow().kind.clone();
    match f_value {
        ValueKind::Int(i) => assert_eq!(i.value, 5 << 1),
        _ => panic!("Expected f to be Int(10) but got {:?}", f_value),
    }

    let g_value = global_mapping.borrow().mapping.get("g").unwrap().borrow().kind.clone();
    match g_value {
        ValueKind::Int(i) => assert_eq!(i.value, 5 >> 1),
        _ => panic!("Expected g to be Int(2) but got {:?}", g_value),
    }
}
