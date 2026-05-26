# virtual_exec_macro

A macro crate to provide compile time sandbox building to increase runtime performance as
it doesn't have to re-evaluate the string expression, as replacement of 
`virtual_exec_parser::parse`.

**Note that this cannot be used as easily as `virtual_exec::exec` as it provide a higher-level API

Example:
```rust
use std::cell::{RefCell, Ref};
use std::collections::HashMap;
use std::rc::Rc;
use bumpalo::Bump;
use virtual_exec_macro::parse;
use virtual_exec_type::ast::core::ASTNode;
use virtual_exec_type::base::{Value, ValueContainer, ValueKind};
use virtual_exec_type::builtin::Mapping;
use virtual_exec_type::exec_ctx::ExecutionContext;

#[test]
fn test_simple_assignment_and_expr() {
    let module = parse!(
        a = 10;
        a = a + 5;
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
```

### Crate List:
- [virtual_exec](https://crates.io/crates/virtual_exec)
- [virtual_exec_type](https://crates.io/crates/virtual_exec_type)
- [virtual_exec_parser](https://crates.io/crates/virtual_exec_parser)
- [virtual_exec_core](https://crates.io/crates/virtual_exec_core)
- [virtual_exec_extern](https://crates.io/crates/virtual_exec_extern)
- [virtual_exec_std](https://crates.io/crates/virtual_exec_std)
- [virtual_exec_repl](https://crates.io/crates/virtual_exec_repl)