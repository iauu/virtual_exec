use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock};

pub struct MemoryError;

pub enum Value<'a> {
    Int(u64),
    Float(f64),
    Bool(bool),
    None,
    Collection(Arc<RwLock<Vec<Value<'a>>>>),
    Dictionary(Arc<RwLock<HashMap<String, Value<'a>>>>),
    #[doc(hidden)]
    _Scope(PhantomData<&'a ()>),
}

pub struct ValueInnerPtr<'a> {
    inner: Value<'a>,
    size: usize,
}

impl<'a> Deref for ValueInnerPtr<'a> {
    type Target = Value<'a>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}


pub struct ValuePtr<'a>(Arc<Mutex<ValueInnerPtr<'a>>>);

impl<'a> Deref for ValuePtr<'a> {
    type Target = Arc<Mutex<ValueInnerPtr<'a>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct MemoryAllocation {
    pub curr: usize,
    pub max: usize,
}

impl MemoryAllocation {
    pub fn new(max: usize) -> MemoryAllocation {
        Self {
            curr: 0, max
        }
    }

    pub fn check_alloc(&self, size: usize) -> bool {
        if (size > self.max) { return false };
        let req_curr = self.max - size;
        req_curr < self.curr
    }

    pub fn check_alloc_err(&self, size: usize) -> Result<(), MemoryError> {
        if self.check_alloc(size) {
            return Ok(());
        }
        Err(MemoryError)
    }
}
