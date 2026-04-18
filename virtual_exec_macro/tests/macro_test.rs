use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use bumpalo::Bump;
use virtual_exec_macro::parse;
use virtual_exec_type::alloc::Allocator;
use virtual_exec_type::ast::core::ASTNode;
use virtual_exec_type::base::ValueKind;
use virtual_exec_type::builtin::Mapping;
use virtual_exec_type::exec_ctx::ExecutionContext;

#[test]
fn test_simple_assignment_and_expr() {
    let module = parse!(
        a = 10;
        a = a + 5;
        a;
    );
    let arena = Bump::new();
    let alloc = Allocator::new(&arena);
    let mut global_scope = Mapping { mapping: HashMap::new() };

    let initial_value = alloc.allocate(ValueKind::None);

    global_scope.mapping.insert("a".to_string(), Rc::new(RefCell::new(initial_value)));

    let mapping = vec![Rc::new(RefCell::new(global_scope))];
    let ctx = Rc::new(RefCell::new(ExecutionContext::new(alloc, 1000, mapping.clone())));

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
    let arena = Bump::new();
    let alloc = Allocator::new(&arena);
    let mut global_scope = Mapping { mapping: HashMap::new() };

    let initial_value = alloc.allocate(ValueKind::None);

    global_scope.mapping.insert("a".to_string(), Rc::new(RefCell::new(initial_value)));

    let mapping = vec![Rc::new(RefCell::new(global_scope))];
    let ctx = Rc::new(RefCell::new(ExecutionContext::new(alloc, 1000, mapping.clone())));

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
    let arena = Bump::new();
    let alloc = Allocator::new(&arena);
    let mut global_scope = Mapping { mapping: HashMap::new() };

    let initial_value = alloc.allocate(ValueKind::None);

    global_scope.mapping.insert("a".to_string(), Rc::new(RefCell::new(initial_value)));

    let mapping = vec![Rc::new(RefCell::new(global_scope))];
    let ctx = Rc::new(RefCell::new(ExecutionContext::new(alloc, 15, mapping.clone())));

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
    let arena = Bump::new();
    let alloc = Allocator::new(&arena);
    let mut global_scope = Mapping { mapping: HashMap::new() };

    let initial_value = alloc.allocate(ValueKind::None);

    global_scope.mapping.insert("a".to_string(), Rc::new(RefCell::new(initial_value)));

    let mapping = vec![Rc::new(RefCell::new(global_scope))];
    let ctx = Rc::new(RefCell::new(ExecutionContext::new(alloc, 1000, mapping.clone())));

    let result = module.eval(ctx);

    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());

    let value = (&mapping).get(0).unwrap().borrow().mapping.get("a").unwrap().borrow().kind.clone();

    match value {
        ValueKind::Int(i) => assert_eq!(i.value, 15),
        _ => panic!("Expected an integer result, but got {:?}", value),
    }
}