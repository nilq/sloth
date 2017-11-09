use std::hash::{Hash, Hasher};
use std::mem;

#[derive(Clone, Copy, PartialEq)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    Char(char),
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match *self {
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
        }
    }
}

impl Value {
    pub fn truthy(self) -> bool {
        self != Value::Bool(false)
    }
}
