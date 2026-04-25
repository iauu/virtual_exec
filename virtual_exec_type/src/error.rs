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
pub enum SandboxExecutionError {
    TimeoutError,
    ReferenceNotExistError(String),
    DivideByZeroError,
    GenericPanicRewindError,
    UndefinedOperatorMethodError,
    InvalidTypeError,
    InvalidSyntaxError,
    SubscriptKeyError,
    AttributeNotFoundError,
    MemoryError
}

impl From<MemoryError> for SandboxExecutionError {
    fn from(err: MemoryError) -> Self {
        SandboxExecutionError::MemoryError
    }
}

impl From<MemoryError> for TypeConversionError {
    fn from(err: MemoryError) -> Self {
        TypeConversionError::MemoryError
    }
}

impl From<TypeConversionError> for SandboxExecutionError {
    fn from(err: TypeConversionError) -> Self {
        match err {
            TypeConversionError::DivideByZeroError => SandboxExecutionError::DivideByZeroError,
            TypeConversionError::UndefinedOperatorMethodError => SandboxExecutionError::UndefinedOperatorMethodError,
            TypeConversionError::InvalidTypeError => SandboxExecutionError::InvalidTypeError,
            TypeConversionError::MemoryError => SandboxExecutionError::MemoryError
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
