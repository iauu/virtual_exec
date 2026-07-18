use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::String;
use crate::HashMap;
use core::marker::PhantomData;
use core::ops::Deref;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use std::collections::HashSet;
use std::ops::DerefMut;
use async_lock::{Mutex, MutexGuardArc, RwLock, RwLockReadGuardArc, RwLockWriteGuardArc};
use crate::error::{MemoryError, ExecutionError, MemoryOutOfBoundError};
use crate::ext::*;

pub type MemoryAllocator<'a> = Arc<Mutex<MemoryAllocation<'a>>>;

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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ValueKind {
    Int,
    Float,
    Bool,
    String,
    None,
    Collection,
    Object,
    Error,
    DPtr,
    FnPtrExternal
}

#[derive(Debug, Clone)]
pub enum OwnedValueInternal {
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

impl PartialEq for OwnedValueInternal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (OwnedValueInternal::Int(a), OwnedValueInternal::Int(b)) => a == b,
            (OwnedValueInternal::Float(a), OwnedValueInternal::Float(b)) => a == b,
            (OwnedValueInternal::Bool(a), OwnedValueInternal::Bool(b)) => a == b,
            (OwnedValueInternal::None, OwnedValueInternal::None) => true,
            (OwnedValueInternal::String(a), OwnedValueInternal::String(b)) => a == b,
            (OwnedValueInternal::Collection(a), OwnedValueInternal::Collection(b)) => a
                .iter().zip(b)
                .all(|(a, b)| Arc::ptr_eq(a, b)),
            (OwnedValueInternal::Object(a), OwnedValueInternal::Object(b))  => 
                a.iter().zip(b).all(|(a, b)| {a.0 == b.0 && Arc::ptr_eq(a.1, b.1)}),
            (OwnedValueInternal::Error(a), OwnedValueInternal::Error(b)) => a == b,
            (OwnedValueInternal::DPtr(a, c), OwnedValueInternal::DPtr(b, d)) => a == b && c == d,
            (OwnedValueInternal::FnPtrExternal(a, c), OwnedValueInternal::FnPtrExternal(b, d)) => a == b && c == d,
            _ => false
        }
    }
}


/// Note: Restoring owned value pointer that also currently exist in the allocator doesn't re-merge them (just so I don't forget :p)
pub type OwnedValue = Arc<RwLock<OwnedValueInternal>>;


impl Into<ValueKind> for OwnedValue {
    fn into(self) -> ValueKind {
        match self.read_arc_safe().deref() {
            OwnedValueInternal::Int(_) => ValueKind::Int,
            OwnedValueInternal::Float(_) => ValueKind::Float,
            OwnedValueInternal::Bool(_) => ValueKind::Bool,
            OwnedValueInternal::String(_) => ValueKind::String,
            OwnedValueInternal::None => ValueKind::None,
            OwnedValueInternal::Collection(_) => ValueKind::Collection,
            OwnedValueInternal::Object(_) => ValueKind::Object,
            OwnedValueInternal::Error(_) => ValueKind::Error,
            OwnedValueInternal::DPtr(_, _) => ValueKind::DPtr,
            OwnedValueInternal::FnPtrExternal(_, _) => ValueKind::FnPtrExternal
        }
    }
}

impl<'a> Into<ValueKind> for &Value<'a> {
    fn into(self) -> ValueKind {
        match self {
            Value::Int(_) => ValueKind::Int,
            Value::Float(_) => ValueKind::Float,
            Value::Bool(_) => ValueKind::Bool,
            Value::String(_) => ValueKind::String,
            Value::None => ValueKind::None,
            Value::Collection(_) => ValueKind::Collection,
            Value::Object(_) => ValueKind::Object,
            Value::Error(_) => ValueKind::Error,
            Value::DPtr(_, _) => ValueKind::DPtr,
            Value::FnPtrExternal(_, _) => ValueKind::FnPtrExternal,
            Value::_Scope(_) => ValueKind::None,
            Value::MemoryChunk(_) => ValueKind::None
        }
    }
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

impl<'a> Into<ValueKind> for ValuePtr<'a> {
    fn into(self) -> ValueKind {
        (&self.read_arc_safe().inner).into()
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


fn drop_value_iter(root: Value) {
    let mut work = Vec::new();
    work.push(root);
    while let Some(value) = work.pop() {
        let children: Option<Vec<ValuePtr>> = match value {
            Value::Collection(arc) => Arc::try_unwrap(arc).ok().map(|lock| lock.into_inner()),
            Value::Object(arc) => Arc::try_unwrap(arc).ok().map(|lock| lock.into_inner().into_values().collect()),
            _ => None,
        };
        if let Some(children) = children {
            for child in children {
                if let Ok(lock) = Arc::try_unwrap(child) {
                    let mut inner = lock.into_inner();
                    work.push(core::mem::replace(&mut inner.inner, Value::None));
                }
            }
        }
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
        drop_value_iter(core::mem::replace(&mut self.inner, Value::None));
    }
}

pub type ValuePtr<'a> = Arc<RwLock<ValueInnerPtr<'a>>>;

enum OwnedValueConstruction {
    List(Vec<usize>),
    Dict(HashMap<String, usize>),
    None
}

fn to_owned_init_transform<'a>(ptr: &ValuePtr<'a>) -> OwnedValue {
    Arc::new(RwLock::new(match &ptr.read_arc_safe().inner {
        Value::Int(x) => OwnedValueInternal::Int(*x),
        Value::Float(x) => OwnedValueInternal::Float(*x),
        Value::Bool(x) => OwnedValueInternal::Bool(*x),
        Value::None => OwnedValueInternal::None,
        Value::String(x) => OwnedValueInternal::String(x.clone()),
        Value::Collection(_) => OwnedValueInternal::Collection(Vec::new()),
        Value::Object(_) => OwnedValueInternal::Object(HashMap::new()),
        Value::_Scope(_) => OwnedValueInternal::None,
        Value::MemoryChunk(_) => OwnedValueInternal::None,
        Value::Error(e) => OwnedValueInternal::Error(e.clone()),
        Value::DPtr(ptr, size) => OwnedValueInternal::DPtr(*ptr, *size),
        Value::FnPtrExternal(name, size) => OwnedValueInternal::FnPtrExternal(name.clone(), *size),
    }))
}

#[derive(Debug)]
pub struct MemoryAllocation<'a> {
    curr: usize,
    pub max: usize,
    _phantom: PhantomData<&'a ()>,
    _obj: Vec<Weak<RwLock<ValueInnerPtr<'a>>>>,
    _obj_watermark: usize
}

impl<'a> MemoryAllocation<'a> {
    pub fn new(max: usize) -> MemoryAllocation<'a> {
        Self {
            curr: 0, max,
            _phantom: Default::default(),
            _obj: Vec::new(),
            _obj_watermark: 16
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
        if self._obj.len() >= self._obj_watermark {
            self.gc_weak();
            self._obj_watermark = self._obj.len() * 2 + 16;
        }
        self._obj.push(Arc::downgrade(obj));
    }

    pub fn curr(&self) -> usize {
        self.curr
    }

    fn get_idx_ref(&self, value: &ValuePtr<'a>) -> Result<usize, MemoryOutOfBoundError> {
        self._obj.iter().position(|obj| if let Some(ptr) = obj.upgrade() {Arc::ptr_eq(&ptr, value)} else {false})
            .ok_or(MemoryOutOfBoundError)
    }

    fn get_obj_construction(&self, value: &ValuePtr<'a>) -> Result<OwnedValueConstruction, MemoryOutOfBoundError> {
        Ok(match value.read_arc_safe().clone() {
            Value::Collection(collect) => OwnedValueConstruction::List(
                collect.read_arc_safe()
                    .iter()
                    .map(
                        |item|
                            self.get_idx_ref(item)
                    ).collect::<Result<_, MemoryOutOfBoundError>>()?),
            Value::Object(obj) => OwnedValueConstruction::Dict(
                obj.read_arc_safe()
                    .iter()
                    .map(
                        |(k, v)|
                            Ok((k.clone(), self.get_idx_ref(v)?))
                    ).collect::<Result<_, MemoryOutOfBoundError>>()?),
            _ => OwnedValueConstruction::None
        })
    }

    pub fn get_owned(&mut self, value: &ValuePtr<'a>) -> Result<OwnedValue, MemoryOutOfBoundError> {
        self.gc_weak();
        let idx_self = self.get_idx_ref(value)?;
        let mut ref_id: HashSet<usize> = HashSet::new();
        let mut non_handled: Vec<usize> = Vec::new();
        let mut obj_map: HashMap<usize, (OwnedValue, OwnedValueConstruction)> = HashMap::new();
        non_handled.push(idx_self);
        while let Some(idx_ref) = non_handled.pop() {
            let value_idx = self._obj[idx_ref].upgrade().ok_or(MemoryOutOfBoundError)?;
            let construction = self.get_obj_construction(&value_idx)?;
            match &construction {
                OwnedValueConstruction::List(list) => {
                    list.iter().for_each(|item| {
                        if !non_handled.contains(item) {
                            ref_id.insert(*item);
                        }
                    });
                },
                OwnedValueConstruction::Dict(map) => {
                    map.values().for_each(|item| {
                        if !non_handled.contains(item) {
                        ref_id.insert(*item);
                        }
                    });
                },
                OwnedValueConstruction::None => {}
            };
            obj_map.insert(idx_ref, (to_owned_init_transform(&value_idx), construction));
        }

        let obj_view_map: HashMap<usize, OwnedValue> = obj_map.iter().map(|(k, v)| (*k, Arc::clone(&v.0))).collect();

        for (_, (ptr, construction)) in obj_map.iter_mut() {
            match construction {
                OwnedValueConstruction::List(list) => {
                    if let OwnedValueInternal::Collection(vec) = ptr.write_arc_safe().deref_mut() {
                        list.iter().for_each(|item| {
                            vec.push(obj_view_map[item].clone());
                        })
                    }
                },
                OwnedValueConstruction::Dict(map) => {
                    if let OwnedValueInternal::Object(obj) = ptr.write_arc_safe().deref_mut() {
                        map.iter_mut().for_each(|(k, v)| {
                            obj.insert(k.clone(), obj_view_map[v].clone());
                        })
                    }
                },
                _ => {}
            }
        }

        Ok(Arc::clone(&obj_view_map[&idx_self]))
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

const NODE_OVERHEAD: usize = 32;
const CONTAINER_OVERHEAD: usize = 32;

impl<'a> GetSize for Value<'a> {
    fn get_size(&self) -> usize {
        match self {
            Value::Int(_i) => 8 + NODE_OVERHEAD,
            Value::Float(_f) => 8 + NODE_OVERHEAD,
            Value::Bool(_b) => 1 + NODE_OVERHEAD,
            Value::Collection(c) => c.read_arc_safe().len() * 8 + 16 + NODE_OVERHEAD + CONTAINER_OVERHEAD,
            Value::Object(d) => {
                let map = d.read_arc_safe();
                map.len() * 8 + map.keys().map(|k| k.len()).sum::<usize>() + 16 + NODE_OVERHEAD + CONTAINER_OVERHEAD
            },
            Value::String(s) => s.len() + NODE_OVERHEAD,
            Value::None => 1 + NODE_OVERHEAD,
            Value::_Scope(_) => 1 + NODE_OVERHEAD,
            Value::MemoryChunk(size) => *size,
            Value::Error(_) => 1024,
            Value::DPtr(_, _) => 16 + NODE_OVERHEAD,
            Value::FnPtrExternal(f, _) => f.len() + NODE_OVERHEAD,
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