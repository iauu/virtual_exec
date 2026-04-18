use bumpalo::Bump;
use virtual_exec_type::base::{ValueContainer, ValueKind};
use virtual_exec_type::builtin::{VirPyFloat, VirPyInt};
use virtual_exec_type::op::op_add;
use virtual_exec_type::alloc::Allocator;

#[test]
fn test_op_add_functionality() {
    let arena = Bump::new();
    let alloc = Allocator::new(&arena);

    let lhs_int = alloc.allocate(ValueKind::Int(VirPyInt::new(15)));
    let rhs_int = alloc.allocate(ValueKind::Int(VirPyInt::new(27)));

    let lhs_float = alloc.allocate(ValueKind::Float(VirPyFloat::new(1.5)));
    let rhs_float = alloc.allocate(ValueKind::Float(VirPyFloat::new(2.25)));

    let result_int = op_add(lhs_int, rhs_int, &alloc).expect("op_add for Int+Int should succeed");
    assert_eq!(result_int.as_int().unwrap().value, 42);
    println!("Int + Int works as expected.");

    let result_float =
        op_add(lhs_float, rhs_float, &alloc).expect("op_add for Float+Float should succeed");
    // Use a small epsilon for float comparison
    assert!((result_float.as_float().unwrap().value - 3.75).abs() < f64::EPSILON);
    println!("Float + Float works as expected.");

    let result_unsupported =
        op_add(lhs_int, lhs_float, &alloc).expect("op_add for Int+Float should succeed");
    assert!(result_unsupported.as_float().unwrap().value - 16.5 < f64::EPSILON);
    println!("Int + Float works as expected.");
}
