use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock};

pub struct MemoryError;

pub type MemoryAllocator<'a> = Arc<Mutex<MemoryAllocation<'a>>>;

pub enum Value<'a> {
    Int(u64),
    Float(f64),
    Bool(bool),
    None,
    Collection(Arc<RwLock<Vec<ValuePtr<'a>>>>),
    Dictionary(Arc<RwLock<HashMap<String, ValuePtr<'a>>>>),
    #[doc(hidden)]
    _Scope(PhantomData<&'a ()>),
}

pub struct ValueInnerPtr<'a> {
    inner: Value<'a>,
    size: usize,
    pub(self) alloc: MemoryAllocator<'a>
}

impl<'a> Deref for ValueInnerPtr<'a> {
    type Target = Value<'a>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a> ValueInnerPtr<'a> {
    pub fn new(inner: Value<'a>, size: usize, alloc: MemoryAllocator<'a>) -> Self {
        Self {
            inner, size, alloc
        }
    }

    pub fn marked_size(&self) -> usize {
        self.size
    }
}


impl<'a> Drop for ValueInnerPtr<'a> {
    fn drop(&mut self) {
        self.alloc.lock().unwrap()._internal_dealloc(self.size);
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
    pub curr: usize,
    pub max: usize,
    _phantom: PhantomData<&'a ()>
}

impl<'a> MemoryAllocation<'a> {
    pub fn new(max: usize) -> MemoryAllocation<'a> {
        Self {
            curr: 0, max,
            _phantom: Default::default(),
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
            Value::Dictionary(d) => {
                let map = d.read().unwrap();
                map.len() * 8 + map.keys().map(|k| k.len()).sum::<usize>()
            },
            Value::None => 1,
            Value::_Scope(_) => 1
        }
    }
}

impl<'a> Allocator for MemoryAllocator<'a> {
    type Input = Value<'a>;
    type Output = ValueInnerPtr<'a>;

    fn alloc(&mut self, input: Self::Input) -> Result<Self::Output, MemoryError> {
        let size = input.get_size();
        self.lock().unwrap()._internal_alloc(size)?;
        Ok(ValueInnerPtr {
            inner: input,
            size,
            alloc: Arc::clone(self),
        })
    }

    fn change_alloc(&mut self, data: &mut Self::Output) -> Result<(), MemoryError> {
        let marked_size = data.marked_size();
        let new_size = data.get_size();
        if marked_size == new_size {
            return Ok(());
        }
        if marked_size < new_size {
            self.lock().unwrap()._internal_alloc(new_size - marked_size)?;
        } else if marked_size > new_size {
            self.lock().unwrap()._internal_dealloc(marked_size - new_size);
        }
        data.size = new_size;
        Ok(())

    }
}