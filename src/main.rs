extern crate colored;
use colored::*;

use std::rc::Rc;

mod sloth;
use sloth::*;

fn main() {
    let test = r#"
a := "hey"
a
    "#;

    let lexer = lexer(&mut test.chars());

    let traveler   = Traveler::new(lexer.collect());
    let mut parser = Parser::new(traveler);

    match parser.parse() {
        Err(err)  => match err {
            ParserError {ref value, ref position} => {
                match *position {
                    Some(ref pos) => {
                        let mut lines = test.lines();

                        for i in 0 .. pos.line - 1 {
                            if i == pos.line - 2 {
                                let source_pos = format!("ln {}      | ", pos.line - 1).yellow();
                                match lines.next() {
                                    Some(line) => println!("{}{}", source_pos, line),
                                    None       => unreachable!(),
                                }
                            } else {
                                lines.next();
                            }
                        }

                        let source_pos = format!("ln {}, cl {}| ", pos.line, pos.col).yellow();

                        match lines.next() {
                            Some(line) => println!("{}{}", source_pos, line),
                            None       => unreachable!(),
                        }

                        let mut error = String::from("");

                        for _ in 0 .. pos.col + source_pos.len() {
                            error.push_str(" ")
                        }

                        error.push_str("^ ");

                        match *value {
                            ParserErrorValue::Constant(ref a) => error.push_str(a),
                        }

                        println!("{}", error.red());
                    },

                    None => (),
                }
            },
        },
        Ok(stuff) => {
            let symtab  = Rc::new(SymTab::new_global());
            let typetab = Rc::new(TypeTab::new_global());
            
            let root = Expression::Block(stuff);

            match root.visit(&symtab, &typetab) {
                Err(err) => match err {
                    CheckError {ref value, ref position} => {
                        match *position {
                            Some(ref pos) => {
                                let mut lines = test.lines();

                                for i in 0 .. pos.line - 1 {
                                    if i == pos.line - 2 {
                                        let source_pos = format!("          | ").yellow();
                                        match lines.next() {
                                            Some(line) => println!("{}{}", source_pos, line),
                                            None       => unreachable!(),
                                        }
                                    } else {
                                        lines.next();
                                    }
                                }

                                let source_pos = format!("ln {}, cl {}| ", pos.line, pos.col).yellow();

                                match lines.next() {
                                    Some(line) => println!("{}{}", source_pos, line),
                                    None       => unreachable!(),
                                }

                                let mut error = String::from("");

                                for _ in 0 .. pos.col + source_pos.len() {
                                    error.push_str(" ")
                                }

                                error.push_str("^ ");

                                match *value {
                                    CheckErrorValue::Constant(ref a) => error.push_str(a),
                                }
                                
                                println!("{}", error.red());
                                
                            },
                            
                            None => (),
                        }
                    },
                },
                _ => {
                    let mut vm       = VirtualMachine::new();
                    let mut compiler = Compiler::new(vm.clone());

                    match compiler.compile(&root) {
                        Ok(compiled) => vm.execute(&compiled),
                        Err(err)     => println!("{}", err),
                    }
                },
            }
        }
    }
}
