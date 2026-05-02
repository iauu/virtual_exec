
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct MemoryError;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy)]
pub enum TypeConversionError {
    DivideByZeroError,
    UndefinedOperatorMethodError,
    InvalidTypeError,
    MemoryError
}

#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub enum RecoverableError {
    TimeoutError(u64),
}

#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub enum NonRecoverableError {
    ReferenceNotExistError(String),
    DivideByZeroError,
    UndefinedOperatorMethodError,
    InvalidTypeError,
    SubscriptKeyError,
    AttributeNotFoundError,
    UndefinedOperendError,
    AttrNotStringError,
    RefNameMissingError,
    UndefinedVarError,
    AttrMisuseError,
    UnexpectedAttrError,
    UnexpectedIdxError,
    IndexOutOfRangeError,
    MemoryError,
    UnexpectedFunctionCall,
    IncorrectArgumentCountError,
    GenericError
}

#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub enum CriticalError {
    GenericPanicRewindError,
    InvalidSyntaxError,
    FnStackUnderflowError,
    VStackUnderflowError,
}

#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub enum ExecutionError {
    Recoverable(RecoverableError),
    NonRecoverable(NonRecoverableError),
    Critical(CriticalError)
}

impl From<RecoverableError> for ExecutionError {
    fn from(err: RecoverableError) -> Self {
        ExecutionError::Recoverable(err)
    }
}

impl From<NonRecoverableError> for ExecutionError {
    fn from(err: NonRecoverableError) -> Self {
        ExecutionError::NonRecoverable(err)
    }
}

impl From<CriticalError> for ExecutionError {
    fn from(err: CriticalError) -> Self {
        ExecutionError::Critical(err)
    }
}

impl From<MemoryError> for ExecutionError {
    fn from(_err: MemoryError) -> Self {
        ExecutionError::NonRecoverable(NonRecoverableError::MemoryError)
    }
}

impl From<MemoryError> for TypeConversionError {
    fn from(_err: MemoryError) -> Self {
        TypeConversionError::MemoryError
    }
}

impl From<TypeConversionError> for ExecutionError {
    fn from(err: TypeConversionError) -> Self {
        match err {
            TypeConversionError::DivideByZeroError => ExecutionError::NonRecoverable(NonRecoverableError::DivideByZeroError),
            TypeConversionError::UndefinedOperatorMethodError => ExecutionError::NonRecoverable(NonRecoverableError::UndefinedOperatorMethodError),
            TypeConversionError::InvalidTypeError => ExecutionError::NonRecoverable(NonRecoverableError::InvalidTypeError),
            TypeConversionError::MemoryError => ExecutionError::NonRecoverable(NonRecoverableError::MemoryError)
        }
    }
}
