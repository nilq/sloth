use std::mem;
use std::ptr;
use std::rc::Rc;

use super::*;

#[derive(Debug, Clone)]
pub struct CallInfo {
    locals: Box<[Value]>,
    pc:     usize,
    func:   *const CompiledBlock,
}


#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    Add,
    Sub,
    Mul,
    Rem,
    Div,
    
    Neg,
    
    Lt,
    Gt,
    LtEq,
    GtEq,
    Eq,
    NotEq,
    
    LoadConst(u16),
    LoadLocal(u16),
    StoreLocal(u16),
    
    BranchTrue(i16),
    BranchFalse(i16),
    Jump(i16),
    
    Pop,
    Return,
    Call(u8),
    Print,
}

#[derive(Debug, Clone)]
pub struct VirtualMachine {
    pub value_stack: Vec<Value>,
    pub call_stack:  Vec<CallInfo>,
    pub next_object: *mut HeapObject,
}

impl VirtualMachine {
    pub fn new() -> VirtualMachine {
        VirtualMachine {
            value_stack: Vec::new(),
            call_stack:  Vec::new(),
            next_object: ptr::null_mut(),
        }
    }
    
    pub fn execute(&mut self, initial_block: *const CompiledBlock) {
        let mut pc = 0;
        let mut func = unsafe {&*initial_block};
        let mut locals = vec![Value::Null; func.locals.len()].into_boxed_slice();
        
        macro_rules! match_binop {
            ($($pat:pat => $block:block)+) => {{
                let _a = self.value_stack.pop().unwrap();
                let _b = self.value_stack.pop().unwrap();

                let _result = match (_b, _a) {
                    $($pat => $block)+,
                    _ => panic!("unexpected operand"),
                };

                self.value_stack.push(_result);
            }}
        }
        
        loop {
            if pc >= func.code.len() {
                break
            }

            let op = func.code[pc];
            
            println!("{:?}", op);

            match op {
                OpCode::LoadConst(i)  => { self.value_stack.push(func.consts[i as usize]); },
                OpCode::LoadLocal(i)  => { self.value_stack.push(locals[i as usize]); },
                OpCode::StoreLocal(i) => locals[i as usize] = self.value_stack.pop().unwrap(),
                
                OpCode::BranchTrue(d) => {
                    if self.value_stack.pop().unwrap().truthy() {
                        pc = pc.wrapping_add((d as isize) as usize)
                    } else {
                        pc = pc.wrapping_add(1)
                    }
                    
                    continue
                }

                OpCode::BranchFalse(d) => {
                    if !self.value_stack.pop().unwrap().truthy() {
                        pc = pc.wrapping_add((d as isize) as usize)
                    } else {
                        pc = pc.wrapping_add(1)
                    }
                    
                    continue
                }
                
                OpCode::Jump(d) => {
                    pc = pc.wrapping_add((d as isize) as usize);
                    continue
                }

                OpCode::Pop => { self.value_stack.pop().unwrap(); },
                
                OpCode::Add => {
                    let a = self.value_stack.pop().unwrap();
                    let b = self.value_stack.pop().unwrap();

                    let result = match (b, a) {
                        (Value::Int(a), Value::Int(b))     => Value::Int(a + b),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
                        (Value::Float(a), Value::Int(b))   => Value::Float(a + b as f64),
                        _ => panic!("unexpected operand"),
                    };

                    self.value_stack.push(result)
                }

                OpCode::Sub => {
                    let a = self.value_stack.pop().unwrap();
                    let b = self.value_stack.pop().unwrap();

                    let result = match (b, a) {
                        (Value::Int(a), Value::Int(b))     => Value::Int(a - b),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
                        (Value::Float(a), Value::Int(b))   => Value::Float(a - b as f64),
                        _ => panic!("unexpected operand"),
                    };

                    self.value_stack.push(result)
                }

                OpCode::Rem => {
                    let a = self.value_stack.pop().unwrap();
                    let b = self.value_stack.pop().unwrap();

                    let result = match (b, a) {
                        (Value::Int(a), Value::Int(b))     => Value::Int(a % b),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a % b),
                        _ => panic!("unexpected operand"),
                    };

                    self.value_stack.push(result)
                }

                OpCode::Div => {
                    let a = self.value_stack.pop().unwrap();
                    let b = self.value_stack.pop().unwrap();

                    let result = match (b, a) {
                        (Value::Int(a), Value::Int(b))     => Value::Int(a / b),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a / b),
                        (Value::Float(a), Value::Int(b))   => Value::Float(a / b as f64),
                        _ => panic!("unexpected operand"),
                    };

                    self.value_stack.push(result)
                }

                OpCode::Lt   => {
                    let a = self.value_stack.pop().unwrap();
                    let b = self.value_stack.pop().unwrap();

                    let result = match (b, a) {
                        (Value::Int(a), Value::Int(b))     => Value::Bool(a < b),
                        (Value::Float(a), Value::Float(b)) => Value::Bool(a < b),
                        (Value::Float(a), Value::Int(b))   => Value::Bool(a < b as f64),
                        (Value::Int(a), Value::Float(b))   => Value::Bool(a < b as i64),
                        _ => panic!("unexpected operand"),
                    };

                    self.value_stack.push(result)
                }

                OpCode::Gt   => {
                    let a = self.value_stack.pop().unwrap();
                    let b = self.value_stack.pop().unwrap();

                    let result = match (b, a) {
                        (Value::Int(a), Value::Int(b))     => Value::Bool(a > b),
                        (Value::Float(a), Value::Float(b)) => Value::Bool(a > b),
                        (Value::Float(a), Value::Int(b))   => Value::Bool(a > b as f64),
                        (Value::Int(a), Value::Float(b))   => Value::Bool(a > b as i64),
                        _ => panic!("unexpected operand"),
                    };

                    self.value_stack.push(result)
                }

                OpCode::LtEq => {
                    let a = self.value_stack.pop().unwrap();
                    let b = self.value_stack.pop().unwrap();

                    let result = match (b, a) {
                        (Value::Int(a), Value::Int(b))     => Value::Bool(a <= b),
                        (Value::Float(a), Value::Float(b)) => Value::Bool(a <= b),
                        (Value::Float(a), Value::Int(b))   => Value::Bool(a <= b as f64),
                        (Value::Int(a), Value::Float(b))   => Value::Bool(a <= b as i64),
                        _ => panic!("unexpected operand"),
                    };

                    self.value_stack.push(result)
                }

                OpCode::GtEq => {
                    let a = self.value_stack.pop().unwrap();
                    let b = self.value_stack.pop().unwrap();

                    let result = match (b, a) {
                        (Value::Int(a), Value::Int(b))     => Value::Bool(a >= b),
                        (Value::Float(a), Value::Float(b)) => Value::Bool(a >= b),
                        (Value::Float(a), Value::Int(b))   => Value::Bool(a >= b as f64),
                        (Value::Int(a), Value::Float(b))   => Value::Bool(a >= b as i64),
                        _ => panic!("unexpected operand"),
                    };

                    self.value_stack.push(result)
                }

                OpCode::Eq => {
                    let a = self.value_stack.pop().unwrap();
                    let b = self.value_stack.pop().unwrap();

                    self.value_stack.push(Value::Bool(a == b));
                }

                OpCode::NotEq => {
                    let a = self.value_stack.pop().unwrap();
                    let b = self.value_stack.pop().unwrap();

                    self.value_stack.push(Value::Bool(a != b));
                }

                OpCode::Neg => {
                    if let Value::Int(n) = self.value_stack.pop().unwrap() {
                        self.value_stack.push(Value::Int(-n));
                    } else if let Value::Float(n) = self.value_stack.pop().unwrap() {
                        self.value_stack.push(Value::Float(-n));
                    } else {
                        panic!("unexpected operand type");
                    }
                }
                
                OpCode::Call(args) => {
                    let args = args as usize;
                    
                    let func_i = self.value_stack.len() - args - 1;
                    let func_v = self.value_stack[func_i];
                    
                    let old_func = func;
                    
                    func = if let Value::HeapObject(p) = func_v {
                        let obj = unsafe {&*p};
                        
                        if let HeapKind::Function(ref func) = obj.kind {
                            func
                        } else {
                            panic!("calling non-func: {:#?}", func_v)
                        }
                    } else {
                        panic!("calling non-func: {:#?}", func_v)
                    };
                    
                    let mut new_locals = vec![Value::Null; func.locals.len()].into_boxed_slice();
                    
                    for i in 0 .. args {
                        new_locals[i] = self.value_stack[func_i + 1 + i]
                    }
                    
                    for _ in 0 .. args + 1 {
                        self.value_stack.pop();
                    }

                    let old_locals = mem::replace(&mut locals, new_locals);
                    
                    self.call_stack.push(CallInfo {
                        pc,
                        locals: old_locals,
                        func: old_func,
                    });
                    
                    pc = 0;
                    
                    continue
                }
                
                OpCode::Print => {
                    println!("{}", self.value_stack.pop().unwrap());
                }
                
                _ => (),
            }

            pc = pc.wrapping_add(1)
        }
    }

    pub fn allocate_object(&mut self, kind: HeapKind) -> Value {
        let obj = Box::into_raw(Box::new(HeapObject {
            next: self.next_object,
            marked: false,
            kind,
        }));
        
        self.next_object = obj;

        Value::HeapObject(obj)
    }
}
