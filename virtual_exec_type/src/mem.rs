use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock, Weak};

#[derive(Debug)]
pub struct MemoryError;

pub type MemoryAllocator<'a> = Arc<Mutex<MemoryAllocation<'a>>>;

pub trait ToOwned {
    type Output;

    fn to_owned(&self) -> Self::Output;
}

pub enum Value<'a> {
    Int(u64),
    Float(f64),
    Bool(bool),
    None,
    String(Box<str>),
    Collection(Arc<RwLock<Vec<ValuePtr<'a>>>>),
    Object(Arc<RwLock<HashMap<String, ValuePtr<'a>>>>),
    #[doc(hidden)]
    _Scope(PhantomData<&'a ()>),
    #[doc(hidden)]
    MemoryChunk(usize)
}

impl ToOwned for Value<'_> {
    type Output = OwnedValue;

    fn to_owned(&self) -> Self::Output {
        match self {
            Value::Int(i) => OwnedValue::Int(*i),
            Value::Float(f) => OwnedValue::Float(*f),
            Value::Bool(b) => OwnedValue::Bool(*b),
            Value::None => OwnedValue::None,
            Value::String(s) => OwnedValue::String(s.to_owned()),
            Value::Collection(c) => {
                OwnedValue::Collection(c.read().unwrap().iter().map(|v| v.lock().unwrap().to_owned()).collect::<Vec<_>>().into())
            },
            Value::Object(d) => {
                OwnedValue::Object(d.read().unwrap().iter().map(|(k, v)| (k.to_owned(), v.lock().unwrap().to_owned())).collect())
            },
            Value::_Scope(_) => OwnedValue::None,
            Value::MemoryChunk(_) => OwnedValue::None
        }
    }
}

pub enum OwnedValue {
    Int(u64),
    Float(f64),
    Bool(bool),
    String(Box<str>),
    None,
    Collection(Vec<OwnedValue>),
    Object(HashMap<String, OwnedValue>),
}

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
            if let Ok(mut alloc) = ptr.lock() {
                alloc._internal_dealloc(self.size);
            }
        }
    }
}

pub struct ValuePtr<'a>(Arc<Mutex<ValueInnerPtr<'a>>>);

impl<'a> Deref for ValuePtr<'a> {
    type Target = Arc<Mutex<ValueInnerPtr<'a>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct MemoryAllocation<'a> {
    curr: usize,
    pub max: usize,
    _phantom: PhantomData<&'a ()>,
    _obj: Vec<Weak<Mutex<ValueInnerPtr<'a>>>>
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
        if (size > self.max) { return false };
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
        if size > self.curr {
            unreachable!();
            self.curr = 0;
        } else {
            self.curr -= size;
        }
    }

    pub fn gc_weak(&mut self) -> () {
        self._obj.retain(|obj| obj.upgrade().is_some());
    }

    pub fn obj_count(&self) -> usize {
        self._obj.len()
    }

    pub(self) fn _index_obj(&mut self, obj: &Arc<Mutex<ValueInnerPtr<'a>>>) -> () {
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
                if let Ok(mut inner) = ptr.lock() {
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

    fn alloc(&mut self, input: Self::Input) -> Result<Self::Output, MemoryError>;

    fn change_alloc(&mut self, data: &mut Self::Output) -> Result<(), MemoryError>;
}

impl<'a> GetSize for Value<'a> {
    fn get_size(&self) -> usize {
        match self {
            Value::Int(i) => 8,
            Value::Float(f) => 8,
            Value::Bool(b) => 1,
            Value::Collection(c) => c.read().unwrap().len() * 8,
            Value::Object(d) => {
                let map = d.read().unwrap();
                map.len() * 8 + map.keys().map(|k| k.len()).sum::<usize>()
            },
            Value::String(s) => s.len(),
            Value::None => 1,
            Value::_Scope(_) => 1,
            Value::MemoryChunk(size) => *size
        }
    }
}

impl<'a> Allocator for MemoryAllocator<'a> {
    type Input = Value<'a>;
    type Output = Arc<Mutex<ValueInnerPtr<'a>>>;

    fn alloc(&mut self, input: Self::Input) -> Result<Self::Output, MemoryError> {
        let size = input.get_size();
        self.lock().unwrap()._internal_alloc(size)?;
        let obj = ValueInnerPtr::new(input, size, self);
        let ptr = Arc::new(Mutex::new(obj));
        self.lock().unwrap()._index_obj(&ptr);
        Ok(ptr)
    }

    fn change_alloc(&mut self, data: &mut Self::Output) -> Result<(), MemoryError> {
        let marked_size = data.lock().unwrap().marked_size();
        let new_size = data.lock().unwrap().get_size();
        if marked_size == new_size {
            return Ok(());
        }
        if marked_size < new_size {
            self.lock().unwrap()._internal_alloc(new_size - marked_size)?;
        } else if marked_size > new_size {
            self.lock().unwrap()._internal_dealloc(marked_size - new_size);
        }
        data.lock().unwrap().size = new_size;
        Ok(())

    }
}