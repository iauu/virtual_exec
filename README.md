# VirtualExec

A rust library to perform sandboxed safe expression evaluation, in a similar syntax to rust

```rust
use virtual_exec::exec;
use virtual_exec_type::exec_ctx::RsValue;

#[test]
fn test_simple_assignment() {
    let code = "a = 1; b = 2; c = 3; if a != b {d = 2;} d;";
    let result = exec(code, 100).unwrap();
    assert_eq!(result.get("a"), Some(&RsValue::Int(1)));
}
```
An example if the execution. In particular, the `100` there defines the lifetime of the calculation, 
which this allowed up to 100 operation, and would raise `TimeoutError` if it take longer than that 
to execute.

The current supported operation is expression calculation, assignment and if-statement.

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
