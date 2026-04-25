use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex, RwLock};
use crate::mem::{Allocator, MemoryAllocator, MemoryError, Value, ValuePtr};

trait TypeCast<'a> {
    fn as_int(&self) -> Option<u64>;
    fn as_float(&self) -> Option<f64>;

    fn as_object(&self) -> Option<Arc<RwLock<HashMap<String, ValuePtr<'a>>>>>;

    fn as_collections(&self) -> Option<Arc<RwLock<Vec<ValuePtr<'a>>>>>;

    fn as_string(&self) -> Option<String>;

    fn as_bool(&self) -> Option<bool>;

    fn as_none(&self) -> Option<()>;

}

impl<'a> TypeCast<'a> for ValuePtr<'a> {
    fn as_int(&self) -> Option<u64> {
        if let Value::Int(v) = self.lock().unwrap().inner {
            Some(v)
        } else {
            None
        }
    }

    fn as_bool(&self) -> Option<bool> {
        if let Value::Bool(b) = self.lock().unwrap().inner {
            Some(b)
        } else {
            None
        }
    }

    fn as_float(&self) -> Option<f64> {
        if let Value::Float(v) = self.lock().unwrap().inner {
            Some(v)
        } else {
            None
        }
    }

    fn as_object(&self) -> Option<Arc<RwLock<HashMap<String, ValuePtr<'a>>>>> {
        if let Value::Object(o) = &self.clone().lock().unwrap().inner {
            Some(o.clone())
        } else {
            None
        }
    }

    fn as_collections(&self) -> Option<Arc<RwLock<Vec<ValuePtr<'a>>>>> {
        if let Value::Collection(c) = &self.clone().lock().unwrap().inner {
            Some(c.clone())
        } else {
            None
        }
    }

    fn as_string(&self) -> Option<String> {
        if let Value::String(s) = &self.lock().unwrap().inner {
            Some(s.to_string())
        } else {
            None
        }
    }

    fn as_none(&self) -> Option<()> {
        let item = &self.lock().unwrap().inner;
        if let Value::None = item {
            Some(())
        } else if let Value::MemoryChunk(_) = item  {
            Some(())
        } else if let Value::_Scope(_) = item {
            Some(())
        }
        else {
            None
        }
    }
}

pub trait Downcast<'ctx>: Sized {
    fn from_value(value: ValuePtr<'ctx>) -> Option<Self>;
}

pub trait Upcast<'ctx>: Sized {
    fn from_value(&self, alloc: &MemoryAllocator<'ctx>) -> Result<ValuePtr<'ctx>, MemoryError>;
}

impl<'ctx> Downcast<'ctx> for bool {
    fn from_value(value: ValuePtr<'ctx>) -> Option<Self> {
        value.as_bool()
    }
}

impl<'ctx> Upcast<'ctx> for bool {
    fn from_value(&self, alloc: &MemoryAllocator<'ctx>) -> Result<ValuePtr<'ctx>, MemoryError> {
        alloc.alloc(Value::Bool(*self))
    }
}

impl<'ctx> Downcast<'ctx> for Arc<RwLock<Vec<ValuePtr<'ctx>>>> {
    fn from_value(value: ValuePtr<'ctx>) -> Option<Self> {
        value.as_collections()
    }
}

impl<'ctx> Upcast<'ctx> for Arc<RwLock<Vec<ValuePtr<'ctx>>>> {
    fn from_value(&self, alloc: &MemoryAllocator<'ctx>) -> Result<ValuePtr<'ctx>, MemoryError> {
        alloc.alloc(Value::Collection(self.clone()))
    }
}

impl<'ctx> Downcast<'ctx> for String {
    fn from_value(value: ValuePtr<'ctx>) -> Option<Self> {
        value.as_string()
    }
}

impl<'ctx> Upcast<'ctx> for String {
    fn from_value(&self, alloc: &MemoryAllocator<'ctx>) -> Result<ValuePtr<'ctx>, MemoryError> {
        alloc.alloc(Value::String(self.clone().into_boxed_str()))
    }
}

impl<'ctx> Downcast<'ctx> for () {
    fn from_value(value: ValuePtr<'ctx>) -> Option<Self> {
        value.as_none()
    }
}

impl<'ctx> Upcast<'ctx> for () {
    fn from_value(&self, alloc: &MemoryAllocator<'ctx>) -> Result<ValuePtr<'ctx>, MemoryError> {
        alloc.alloc(Value::None)
    }
}
