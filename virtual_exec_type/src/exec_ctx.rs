use crate::base::{Value, ValueContainer, ValueKind};
use crate::builtin::Mapping;
use crate::error::SandboxExecutionError;
use bumpalo::Bump;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::alloc::Allocator;
use crate::exec_ctx::RsValue::Vector;

pub type Result<T> = core::result::Result<T, SandboxExecutionError>;

#[derive(Debug, Clone, PartialEq)]
pub enum RsValue {
    Int(i64),
    Float(f64),
    Object(HashMap<String, RsValue>),
    Bool(bool),
    String(String),
    Vector(Vec<RsValue>),
    None,
}

fn value_kind_to_rs_value(kind: &ValueKind) -> RsValue {
    match kind {
        ValueKind::Int(i) => RsValue::Int(i.value),
        ValueKind::Float(f) => RsValue::Float(f.value),
        ValueKind::Bool(b) => RsValue::Bool(*b),
        ValueKind::String(s) => RsValue::String(s.clone()),
        ValueKind::None => RsValue::None,
        ValueKind::Object(o) => {
            let mut map = HashMap::new();
            for (key, value_rc) in o.mapping.borrow().mapping.iter() {
                let value_ref = value_rc.borrow();
                map.insert(key.clone(), value_kind_to_rs_value(&value_ref.kind));
            }
            RsValue::Object(map)
        }
        // Errors are not representable as a PyValue and are skipped.
        ValueKind::ErrorWrapped(_) => RsValue::None,
        ValueKind::Collection(v) => {
            let mut vec = Vec::new();
            for value in v.borrow().clone() {
                vec.push(value_kind_to_rs_value(&value.kind));
            }
            RsValue::Vector(vec)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionContext<'ctx> {
    pub arena: Allocator<'ctx>,
    pub ttl: i64,
    pub mapping: Vec<Rc<RefCell<Mapping<'ctx>>>>, // Top layer ([0]): most local scope
}

// By implementing RefUnwindSafe, we are asserting that even if a panic
// occurs during a method call that mutates ExecutionContext through a
// shared reference, the context is left in a state that won't cause
// undefined behavior *for the caller*. This is acceptable for our interpreter
// because the API contract of Module::eval is that if a panic is caught,
// the entire ExecutionContext must be discarded by the caller.
impl<'ctx> std::panic::RefUnwindSafe for ExecutionContext<'ctx> {}

impl<'ctx> ExecutionContext<'ctx> {
    pub fn new(
        arena: Allocator<'ctx>,
        ttl: i64,
        mapping: Vec<Rc<RefCell<Mapping<'ctx>>>>,
    ) -> Self {
        Self {
            arena,
            ttl,
            mapping,
        }
    }

    pub fn to_hashmap(&self) -> HashMap<String, RsValue> {
        let mut dict = HashMap::new();
        for scope_rc in self.mapping.iter().rev() {
            let scope = scope_rc.borrow();
            for (key, value_rc) in scope.mapping.iter() {
                let value_ref = value_rc.borrow();
                dict.insert(key.clone(), value_kind_to_rs_value(&value_ref.kind));
            }
        }
        dict
    }

    pub fn consume_one(&mut self) -> Result<()> {
        self.consume(1)
    }

    pub fn consume(&mut self, amount: i64) -> Result<()> {
        if amount > self.ttl {
            return Err(SandboxExecutionError::TimeoutError);
        }
        self.ttl -= amount;
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<Rc<RefCell<Value<'ctx>>>> {
        let mut r: Option<Rc<RefCell<Value<'ctx>>>> = None;
        for mapping in self.mapping.clone() {
            if mapping.borrow().mapping.contains_key(name) {
                r = Some(mapping.borrow().mapping.get(name).unwrap().clone());
                break; // Found in the most local scope, stop searching
            }
        }
        match r {
            Some(v) => Ok(v),
            None => Err(SandboxExecutionError::ReferenceNotExistError(
                name.to_string(),
            )),
        }
    }

    pub fn get_ignore_missing(
        &mut self,
        name: &str,
        value: Value<'ctx>,
    ) -> Result<Rc<RefCell<Value<'ctx>>>> {
        for mapping in &self.mapping {
            if mapping.borrow().mapping.contains_key(name) {
                let r = mapping.borrow().mapping.get(name).unwrap().clone();
                r.replace(value);
                return Ok(r);
            }
        }

        if self.mapping.is_empty() {
            return Err(SandboxExecutionError::ReferenceNotExistError(
                name.to_string(),
            ));
        }

        let new_value = Rc::new(RefCell::new(value));
        self.mapping[0]
            .borrow_mut()
            .mapping
            .insert(name.to_string(), new_value.clone());
        Ok(new_value)
    }
}
