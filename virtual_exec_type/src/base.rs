use std::cell::RefCell;
use crate::builtin::{VirFloat, VirInt, VirObject};
use crate::error::SandboxExecutionError;
use bumpalo::Bump;
use std::fmt::Debug;
use std::rc::Rc;

pub type Value<'ctx> = &'ctx ValueContainer<'ctx>;

#[derive(Debug, Clone)]
pub enum ValueKind<'ctx> {
    Int(VirInt),
    Float(VirFloat),
    Object(VirObject<'ctx>),
    ErrorWrapped(SandboxExecutionError),
    Bool(bool),
    String(String),
    Collection(Rc<RefCell<Vec<Value<'ctx>>>>),
    None,
}

pub trait Downcast<'ctx>: Sized {
    fn from_value(value: Value<'ctx>) -> Option<&'ctx Self>;
}

pub trait Upcast<'ctx>: Sized {
    fn from_value(&'ctx self) -> ValueKind<'ctx>;
}

impl<'ctx> Downcast<'ctx> for bool {
    fn from_value(value: Value<'ctx>) -> Option<&'ctx Self> {
        value.as_bool()
    }
}

impl<'ctx> Upcast<'ctx> for bool {
    fn from_value(&'ctx self) -> ValueKind<'ctx> {
        ValueKind::Bool(*self)
    }
}

impl<'ctx> Downcast<'ctx> for Rc<RefCell<Vec<Value<'ctx>>>> {
    fn from_value(value: Value<'ctx>) -> Option<&'ctx Self> {
        value.as_collection()
    }
}

impl<'ctx> Upcast<'ctx> for Rc<RefCell<Vec<Value<'ctx>>>> {
    fn from_value(&'ctx self) -> ValueKind<'ctx> {
        ValueKind::Collection(self.clone())
    }
}

impl<'ctx> Downcast<'ctx> for String {
    fn from_value(value: Value<'ctx>) -> Option<&'ctx Self> {
        value.as_string()
    }
}

impl<'ctx> Upcast<'ctx> for String {
    fn from_value(&'ctx self) -> ValueKind<'ctx> {
        ValueKind::String((*self).clone())
    }
}

impl<'ctx> Downcast<'ctx> for () {
    fn from_value(value: Value<'ctx>) -> Option<&'ctx Self> {
        value.as_none()
    }
}

impl<'ctx> Upcast<'ctx> for () {
    fn from_value(&'ctx self) -> ValueKind<'ctx> {
        ValueKind::None
    }
}

#[derive(Debug, Clone)]
pub struct ValueContainer<'ctx> {
    pub kind: ValueKind<'ctx>,
}

impl<'ctx> ValueContainer<'ctx> {
    pub fn new(kind: ValueKind<'ctx>, arena: &'ctx Bump) -> Value<'ctx> {
        arena.alloc(ValueContainer { kind })
    }

    pub fn clone_in_arena(&self, arena: &'ctx Bump) -> Value<'ctx> {
        let new_kind = match &self.kind {
            ValueKind::Int(i) => ValueKind::Int(*i),
            ValueKind::Float(f) => ValueKind::Float(*f),
            ValueKind::Object(o) => ValueKind::Object(o.clone()),
            ValueKind::ErrorWrapped(e) => ValueKind::ErrorWrapped(e.clone()),
            ValueKind::Bool(b) => ValueKind::Bool(*b),
            ValueKind::String(s) => ValueKind::String(s.clone()),
            ValueKind::None => ValueKind::None,
            ValueKind::Collection(c) => ValueKind::Collection(c.clone()),
        };
        ValueContainer::new(new_kind, arena)
    }

    pub fn is_truthy(&self) -> bool {
        match &self.kind {
            ValueKind::Int(i) => i.value != 0,
            ValueKind::Float(f) => f.value != 0.0,
            ValueKind::Bool(b) => *b,
            ValueKind::None => false,
            ValueKind::String(s) => !s.is_empty(),
            ValueKind::Collection(c) => !c.borrow().is_empty(),
            ValueKind::Object(_) => true,
            ValueKind::ErrorWrapped(_) => true,
        }
    }

    pub fn as_int(&self) -> Option<&VirInt> {
        match &self.kind {
            ValueKind::Int(i) => Some(i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<&VirFloat> {
        match &self.kind {
            ValueKind::Float(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&VirObject<'ctx>> {
        match &self.kind {
            ValueKind::Object(o) => Some(o),
            _ => None,
        }
    }

    pub fn as_error(&self) -> Option<&SandboxExecutionError> {
        match &self.kind {
            ValueKind::ErrorWrapped(e) => Some(e),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<&bool> {
        match &self.kind {
            ValueKind::Bool(e) => Some(e),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        match &self.kind {
            ValueKind::String(e) => Some(e),
            _ => None,
        }
    }

    pub fn as_none(&self) -> Option<&()> {
        match &self.kind {
            ValueKind::None => Some(&()),
            _ => None,
        }
    }

    pub fn as_collection(&self) -> Option<&Rc<RefCell<Vec<Value<'ctx>>>>> {
        match &self.kind {
            ValueKind::Collection(e) => Some(e),
            _ => None,
        }
    }
}
