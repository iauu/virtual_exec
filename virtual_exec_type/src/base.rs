use alloc::format;
use crate::HashMap;
use alloc::sync::{Arc};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::Display;
use async_lock::{RwLock};
use crate::config::recurse::{RecurseRestricter, RecursionError};
use crate::mem::{Allocator, MemoryAllocator, Value, ValuePtr};
use crate::error::{MemoryError, ExecutionError};
use crate::ext::*;

pub trait IsTruhy {
    fn is_truthy(&self) -> bool;
}

impl IsTruhy for Value<'_> {
    fn is_truthy(&self) -> bool {
        match self {
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::Bool(b) => *b,
            Value::None => false,
            Value::String(s) => s.len() > 0,
            Value::Collection(v) => v.read_arc_safe().len() > 0,
            Value::Object(v) => v.read_arc_safe().len() > 0,
            Value::_Scope(_) => false,
            Value::MemoryChunk(_) => false,
            Value::Error(_) => false,
            Value::DPtr(_, _) => true,
            Value::FnPtrExternal(_, _) => true,
        }
    }
}

impl IsTruhy for ValuePtr<'_> {
    fn is_truthy(&self) -> bool {
        self.lock_arc_safe().is_truthy()
    }
}


pub trait ToStringSafe {
    fn to_string_safe(&self, recurse_restricter: RecurseRestricter) -> Result<String, RecursionError>;
}

macro_rules! consume_fmt {
    ($rest:expr, $fmt:literal $(, $args:tt)* $(,)?) => {
        {
            let x = format!($fmt $(, $args)*);
            $rest.consume_mem(x.len() as u64)?;
            x
        }
    };
}

impl ToStringSafe for Value<'_> {
    fn to_string_safe(&self, recurse_restricter: RecurseRestricter) -> Result<String, RecursionError> {
        recurse_restricter.consume_inst(1)?;
        Ok(match self {
            Value::Int(i) => consume_fmt!(recurse_restricter, "{}", i),
            Value::Float(f) => consume_fmt!(recurse_restricter, "{}", f),
            Value::Bool(b) => consume_fmt!(recurse_restricter, "{}", b),
            Value::String(s) => consume_fmt!(recurse_restricter, "\"{}\"", s),
            Value::Collection(v) => {
                recurse_restricter.consume_mem((v.read_arc_safe().len() as u64 + 1)*2)?;
                format!("[{}]", v.read_arc_safe().iter().map(|v| Ok(
                    v.to_string_safe(recurse_restricter.incr()?)?
                )).collect::<Result<Vec<String>, RecursionError>>()?.join(", "))
            },
            Value::Object(v) => {
                recurse_restricter.consume_mem((v.read_arc_safe().len() as u64 + 1)*4)?;
                let key_lens: u64 = v.read_arc_safe().iter().map(|v| v.0.len()).sum::<usize>() as u64;
                recurse_restricter.consume_mem(key_lens)?;
                format!("{{{}}}", v.read_arc_safe().iter().map(|v| Ok(
                    format!("\"{}\": {}", v.0, v.1.to_string_safe(recurse_restricter.incr()?)?)
                )).collect::<Result<Vec<String>, RecursionError>>()?.join(", "))
            }
            Value::None => consume_fmt!(recurse_restricter, "None"),
            Value::_Scope(_) => consume_fmt!(recurse_restricter, "_Scoped"),
            Value::MemoryChunk(size) => consume_fmt!(recurse_restricter, "_MemChunk(size: {})", size),
            Value::Error(e) => consume_fmt!(recurse_restricter, "_Error({:?})", e),
            Value::DPtr(ptr, size) => consume_fmt!(recurse_restricter, "DynFuncPtr(loc: {}, arg_len: {})", ptr, size),
            Value::FnPtrExternal(name, size) => consume_fmt!(recurse_restricter, "DynExternFuncPtr(loc: {}, arg_len: {})", name, size)
        })
    }
}

impl ToStringSafe for ValuePtr<'_> {
    fn to_string_safe(&self, recurse_restricter: RecurseRestricter) -> Result<String, RecursionError> {
        self.lock_arc_safe().to_string_safe(recurse_restricter)
    }
}

pub trait TypeCast<'a> {
    fn as_int(&self) -> Option<i64>;
    fn as_float(&self) -> Option<f64>;

    fn as_object(&self) -> Option<Arc<RwLock<HashMap<String, ValuePtr<'a>>>>>;

    fn as_collections(&self) -> Option<Arc<RwLock<Vec<ValuePtr<'a>>>>>;

    fn as_string(&self) -> Option<String>;

    fn as_bool(&self) -> Option<bool>;

    fn as_none(&self) -> Option<()>;

    fn as_error(&self) -> Option<ExecutionError>;
    
    fn as_dptr(&self) -> Option<(u64, usize)>;
    
    fn as_fn_ptr_extern(&self) -> Option<(String, usize)>;
}

impl<'a> TypeCast<'a> for ValuePtr<'a> {
    fn as_int(&self) -> Option<i64> {
        if let Value::Int(v) = self.lock_arc_safe().inner {
            Some(v)
        } else {
            None
        }
    }

    fn as_bool(&self) -> Option<bool> {
        if let Value::Bool(b) = self.lock_arc_safe().inner {
            Some(b)
        } else {
            None
        }
    }

    fn as_float(&self) -> Option<f64> {
        if let Value::Float(v) = self.lock_arc_safe().inner {
            Some(v)
        } else {
            None
        }
    }

    fn as_object(&self) -> Option<Arc<RwLock<HashMap<String, ValuePtr<'a>>>>> {
        if let Value::Object(o) = &self.clone().lock_arc_safe().inner {
            Some(o.clone())
        } else {
            None
        }
    }

    fn as_collections(&self) -> Option<Arc<RwLock<Vec<ValuePtr<'a>>>>> {
        if let Value::Collection(c) = &self.clone().lock_arc_safe().inner {
            Some(c.clone())
        } else {
            None
        }
    }

    fn as_string(&self) -> Option<String> {
        if let Value::String(s) = &self.lock_arc_safe().inner {
            Some(s.to_string())
        } else {
            None
        }
    }

    fn as_none(&self) -> Option<()> {
        let item = &self.lock_arc_safe().inner;
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

    fn as_error(&self) -> Option<ExecutionError> {
        if let Value::Error(e) = &self.lock_arc_safe().inner {
            Some(e.clone())
        } else {
            None
        }
    }

    fn as_dptr(&self) -> Option<(u64, usize)> {
        if let Value::DPtr(d, s) = &self.clone().lock_arc_safe().inner {
            Some((*d, *s))
        } else {
            None
        }
    }
    
    fn as_fn_ptr_extern(&self) -> Option<(String, usize)> {
        if let Value::FnPtrExternal(f, s) = &self.clone().lock_arc_safe().inner {
            Some((f.to_string(), *s))
        } else {
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

impl<'ctx> Downcast<'ctx> for i64 {
    fn from_value(value: ValuePtr<'ctx>) -> Option<Self> {
        value.as_int()
    }
}

impl<'ctx> Upcast<'ctx> for i64 {
    fn from_value(&self, alloc: &MemoryAllocator<'ctx>) -> Result<ValuePtr<'ctx>, MemoryError> {
        alloc.alloc(Value::Int(*self))
    }
}

impl<'ctx> Downcast<'ctx> for f64 {
    fn from_value(value: ValuePtr<'ctx>) -> Option<Self> {
        value.as_float()
    }
}

impl<'ctx> Upcast<'ctx> for f64 {
    fn from_value(&self, alloc: &MemoryAllocator<'ctx>) -> Result<ValuePtr<'ctx>, MemoryError> {
        alloc.alloc(Value::Float(*self))
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

impl<'ctx> Downcast<'ctx> for ExecutionError {
    fn from_value(value: ValuePtr<'ctx>) -> Option<Self> {
        value.as_error()
    }
}

impl<'ctx> Upcast<'ctx> for ExecutionError {
    fn from_value(&self, alloc: &MemoryAllocator<'ctx>) -> Result<ValuePtr<'ctx>, MemoryError> {
        alloc.alloc(Value::Error(self.clone()))
    }
}

impl<'ctx> Downcast<'ctx> for ValuePtr<'ctx> {
    fn from_value(value: ValuePtr<'ctx>) -> Option<Self> {
        Some(value)
    }
}

impl<'ctx> Upcast<'ctx> for ValuePtr<'ctx> {
    fn from_value(&self, _alloc: &MemoryAllocator<'ctx>) -> Result<ValuePtr<'ctx>, MemoryError> {
        Ok(self.clone())
    }
}