use std::cell::RefCell;
use std::rc::Rc;
use bumpalo::Bump;
use crate::base::{Value, ValueContainer, ValueKind};

#[derive(Clone, Debug)]
pub struct Allocator<'ctx> {
    pub arena: Rc<RefCell<&'ctx Bump>>,
}

impl<'ctx> Allocator<'ctx> {
    pub fn new(arena: &'ctx Bump) -> Self {
        Self { arena: Rc::new(RefCell::new(arena)) }
    }

    pub fn allocate(&self, kind: ValueKind<'ctx>) -> Value<'ctx> {
        ValueContainer::new(kind, *self.arena.borrow())
    }
}

pub fn create_arena(size: Option<usize>) -> Bump {
    match size {
        Some(size) => Bump::with_capacity(size),
        None => Bump::new(),
    }
}