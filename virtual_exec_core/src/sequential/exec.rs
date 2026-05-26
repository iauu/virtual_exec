use std::collections::HashMap;
use std::sync::{Arc};
use async_lock::RwLock;
use virtual_exec_type::mem::{Allocator, MemoryAllocator, Value, ValuePtr};
use virtual_exec_type::op::*;
use crate::sequential::instructions::{Instruction, SubscriptLoad};
use virtual_exec_type::base::{IsTruhy, TypeCast};
use virtual_exec_type::error::{NonRecoverableError, CriticalError, RecoverableError};
pub use virtual_exec_type::error::ExecutionError;

type AttrReference<'ctx> = (Option<ValuePtr<'ctx>>, String);
type IdxReference<'ctx> = (ValuePtr<'ctx>, i64);

pub type ArgumentPackage<'ctx> = Vec<ValuePtr<'ctx>>;

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

#[derive(Debug, Clone)]
pub struct FnStackFrame<'ctx> {
    pub ptr: u64,
    pub mapping: Arc<RwLock<HashMap<String, ValuePtr<'ctx>>>>,
}

#[derive(Debug, Clone)]
pub struct InstStateMachine<'ctx> {
    pub lim: u64,
    pub fn_stack_frame: Vec<FnStackFrame<'ctx>>,
    pub alloc: MemoryAllocator<'ctx>,
    pub instructions: Vec<Instruction>,
    pub state: Result<State<'ctx>, ExecutionError>,
    pub stack: Vec<StackItem<'ctx>>,
}

#[derive(Debug, Clone)]
pub enum State<'ctx> {
    Ok,
    Terminated,
    Interrupt,
    Timeout(u64),
    FnExternInput(String, usize),
    FnExternOutput(String, ArgumentPackage<'ctx>)
}

macro_rules! __binary_autogen {
    ($f:ident, $ss:ident) => {
        {
            let b = $ss.pop_get()?;
            let a = $ss.pop_get()?;
            let result = $f(a, b, &$ss.alloc).map_err(|_| ExecutionError::NonRecoverable(NonRecoverableError::UndefinedOperendError))?;
            $ss.push_value(result);
        }
    };
}


macro_rules! __unary_autogen {
    ($f:ident, $ss:ident) => {
        {
            let a = $ss.pop_get()?;
            let result = $f(a, &$ss.alloc).map_err(|_| ExecutionError::NonRecoverable(NonRecoverableError::UndefinedOperendError))?;
            $ss.push_value(result);
        }
    };
}

impl<'ctx> InstStateMachine<'ctx> {

    #[allow(unused)]
    fn pop_value(&mut self) -> Result<ValuePtr<'ctx>, ExecutionError> {
        let result = self.stack.pop().ok_or(ExecutionError::Critical(CriticalError::VStackUnderflowError))?;
        match result {
            StackItem::Value(value) => Ok(value),
            StackItem::AttrReference(_) | StackItem::IdxReference(_) => {
                Err(ExecutionError::NonRecoverable(NonRecoverableError::UndefinedVarError))
            }
        }
    }

    fn push(&mut self, value: StackItem<'ctx>) {
        self.stack.push(value);
    }

    fn push_value(&mut self, value: ValuePtr<'ctx>) {
        self.stack.push(value.into());
    }

    #[allow(unused)]
    fn pop_ref(&mut self) -> Result<AttrReference<'ctx>, ExecutionError> {
        let result = self.stack.pop().ok_or(ExecutionError::Critical(CriticalError::VStackUnderflowError))?;
        match result {
            StackItem::Value(_) | StackItem::IdxReference(_) => {
                Err(ExecutionError::NonRecoverable(NonRecoverableError::AttrMisuseError))
            },
            StackItem::AttrReference(reference) => Ok(reference)

        }
    }

    fn push_ref(&mut self, reference: AttrReference<'ctx>) {
        self.stack.push(reference.into());
    }

    #[allow(unused)]
    fn pop_idx_ref(&mut self) -> Result<IdxReference<'ctx>, ExecutionError> {
        let result = self.stack.pop().ok_or(ExecutionError::Critical(CriticalError::VStackUnderflowError))?;
        match result {
            StackItem::Value(_) | StackItem::AttrReference(_) => {
                Err(ExecutionError::NonRecoverable(NonRecoverableError::AttrMisuseError))
            },
            StackItem::IdxReference(reference) => Ok(reference)
        }
    }

    fn push_idx_ref(&mut self, reference: IdxReference<'ctx>) {
        self.stack.push(reference.into());
    }

    fn pop(&mut self) -> Result<StackItem<'ctx>, ExecutionError> {
        self.stack.pop().ok_or(ExecutionError::Critical(CriticalError::VStackUnderflowError))
    }

    fn resolve(&self, name: &str) -> Result<ValuePtr<'ctx>, ExecutionError> {
        for frame in self.fn_stack_frame.iter().rev() {
            if let Some(val) = frame.mapping.read_arc_blocking().get(name) {
                return Ok(val.clone());
            }
        }
        
        Err(ExecutionError::NonRecoverable(NonRecoverableError::ReferenceNotExistError(name.to_string())))
    }

    fn pop_get(&mut self) -> Result<ValuePtr<'ctx>, ExecutionError> {
        let result = {
            self.pop()?
        };

        match result {
            StackItem::Value(v) => Ok(v),
            StackItem::AttrReference(target) => {
                match target.0 {
                    Some(obj) => {
                        if let Some(o) = obj.as_object() {
                            Ok(o.read_arc_blocking().get(&target.1).ok_or_else(|| ExecutionError::NonRecoverable(NonRecoverableError::ReferenceNotExistError(target.1.clone())))?.clone())
                        } else {
                            Err(ExecutionError::NonRecoverable(NonRecoverableError::UnexpectedAttrError))
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
                        idx += arr.read_arc_blocking().len() as i64;
                    }
                    if idx >= 0 && (idx as usize) < arr.read_arc_blocking().len() {
                        Ok(arr.read_arc_blocking()[idx as usize].clone())
                    } else {
                        Err(ExecutionError::NonRecoverable(NonRecoverableError::IndexOutOfRangeError))
                    }
                } else {
                    Err(ExecutionError::NonRecoverable(NonRecoverableError::UnexpectedIdxError))
                }
            }
        }
    }

    fn get_mut_stack_ref<'a>(&'a mut self) -> Result<&'a mut FnStackFrame<'ctx>, ExecutionError> {
        self.fn_stack_frame.last_mut().ok_or(ExecutionError::Critical(CriticalError::FnStackUnderflowError))
    }

    fn get_stack_ref<'a>(&'a self) -> Result<&'a FnStackFrame<'ctx>, ExecutionError> {
        self.fn_stack_frame.last().ok_or(ExecutionError::Critical(CriticalError::FnStackUnderflowError))
    }
    
    fn alloc(&self, data: Value<'ctx>) -> Result<ValuePtr<'ctx>, ExecutionError> {
        self.alloc.alloc(data).map_err(|e| e.into())
    }

    fn inst_eval(&mut self, instruction: Instruction) -> Result<State<'ctx>, ExecutionError> {
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
                let value = self.pop_get()?;
                let target = self.pop()?;
                match target {
                    StackItem::Value(_value) => {
                        self.state = Err(ExecutionError::NonRecoverable(NonRecoverableError::UndefinedVarError));
                        return self.state.clone()
                    },
                    StackItem::AttrReference((None, target)) => {
                        let stack =  self.get_mut_stack_ref()?;
                        stack.mapping.write_arc_blocking().insert(target, value.clone());
                    },
                    StackItem::AttrReference((Some(value), target)) => {
                        if let Some(obj) = value.as_object() {
                            obj.write_arc_blocking().insert(target, value.clone());
                        } else {
                            self.state = Err(ExecutionError::NonRecoverable(NonRecoverableError::UnexpectedAttrError));
                            return self.state.clone()
                        }
                    }
                    StackItem::IdxReference(target) => {
                        if let Some(arr) = target.0.as_collections() {
                            let mut idx = target.1;
                            if idx < 0 {
                                idx += arr.read_arc_blocking().len() as i64
                            }
                            if idx >= 0 && (idx as usize) < arr.read_arc_blocking().len() {
                                arr.write_arc_blocking()[idx as usize] = value;
                            }
                            else {
                                self.state = Err(ExecutionError::NonRecoverable(NonRecoverableError::IndexOutOfRangeError));
                                return self.state.clone()
                            }
                        }
                    }
                }
            }
            Instruction::JmpNz(loc) => {
                let a = self.pop_get()?;
                if a.is_truthy() {
                    let stack =  self.get_mut_stack_ref()?;
                    stack.ptr = loc;
                }
            }
            Instruction::JmpZ(loc) => {
                let a = self.pop_get()?;
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
                let ptr = self.pop_get()?;
                if let Some((ptr, fn_size)) = ptr.as_dptr() {
                    let given_size = self.pop_get()?.as_int().ok_or(ExecutionError::NonRecoverable(NonRecoverableError::UnexpectedFunctionCall));
                    let given_size = match given_size {
                        Ok(v) => v,
                        Err(e) => {
                            self.state = Err(e);
                            return self.state.clone();
                        }
                    };

                    if given_size as usize != fn_size {
                        self.state = Err(ExecutionError::NonRecoverable(NonRecoverableError::IncorrectArgumentCountError));
                        return self.state.clone();
                    }

                    self.fn_stack_frame.push(
                        FnStackFrame {
                            ptr,
                            mapping: Arc::new(Default::default()),
                        }
                    )
                } else if let Some((name, fn_size)) = ptr.as_fn_ptr_extern() {
                    let given_size = self.pop_get()?.as_int().ok_or(ExecutionError::NonRecoverable(NonRecoverableError::UnexpectedFunctionCall));
                    let given_size = match given_size {
                        Ok(v) => v,
                        Err(e) => {
                            self.state = Err(e);
                            return self.state.clone();
                        }
                    };

                    if given_size as usize != fn_size {
                        self.state = Err(ExecutionError::NonRecoverable(NonRecoverableError::IncorrectArgumentCountError));
                        return self.state.clone();
                    }

                    self.state = Ok(State::FnExternInput(name, fn_size))
                }
                else {
                    self.state = Err(ExecutionError::NonRecoverable(NonRecoverableError::UnexpectedFunctionCall));
                    return self.state.clone()
                }
            }
            Instruction::Ret => {
                self.fn_stack_frame.pop().ok_or(ExecutionError::Critical(CriticalError::FnStackUnderflowError))?;
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
                    arr.push(self.pop_get()?);
                }
                self.push_value(self.alloc(Value::Collection(Arc::new(RwLock::new(arr))))?);
            }
            Instruction::ConstructObj(len2) => {
                let mut obj = HashMap::new();
                for idx in 0..len2 {
                    let name = self.pop_get()?;
                    let value = self.pop_get()?;
                    if name.as_string().is_none() {
                        let remaining_stackdrop = (len2 - idx) * 2;
                        for _ in 0..remaining_stackdrop {
                            let _ = self.pop_get(); // Drop error since AttrNotStringError is the primary issue, although otherwise this would cause error as well for stack underflow
                        }
                        self.state = Err(ExecutionError::NonRecoverable(NonRecoverableError::AttrNotStringError));
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
                let value = self.pop_get()?;
                if let Some(_) = value.as_object() {
                    self.push_ref((Some(value), name.into_string()));
                }
                else {
                    self.state = Err(ExecutionError::NonRecoverable(NonRecoverableError::UnexpectedAttrError));
                    return self.state.clone()
                }

            }
            Instruction::LoadObjectIndex(idx) => {
                let value = self.pop_get()?;
                if let Some(_) = value.as_collections() {
                    if let SubscriptLoad::Idx(idx) = idx {
                        self.push_idx_ref((value, idx));
                    }
                }
                else if let Some(_) = value.as_object() {
                    if let SubscriptLoad::String(s) = idx {
                        self.push_ref((Some(value), s.into_string()));
                    }
                }
                else {
                    self.state = Err(ExecutionError::NonRecoverable(NonRecoverableError::UnexpectedIdxError));
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
                self.pop()?;
            },
            Instruction::Swap => {
                let a = self.pop()?;
                let b = self.pop()?;
                self.push(a);
                self.push(b);
            },
            Instruction::LoadDPtr(ptr, arg_len) => {
                self.push(StackItem::Value(self.alloc(Value::DPtr(ptr, arg_len))?))
            }
        };
        self.state.clone()
    }

    pub fn run_once(&mut self) -> Result<State<'ctx>, ExecutionError> {
        match self.state.clone() {
            Ok(State::Terminated) => {
                return Ok(State::Terminated)
            },
            Ok(State::Interrupt) => {
                return Ok(State::Interrupt)
            },
            Ok(State::FnExternInput(a, b)) => {
              return Ok(State::FnExternInput(a, b))  
            },
            Ok(State::FnExternOutput(a, b)) => {
                return Ok(State::FnExternOutput(a,b))
            },
            Err(ExecutionError::Recoverable(rec)) => {
                self.state = Err(ExecutionError::NonRecoverable(NonRecoverableError::NonRecoveredRecoverableError(rec)));
                return self.state.clone();
            },
            Err(err) => {
                return Err(err)
            },
            Ok(State::Ok) | Ok(State::Timeout(_)) => {}
        };
        let instruction;
        {
            let stack =  self.get_stack_ref()?;
            let ptr = stack.ptr as usize;
            instruction = self.instructions[ptr].clone();
            let stack =  self.get_mut_stack_ref()?;
            stack.ptr += 1;
        }
        if let Ok(State::Timeout(req)) = self.state && req > self.lim {
            return self.state.clone();
        }
        if self.lim == 0 {
            self.state = Ok(State::Timeout(1));
            return self.state.clone()
        } else {
            self.state = Ok(State::Ok)
        }
        self.lim -= 1;

        self.state = self.inst_eval(instruction);
        match self.state.clone() {
            Err(e) => {
                return self.state.clone();
            }
            _ => {}
        }

        if self.fn_stack_frame.last().unwrap().ptr as usize == self.instructions.len()  {
            self.state = Ok(State::Terminated);
            return self.state.clone()
        }
        self.state.clone()
    }
    
    pub fn retrieve_fn_input(&mut self) -> Result<Option<(String, ArgumentPackage<'ctx>)>, ExecutionError> {
        if let Ok(State::FnExternInput(fn_name, b)) = self.state.clone() {
            let values = (0..b)
                .map(|_| self.pop_get())
                .collect::<Result<Vec<_>, ExecutionError>>()
                .inspect_err(|e| self.state = Err(e.clone()))?;
            self.state = Ok(State::FnExternOutput(fn_name.clone(), values.clone()));
            Ok(Some((fn_name, values)))
        } else {
            Ok(None)
        }
    }
    
    pub fn push_fn_output(&mut self, ptr: Result<ValuePtr<'ctx>, ExecutionError>) -> bool {
        if let Ok(State::FnExternOutput(_, values)) = self.state.clone() {
            match ptr {
                Ok(ptr) => {
                    self.push_value(ptr);
                    self.state = Ok(State::Ok);
                },
                Err(e) => {
                    if let ExecutionError::Recoverable(rec) = e {
                        match rec {
                            RecoverableError::TimeoutError(amount) => {
                                self.state = Err(ExecutionError::Recoverable(RecoverableError::TimeoutError(amount)));
                                let _ = values.iter().rev().map(|v| self.push_value(v.clone())).collect::<Vec<_>>();
                                match self.get_mut_stack_ref() {
                                    Ok(stack_ref) => {
                                        stack_ref.ptr -= 1
                                    }
                                    Err(e) => {
                                        self.state = Err(e.clone());
                                    }
                                }
                            }
                        }
                    } else {
                        self.state = Err(e);
                    }
                }
            }
            true
        } else {
            false
        }
    }

    pub fn check_use(&self, size: u64) -> bool {
        size > self.lim
    }

    pub fn check_use_err(&self, size: u64) -> Result<(), RecoverableError> {
        if self.check_use(size) {
            return Ok(());
        }
        Err(RecoverableError::TimeoutError(size))
    }

    pub fn reduce_lim(&mut self, size: u64) -> Result<(), RecoverableError> {
        self.check_use_err(size)?;
        self.lim -= size;
        Ok(())
    }
}
