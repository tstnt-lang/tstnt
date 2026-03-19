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
    let cache_path = path.replace(".tstnt", ".tst_cache");
    let src_meta = fs::metadata(path).ok();
    let cache_meta = fs::metadata(&cache_path).ok();
    let use_cache = if let (Some(sm), Some(cm)) = (&src_meta, &cache_meta) {
        cm.modified().ok() > sm.modified().ok()
    } else { false };
    let ast = if use_cache && !debug {
        let bytes = fs::read(&cache_path).unwrap_or_default();
        let src2 = String::from_utf8_lossy(&bytes).to_string();
        let tokens = Lexer::new(&src2).tokenize();
        match Parser::new(tokens).parse() {
            Ok(a) => a,
            Err(_) => {
                let tokens = Lexer::new(&src).tokenize();
                match Parser::new(tokens).parse() { Ok(a) => a, Err(e) => { pretty_error(&src, &e); std::process::exit(1); } }
            }
        }
    } else {
        let tokens = Lexer::new(&src).tokenize();
        match Parser::new(tokens).parse() {
            Ok(a) => { fs::write(&cache_path, &src).ok(); a }
            Err(e) => { pretty_error(&src, &e); std::process::exit(1); }
        }
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
        Some("check") => {
            if let Some(f) = args.get(2) {
                let src = match fs::read_to_string(f) {
                    Ok(s) => s, Err(e) => { eprintln!("\x1b[31merror\x1b[0m: {}", e); std::process::exit(1); }
                };
                let tokens = Lexer::new(&src).tokenize();
                match Parser::new(tokens).parse() {
                    Ok(_) => println!("\x1b[32m✓\x1b[0m {} — no syntax errors", f),
                    Err(e) => { pretty_error(&src, &e); std::process::exit(1); }
                }
            } else { eprintln!("Usage: tstnt check <file>"); }
        }
        Some("bench") => {
            if let Some(f) = args.get(2) {
                let n: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(10);
                let src = match fs::read_to_string(f) {
                    Ok(s) => s, Err(e) => { eprintln!("\x1b[31merror\x1b[0m: {}", e); std::process::exit(1); }
                };
                println!("\x1b[36mbench\x1b[0m running {} x{}", f, n);
                let tokens = Lexer::new(&src).tokenize();
                let ast = match Parser::new(tokens).parse() {
                    Ok(a) => a, Err(e) => { pretty_error(&src, &e); std::process::exit(1); }
                };
                let start = std::time::Instant::now();
                for _ in 0..n {
                    let mut interp = Interpreter::new();
                    let _ = interp.run(&ast);
                }
                let elapsed = start.elapsed().as_millis();
                println!("\x1b[32m{}\x1b[0m runs in \x1b[33m{}ms\x1b[0m total, \x1b[33m{}ms\x1b[0m avg",
                    n, elapsed, elapsed / n.max(1) as u128);
            } else { eprintln!("Usage: tstnt bench <file> [runs]"); }
        }
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
        Some("new") => {
            if let Some(name) = args.get(2) {
                let name = name.clone();
                let dirs = [format!("{}/src", name), format!("{}/tests", name)];
                for d in &dirs { std::fs::create_dir_all(d).ok(); }

                let main_code = format!("do main {{\n    print(\"Hello from {}!\")\n}}", name);
                fs::write(format!("{}/src/main.tstnt", name), &main_code).ok();

                let test_code = "test hello {\n    assert_eq(1 + 1, 2)\n}";
                fs::write(format!("{}/tests/test.tstnt", name), test_code).ok();

                let pkg_name = name.clone();
                let pkg_json = format!(
                    "{{\n  \"name\": \"{}\",\n  \"version\": \"0.1.0\",\n  \"main\": \"src/main.tstnt\"\n}}",
                    pkg_name
                );
                fs::write(format!("{}/pkg.json", name), &pkg_json).ok();

                let gitignore = "*.tst\n.env\n*.tsdb\n";
                fs::write(format!("{}/.gitignore", name), gitignore).ok();

                let readme = format!("# {}\n\nA TSTNT project.\n\n```bash\ntstnt src/main.tstnt\n```\n", name);
                fs::write(format!("{}/README.md", name), &readme).ok();

                println!("[32m✓[0m Created project: [1m{}[0m", name);
                println!("");
                println!("  {}/", name);
                println!("  ├── src/");
                println!("  │   └── main.tstnt");
                println!("  ├── tests/");
                println!("  │   └── test.tstnt");
                println!("  ├── pkg.json");
                println!("  ├── .gitignore");
                println!("  └── README.md");
                println!("");
                println!("  [90mcd {} && tstnt src/main.tstnt[0m", name);
            } else {
                eprintln!("Usage: tstnt new <project-name>");
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
            println!("  v1.1.0  [90m— The TSTNT Language[0m");
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
