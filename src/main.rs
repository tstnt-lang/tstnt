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
        Some("watch") => {
            if let Some(f) = args.get(2) {
                let f = f.clone();
                println!("Watching: {}", f);
                let mut last_mod = std::fs::metadata(&f).and_then(|m| m.modified()).ok();
                run_file(&f, false, debug);
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    if let Ok(meta) = std::fs::metadata(&f) {
                        if let Ok(modified) = meta.modified() {
                            if Some(modified) != last_mod {
                                last_mod = Some(modified);
                                println!("\n--- Restarting ---\n");
                                run_file(&f, false, debug);
                            }
                        }
                    }
                }
            }
        }
        Some("transpile") => {
            if let Some(f) = args.get(2) {
                let target = args.get(3).map(|s| s.as_str()).unwrap_or("py");
                let src = fs::read_to_string(f).unwrap_or_default();
                let tokens = Lexer::new(&src).tokenize();
                let ast = Parser::new(tokens).parse().unwrap_or_default();
                let out = match target {
                    "py" | "python" => transpiler::to_python(&ast),
                    "js" | "javascript" => transpiler::to_js(&ast),
                    _ => { eprintln!("Unknown target: {} (use py or js)", target); return; }
                };
                let ext = if target.starts_with("py") { "py" } else { "js" };
                let outfile = f.replace(".tstnt", &format!(".{}", ext));
                fs::write(&outfile, &out).ok();
                println!("Transpiled to: {}", outfile);
            }
        }
        Some("version") | Some("--version") | Some("-v") => {
            println!("[36m");
            println!(r"  _____  ___  _____  _  _ _____");
            println!(r" |_   _|/ __||_   _|| \| |_   _|");
            println!(r"   | |  \__ \  | |  | .` | | |  ");
            println!(r"   |_|  |___/  |_|  |_|\_| |_|  ");
            println!("[0m");
            println!("  v0.8.0  [90m— The TSTNT Language[0m");
            println!("  [90mgithub.com/tstnt-lang[0m
");
        }
        Some("--secret") => {
            println!("[33m🐉 You found the secret![0m");
            println!("  TSTNT was built with love on Android.");
        }
        Some(f) => run_file(f, false, debug),
        None => repl::run(),
    }
}
