use std::cell::{RefCell, Ref};
use std::collections::HashMap;
use std::rc::Rc;
use bumpalo::Bump;
use virtual_exec_macro::parse;
use virtual_exec_type::ast::core::ASTNode;
use virtual_exec_type::base::{Value, ValueContainer, ValueKind};
use virtual_exec_type::builtin::Mapping;
use virtual_exec_type::exec_ctx::{CtxBump, ExecutionContext};

#[test]
fn test_simple_assignment_and_expr<'ctx>() {
    let module = parse!(
        a = 10;
        a = a + 5;
        a;
    );
    let arena = Bump::new();
    let ptr: &'ctx Bump = &arena;
    let arena_rc = Rc::new(RefCell::new(ptr));
    let mut global_scope = Mapping { mapping: HashMap::new() };

    let initial_value: Value<'static> = {
        let arena_borrow: Ref<CtxBump> = arena_rc.borrow();
        let arena_ref: &Bump = &arena_borrow;
        // let long_lived_arena: &'static Bump = unsafe { std::mem::transmute(arena_ref) };
        ValueContainer::new(ValueKind::None, arena_ref)
    };

    global_scope.mapping.insert("a".to_string(), Rc::new(RefCell::new(initial_value)));

    let mapping = vec![Rc::new(RefCell::new(global_scope))];
    let ctx = Rc::new(RefCell::new(ExecutionContext::new(arena_rc.clone(), 1000, mapping.clone())));

    let result = module.eval(ctx);

    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());

    let value = (&mapping).get(0).unwrap().borrow().mapping.get("a").unwrap().borrow().kind.clone();

    match value {
        ValueKind::Int(i) => assert_eq!(i.value, 15),
        _ => panic!("Expected an integer result, but got {:?}", value),
    }
}

#[test]
fn test_more() {
    let module = parse!(
        a = 10;
        a = a + 5;
        if a == 15 {
            a = 2;
        }
        a;
    );
    let arena_rc = Rc::new(RefCell::new(Bump::new()));
    let mut global_scope = Mapping { mapping: HashMap::new() };

    let initial_value: Value<'static> = {
        let arena_borrow: Ref<Bump> = arena_rc.borrow();
        let arena_ref: &Bump = &arena_borrow;
        let long_lived_arena: &'static Bump = unsafe { std::mem::transmute(arena_ref) };
        ValueContainer::new(ValueKind::None, long_lived_arena)
    };

    global_scope.mapping.insert("a".to_string(), Rc::new(RefCell::new(initial_value)));

    let mapping = vec![Rc::new(RefCell::new(global_scope))];
    let ctx = Rc::new(RefCell::new(ExecutionContext::new(arena_rc.clone(), 1000, mapping.clone())));

    let result = module.eval(ctx);

    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());

    let value = (&mapping).get(0).unwrap().borrow().mapping.get("a").unwrap().borrow().kind.clone();

    match value {
        ValueKind::Int(i) => assert_eq!(i.value, 2),
        _ => panic!("Expected an integer result, but got {:?}", value),
    }
}

#[test]
fn test_timeout() {
    let module = parse!(
        a = 10;
        a = a + 5;
        if a == 15 {
            a = 2;
        }
        a;
    );
    let arena_rc = Rc::new(RefCell::new(Bump::new()));
    let mut global_scope = Mapping { mapping: HashMap::new() };

    let initial_value: Value<'static> = {
        let arena_borrow: Ref<Bump> = arena_rc.borrow();
        let arena_ref: &Bump = &arena_borrow;
        let long_lived_arena: &'static Bump = unsafe { std::mem::transmute(arena_ref) };
        ValueContainer::new(ValueKind::None, long_lived_arena)
    };

    global_scope.mapping.insert("a".to_string(), Rc::new(RefCell::new(initial_value)));

    let mapping = vec![Rc::new(RefCell::new(global_scope))];
    let ctx = Rc::new(RefCell::new(ExecutionContext::new(arena_rc.clone(), 15, mapping.clone())));

    let result = module.eval(ctx);

    assert!((&result).is_err(), "Evaluation successful when TimeoutError is expected: {:?}", result.ok());
    assert!(match (result.clone().err()) {
        Some(virtual_exec_type::error::SandboxExecutionError::TimeoutError) => true,
        _ => false
    }, "Expected TimeoutError, but got {:?}", result.err());
}

#[test]
fn test_if_fail_path() {
    let module = parse!(
        a = 10;
        a = a + 5;
        if a == 14 {
            a = 2;
        }
        a;
    );
    let arena_rc = Rc::new(RefCell::new(Bump::new()));
    let mut global_scope = Mapping { mapping: HashMap::new() };

    let initial_value: Value<'static> = {
        let arena_borrow: Ref<Bump> = arena_rc.borrow();
        let arena_ref: &Bump = &arena_borrow;
        let long_lived_arena: &'static Bump = unsafe { std::mem::transmute(arena_ref) };
        ValueContainer::new(ValueKind::None, long_lived_arena)
    };

    global_scope.mapping.insert("a".to_string(), Rc::new(RefCell::new(initial_value)));

    let mapping = vec![Rc::new(RefCell::new(global_scope))];
    let ctx = Rc::new(RefCell::new(ExecutionContext::new(arena_rc.clone(), 1000, mapping.clone())));

    let result = module.eval(ctx);

    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());

    let value = (&mapping).get(0).unwrap().borrow().mapping.get("a").unwrap().borrow().kind.clone();

    match value {
        ValueKind::Int(i) => assert_eq!(i.value, 15),
        _ => panic!("Expected an integer result, but got {:?}", value),
    }
}