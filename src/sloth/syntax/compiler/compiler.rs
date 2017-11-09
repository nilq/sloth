use std::collections::HashMap;
use std::mem;

use std::collections::hash_map::Entry;

use super::*;

#[derive(Clone, Copy)]
struct JumpPatch(usize);

#[derive(Clone, Copy)]
struct BranchTarget(usize);

#[derive(Debug, Clone)]
pub struct CompiledBlock {
    pub code:   Box<[OpCode]>,
    pub consts: Box<[Value]>,
    pub locals: Box<[String]>,
}

pub struct Compiler {
    locals: HashMap<String, u16>,
    code:   Vec<OpCode>,
    consts: Vec<Value>,
    vm:     VirtualMachine,
}

impl Compiler {
    pub fn new(vm: VirtualMachine) -> Compiler {
        Compiler {
            locals: HashMap::new(),
            code:   Vec::new(),
            consts: Vec::new(),
            vm,
        }
    }

    pub fn compile(&mut self, block: &Expression) -> CompileResult<CompiledBlock> {
        self.compile_main(block)
    }
    
    fn declare_local(&mut self, name: &str) -> CompileResult<u16> {
        let i = self.locals.len();
        
        if i > (u16::max_value() as usize) {
            return Err(CompileError::new(&format!("local overflow: {}", u16::max_value())))
        }
        
        let i = i as u16;
        let e = self.locals.entry(name.to_string());
        
        match e {
            Entry::Occupied(_) => Err(CompileError::new(&format!("redeclared local: {}", name))),
            Entry::Vacant(v)   => {
                v.insert(i);
                Ok(i)
            }
        }
    }
    
    fn fetch_local(&mut self, name: &str) -> CompileResult<u16> {
        self.locals.get(name).map(|i| *i).ok_or(CompileError::new(&format!("undeclared local: {}", name)))
    }
    
    fn emit(&mut self, op: OpCode) {
        self.code.push(op)
    }
    
    fn emit_load_const(&mut self, value: Value) -> CompileResult<()> {
        let i = self.consts.len();
        
        if i > (u16::max_value() as usize) {
            Err(CompileError::new(&format!("constant overflow: {}", u16::max_value())))
        } else {
            let i = i as u16;
            
            self.consts.push(value);
            self.emit(OpCode::LoadConst(i));

            Ok(())
        }
    }
    
    fn emit_branch_false(&mut self) -> JumpPatch {
        let result = JumpPatch(self.code.len());
        self.emit(OpCode::BranchFalse(0));
        result
    }

    fn emit_branch_true(&mut self) -> JumpPatch {
        let result = JumpPatch(self.code.len());
        self.emit(OpCode::BranchTrue(0));
        result
    }

    fn emit_jump(&mut self) -> JumpPatch {
        let result = JumpPatch(self.code.len());
        self.emit(OpCode::Jump(0));
        result
    }

    fn save_branch_target(&self) -> BranchTarget {
        BranchTarget(self.code.len())
    }

    fn patch_jump(&mut self, patch: JumpPatch) -> CompileResult<()> {
        let cur        = self.code.len();
        let branch_loc = patch.0;
        let diff       = (cur as isize) - (branch_loc as isize);

        if diff > (i16::max_value() as isize) || diff < (i16::min_value() as isize) {
            Err(CompileError::new("branch went too far"))
        } else {
            let diff = diff as i16;

            match self.code[branch_loc] {
                OpCode::Jump(_)        => self.code[branch_loc] = OpCode::Jump(diff),
                OpCode::BranchTrue(_)  => self.code[branch_loc] = OpCode::BranchTrue(diff),
                OpCode::BranchFalse(_) => self.code[branch_loc] = OpCode::BranchFalse(diff),
                _                      => unreachable!(),
            }

            Ok(())
        }
    }

    fn compile_statement(&mut self, s: &Statement) -> CompileResult<()> {
        match *s {
            Statement::Definition(ref def) => {
                let id = match *def.name {
                    Expression::Identifier(ref i, _) => i,
                    _                                => unreachable!(),
                };

                match def.right {
                    Some(ref e) => {
                        self.compile_expression(&e)?;
                        let i = self.declare_local(id)?;
                        self.emit(OpCode::StoreLocal(i))
                    },
                    None    => { self.declare_local(id)?; },
                }

                Ok(())
            },

            Statement::Assignment(ref ass) => {
                let id = match *ass.left {
                    Expression::Identifier(ref i, _) => i,
                    _                                => unreachable!(),
                };

                self.compile_expression(&ass.right)?;
                let i = self.fetch_local(id)?;
                self.emit(OpCode::StoreLocal(i));

                Ok(())
            },

            Statement::Expression(ref e) => {
                self.compile_expression(e)?;
                self.emit(OpCode::Pop);

                Ok(())
            }
        }
    }
    
    fn compile_expression(&mut self, e: &Expression) -> CompileResult<()> {
        match *e {
            Expression::Int(ref n)   => self.emit_load_const(Value::Int(*n)),
            Expression::Float(ref n) => self.emit_load_const(Value::Float(*n)),
            Expression::Bool(ref n)  => self.emit_load_const(Value::Bool(*n)),
            Expression::Char(ref n)  => self.emit_load_const(Value::Char(*n)),
            Expression::Str(ref n)   => {
                let value = self.vm.allocate_object(HeapKind::Str((*n).clone()));
                self.emit_load_const(value)
            },
            Expression::Identifier(ref id, _) => {
                let i = self.fetch_local(id)?;
                self.emit(OpCode::LoadLocal(i));
                Ok(())
            },
            
            Expression::Block(ref statements) => {
                for s in statements {
                    self.compile_statement(s)?;
                }
                Ok(())
            },

            _ => Ok(()),
        }
    }

    fn compile_main(&mut self, block: &Expression) -> CompileResult<CompiledBlock> {
        self.compile_expression(block)?;
        self.emit_load_const(Value::Null)?;
        self.code.push(OpCode::Return);
        
        let mut locals = vec![String::new(); self.locals.len()];
        
        for (name, i) in self.locals.drain() {
            locals[i as usize] = name
        }
        
        Ok(CompiledBlock {
            code: mem::replace(&mut self.code, Vec::new()).into_boxed_slice(),
            consts: mem::replace(&mut self.consts, Vec::new()).into_boxed_slice(),
            locals: locals.into_boxed_slice(),
        })
    }
}
