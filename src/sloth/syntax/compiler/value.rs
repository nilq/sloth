use std::hash::{Hash, Hasher};
use std::mem;
use std::rc::Rc;
use std::fmt::*;

use super::*;

#[derive(Debug, Clone)]
pub enum HeapKind {
    Str(Rc<String>),
    Function(CompiledBlock),
}

#[derive(Debug, Clone)]
pub struct HeapObject {
    pub next:   *mut HeapObject,
    pub marked: bool,
    pub kind:   HeapKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    Char(char),
    Null,
    HeapObject(*mut HeapObject),
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match *self {
            Value::Null => state.write_u8(0),

            Value::Bool(b) => {
                state.write_u8(1);
                state.write_u8(b as u8)
            }

            Value::Int(n) => {
                state.write_u8(2);
                state.write_u64(unsafe {mem::transmute(n)})
            }

            Value::Float(n) => {
                state.write_u8(2);
                state.write_u64(unsafe {mem::transmute(n)})
            }

            Value::Char(n) => {
                state.write_u8(2);
                state.write_u32(unsafe {mem::transmute(n)})
            }

            Value::HeapObject(p) => {
                state.write_u8(3);
                state.write_usize(p as usize);
            }
        }
    }
}

impl Value {
    pub fn truthy(self) -> bool {
        self != Value::Null && self != Value::Bool(false)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            Value::Null          => write!(f, "null")?,
            Value::Bool(b)       => write!(f, "{}", b)?,
            Value::Int(n)        => write!(f, "{}", n)?,
            Value::Float(n)      => write!(f, "{}", n)?,
            Value::Char(n)       => write!(f, "{}", n)?,
            Value::HeapObject(p) => {
                let obj = unsafe { &*p };
                match obj.kind {
                    HeapKind::Str(ref s)         => write!(f, "{}", s)?,
                    HeapKind::Function(_) => write!(f, "lambda<{:#?}>", p)?,
                }
            }
        }

        Ok(())
    }
}
