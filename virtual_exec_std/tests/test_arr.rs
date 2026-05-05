mod test_sys;
mod test_mem;

use virtual_exec_core::{Machine, parse, compile};
use virtual_exec_core::sequential::exec::State;
use virtual_exec_type::error::ExecutionError;
use virtual_exec_type::mem::OwnedValue;
use virtual_exec_std::BASIC;

#[test]
fn test_arr() {
    let code = "arr = create_array(); push_array(arr, 100); value = arr_get_from_idx(arr, 0); size = arr_get_len(arr); value2 = pop_array(arr);";
    let compiled = compile(&parse(code).unwrap());
    println!("{:?}", compiled);
    let mut machine = Machine::new(compiled, 200, 200, vec![BASIC.clone()]).unwrap();
    match machine.sync_run_all() {
        Ok(State::Ok) => {},
        Ok(reason) => {
            println!("Machine: {:?}, state: {:?}", machine, reason);
        },
        Err(e) => {
            println!("Machine: {:?}, err: {:?}", machine, e);
        }
    }
    assert_eq!(machine.get("value"), Some(OwnedValue::Int(100)));
    assert_eq!(machine.get("value2"), Some(OwnedValue::Int(100)));
    assert_eq!(machine.get("size"), Some(OwnedValue::Int(1)));
}

