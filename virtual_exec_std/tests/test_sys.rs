use virtual_exec_core::{Machine, parse, compile};
use virtual_exec_core::sequential::exec::State;
use virtual_exec_std::{BASIC, SYS};

#[test]
fn test_print() {
    let code = "a = \"test\n\"; print(a);b = None; println(b);";
    let compiled = compile(&parse(code).unwrap());
    println!("{:?}", compiled);
    let mut machine = Machine::new(compiled, 200, 200, vec![BASIC.clone(), SYS.clone()]).unwrap();
    match machine.sync_run_all() {
        Ok(State::Ok) => {},
        Ok(reason) => {
            println!("Machine: {:?}, state: {:?}", machine, reason);
        },
        Err(e) => {
            println!("Machine: {:?}, err: {:?}", machine, e);
        }
    }
}

