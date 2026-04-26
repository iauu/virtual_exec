use crate::base::{Downcast};
use crate::mem::ValuePtr;

#[derive(Debug)]
pub struct MemoryError;

#[derive(Clone, Debug)]
pub enum TypeConversionError {
    DivideByZeroError,
    UndefinedOperatorMethodError,
    InvalidTypeError,
    MemoryError
}

#[derive(Clone, Debug)]
pub enum ExecutionError {
    TimeoutError,
    ReferenceNotExistError(String),
    DivideByZeroError,
    GenericPanicRewindError,
    UndefinedOperatorMethodError,
    InvalidTypeError,
    InvalidSyntaxError,
    SubscriptKeyError,
    AttributeNotFoundError,
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

impl From<MemoryError> for ExecutionError {
    fn from(err: MemoryError) -> Self {
        ExecutionError::MemoryError
    }
}

impl From<MemoryError> for TypeConversionError {
    fn from(err: MemoryError) -> Self {
        TypeConversionError::MemoryError
    }
}

impl From<TypeConversionError> for ExecutionError {
    fn from(err: TypeConversionError) -> Self {
        match err {
            TypeConversionError::DivideByZeroError => ExecutionError::DivideByZeroError,
            TypeConversionError::UndefinedOperatorMethodError => ExecutionError::UndefinedOperatorMethodError,
            TypeConversionError::InvalidTypeError => ExecutionError::InvalidTypeError,
            TypeConversionError::MemoryError => ExecutionError::MemoryError
        }
    }
}

//
// pub type Result<T> = ::core::result::Result<T, SandboxExecutionError>;
//
// impl<'ctx> Downcast<'ctx> for SandboxExecutionError {
//     fn from_value(value: ValuePtr<'ctx>) -> Option<&'ctx Self> {
//         value.as_error()
//     }
// }
