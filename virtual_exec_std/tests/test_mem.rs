use virtual_exec_core::sequential::exec::State;
use virtual_exec_core::{Machine, compile, parse};
use virtual_exec_std::BASIC;
use virtual_exec_type::error::{ExecutionError, NonRecoverableError};
use virtual_exec_type::mem::OwnedValue;

#[test]
fn test_mem() {
    let code = "i = 0; a = \"a\"; while i < 18 { a = concat(a, a); i+=1; }";
    let compiled = compile(&parse(code).unwrap());
    println!("{:?}", compiled);
    for _ in 0..10000 {
        let mut machine =
            Machine::new(compiled.clone(), 1 << 19, 500, vec![BASIC.clone()]).unwrap();
        match machine.sync_run_all() {
            Ok(State::Ok) | Ok(State::Terminated) => {}
            Ok(reason) => {
                println!("state: {:?}", reason);
            }
            Err(e) => {
                println!("err: {:?}", e);
                match e {
                    ExecutionError::NonRecoverable(NonRecoverableError::MemoryError) => {
                        let curr = machine.alloc.lock_arc_blocking().curr();
                        let max = machine.alloc.lock_arc_blocking().max;
                        println!("{}/{}", curr, max);
                    }
                    _ => {}
                }
            }
        }
    }
}
