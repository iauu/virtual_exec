use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use bumpalo::Bump;
use virtual_exec_macro::compile;
use virtual_exec_parser::sequential::exec::{FnStackFrame, InstStateMachine, State};
use virtual_exec_parser::sequential::instructions::Instruction;
use virtual_exec_type::alloc::{create_arena, Allocator};
use virtual_exec_type::ast::core::ASTNode;
use virtual_exec_type::base::ValueKind;
use virtual_exec_type::builtin::Mapping;
use virtual_exec_parser::sequential::exec::SandboxExecutionError;
use virtual_exec_type::exec_ctx::ExecutionContext;

#[test]
fn test_simple_assignment_and_expr() {
    let insts = compile!(
        a = 10;
        a = a + 5;
        a;
    );
    let arena = create_arena(None);
    let alloc = Allocator::new(&arena);

    let global_mapping = Rc::new(RefCell::new(Mapping { mapping: HashMap::new() }));

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


    let value = (&global_mapping).borrow().mapping.get("a").unwrap().borrow().kind.clone();

    match value {
        ValueKind::Int(i) => assert_eq!(i.value, 15),
        _ => panic!("Expected an integer result, but got {:?}", value),
    }
}

#[test]
fn test_more() {
    let insts = compile!(
        a = 10;
        a = a + 5;
        if a == 15 {
            a = 2;
        }
        a;
    );
    let arena = create_arena(None);
    let alloc = Allocator::new(&arena);

    let global_mapping = Rc::new(RefCell::new(Mapping { mapping: HashMap::new() }));

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

    let value = (&global_mapping).borrow().mapping.get("a").unwrap().borrow().kind.clone();

    match value {
        ValueKind::Int(i) => assert_eq!(i.value, 2),
        _ => panic!("Expected an integer result, but got {:?}", value),
    }
}

#[test]
fn test_timeout() {
    let insts = compile!(
        a = 10;
        a = a + 5;
        if a == 15 {
            a = 2;
        }
        a;
    );
    let arena = create_arena(None);
    let alloc = Allocator::new(&arena);

    let global_mapping = Rc::new(RefCell::new(Mapping { mapping: HashMap::new() }));

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

    assert!(state_machine.state.is_err(), "Evaluation successful when TimeoutError is expected: {:?}", state_machine.state);
    assert!(match &state_machine.state {
        Ok(State::Timeout) => true,
        _ => false
    }, "Expected TimeoutError, but got {:?}", state_machine.state);
}

#[test]
fn test_if_fail_path() {
    let insts = compile!(
        a = 10;
        a = a + 5;
        if a == 14 {
            a = 2;
        }
        a;
    );
    let arena = create_arena(None);
    let alloc = Allocator::new(&arena);

    let global_mapping = Rc::new(RefCell::new(Mapping { mapping: HashMap::new() }));

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

    let value = (&global_mapping).borrow().mapping.get("a").unwrap().borrow().kind.clone();

    match value {
        ValueKind::Int(i) => assert_eq!(i.value, 15),
        _ => panic!("Expected an integer result, but got {:?}", value),
    }
}
