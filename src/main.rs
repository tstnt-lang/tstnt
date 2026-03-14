mod lexer; mod parser; mod value; mod interpreter; mod stdlib;
mod repl; mod formatter; mod pkg; mod compiler;
mod vm;
mod transpiler;

use std::fs;
use lexer::Lexer;
use parser::Parser;
use interpreter::Interpreter;

fn pretty_error(src: &str, err: &str) {
    eprintln!("\x1b[31merror\x1b[0m: {}", err);
    // Извлекаем номер строки из "Line N:"
    if let Some(rest) = err.strip_prefix("Line ") {
        if let Some(colon) = rest.find(':') {
            if let Ok(line_num) = rest[..colon].parse::<usize>() {
                let lines: Vec<&str> = src.lines().collect();
                if let Some(line) = lines.get(line_num.saturating_sub(1)) {
                    eprintln!("  \x1b[34m{}\x1b[0m | {}", line_num, line);
                    eprintln!("    {} \x1b[31m^\x1b[0m", " ".repeat(line_num.to_string().len()));
                }
            }
        }
    }
}

fn run_file(path: &str, test_mode: bool, debug: bool) {
    let src = match fs::read_to_string(path) {
        Ok(s) => s, Err(e) => { eprintln!("\x1b[31merror\x1b[0m: {}", e); std::process::exit(1); }
    };
    let tokens = Lexer::new(&src).tokenize();
    let ast = match Parser::new(tokens).parse() {
        Ok(a) => a, Err(e) => { pretty_error(&src, &e); std::process::exit(1); }
    };
    let mut interp = Interpreter::new();
    interp.debug = debug;
    let result = if test_mode { interp.run_tests(&ast) } else { interp.run(&ast) };
    if let Err(e) = result { pretty_error(&src, &e); std::process::exit(1); }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let debug = args.contains(&"--debug".to_string());
    match args.get(1).map(|s| s.as_str()) {
        Some("repl") => repl::run(),
        Some("fmt") => { if let Some(f) = args.get(2) { if let Ok(s) = fs::read_to_string(f) { println!("{}", formatter::format(&s)); } } }
        Some("test") => { if let Some(f) = args.get(2) { run_file(f, true, debug); } else { eprintln!("Usage: tstnt test <file>"); } }
        Some("build") => {
            if let Some(f) = args.get(2) {
                let src = fs::read_to_string(f).unwrap_or_default();
                let tokens = Lexer::new(&src).tokenize();
                let ast = Parser::new(tokens).parse().unwrap_or_default();
                let out = f.replace(".tstnt", ".tst");
                let bytes = compiler::compile(&ast); std::fs::write(&out, bytes).ok();
                println!("Built: {}", out);
            }
        }
        Some("run") => { if let Some(f) = args.get(2) { run_file(f, false, debug); } }
        Some("pkg") => {
            match args.get(2).map(|s| s.as_str()) {
                Some("install") => pkg::install(args.get(3).map(|s| s.as_str()).unwrap_or("")),
                Some("uninstall") => pkg::uninstall(args.get(3).map(|s| s.as_str()).unwrap_or("")),
                Some("list") => pkg::list(),
                Some("search") => pkg::search(args.get(3).map(|s| s.as_str()).unwrap_or("")),
                _ => eprintln!("Usage: tstnt pkg [install|uninstall|list|search]"),
            }
        }
        Some("version") => println!("tstnt v0.4.0"),
        Some(f) => run_file(f, false, debug),
        None => repl::run(),
    }
}
