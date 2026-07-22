use crate::HashMap;
use alloc::sync::{Arc};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use async_lock::RwLock;
use virtual_exec_type::mem::{Allocator, GetSize, MemoryAllocator, Value, ValuePtr};
use virtual_exec_type::op::*;
use crate::sequential::instructions::{Instruction, SubscriptLoad};
use virtual_exec_type::base::{IsTruhy, TypeCast};
use virtual_exec_type::error::{NonRecoverableError, CriticalError, RecoverableError};
pub use virtual_exec_type::error::ExecutionError;
use virtual_exec_type::ext::*;

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

impl<'ctx> GetSize for StackItem<'ctx> {
    fn get_size(&self) -> usize {
        // A stack entry is a real heap node plus its `_acct` MemoryChunk node,
        // so a slot costs more virtual than a bare collection element (8 B).
        const SLOT: usize = 32;
        match self {
            StackItem::Value(_) => SLOT,
            StackItem::AttrReference((_, name)) => SLOT + name.len(),
            StackItem::IdxReference(_) => SLOT,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StackEntry<'ctx> {
    item: StackItem<'ctx>,
    #[allow(dead_code)]
    _acct: ValuePtr<'ctx>,
}

#[derive(Debug, Clone)]
pub struct FnStackFrame<'ctx> {
    pub ptr: u64,
    pub mapping: Arc<RwLock<HashMap<String, ValuePtr<'ctx>>>>,
    #[doc(hidden)]
    pub _acct: Option<ValuePtr<'ctx>>,
}

#[derive(Debug, Clone)]
pub struct InstStateMachine<'ctx> {
    pub lim: u64,
    pub fn_stack_frame: Vec<FnStackFrame<'ctx>>,
    pub alloc: MemoryAllocator<'ctx>,
    pub instructions: Vec<Instruction>,
    pub state: Result<State<'ctx>, ExecutionError>,
    pub stack: Vec<StackEntry<'ctx>>,
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
            let result = $f(a, b, &$ss.alloc).map_err(ExecutionError::from)?;
            $ss.push_value(result)?;
        }
    };
    ($f:ident, $ss:ident, $v:expr) => {
        {
            let b = $ss.pop_get()?;
            let a = $ss.pop_get()?;
            let result = $f(a, b, &$ss.alloc).map_err(ExecutionError::from).or_else(|_| $v)?;
            $ss.push_value(result)?;
        }
    };
}


macro_rules! __unary_autogen {
    ($f:ident, $ss:ident) => {
        {
            let a = $ss.pop_get()?;
            let result = $f(a, &$ss.alloc).map_err(ExecutionError::from)?;
            $ss.push_value(result)?;
        }
    };
    ($f:ident, $ss:ident, $v:expr) => {
        {
            let b = $ss.pop_get()?;
            let result = $f(a, &$ss.alloc).map_err(ExecutionError::from).or_else(|_| $v)?;
            $ss.push_value(result)?;
        }
    };
}

impl<'ctx> InstStateMachine<'ctx> {
    
    const FN_FRAME_COST: usize = 128;
    
    fn stack_push(&mut self, item: StackItem<'ctx>) -> Result<(), ExecutionError> {
        let acct = self.alloc(Value::MemoryChunk(item.get_size()))?;
        self.stack.push(StackEntry { item, _acct: acct });
        Ok(())
    }
    
    fn stack_pop(&mut self) -> Result<StackItem<'ctx>, ExecutionError> {
        let StackEntry { item, _acct } = self.stack.pop().ok_or(ExecutionError::Critical(CriticalError::VStackUnderflowError))?;
        Ok(item)
    }
    
    fn push_frame(&mut self, ptr: u64, mapping: Arc<RwLock<HashMap<String, ValuePtr<'ctx>>>>) -> Result<(), ExecutionError> {
        let acct = self.alloc(Value::MemoryChunk(Self::FN_FRAME_COST))?;
        self.fn_stack_frame.push(FnStackFrame { ptr, mapping, _acct: Some(acct) });
        Ok(())
    }


    fn pop_frame(&mut self) -> Result<FnStackFrame<'ctx>, ExecutionError> {
        self.fn_stack_frame.pop().ok_or(ExecutionError::Critical(CriticalError::FnStackUnderflowError))
    }

    fn reaccount_current_frame(&mut self) -> Result<(), ExecutionError> {
        let (acct, footprint) = {
            let frame = self.get_stack_ref()?;
            match &frame._acct {
                Some(acct) => {
                    let keys = frame.mapping.read_arc_safe().keys().map(|k| k.len()).sum::<usize>();
                    (acct.clone(), Self::FN_FRAME_COST + keys)
                }
                None => return Ok(()),
            }
        };
        acct.write_arc_safe().inner = Value::MemoryChunk(footprint);
        self.alloc.change_alloc(&acct).map_err(ExecutionError::from)
    }

    #[allow(unused)]
    fn pop_value(&mut self) -> Result<ValuePtr<'ctx>, ExecutionError> {
        let result = self.stack_pop()?;
        match result {
            StackItem::Value(value) => Ok(value),
            StackItem::AttrReference(_) | StackItem::IdxReference(_) => {
                Err(ExecutionError::NonRecoverable(NonRecoverableError::UndefinedVarError))
            }
        }
    }

    fn push(&mut self, value: StackItem<'ctx>) -> Result<(), ExecutionError> {
        self.stack_push(value)
    }

    fn push_value(&mut self, value: ValuePtr<'ctx>) -> Result<(), ExecutionError> {
        self.stack_push(value.into())
    }

    #[allow(unused)]
    fn pop_ref(&mut self) -> Result<AttrReference<'ctx>, ExecutionError> {
        let result = self.stack_pop()?;
        match result {
            StackItem::Value(_) | StackItem::IdxReference(_) => {
                Err(ExecutionError::NonRecoverable(NonRecoverableError::AttrMisuseError))
            },
            StackItem::AttrReference(reference) => Ok(reference)

        }
    }

    fn push_ref(&mut self, reference: AttrReference<'ctx>) -> Result<(), ExecutionError> {
        self.stack_push(reference.into())
    }

    #[allow(unused)]
    fn pop_idx_ref(&mut self) -> Result<IdxReference<'ctx>, ExecutionError> {
        let result = self.stack_pop()?;
        match result {
            StackItem::Value(_) | StackItem::AttrReference(_) => {
                Err(ExecutionError::NonRecoverable(NonRecoverableError::AttrMisuseError))
            },
            StackItem::IdxReference(reference) => Ok(reference)
        }
    }

    fn push_idx_ref(&mut self, reference: IdxReference<'ctx>) -> Result<(), ExecutionError> {
        self.stack_push(reference.into())
    }

    fn pop(&mut self) -> Result<StackItem<'ctx>, ExecutionError> {
        self.stack_pop()
    }

    fn resolve(&self, name: &str) -> Result<ValuePtr<'ctx>, ExecutionError> {
        for frame in self.fn_stack_frame.iter().rev() {
            if let Some(val) = frame.mapping.read_arc_safe().get(name) {
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
                            Ok(o.read_arc_safe().get(&target.1).ok_or_else(|| ExecutionError::NonRecoverable(NonRecoverableError::ReferenceNotExistError(target.1.clone())))?.clone())
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
                        idx += arr.read_arc_safe().len() as i64;
                    }
                    if idx >= 0 && (idx as usize) < arr.read_arc_safe().len() {
                        Ok(arr.read_arc_safe()[idx as usize].clone())
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
            Instruction::Eq => { __binary_autogen!(err_op_eq, self, self.alloc(Value::Bool(false))); },
            Instruction::NotEq => { __binary_autogen!(err_op_ne, self, self.alloc(Value::Bool(true))); }
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
                        self.get_mut_stack_ref()?.mapping.write_arc_safe().insert(target, value.clone());
                        self.reaccount_current_frame()?;
                    },
                    StackItem::AttrReference((Some(obj_ptr), target)) => {
                        if let Some(obj) = obj_ptr.as_object() {
                            obj.write_arc_safe().insert(target, value.clone());
                        } else {
                            self.state = Err(ExecutionError::NonRecoverable(NonRecoverableError::UnexpectedAttrError));
                            return self.state.clone()
                        }
                        self.alloc.change_alloc(&obj_ptr)?;
                    }
                    StackItem::IdxReference(target) => {
                        if let Some(arr) = target.0.as_collections() {
                            let mut idx = target.1;
                            if idx < 0 {
                                idx += arr.read_arc_safe().len() as i64
                            }
                            if idx >= 0 && (idx as usize) < arr.read_arc_safe().len() {
                                arr.write_arc_safe()[idx as usize] = value;
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

                    self.push_frame(ptr, Arc::new(Default::default()))?
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
                self.pop_frame()?;
            }
            Instruction::LoadNone => {
                self.push_value(self.alloc(Value::None)?)?;
            }
            Instruction::LoadLitFloat(v) => {
                self.push_value(self.alloc(Value::Float(v))?)?;
            }
            Instruction::LoadLitInt(v) => {
                self.push_value(self.alloc(Value::Int(v))?)?;
            }
            Instruction::LoadLitString(v) => {
                self.push_value(self.alloc(Value::String(v))?)?;
            }
            Instruction::LoadLitBool(v) => {
                self.push_value(self.alloc(Value::Bool(v))?)?;
            }
            Instruction::ConstructArr(len) => {
                if len > self.stack.len() as u64 {
                    self.state = Err(ExecutionError::Critical(CriticalError::VStackUnderflowError));
                    return self.state.clone()
                }
                let mut arr = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    arr.push(self.pop_get()?);
                }
                self.push_value(self.alloc(Value::Collection(Arc::new(RwLock::new(arr))))?)?;
            }
            Instruction::ConstructObj(len2) => {
                if len2.checked_mul(2).map_or(true, |needed| needed > self.stack.len() as u64) {
                    self.state = Err(ExecutionError::Critical(CriticalError::VStackUnderflowError));
                    return self.state.clone()
                }
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
                self.push_value(self.alloc(Value::Object(Arc::new(RwLock::new(obj))))?)?;
            }
            Instruction::LoadName(name) => {
                self.push_ref((None, name.into_string()))?;
            }
            Instruction::LoadObjectAttr(name) => {
                let value = self.pop_get()?;
                if let Some(_) = value.as_object() {
                    self.push_ref((Some(value), name.into_string()))?;
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
                        self.push_idx_ref((value, idx))?;
                    }
                }
                else if let Some(_) = value.as_object() {
                    if let SubscriptLoad::String(s) = idx {
                        self.push_ref((Some(value), s.into_string()))?;
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
                self.push(a)?;
                self.push(b)?;
            },
            Instruction::LoadDPtr(ptr, arg_len) => {
                self.push(StackItem::Value(self.alloc(Value::DPtr(ptr, arg_len))?))?
            }
        };
        self.state.clone()
    }

    #[cfg(feature = "std")]
    fn eval_guarded(&mut self, instruction: Instruction) -> Result<State<'ctx>, ExecutionError> {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.inst_eval(instruction))).unwrap_or_else(|_| Err(ExecutionError::Critical(CriticalError::GenericPanicRewindError)))
    }

    #[cfg(not(feature = "std"))]
    fn eval_guarded(&mut self, instruction: Instruction) -> Result<State<'ctx>, ExecutionError> {
        self.inst_eval(instruction)
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
        if self.get_stack_ref()?.ptr as usize >= self.instructions.len() {
            self.state = Ok(State::Terminated);
            return self.state.clone()
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
        let instruction;
        {
            let stack =  self.get_stack_ref()?;
            let ptr = stack.ptr as usize;
            instruction = self.instructions[ptr].clone();
            let stack =  self.get_mut_stack_ref()?;
            stack.ptr += 1;
        }
        self.lim -= 1;

        self.state = self.eval_guarded(instruction);
        match self.state.clone() {
            Err(_e) => {
                return self.state.clone();
            }
            _ => {}
        }

        // Only a plain Ok state may terminate here: a pending FnExternInput from a
        // trailing Call must survive so the host can still run the extern function.
        if let Ok(State::Ok) = self.state && let Some(frame) = self.fn_stack_frame.last() && frame.ptr as usize == self.instructions.len() {
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
        if let Ok(State::FnExternOutput(name, values)) = self.state.clone() {
            match ptr {
                Ok(ptr) => {
                    match self.push_value(ptr) {
                        Ok(()) => self.state = Ok(State::Ok),
                        Err(e) => self.state = Err(e),
                    }
                },
                Err(e) => {
                    if let ExecutionError::Recoverable(rec) = e {
                        match rec {
                            RecoverableError::TimeoutError(amount) => {
                                self.state = Err(ExecutionError::Recoverable(RecoverableError::TimeoutError(amount)));
                                for v in values.iter().rev() {
                                    if let Err(e) = self.push_value(v.clone()) {
                                        self.state = Err(e);
                                        return true;
                                    }
                                }
                                // Call also popped the argument count and function pointer
                                // before the extern call was made; both must be restored so
                                // re-executing Call sees the same stack as the original call.
                                let count = self.alloc(Value::Int(values.len() as i64));
                                let fn_ptr = self.alloc(Value::FnPtrExternal(name.into_boxed_str(), values.len()));
                                match (count, fn_ptr) {
                                    (Ok(count), Ok(fn_ptr)) => {
                                        if let Err(e) = self.push_value(count) {
                                            self.state = Err(e);
                                            return true;
                                        }
                                        if let Err(e) = self.push_value(fn_ptr) {
                                            self.state = Err(e);
                                            return true;
                                        }
                                        match self.get_mut_stack_ref() {
                                            Ok(stack_ref) => {
                                                stack_ref.ptr -= 1
                                            }
                                            Err(e) => {
                                                self.state = Err(e.clone());
                                            }
                                        }
                                    }
                                    (Err(e), _) | (_, Err(e)) => {
                                        self.state = Err(e);
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

    pub fn grant_lim(&mut self, additional: u64) {
        self.lim = self.lim.saturating_add(additional);
        match &self.state {
            Err(ExecutionError::Recoverable(RecoverableError::TimeoutError(_))) | Ok(State::Timeout(_)) => {
                self.state = Ok(State::Ok);
            }
            _ => {}
        }
    }

    pub fn check_use(&self, size: u64) -> bool {
        size <= self.lim
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

    pub fn fork<'alt>(&self) -> InstStateMachine<'alt> {
        self.alloc.lock_blocking().gc_weak();
        let vals = self.alloc.lock_arc_blocking().fork();
        let new_alloc: MemoryAllocator<'alt> = vals.0;
        let vec_items: Vec<ValuePtr<'alt>> = vals.1;
        let new_frames: Vec<FnStackFrame<'alt>> = self.fn_stack_frame.iter().map(
            |frame| FnStackFrame {
                ptr: frame.ptr,
                mapping: Arc::new(RwLock::new(
                    frame.mapping.read_arc_safe().iter().map(
                        |entry| {
                            let idx: usize = self.alloc.lock_arc_blocking()
                                .get_idx_ref(entry.1).unwrap();
                            (
                                entry.0.clone(), vec_items[idx].clone()
                            )

                        }
                    ).collect::<HashMap<String, ValuePtr<'alt>>>()
                )),
                _acct: match &frame._acct {
                    Some(e) => Some(
                        vec_items[self.alloc.lock_arc_blocking().get_idx_ref(&e).unwrap()].clone()
                    ),
                    None => None
                },
            }
        ).collect();
        let new_states: Result<State<'alt>, ExecutionError> = match &self.state {
            Ok(s) => {
                Ok(match s {
                    State::FnExternOutput(s, a) => State::FnExternOutput(
                        s.clone(),
                        a.iter()
                            .map(
                                |a| vec_items[self.alloc.lock_arc_blocking().get_idx_ref(a).unwrap()].clone()
                            )
                            .collect()
                    ),
                    State::FnExternInput(s, size) => State::FnExternInput(s.clone(), *size),
                    State::Ok => State::Ok,
                    State::Terminated => State::Terminated,
                    State::Interrupt => State::Interrupt,
                    State::Timeout(inst) => State::Timeout(*inst)
                })
            },
            Err(e) => Err(e.clone())
        };
        todo!()
    }
}
