pub mod error;
pub mod value;

pub use super::*;

pub use self::error::*;
pub use self::value::*;

pub type CompileResult<T> = Result<T, CompileError>;
