use crate::base::{Downcast, Upcast, Value, ValueKind};
use crate::error::Result;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Add, Div, Mul, Rem, Sub};
use std::rc::Rc;

#[derive(Debug, Clone, Copy)]
pub struct VirInt {
    pub value: i64,
}
impl VirInt {
    pub fn new(value: i64) -> Self {
        Self { value }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VirFloat {
    pub value: f64,
}
impl VirFloat {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

#[derive(Debug, Clone)]
pub struct Mapping<'ctx> {
    pub mapping: HashMap<String, Rc<RefCell<Value<'ctx>>>>,
}

#[derive(Debug, Clone)]
pub struct VirObject<'ctx> {
    pub mapping: Rc<RefCell<Mapping<'ctx>>>,
}

impl<'ctx> VirObject<'ctx> {
    pub fn new() -> Self {
        Self {
            mapping: Rc::new(RefCell::new(Mapping {
                mapping: HashMap::new(),
            })),
        }
    }
    pub fn get(&self, key: &str) -> Option<Rc<RefCell<Value<'ctx>>>> {
        self.mapping.borrow().mapping.get(key).cloned()
    }
    pub fn set(&self, key: String, value: Value<'ctx>) {
        let value_cell = Rc::new(RefCell::new(value));
        self.mapping.borrow_mut().mapping.insert(key, value_cell);
    }
    pub fn clone(&self) -> Self {
        Self {
            mapping: Rc::clone(&self.mapping),
        }
    }
}

impl<'ctx> Default for VirObject<'ctx> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'ctx> Downcast<'ctx> for VirInt {
    fn from_value(value: Value<'ctx>) -> Option<&'ctx Self> {
        value.as_int()
    }
}

impl<'ctx> Downcast<'ctx> for VirFloat {
    fn from_value(value: Value<'ctx>) -> Option<&'ctx Self> {
        value.as_float()
    }
}

impl<'ctx> Upcast<'ctx> for VirInt {
    fn from_value(&'ctx self) -> ValueKind<'ctx> {
        ValueKind::Int(*self)
    }
}

impl<'ctx> Upcast<'ctx> for VirFloat {
    fn from_value(&'ctx self) -> ValueKind<'ctx> {
        ValueKind::Float(*self)
    }
}

impl Add for VirInt {
    type Output = Result<Self>;
    fn add(self, rhs: Self) -> Self::Output {
        Ok(Self::new(self.value + rhs.value))
    }
}
impl Add for VirFloat {
    type Output = Result<Self>;
    fn add(self, rhs: Self) -> Self::Output {
        Ok(Self::new(self.value + rhs.value))
    }
}
impl Add<VirInt> for VirFloat {
    type Output = Result<Self>;
    fn add(self, rhs: VirInt) -> Self::Output {
        Ok(Self::new(self.value + (rhs.value as f64)))
    }
}
impl Add<VirFloat> for VirInt {
    type Output = Result<VirFloat>;
    fn add(self, rhs: VirFloat) -> Self::Output {
        Ok(VirFloat::new(self.value as f64 + rhs.value))
    }
}

impl Sub for VirInt {
    type Output = Result<Self>;
    fn sub(self, rhs: Self) -> Self::Output {
        Ok(Self::new(self.value - rhs.value))
    }
}
impl Sub for VirFloat {
    type Output = Result<Self>;
    fn sub(self, rhs: Self) -> Self::Output {
        Ok(Self::new(self.value - rhs.value))
    }
}
impl Sub<VirInt> for VirFloat {
    type Output = Result<Self>;
    fn sub(self, rhs: VirInt) -> Self::Output {
        Ok(Self::new(self.value - (rhs.value as f64)))
    }
}
impl Sub<VirFloat> for VirInt {
    type Output = Result<VirFloat>;
    fn sub(self, rhs: VirFloat) -> Self::Output {
        Ok(VirFloat::new(self.value as f64 - rhs.value))
    }
}

impl Mul for VirInt {
    type Output = Result<Self>;
    fn mul(self, rhs: Self) -> Self::Output {
        Ok(Self::new(self.value * rhs.value))
    }
}
impl Mul for VirFloat {
    type Output = Result<Self>;
    fn mul(self, rhs: Self) -> Self::Output {
        Ok(Self::new(self.value * rhs.value))
    }
}
impl Mul<VirInt> for VirFloat {
    type Output = Result<Self>;
    fn mul(self, rhs: VirInt) -> Self::Output {
        Ok(Self::new(self.value * (rhs.value as f64)))
    }
}
impl Mul<VirFloat> for VirInt {
    type Output = Result<VirFloat>;
    fn mul(self, rhs: VirFloat) -> Self::Output {
        Ok(VirFloat::new(self.value as f64 * rhs.value))
    }
}

impl Div for VirInt {
    type Output = Result<VirFloat>;
    fn div(self, rhs: Self) -> Self::Output {
        if rhs.value == 0 {
            return Err(crate::error::SandboxExecutionError::DivideByZeroError);
        }
        Ok(VirFloat::new((self.value as f64) / (rhs.value as f64)))
    }
}
impl Div for VirFloat {
    type Output = Result<Self>;
    fn div(self, rhs: Self) -> Self::Output {
        if rhs.value == 0f64 {
            return Err(crate::error::SandboxExecutionError::DivideByZeroError);
        }
        Ok(Self::new(self.value / rhs.value))
    }
}
impl Div<VirInt> for VirFloat {
    type Output = Result<Self>;
    fn div(self, rhs: VirInt) -> Self::Output {
        if rhs.value == 0 {
            return Err(crate::error::SandboxExecutionError::DivideByZeroError);
        }
        Ok(Self::new(self.value / (rhs.value as f64)))
    }
}
impl Div<VirFloat> for VirInt {
    type Output = Result<VirFloat>;
    fn div(self, rhs: VirFloat) -> Self::Output {
        if rhs.value == 0f64 {
            return Err(crate::error::SandboxExecutionError::DivideByZeroError);
        }
        Ok(VirFloat::new(self.value as f64 / rhs.value))
    }
}

impl Rem for VirInt {
    type Output = Result<VirInt>;
    fn rem(self, rhs: Self) -> Self::Output {
        if rhs.value == 0 {
            return Err(crate::error::SandboxExecutionError::DivideByZeroError);
        }
        let mut v = (self.value) % (rhs.value);
        if v < 0 {
            v += rhs.value;
        }
        Ok(VirInt::new(v))
    }
}
impl Rem for VirFloat {
    type Output = Result<VirFloat>;
    fn rem(self, rhs: Self) -> Self::Output {
        if rhs.value == 0f64 {
            return Err(crate::error::SandboxExecutionError::DivideByZeroError);
        }
        let mut v = (self.value) % (rhs.value);
        if v < 0f64 {
            v += rhs.value;
        }
        Ok(VirFloat::new(v))
    }
}
impl Rem<VirInt> for VirFloat {
    type Output = Result<VirFloat>;
    fn rem(self, rhs: VirInt) -> Self::Output {
        if rhs.value == 0 {
            return Err(crate::error::SandboxExecutionError::DivideByZeroError);
        }
        let mut v = (self.value) % (rhs.value as f64);
        if v < 0f64 {
            v += rhs.value as f64;
        }
        Ok(VirFloat::new(v))
    }
}
impl Rem<VirFloat> for VirInt {
    type Output = Result<VirFloat>;
    fn rem(self, rhs: VirFloat) -> Self::Output {
        if rhs.value == 0f64 {
            return Err(crate::error::SandboxExecutionError::DivideByZeroError);
        }
        let mut v = (self.value as f64) % (rhs.value);
        if v < 0f64 {
            v += rhs.value;
        }
        Ok(VirFloat::new(v))
    }
}
