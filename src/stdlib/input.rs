use crate::value::Value;
use std::io::{self, Write, BufRead};

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "read" => {
            if let Some(prompt) = args.first() { print!("{}", prompt); io::stdout().flush().ok(); }
            let mut s = String::new();
            io::stdin().lock().read_line(&mut s).ok();
            Ok(Value::Str(s.trim().to_string()))
        }
        "read_int" => {
            if let Some(prompt) = args.first() { print!("{}", prompt); io::stdout().flush().ok(); }
            let mut s = String::new();
            io::stdin().lock().read_line(&mut s).ok();
            Ok(Value::Int(s.trim().parse().unwrap_or(0)))
        }
        "read_float" => {
            if let Some(prompt) = args.first() { print!("{}", prompt); io::stdout().flush().ok(); }
            let mut s = String::new();
            io::stdin().lock().read_line(&mut s).ok();
            Ok(Value::Float(s.trim().parse().unwrap_or(0.0)))
        }
        "confirm" => {
            let msg = args.first().map(|v| v.to_string()).unwrap_or_else(|| "Continue? (y/n): ".into());
            print!("{}", msg); io::stdout().flush().ok();
            let mut s = String::new();
            io::stdin().lock().read_line(&mut s).ok();
            Ok(Value::Bool(matches!(s.trim().to_lowercase().as_str(), "y" | "yes" | "да")))
        }
        "menu" => {
            // input.menu("Choose:", ["Option 1", "Option 2"]) -> int (index)
            let title = args.first().map(|v| v.to_string()).unwrap_or_default();
            let options = match args.get(1) { Some(Value::Array(a)) => a.clone(), _ => return Err("input.menu: (title, [options])".into()) };
            println!("{}", title);
            for (i, opt) in options.iter().enumerate() { println!("  {}. {}", i+1, opt); }
            print!("> "); io::stdout().flush().ok();
            let mut s = String::new();
            io::stdin().lock().read_line(&mut s).ok();
            let choice: usize = s.trim().parse::<usize>().unwrap_or(1).saturating_sub(1);
            Ok(Value::Int(choice.min(options.len().saturating_sub(1)) as i64))
        }
        _ => Err(format!("input.{}: unknown", func))
    }
}
