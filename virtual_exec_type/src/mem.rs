use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::String;
use crate::HashMap;
use core::marker::PhantomData;
use core::ops::Deref;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use async_lock::{Mutex, MutexGuardArc, RwLock, RwLockReadGuardArc, RwLockWriteGuardArc};
use crate::error::{MemoryError, ExecutionError};
use crate::ext::*;

pub type MemoryAllocator<'a> = Arc<Mutex<MemoryAllocation<'a>>>;

pub trait ToOwnedValue {
    type Output;

    fn to_owned_value(&self) -> Self::Output;
}

#[derive(Debug, Clone)]
pub enum Value<'a> {
    Int(i64),
    Float(f64),
    Bool(bool),
    None,
    String(Box<str>),
    Collection(Arc<RwLock<Vec<ValuePtr<'a>>>>),
    Object(Arc<RwLock<HashMap<String, ValuePtr<'a>>>>),
    #[doc(hidden)]
    _Scope(PhantomData<&'a ()>),
    #[doc(hidden)]
    MemoryChunk(usize),
    Error(ExecutionError),
    DPtr(u64, usize),
    FnPtrExternal(Box<str>, usize)
}

impl PartialEq for Value<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::None, Value::None) => true,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Collection(a), Value::Collection(b)) => Arc::ptr_eq(a, b),
            (Value::Object(a), Value::Object(b)) => Arc::ptr_eq(a, b),
            (Value::MemoryChunk(a), Value::MemoryChunk(b)) => a == b,
            (Value::Error(a), Value::Error(b)) => a == b,
            (Value::DPtr(a, c), Value::DPtr(b, d)) => a == b && c == d,
            _ => false
        }
    }
}

impl ToOwnedValue for Value<'_> {
    type Output = OwnedValue;

    fn to_owned_value(&self) -> Self::Output {
        match self {
            Value::Int(i) => OwnedValue::Int(*i),
            Value::Float(f) => OwnedValue::Float(*f),
            Value::Bool(b) => OwnedValue::Bool(*b),
            Value::None => OwnedValue::None,
            Value::String(s) => OwnedValue::String(s.to_owned()),
            Value::Collection(c) => {
                OwnedValue::Collection(c.read_arc_safe().iter().map(|v| v.read_arc_safe().to_owned_value()).collect::<Vec<_>>().into())
            },
            Value::Object(d) => {
                OwnedValue::Object(d.read_arc_safe().iter().map(|(k, v)| (k.to_owned(), v.read_arc_safe().to_owned_value())).collect())
            },
            Value::_Scope(_) => OwnedValue::None,
            Value::MemoryChunk(_) => OwnedValue::None,
            Value::Error(e) => OwnedValue::Error(e.clone()),
            Value::DPtr(d, s) => OwnedValue::DPtr(*d, *s),
            Value::FnPtrExternal(f, s) => OwnedValue::FnPtrExternal(f.clone(), *s),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OwnedValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(Box<str>),
    None,
    Collection(Vec<OwnedValue>),
    Object(HashMap<String, OwnedValue>),
    Error(ExecutionError),
    DPtr(u64, usize),
    FnPtrExternal(Box<str>, usize)
}

#[derive(Debug)]
pub struct ValueInnerPtr<'a> {
    pub inner: Value<'a>,
    size: usize,
    pub(self) alloc: Weak<Mutex<MemoryAllocation<'a>>>
}

impl<'a> Deref for ValueInnerPtr<'a> {
    type Target = Value<'a>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a> ValueInnerPtr<'a> {
    pub fn new(inner: Value<'a>, size: usize, alloc: &MemoryAllocator<'a>) -> Self {
        Self {
            inner, size, alloc: Arc::downgrade(alloc)
        }
    }

    pub fn marked_size(&self) -> usize {
        self.size
    }
}


impl<'a> Drop for ValueInnerPtr<'a> {
    fn drop(&mut self) {
        let ptr = self.alloc.upgrade();
        if let Some(ptr) = ptr {
            if let Some(mut alloc) = ptr.try_lock() {
                alloc._internal_dealloc(self.size);
            }
        }
    }
}

pub type ValuePtr<'a> = Arc<RwLock<ValueInnerPtr<'a>>>;

#[derive(Debug)]
pub struct MemoryAllocation<'a> {
    curr: usize,
    pub max: usize,
    _phantom: PhantomData<&'a ()>,
    _obj: Vec<Weak<RwLock<ValueInnerPtr<'a>>>>
}

impl<'a> MemoryAllocation<'a> {
    pub fn new(max: usize) -> MemoryAllocation<'a> {
        Self {
            curr: 0, max,
            _phantom: Default::default(),
            _obj: Vec::new()
        }
    }

    pub fn check_alloc(&self, size: usize) -> bool {
        if size > self.max  { return false };
        let req_curr = self.max - size;
        req_curr >= self.curr
    }

    pub fn check_alloc_err(&self, size: usize) -> Result<(), MemoryError> {
        if self.check_alloc(size) {
            return Ok(());
        }
        Err(MemoryError)
    }

    pub(self) fn _internal_alloc(&mut self, size: usize) -> Result<(), MemoryError> {
        if self.check_alloc(size) {
            self.curr += size;
            return Ok(());
        }
        Err(MemoryError)
    }

    pub(self) fn _internal_dealloc(&mut self, size: usize) -> () {
        // `size` should never exceed `curr` while accounting is consistent
        // (`curr` is the sum of every live object's marked size). In debug/test
        // builds (`debug_assertions` on) panic loudly so a desync never goes
        // silent; in release, saturate so a bookkeeping bug degrades into a
        // recoverable over-count instead of aborting the whole process.
        debug_assert!(
            size <= self.curr,
            "memory accounting desync: dealloc of {} exceeds tracked {}",
            size,
            self.curr
        );
        self.curr = self.curr.saturating_sub(size);
    }

    pub fn gc_weak(&mut self) -> () {
        self._obj.retain(|obj| obj.upgrade().is_some());
    }

    pub fn obj_count(&self) -> usize {
        self._obj.len()
    }

    pub(self) fn _index_obj(&mut self, obj: &ValuePtr<'a>) -> () {
        self._obj.push(Arc::downgrade(obj));
    }

    pub fn curr(&self) -> usize {
        self.curr
    }
}

impl<'a> Drop for MemoryAllocation<'a> {
    fn drop(&mut self) {
        self.gc_weak();
        self._obj.iter().for_each(|obj| {
            if let Some(ptr) = obj.upgrade() {
                if let Some(mut inner) = ptr.try_write() {
                    inner.inner = Value::None;
                }
            }
        })
    }
}

pub trait GetSize {
    fn get_size(&self) -> usize;
}

pub trait Allocator {
    type Input : GetSize;
    type Output;

    fn alloc(&self, input: Self::Input) -> Result<Self::Output, MemoryError>;

    fn change_alloc(&self, data: & Self::Output) -> Result<(), MemoryError>;
}

impl<'a> GetSize for Value<'a> {
    fn get_size(&self) -> usize {
        match self {
            Value::Int(_i) => 8,
            Value::Float(_f) => 8,
            Value::Bool(_b) => 1,
            Value::Collection(c) => c.read_arc_safe().len() * 8 + 16,
            Value::Object(d) => {
                let map = d.read_arc_safe();
                map.len() * 8 + map.keys().map(|k| k.len()).sum::<usize>() + 16
            },
            Value::String(s) => s.len(),
            Value::None => 1,
            Value::_Scope(_) => 1,
            Value::MemoryChunk(size) => *size,
            Value::Error(_) => 1024,
            Value::DPtr(_, _) => 16,
            Value::FnPtrExternal(f, _) => f.len(),
        }
    }
}

pub trait MemoryAllocatorConstructor<'a> {
    fn construct(max: usize) -> MemoryAllocator<'a> {
        Arc::new(Mutex::new(MemoryAllocation::new(max)))
    }
}

impl<'a> MemoryAllocatorConstructor<'a> for MemoryAllocator<'a> {}

impl<'a> Allocator for MemoryAllocator<'a> {
    type Input = Value<'a>;
    type Output = Arc<RwLock<ValueInnerPtr<'a>>>;

    fn alloc(&self, input: Self::Input) -> Result<Self::Output, MemoryError> {
        let size = input.get_size();
        self.lock_arc_safe()._internal_alloc(size)?;
        let obj = ValueInnerPtr::new(input, size, self);
        let ptr = Arc::new(RwLock::new(obj));
        self.lock_arc_safe()._index_obj(&ptr);
        Ok(ptr)
    }

    fn change_alloc(&self, data: &Self::Output) -> Result<(), MemoryError> {
        let marked_size = data.read_arc_safe().marked_size();
        let new_size = data.read_arc_safe().get_size();
        if marked_size == new_size {
            return Ok(());
        }
        if marked_size < new_size {
            self.lock_arc_safe()._internal_alloc(new_size - marked_size)?;
        } else if marked_size > new_size {
            self.lock_arc_safe()._internal_dealloc(marked_size - new_size);
        }
        data.write_arc_safe().size = new_size;
        Ok(())

    }
}