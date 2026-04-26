use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use virtual_exec_type::mem::{Allocator, MemoryAllocator, Value, ValuePtr};
use virtual_exec_type::op::*;
use crate::sequential::instructions::Instruction;
use virtual_exec_type::base::{IsTruhy, TypeCast};
use virtual_exec_type::error::MemoryError;

type AttrReference<'ctx> = (Option<ValuePtr<'ctx>>, String);
type IdxReference<'ctx> = (ValuePtr<'ctx>, i64);

impl<'ctx> From<AttrReference<'ctx>> for StackItem<'ctx> {
    fn from(value: AttrReference<'ctx>) -> Self {
        StackItem::AttrReference(value)
    }
}

impl<'ctx> From<IdxReference<'ctx>> for StackItem<'ctx> {
    fn from(value: IdxReference<'ctx>) -> Self {
        StackItem::IdxReference(value)
    }
}

#[derive(Debug, Clone)]
pub enum StackItem<'ctx> {
    Value(ValuePtr<'ctx>),
    AttrReference(AttrReference<'ctx>),
    IdxReference(IdxReference<'ctx>)
}

impl<'ctx> From<ValuePtr<'ctx>> for StackItem<'ctx> {
    fn from(value: ValuePtr<'ctx>) -> Self {
        StackItem::Value(value)
    }
}

pub struct FnStackFrame<'ctx> {
    pub ptr: u64,
    pub mapping: Arc<RwLock<HashMap<String, ValuePtr<'ctx>>>>,
}

impl From<MemoryError> for SandboxExecutionError {
    fn from(value: MemoryError) -> Self {
        SandboxExecutionError::MemoryError
    }
}

#[derive(Clone, Debug)]
pub enum SandboxExecutionError {
    ReferenceNotExistError(String),
    DivideByZeroError,
    FnStackUnderflowError,
    VStackUnderflowError,
    UndefinedOperendError,
    AttrNotStringError,
    RefNameMissingError,
    UndefinedVarError,
    AttrMisuseError,
    UnexpectedAttrError,
    UnexpectedIdxError,
    IndexOutOfRangeError,
    MemoryError
}


pub struct InstStateMachine<'ctx> {
    pub lim: u64,
    pub fn_stack_frame: Vec<FnStackFrame<'ctx>>,
    pub alloc: MemoryAllocator<'ctx>,
    pub instructions: Vec<Instruction>,
    pub state: Result<State, SandboxExecutionError>,
    pub stack: Vec<StackItem<'ctx>>,
}

#[derive(Debug, Clone)]
pub enum State {
    Ok,
    Terminated,
    Interrupt,
    Timeout
}

macro_rules! __binary_autogen {
    ($f:ident, $ss:ident) => {
        {
            let b = $ss.pop_get()?;
            let a = $ss.pop_get()?;
            let result = $f(a, b, &$ss.alloc).map_err(|_| SandboxExecutionError::UndefinedOperendError)?;
            $ss.push_value(result);
        }
    };
}


macro_rules! __unary_autogen {
    ($f:ident, $ss:ident) => {
        {
            let a = $ss.pop_get()?;
            let result = $f(a, &$ss.alloc).map_err(|_| SandboxExecutionError::UndefinedOperendError)?;
            $ss.push_value(result);
        }
    };
}

impl<'ctx> InstStateMachine<'ctx> {
    fn pop_value(&mut self) -> Result<ValuePtr<'ctx>, SandboxExecutionError> {
        let result = self.stack.pop().ok_or(SandboxExecutionError::VStackUnderflowError)?;
        match result {
            StackItem::Value(value) => Ok(value),
            StackItem::AttrReference(_) | StackItem::IdxReference(_) => {
                Err(SandboxExecutionError::UndefinedVarError)
            }
        }
    }

    fn push_value(&mut self, value: ValuePtr<'ctx>) {
        self.stack.push(value.into());
    }

    fn pop_ref(&mut self) -> Result<AttrReference<'ctx>, SandboxExecutionError> {
        let result = self.stack.pop().ok_or(SandboxExecutionError::VStackUnderflowError)?;
        match result {
            StackItem::Value(_) | StackItem::IdxReference(_) => {
                Err(SandboxExecutionError::AttrMisuseError)
            },
            StackItem::AttrReference(reference) => Ok(reference)

        }
    }

    fn push_ref(&mut self, reference: AttrReference<'ctx>) {
        self.stack.push(reference.into());
    }

    fn pop_idx_ref(&mut self) -> Result<IdxReference<'ctx>, SandboxExecutionError> {
        let result = self.stack.pop().ok_or(SandboxExecutionError::VStackUnderflowError)?;
        match result {
            StackItem::Value(_) | StackItem::AttrReference(_) => {
                Err(SandboxExecutionError::AttrMisuseError)
            },
            StackItem::IdxReference(reference) => Ok(reference)
        }
    }

    fn push_idx_ref(&mut self, reference: IdxReference<'ctx>) {
        self.stack.push(reference.into());
    }

    fn pop(&mut self) -> Result<StackItem<'ctx>, SandboxExecutionError> {
        self.stack.pop().ok_or(SandboxExecutionError::VStackUnderflowError)
    }

    fn resolve(&self, name: &str) -> Result<ValuePtr<'ctx>, SandboxExecutionError> {
        for frame in self.fn_stack_frame.iter().rev() {
            if let Some(val) = frame.mapping.read().unwrap().get(name) {
                return Ok(val.clone());
            }
        }
        
        Err(SandboxExecutionError::ReferenceNotExistError(name.to_string()))
    }

    fn pop_get(&mut self) -> Result<ValuePtr<'ctx>, SandboxExecutionError> {
        let result = {
            self.pop()?
        };

        match result {
            StackItem::Value(v) => Ok(v),
            StackItem::AttrReference(target) => {
                match target.0 {
                    Some(obj) => {
                        if let Some(o) = obj.as_object() {
                            Ok(o.read().unwrap().get(&target.1).ok_or_else(|| SandboxExecutionError::ReferenceNotExistError(target.1))?.clone())
                        } else {
                            Err(SandboxExecutionError::UnexpectedAttrError)
                        }
                    },
                    None => {
                        Ok(self.resolve(&target.1)?.clone())
                    }
                }
            },
            StackItem::IdxReference(target) => {
                if let Some(arr)= target.0.as_collections() {
                    let mut idx = target.1;
                    if idx < 0 {
                        idx += arr.read().unwrap().len() as i64;
                    }
                    if idx >= 0 && (idx as usize) < arr.read().unwrap().len() {
                        Ok(arr.read().unwrap()[idx as usize].clone())
                    } else {
                        Err(SandboxExecutionError::IndexOutOfRangeError)
                    }
                } else {
                    Err(SandboxExecutionError::UnexpectedIdxError)
                }
            }
        }
    }

    fn get_mut_stack_ref<'a>(&'a mut self) -> Result<&'a mut FnStackFrame<'ctx>, SandboxExecutionError> {
        self.fn_stack_frame.last_mut().ok_or(SandboxExecutionError::FnStackUnderflowError)
    }

    fn get_stack_ref<'a>(&'a self) -> Result<&'a FnStackFrame<'ctx>, SandboxExecutionError> {
        self.fn_stack_frame.last().ok_or(SandboxExecutionError::FnStackUnderflowError)
    }
    
    fn alloc(&self, data: Value<'ctx>) -> Result<ValuePtr<'ctx>, SandboxExecutionError> {
        self.alloc.alloc(data).map_err(|e| e.into())
    }

    pub fn run_once(&mut self) -> Result<State, SandboxExecutionError> {
        match self.state.clone() {
            Ok(State::Terminated) => {
                return Ok(State::Terminated)
            },
            Ok(State::Interrupt) => {
                return Ok(State::Interrupt)
            },
            Err(err) => {
                return Err(err)
            },
            Ok(State::Ok) | Ok(State::Timeout) => {}
        };
        let instruction;
        {
            let stack =  self.get_stack_ref()?;
            let ptr = stack.ptr as usize;
            instruction = self.instructions[ptr].clone();
            let stack =  self.get_mut_stack_ref()?;
            stack.ptr += 1;
        }
        if self.lim == 0 {
            self.state = Ok(State::Timeout);
            return self.state.clone()
        } else {
            self.state = Ok(State::Ok)
        }
        self.lim -= 1;
        match instruction {
            Instruction::Add => { __binary_autogen!(err_op_add, self); },
            Instruction::Sub => { __binary_autogen!(err_op_sub, self); },
            Instruction::Mul => { __binary_autogen!(err_op_mul, self); },
            Instruction::Div => { __binary_autogen!(err_op_div, self); },
            Instruction::Mod => { __binary_autogen!(err_op_moduls, self); },
            Instruction::BitwiseAnd => { __binary_autogen!(err_op_band, self); },
            Instruction::BitwiseOr => { __binary_autogen!(err_op_bor, self); },
            Instruction::BitwiseXor => { __binary_autogen!(err_op_bxor, self); },
            Instruction::Shl => { __binary_autogen!(err_op_bsl, self); },
            Instruction::Shr => { __binary_autogen!(err_op_bsr, self); },
            Instruction::UnaryPlus => { __unary_autogen!(err_op_pos, self); },
            Instruction::UnaryMinus => { __unary_autogen!(err_op_neg, self); }
            Instruction::Not => { __unary_autogen!(err_op_not, self); },
            Instruction::BitwiseNot => { __unary_autogen!(err_op_bnot, self); }
            Instruction::Eq => { __binary_autogen!(err_op_eq, self); },
            Instruction::NotEq => { __binary_autogen!(err_op_ne, self); }
            Instruction::Lt => { __binary_autogen!(err_op_lt, self); },
            Instruction::Lte => { __binary_autogen!(err_op_le, self); },
            Instruction::Gt => { __binary_autogen!(err_op_gt, self); },
            Instruction::Gte => { __binary_autogen!(err_op_ge, self); },
            Instruction::Assign => {
                let value = self.pop_value()?;
                let target = self.pop()?;
                match target {
                    StackItem::Value(value) => {
                        self.state = Err(SandboxExecutionError::UndefinedVarError);
                        return self.state.clone()
                    },
                    StackItem::AttrReference((None, target)) => {
                        let stack =  self.get_mut_stack_ref()?;
                        stack.mapping.write().unwrap().insert(target, value.clone());
                    },
                    StackItem::AttrReference((Some(value), target)) => {
                        if let Some(obj) = value.as_object() {
                            obj.write().unwrap().insert(target, value.clone());
                        } else {
                            self.state = Err(SandboxExecutionError::UnexpectedAttrError);
                            return self.state.clone()
                        }
                    }
                    StackItem::IdxReference(target) => {
                        if let Some(arr) = target.0.as_collections() {
                            let mut idx = target.1;
                            if idx < 0 {
                                idx += arr.read().unwrap().len() as i64
                            }
                            if idx >= 0 && (idx as usize) < arr.read().unwrap().len() {
                                arr.write().unwrap()[idx as usize] = value;
                            }
                            else {
                                self.state = Err(SandboxExecutionError::IndexOutOfRangeError);
                                return self.state.clone()
                            }
                        }
                    }
                }
            }
            Instruction::JmpNz(loc) => {
                let a = self.pop_value()?;
                if a.is_truthy() {
                    let stack =  self.get_mut_stack_ref()?;
                    stack.ptr = loc;
                }
            }
            Instruction::JmpZ(loc) => {
                let a = self.pop_value()?;
                if !a.is_truthy() {
                    let stack =  self.get_mut_stack_ref()?;
                    stack.ptr = loc;
                }
            }
            Instruction::Jmp(loc) => {
                let stack =  self.get_mut_stack_ref()?;
                stack.ptr = loc;
            }
            Instruction::Call => {
                todo!("Function not exist yet")
            }
            Instruction::Ret => {
                todo!("Function not exist yet")
            }
            Instruction::LoadNone => {
                self.push_value(self.alloc(Value::None)?);
            }
            Instruction::LoadLitFloat(v) => {
                self.push_value(self.alloc(Value::Float(v))?);
            }
            Instruction::LoadLitInt(v) => {
                self.push_value(self.alloc(Value::Int(v))?);
            }
            Instruction::LoadLitString(v) => {
                self.push_value(self.alloc(Value::String(v))?);
            }
            Instruction::LoadLitBool(v) => {
                self.push_value(self.alloc(Value::Bool(v))?);
            }
            Instruction::ConstructArr(len) => {
                let mut arr = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    arr.push(self.pop_value()?);
                }
                self.push_value(self.alloc(Value::Collection(Arc::new(RwLock::new(arr))))?);
            }
            Instruction::ConstructObj(len2) => {
                let mut obj = HashMap::new();
                for idx in 0..len2 {
                    let name = self.pop_value()?;
                    let value = self.pop_value()?;
                    if name.as_string().is_none() {
                        let remaining_stackdrop = (len2 - idx) * 2;
                        for _ in 0..remaining_stackdrop {
                            let _ = self.pop_value(); // Drop error since AttrNotStringError is the primary issue, although otherwise this would cause error as well for stack underflow
                        }
                        self.state = Err(SandboxExecutionError::AttrNotStringError);
                        return self.state.clone()
                    }
                    obj.insert(name.as_string().unwrap().clone(), value);
                }
                self.push_value(self.alloc(Value::Object(Arc::new(RwLock::new(obj))))?);
            }
            Instruction::LoadName(name) => {
                self.push_ref((None, name.into_string()));
            }
            Instruction::LoadObjectAttr(name) => {
                let value = self.pop_value()?;
                if let Some(_) = value.as_object() {
                    self.push_ref((Some(value), name.into_string()));
                }
                else {
                    self.state = Err(SandboxExecutionError::UnexpectedAttrError);
                    return self.state.clone()
                }

            }
            Instruction::LoadObjectIndex(idx) => {
                let value = self.pop_value()?;
                if let Some(_) = value.as_collections() {
                    self.push_idx_ref((value, idx));
                }
                else {
                    self.state = Err(SandboxExecutionError::UnexpectedIdxError);
                    return self.state.clone()
                }
            }
            Instruction::Terminate => {
                self.state = Ok(State::Terminated);
                return self.state.clone()
            }
            Instruction::Interrupt => {
                self.state = Ok(State::Interrupt);
                return self.state.clone()
            }
            Instruction::Pop => {
                self.pop_value()?;
            }
        }
        Ok(State::Ok)
    }
}
