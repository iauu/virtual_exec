# VirtualExec

A rust library to perform sandboxed safe expression evaluation, in a similar syntax to rust

```rust
use virtual_exec::{Machine, parse, compile};
use virtual_exec_parser::sequential::exec::State;
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

```
An example if the execution. In particular, the `100` and `1000` in the `test_fn` defines the memory and lifetime of the machine respectively, 
which this allowed up to 1000 operation and 100 virtual byte allocation, and would raise stop with Ok(State::Timeout) if it take longer than that.

Compile have also been added to convert code into a linear instruction

```rust
use virtual_exec_parser::parser::parse;
use virtual_exec_parser::sequential::compile::compile;
use virtual_exec_parser::sequential::instructions::Instruction;

#[test]
fn test_value_creation_and_downcast() {
    let code = "a = 1; b = 2; c = 3; if a != b {d = 2;} d += d; d;";
    let parsed = parse(code).unwrap();
    let compiled = compile(&parsed);
    assert_eq!(compiled, vec![
        Instruction::LoadName(Box::from("a")),
        Instruction::LoadLitInt(1),
        Instruction::Assign,
        Instruction::LoadName(Box::from("a")),
        Instruction::LoadLitInt(2),
        Instruction::Assign,
        Instruction::LoadName(Box::from("c")),
        Instruction::LoadLitInt(3),
        Instruction::Assign,
        Instruction::LoadName(Box::from("a")),
        Instruction::LoadName(Box::from("b")),
        Instruction::NotEq,
        Instruction::JmpZ(16),
        Instruction::LoadName(Box::from("d")),
        Instruction::LoadLitInt(2),
        Instruction::Assign,
        Instruction::LoadName(Box::from("d")),
        Instruction::LoadName(Box::from("d")),
        Instruction::LoadName(Box::from("d")),
        Instruction::Add,
        Instruction::Assign,
        Instruction::LoadName(Box::from("d")),
        Instruction::Pop
    ]);
}

```

WIP Feature list:
- [x] Variable assignment
- [ ] Attribute assignment
- [ ] Subscript assignment (i.e. `x[a]`)
- [x] Expression evaluation
- [x] A parser and type system
- [ ] Attribute system
- [x] Function call
- [x] `while` loop
- [ ] `for` loop
- [x] FFI function (Calling rust function from sandbox code with custom lifetime consumption)
- [x] Function definition
- [x] `if` statement
- [ ] Custom object definition
- [x] Use `await` in rust to allow context switching to other part of program to make it not blocking
  - [x] Switch to async-agnostic `Arc`, `Mutex`, `RwLock`
- [x] Linear instruction system (this allows `await` system later)
- [ ] `try` `catch` with stack unwinding and memory allocation recalculation

### Sub-crate List:
- [virtual_exec_type](https://crates.io/crates/virtual_exec_type)
- [virtual_exec_parser](https://crates.io/crates/virtual_exec_parser)
- [virtual_exec_macro](https://crates.io/crates/virtual_exec_macro)
- [virtual_exec_core](https://crates.io/crates/virtual_exec_core)
- [virtual_exec_extern](https://crates.io/crates/virtual_exec_extern)
- [virtual_exec_std](https://crates.io/crates/virtual_exec_std)

### Video Demo

https://github.com/user-attachments/assets/9a15c9ba-6932-466f-8d96-412dca2aa888
