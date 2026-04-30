use std::sync::{Arc};
use async_lock::Mutex;
use virtual_exec_type::base::TypeCast;
use virtual_exec_type::mem::{Allocator, MemoryAllocation, Value};
use virtual_exec_type::op::err_op_add;

#[test]
fn test_op_add_functionality() {
    let alloc = MemoryAllocation::new(1000);
    let alloc = Arc::new(Mutex::new(alloc));

    let lhs_int = alloc.alloc(Value::Int(15)).unwrap();
    let rhs_int = alloc.alloc(Value::Int(27)).unwrap();

    let lhs_float = alloc.alloc(Value::Float(1.5)).unwrap();
    let rhs_float = alloc.alloc(Value::Float(2.25)).unwrap();

    let result_int = err_op_add(lhs_int.clone(), rhs_int.clone(), &alloc).expect("op_add for Int+Int should succeed");
    assert_eq!(result_int.as_int().unwrap(), 42);
    println!("Int + Int works as expected.");

    let result_float =
        err_op_add(lhs_float.clone(), rhs_float.clone(), &alloc).expect("op_add for Float+Float should succeed");
    // Use a small epsilon for float comparison
    assert!((result_float.as_float().unwrap() - 3.75).abs() < f64::EPSILON);
    println!("Float + Float works as expected.");

    let result_unsupported =
        err_op_add(lhs_int.clone(), lhs_float.clone(), &alloc).expect("op_add for Int+Float should succeed");
    assert!(result_unsupported.as_float().unwrap() - 16.5 < f64::EPSILON);
    println!("Int + Float works as expected.");
}
