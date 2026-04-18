use std::cell::RefCell;
use std::collections::HashMap;
use crate::builtin::{Mapping, VirPyFloat, VirPyInt, VirPyObject};
use crate::error::SandboxExecutionError;
use bumpalo::Bump;
use std::fmt::Debug;
use std::rc::Rc;

pub type Value<'ctx> = &'ctx ValueContainer<'ctx>;

#[derive(Debug, Clone)]
pub enum ValueKind<'ctx> {
    Int(VirPyInt),
    Float(VirPyFloat),
    Object(VirPyObject<'ctx>),
    ErrorWrapped(SandboxExecutionError),
    Bool(bool),
    String(String),
    Collection(Vec<Value<'ctx>>),
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

impl<'ctx> Downcast<'ctx> for Vec<Value<'ctx>> {
    fn from_value(value: Value<'ctx>) -> Option<&'ctx Self> {
        value.as_collection()
    }
}

impl<'ctx> Upcast<'ctx> for Vec<Value<'ctx>> {
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

#[derive(Debug)]
pub struct ValueContainer<'ctx> {
    pub kind: ValueKind<'ctx>,
}

impl<'ctx> ValueContainer<'ctx> {
    pub fn new(kind: ValueKind<'ctx>, arena: &'ctx Bump) -> Value<'ctx> {
        arena.alloc(ValueContainer { kind })
    }

    pub fn new_static(kind: ValueKind<'static>, arena: &'static Bump) -> Value<'static> {
        arena.alloc(ValueContainer { kind })
    }

    pub fn export(&self) -> ValueContainer<'static> {
        Self {
            kind: match &self.kind {
                ValueKind::Int(i) => ValueKind::Int(i.clone()),
                ValueKind::Float(f) => ValueKind::Float(f.clone()),
                ValueKind::Object(o) => {
                    let mapping = o.mapping.borrow().clone();
                    let inner = mapping.mapping.clone();
                    let map: HashMap<String, Rc<RefCell<Value<'static>>>> = inner.iter().map(
                        |(k, v)|
                            (
                                k.clone(),
                                {
                                    let container= v.borrow();
                                    let container = container.export().kind;
                                    {
                                        use std::borrow::Borrow;
                                        Rc::new(RefCell::new(ValueContainer::<'static>::new_static(container, &static_bumpalo.borrow().lock().unwrap())))
                                    }
                                }
                            )
                    ).collect();
                    ValueKind::Object(VirPyObject {mapping: Rc::new(RefCell::new(Mapping { mapping: map}))})
                },
                ValueKind::ErrorWrapped(e) => ValueKind::ErrorWrapped(e.clone()),
                ValueKind::Bool(b) => ValueKind::Bool(*b),
                ValueKind::String(s) => ValueKind::String(s.clone()),
                ValueKind::None => ValueKind::None,
                ValueKind::Collection(c) => {
                    {
                        use std::borrow::Borrow;
                        ValueKind::Collection(c.iter().map(|v| ValueContainer::<'static>::new_static(v.export().kind, &static_bumpalo.lock().unwrap())).collect())
                    }
                }
            }
        }
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

    pub fn as_int(&self) -> Option<&VirPyInt> {
        match &self.kind {
            ValueKind::Int(i) => Some(i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<&VirPyFloat> {
        match &self.kind {
            ValueKind::Float(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&VirPyObject<'ctx>> {
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

    pub fn as_collection(&self) -> Option<&Vec<Value<'ctx>>> {
        match &self.kind {
            ValueKind::Collection(e) => Some(e),
            _ => None,
        }
    }
}
