use std::sync::{Arc, Mutex};
use virtual_exec_type::mem::*;

#[test]
fn test_mem_alloc() {
    use virtual_exec_type::mem::Value::MemoryChunk;
    let alloc = MemoryAllocation::new(100);
    let alloc = Arc::new(Mutex::new(alloc));
    let obj_1 = alloc.alloc(MemoryChunk(50)).unwrap();
    assert_eq!(alloc.lock().unwrap().curr(), 50);
    let obj_2 = alloc.alloc(MemoryChunk(50)).unwrap();
    assert_eq!(alloc.lock().unwrap().curr(), 100);
    assert_eq!(alloc.alloc(MemoryChunk(1)).is_err(), true, "Memory allocation should fail as it is allocating the 101th slot when limit is 100");
}

#[test]
fn test_mem_dealloc() {
    use virtual_exec_type::mem::Value::MemoryChunk;
    let alloc = MemoryAllocation::new(100);
    let alloc = Arc::new(Mutex::new(alloc));
    let obj_1 = alloc.alloc(MemoryChunk(50)).unwrap();
    assert_eq!(alloc.lock().unwrap().curr(), 50);
    let obj_2 = alloc.alloc(MemoryChunk(50)).unwrap();
    assert_eq!(alloc.lock().unwrap().curr(), 100);
    std::mem::drop(obj_1);
    assert_eq!(alloc.lock().unwrap().curr(), 50);
    assert_eq!(alloc.alloc(MemoryChunk(50)).is_ok(), true, "Memory allocation should access as only 50 slot used as original obj_1 is deallocated");
    assert_eq!(alloc.alloc(MemoryChunk(50)).is_ok(), true, "Memory allocation should access as only 50 slot used as original obj_1 and temporary object is deallocated");
    assert_eq!(alloc.lock().unwrap().curr(), 50);
}

#[test]
fn test_mem_change_alloc() {
    use virtual_exec_type::mem::Value::MemoryChunk;
    let alloc = MemoryAllocation::new(100);
    let alloc = Arc::new(Mutex::new(alloc));
    let mut obj_1 = alloc.alloc(MemoryChunk(50)).unwrap();
    let mut obj_2 = alloc.alloc(MemoryChunk(50)).unwrap();
    assert_eq!(alloc.lock().unwrap().curr(), 100);
    obj_1.lock().unwrap().inner = MemoryChunk(60);
    assert_eq!(alloc.change_alloc(&mut obj_1).is_err(), true, "Memory extend shouldn't be possible when there are no memory remaining");
    obj_1.lock().unwrap().inner = MemoryChunk(50);
    std::mem::drop(obj_1);
    assert_eq!(alloc.lock().unwrap().curr(), 50);
    obj_2.lock().unwrap().inner = MemoryChunk(100);
    assert_eq!(alloc.change_alloc(&mut obj_2).is_ok(), true, "Memory extend should be possible as there are 100 memory slot in total");
    assert_eq!(alloc.lock().unwrap().curr(), 100);
}
