use std::hash::{Hash, Hasher};
use std::mem;
use std::rc::Rc;

use super::*;

pub enum HeapKind {
    Str(Rc<String>),
    Function(CompiledBlock),
}

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
