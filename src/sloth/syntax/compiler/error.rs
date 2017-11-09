use super::*;
use std::fmt;

#[derive(Debug)]
pub enum CompileErrorValue {
    Constant(String),
}

#[derive(Debug)]
pub struct CompileError {
    pub value: CompileErrorValue,
    pub position: Option<TokenPosition>,
}

impl CompileError {
    pub fn new(value: &str) -> CompileError {
        CompileError {
            value:    CompileErrorValue::Constant(value.to_owned()),
            position: None,
        }
    }
    
    pub fn new_pos(value: &str, position: TokenPosition) -> CompileError {
        CompileError {
            value:    CompileErrorValue::Constant(value.to_owned()),
            position: Some(position),
        }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.value {
            CompileErrorValue::Constant(ref s) => write!(f, "{}", s),
        }
    }
}
