use std::rc::Rc;

pub mod symtab;
pub mod typetab;
pub mod error;

pub use super::*;

pub use self::symtab::*;
pub use self::typetab::*;
pub use self::error::*;

pub type CheckResult<T> = Result<T, CheckError>;

pub trait Visitor {
    fn visit(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>) -> CheckResult<()>;
}

pub trait Typer {
    fn get_type(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>) -> CheckResult<Type>;
}

impl Visitor for Expression {
    fn visit(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>) -> CheckResult<()> {
        match *self {
            Expression::Block(ref statements) => {
                for s in statements {
                    s.visit(sym, env)?
                }
                Ok(())
            },

            Expression::Identifier(ref id, ref position) => match sym.get_name(&*id) {
                None    => Err(CheckError::new_pos("undeclared use", position.clone())),
                Some(_) => Ok(())
            },

            Expression::Operation(ref operation) => operation.visit(sym, env),
            Expression::Function(ref function)   => function.visit(sym, env),

            _ => Ok(())
        }
    }
}

impl Typer for Expression {
    fn get_type(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>) -> CheckResult<Type> {
        match *self {
            Expression::Int(_)            => Ok(Type::Int),
            Expression::Float(_)          => Ok(Type::Float),
            Expression::Str(_)            => Ok(Type::Str),
            Expression::Char(_)           => Ok(Type::Char),
            Expression::Bool(_)           => Ok(Type::Bool),
            Expression::Identifier(ref n, ref position) => match sym.get_name(&*n) {
                Some((i, env_index)) => Ok(env.get_type(i, env_index).unwrap()),
                None                 => Err(CheckError::new_pos("can't get type of undeclared", position.clone())),
            },
            _ => Ok(Type::Undefined),
        }
    }
}

impl Visitor for Operation {
    fn visit(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>) -> CheckResult<()> {
        self.left.visit(sym, env)?;
        self.right.visit(sym, env)
    }
}

impl Visitor for Function {
    fn visit(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>) -> CheckResult<()> {
        for arm in &self.arms {
            let mut param_names = Vec::new();
            
            match **arm {
                Expression::Arm(ref arm) => {
                    for p in &arm.params {
                        match **p {
                            Expression::Identifier(ref i, _) => param_names.push(i.clone()),
                            _ => (),
                        }
                    }

                    let local_sym = Rc::new(SymTab::new(sym.clone(), param_names.as_slice()));
                    let local_env = Rc::new(TypeTab::new(env.clone(), &arm.params.iter().map(|_| Type::Any).collect()));

                    arm.body.visit(&local_sym, &local_env)?;
                },
                
                ref c => c.visit(sym, env)?
            }
        }

        Ok(())
    }
}

impl Visitor for Statement {
    fn visit(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>) -> CheckResult<()> {
        match *self {
            Statement::Expression(ref e) => e.visit(sym, env),
            Statement::Definition(ref b) => b.visit(sym, env),
            _ => Ok(())
        }
    }
}

impl Visitor for Definition {
    fn visit(&self, sym: &Rc<SymTab>, env: &Rc<TypeTab>) -> CheckResult<()> {
        match *self.name {
            Expression::Identifier(ref name, _) => {
                let index = sym.add_name(name);
                if index >= env.size() {
                    env.grow();
                }

                match self.right {
                    Some(ref right) => right.visit(&sym, &env),
                    None => Ok(()),
                }
            }

            _ => Err(CheckError::new_pos("unexpected declaration", self.position)),
        }
    }
}
