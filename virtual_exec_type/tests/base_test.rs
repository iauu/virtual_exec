use std::sync::{Arc};
use async_lock::Mutex;
use virtual_exec_type::mem::{Value, MemoryAllocation, Allocator};
use virtual_exec_type::base::TypeCast;

#[test]
fn test_value_creation_and_downcast() {
    let alloc = MemoryAllocation::new(100);
    let alloc = Arc::new(Mutex::new(alloc));
    let int_v = Value::Int(42);
    let int = alloc.alloc(int_v).unwrap();
    let extracted_int = int.as_int().expect("Downcast to Int failed");
    assert_eq!(extracted_int, 42);
    println!(
        "Successfully created and downcasted value: {:?}",
        int
    );
}
