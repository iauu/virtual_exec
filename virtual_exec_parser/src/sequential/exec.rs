use std::cell::RefCell;
use std::rc::Rc;
use virtual_exec_type::alloc::Allocator;
use virtual_exec_type::base::{Value, ValueContainer, ValueKind};
use virtual_exec_type::builtin::{Mapping, VirPyFloat, VirPyInt, VirPyObject};
use virtual_exec_type::op::*;
use crate::sequential::instructions::Instruction;

type AttrReference<'ctx> = (Option<Value<'ctx>>, String);
type IdxReference<'ctx> = (Value<'ctx>, i64);

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
    Value(Value<'ctx>),
    AttrReference(AttrReference<'ctx>),
    IdxReference(IdxReference<'ctx>)
}

impl<'ctx> From<Value<'ctx>> for StackItem<'ctx> {
    fn from(value: Value<'ctx>) -> Self {
        StackItem::Value(value)
    }
}

pub struct FnStackFrame<'ctx> {
    pub ptr: u64,
    pub stack: Vec<StackItem<'ctx>>,
    pub mapping: Rc<RefCell<Mapping<'ctx>>>,
}

impl<'ctx> FnStackFrame<'ctx> {
    pub fn pop_value(&mut self) -> Result<Value<'ctx>, SandboxExecutionError> {
        let result = self.stack.pop().ok_or(SandboxExecutionError::VStackUnderflowError)?;
        match result {
            StackItem::Value(value) => Ok(value),
            StackItem::AttrReference(_) | StackItem::IdxReference(_) => {
                Err(SandboxExecutionError::UndefinedVarError)
            }
        }
    }

    pub fn push_value(&mut self, value: Value<'ctx>) {
        self.stack.push(value.into());
    }

    pub fn pop_ref(&mut self) -> Result<AttrReference<'ctx>, SandboxExecutionError> {
        let result = self.stack.pop().ok_or(SandboxExecutionError::VStackUnderflowError)?;
        match result {
            StackItem::Value(_) | StackItem::IdxReference(_) => {
                Err(SandboxExecutionError::AttrMisuseError)
            },
            StackItem::AttrReference(reference) => Ok(reference)

        }
    }

    pub fn push_ref(&mut self, reference: AttrReference<'ctx>) {
        self.stack.push(reference.into());
    }

    pub fn pop_idx_ref(&mut self) -> Result<IdxReference<'ctx>, SandboxExecutionError> {
        let result = self.stack.pop().ok_or(SandboxExecutionError::VStackUnderflowError)?;
        match result {
            StackItem::Value(_) | StackItem::AttrReference(_) => {
                Err(SandboxExecutionError::AttrMisuseError)
            },
            StackItem::IdxReference(reference) => Ok(reference)
        }
    }

    pub fn push_idx_ref(&mut self, reference: IdxReference<'ctx>) {
        self.stack.push(reference.into());
    }

    pub fn pop(&mut self) -> Result<StackItem<'ctx>, SandboxExecutionError> {
        self.stack.pop().ok_or(SandboxExecutionError::VStackUnderflowError)
    }
}


#[derive(Clone, Debug)]
pub enum SandboxExecutionError {
    TimeoutError,
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
    IndexOutOfRangeError
}


pub struct InstStateMachine<'ctx> {
    pub lim: u64,
    pub fn_stack_frame: Vec<FnStackFrame<'ctx>>,
    pub alloc: Allocator<'ctx>,
    pub instructions: Vec<Instruction>,
    pub state: Result<State, SandboxExecutionError>
}

#[derive(Debug, Clone)]
pub enum State {
    Ok,
    Terminated,
    Interrupt,
}

macro_rules! __binary_autogen {
    ($f:ident, $ss:ident) => {
        {
            let b = $ss.pop_get()?;
            let a = $ss.pop_get()?;
            let result = $f(a, b, &$ss.alloc).ok_or(SandboxExecutionError::UndefinedOperendError)?;
            $ss.fn_stack_frame.last_mut().unwrap().push_value(result);
        }
    };
}


macro_rules! __unary_autogen {
    ($f:ident, $ss:ident) => {
        {
            let a = $ss.pop_get()?;
            let result = $f(a, &$ss.alloc).ok_or(SandboxExecutionError::UndefinedOperendError)?;
            $ss.fn_stack_frame.last_mut().unwrap().push_value(result);
        }
    };
}

impl<'ctx> InstStateMachine<'ctx> {
    fn resolve(&self, name: &str) -> Result<Rc<RefCell<Value<'ctx>>>, SandboxExecutionError> {
        for frame in self.fn_stack_frame.iter().rev() {
            if let Some(val) = frame.mapping.borrow().mapping.get(name) {
                return Ok(Rc::clone(val));
            }
        }
        
        Err(SandboxExecutionError::ReferenceNotExistError(name.to_string()))
    }

    fn pop_get(&mut self) -> Result<Value<'ctx>, SandboxExecutionError> {
        let result = {
            let frame = self.fn_stack_frame.last_mut().ok_or(SandboxExecutionError::FnStackUnderflowError)?;
            frame.pop()?
        };

        match result {
            StackItem::Value(v) => Ok(v),
            StackItem::AttrReference(target) => {
                match target.0 {
                    Some(obj) => {
                        if let Some(o) = obj.as_object() {
                            Ok(*o.get(&target.1).ok_or_else(|| SandboxExecutionError::ReferenceNotExistError(target.1))?.borrow_mut())
                        } else {
                            Err(SandboxExecutionError::UnexpectedAttrError)
                        }
                    },
                    None => {
                        Ok(*self.resolve(&target.1)?.borrow())
                    }
                }
            },
            StackItem::IdxReference(target) => {
                if let Some(arr) = target.0.as_collection() {
                    let mut idx = target.1;
                    if idx < 0 {
                        idx += arr.borrow().len() as i64;
                    }
                    if idx >= 0 && (idx as usize) < arr.borrow().len() {
                        Ok(arr.borrow()[idx as usize])
                    } else {
                        Err(SandboxExecutionError::IndexOutOfRangeError)
                    }
                } else {
                    Err(SandboxExecutionError::UnexpectedIdxError)
                }
            }
        }
    }

    pub fn run_once(&mut self) -> Result<State, SandboxExecutionError> {
        match self.state.clone() {
            Ok(State::Terminated) => {
                return Ok(State::Terminated)
            },
            Ok(State::Interrupt) => {
                return Ok(State::Interrupt)
            }
            Err(err) => {
                return Err(err)
            },
            _ => {}
        };
        let instruction;
        {
            let stack =  self.fn_stack_frame.last_mut();
            let mut stack = match stack {
                Some(stack) => stack,
                None => {
                    self.state = Err(SandboxExecutionError::FnStackUnderflowError);
                    return self.state.clone()
                }
            };
            instruction = self.instructions[stack.ptr as usize].clone();
            stack.ptr += 1;
        }
        self.lim -= 1;
        if self.lim == 0 {
            self.state = Err(SandboxExecutionError::TimeoutError);
            return self.state.clone()
        }
        match instruction {
            Instruction::Add => { __binary_autogen!(op_add, self); },
            Instruction::Sub => { __binary_autogen!(op_sub, self); },
            Instruction::Mul => { __binary_autogen!(op_mul, self); },
            Instruction::Div => { __binary_autogen!(op_div, self); },
            Instruction::Mod => { __binary_autogen!(op_moduls, self); },
            Instruction::BitwiseAnd => { __binary_autogen!(op_band, self); },
            Instruction::BitwiseOr => { __binary_autogen!(op_bor, self); },
            Instruction::BitwiseXor => { __binary_autogen!(op_bxor, self); },
            Instruction::Shl => { __binary_autogen!(op_bsl, self); },
            Instruction::Shr => { __binary_autogen!(op_bsr, self); },
            Instruction::UnaryPlus => { __unary_autogen!(op_pos, self); },
            Instruction::UnaryMinus => { __unary_autogen!(op_neg, self); }
            Instruction::Not => { __unary_autogen!(op_not, self); },
            Instruction::BitwiseNot => { __unary_autogen!(op_bnot, self); }
            Instruction::Eq => { __binary_autogen!(op_eq, self); },
            Instruction::NotEq => { __binary_autogen!(op_ne, self); }
            Instruction::Lt => { __binary_autogen!(op_lt, self); },
            Instruction::Lte => { __binary_autogen!(op_le, self); },
            Instruction::Gt => { __binary_autogen!(op_gt, self); },
            Instruction::Gte => { __binary_autogen!(op_ge, self); },
            Instruction::Assign => {
                let stack =  self.fn_stack_frame.last_mut().unwrap();
                let value = stack.pop_value()?;
                let target = stack.pop()?;
                match target {
                    StackItem::Value(value) => {
                        self.state = Err(SandboxExecutionError::UndefinedVarError);
                        return self.state.clone()
                    },
                    StackItem::AttrReference((None, target)) => {
                        stack.mapping.borrow_mut().mapping.insert(target, Rc::new(RefCell::new(value)));
                    },
                    StackItem::AttrReference((Some(value), target)) => {
                        if let Some(obj) = value.as_object() {
                            obj.set(target, value);
                        } else {
                            self.state = Err(SandboxExecutionError::UnexpectedAttrError);
                            return self.state.clone()
                        }
                    }
                    StackItem::IdxReference(target) => {
                        if let Some(arr) = target.0.as_collection() {
                            let mut idx = target.1;
                            if idx < 0 {
                                idx += arr.borrow().len() as i64
                            }
                            if idx >= 0 && (idx as usize) < arr.borrow().len() {
                                arr.borrow_mut()[idx as usize] = value;
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
                let stack =  self.fn_stack_frame.last_mut().unwrap();
                let a = stack.pop_value()?;
                if a.is_truthy() {
                    stack.ptr = loc;
                }
            }
            Instruction::JmpZ(loc) => {
                let stack =  self.fn_stack_frame.last_mut().unwrap();
                let a = stack.pop_value()?;
                if !a.is_truthy() {
                    stack.ptr = loc;
                }
            }
            Instruction::Jmp(loc) => {
                let stack =  self.fn_stack_frame.last_mut().unwrap();
                stack.ptr = loc;
            }
            Instruction::Call => {
                todo!("Function not exist yet")
            }
            Instruction::Ret => {
                todo!("Function not exist yet")
            }
            Instruction::LoadNone => {
                let stack =  self.fn_stack_frame.last_mut().unwrap();
                stack.push_value(self.alloc.allocate(ValueKind::None));
            }
            Instruction::LoadLitFloat(v) => {
                let stack =  self.fn_stack_frame.last_mut().unwrap();
                stack.push_value(self.alloc.allocate(ValueKind::Float(VirPyFloat { value: v })));
            }
            Instruction::LoadLitInt(v) => {
                let stack =  self.fn_stack_frame.last_mut().unwrap();
                stack.push_value(self.alloc.allocate(ValueKind::Int(VirPyInt { value: v })));
            }
            Instruction::LoadLitString(v) => {
                let stack =  self.fn_stack_frame.last_mut().unwrap();
                stack.push_value(self.alloc.allocate(ValueKind::String(v.clone())));
            }
            Instruction::LoadLitBool(v) => {
                let stack =  self.fn_stack_frame.last_mut().unwrap();
                stack.push_value(self.alloc.allocate(ValueKind::Bool(v)));
            }
            Instruction::ConstructArr(len) => {
                let stack =  self.fn_stack_frame.last_mut().unwrap();
                let mut arr = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    arr.push(stack.pop_value()?);
                }
                stack.push_value(self.alloc.allocate(ValueKind::Collection(Rc::new(RefCell::new(arr)))));
            }
            Instruction::ConstructObj(len2) => {
                let stack =  self.fn_stack_frame.last_mut().unwrap();
                let mut obj = VirPyObject::new();
                for idx in 0..len2 {
                    let name = stack.pop_value()?;
                    let value = stack.pop_value()?;
                    if name.as_string().is_none() {
                        let remaining_stackdrop = (len2 - idx) * 2;
                        for _ in 0..remaining_stackdrop {
                            let _ = stack.pop_value(); // Drop error since AttrNotStringError is the primary issue, although otherwise this would cause error as well for stack underflow
                        }
                        self.state = Err(SandboxExecutionError::AttrNotStringError);
                        return self.state.clone()
                    }
                    obj.set(name.as_string().unwrap().clone(), value);
                }
                stack.push_value(self.alloc.allocate(ValueKind::Object(obj)));
            }
            Instruction::LoadName(name) => {
                let stack =  self.fn_stack_frame.last_mut().unwrap();
                stack.push_ref((None, name.clone()));
            }
            Instruction::LoadObjectAttr(name) => {
                let stack =  self.fn_stack_frame.last_mut().unwrap();
                let value = stack.pop_value()?;
                if let Some(_) = value.as_object() {
                    stack.push_ref((Some(value), name.clone()));
                }
                else {
                    self.state = Err(SandboxExecutionError::UnexpectedAttrError);
                    return self.state.clone()
                }

            }
            Instruction::LoadObjectIndex(idx) => {
                let stack =  self.fn_stack_frame.last_mut().unwrap();
                let value = stack.pop_value()?;
                if let Some(_) = value.as_collection() {
                    stack.push_idx_ref((value, idx));
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
                let stack =  self.fn_stack_frame.last_mut().unwrap();
                stack.pop_value()?;
            }
        }
        Ok(State::Ok)
    }
}
