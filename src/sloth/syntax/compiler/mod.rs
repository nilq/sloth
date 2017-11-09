pub mod error;

pub use super::*;

pub use self::error::*;

pub type CompileResult<T> = Result<T, CompileError>;
