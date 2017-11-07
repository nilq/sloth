mod sloth;

use sloth::*;

fn main() {
    let test = r#"
fib := {
    |0|
    |1|
    |a| (fib a - 1) + fib a - 2
}
    "#;
    
    for t in lexer(&mut test.chars()) {
        println!("{:#?}", t);
    }
}
