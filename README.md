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

```
An example if the execution. In particular, the `100` there defines the lifetime of the calculation, 
which this allowed up to 100 operation, and would raise `TimeoutError` if it take longer than that 
to execute.

The current supported operation is expression calculation, assignment and if-statement.

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
- [ ] Function call
- [ ] `while` loop
- [ ] `for` loop
- [ ] FFI function (Calling rust function from sandbox code with custom lifetime consumption) **The planned behaviour is it would terminate after the function call if it is dynamic lifetime, while terminate before the function call if it is static lifetime**
- [ ] Function definition
- [x] `if` statement
- [ ] Custom object definition
- [ ] Use `await` in rust to allow context switching to other part of program to make it not blocking
- [x] Linear instruction system (this allows `await` system later)

### Sub-crate List:
- [virtual_exec_type](https://crates.io/crates/virtual_exec_type)
- [virtual_exec_parser](https://crates.io/crates/virtual_exec_parser)
- [virtual_exec_macro](https://crates.io/crates/virtual_exec_macro)

### Video Demo

https://github.com/user-attachments/assets/9a15c9ba-6932-466f-8d96-412dca2aa888
