use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock};

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

pub struct ValuePtr<'a>(Arc<Mutex<Value<'a>>>);

impl<'a> Deref for ValuePtr<'a> {
    type Target = Arc<Mutex<Value<'a>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
