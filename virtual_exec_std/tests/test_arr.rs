mod test_mem;
mod test_sys;

use std::ops::Deref;
use std::sync::Arc;
use virtual_exec_core::sequential::exec::State;
use virtual_exec_core::{Machine, compile, parse};
use virtual_exec_std::BASIC;
use virtual_exec_type::ext::SafeReadArcExt;
use virtual_exec_type::mem::OwnedValueInternal;

#[test]
fn test_arr() {
    let code = "arr = create_array(); push_array(arr, 100); value = arr_get_from_idx(arr, 0); size = arr_get_len(arr); value2 = pop_array(arr);";
    let compiled = compile(&parse(code).unwrap());
    println!("{:?}", compiled);
    let mut machine = Machine::new(compiled, 2000, 200, vec![BASIC.clone()]).unwrap();
    match machine.sync_run_all() {
        Ok(State::Ok) => {}
        Ok(reason) => {
            println!("Machine: {:?}, state: {:?}", machine, reason);
        }
        Err(e) => {
            println!("Machine: {:?}, err: {:?}", machine, e);
        }
    }
    let value = machine.get("value").expect("Variable `value` should exist");
    assert_eq!(value.read_arc_safe().deref(), &OwnedValueInternal::Int(100));
    let value2 = machine
        .get("value2")
        .expect("Variable `value2` should exist");
    assert_eq!(
        value2.read_arc_safe().deref(),
        &OwnedValueInternal::Int(100)
    );
    let size = machine.get("size").expect("Variable `size` should exist");
    assert_eq!(size.read_arc_safe().deref(), &OwnedValueInternal::Int(1));
}
