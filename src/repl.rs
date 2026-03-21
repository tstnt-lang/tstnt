use std::io::{self, Write};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::interpreter::Interpreter;

pub fn run() {
    let mut interp = Interpreter::new();
    let mut history: Vec<String> = Vec::new();
    println!("TSTNT REPL v{}  (exit/quit to leave, .history to show history)", env!("CARGO_PKG_VERSION"));
    loop {
        print!(">> ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() { break; }
        let input = line.trim().to_string();
        if input.is_empty() { continue; }
        match input.as_str() {
            "exit" | "quit" => break,
            ".history" => { for (i, h) in history.iter().enumerate() { println!("{}: {}", i + 1, h); } continue; }
            ".clear" => { history.clear(); println!("History cleared."); continue; }
            _ => {}
        }
        history.push(input.clone());
        let tokens = Lexer::new(&input).tokenize();
        let ast = match Parser::new(tokens).parse() {
            Ok(a) => a,
            Err(e) => { eprintln!("Parse error: {}", e); continue; }
        };
        for node in &ast {
            match interp.eval(node) {
                Ok(val) => {
                    use crate::value::Value;
                    if !matches!(val, Value::Null) { println!("{}", val); }
                }
                Err(_) => {
                    if let Err(e2) = interp.run(&ast) {
                        eprintln!("Error: {}", e2);
                    }
                    break;
                }
            }
        }
    }
    println!("Bye.");
}
