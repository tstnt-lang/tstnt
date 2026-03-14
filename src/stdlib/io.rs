use std::io::{self, BufRead, Write};
use std::fs;
use crate::value::Value;

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "print" => {
            let parts: Vec<String> = args.iter().map(|v| v.to_string()).collect();
            println!("{}", parts.join(" "));
            Ok(Value::Null)
        }
        "input" => {
            if let Some(Value::Str(prompt)) = args.first() {
                print!("{}", prompt);
                io::stdout().flush().ok();
            }
            let stdin = io::stdin();
            let mut line = String::new();
            stdin.lock().read_line(&mut line).ok();
            Ok(Value::Str(line.trim_end().to_string()))
        }
        "read_file" => {
            match args.first() {
                Some(Value::Str(path)) => {
                    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
                    Ok(Value::Str(content))
                }
                _ => Err("read_file: expected str path".into())
            }
        }
        "write_file" => {
            match (args.first(), args.get(1)) {
                (Some(Value::Str(path)), Some(Value::Str(content))) => {
                    fs::write(path, content).map_err(|e| e.to_string())?;
                    Ok(Value::Null)
                }
                _ => Err("write_file: expected (str, str)".into())
            }
        }
        _ => Err(format!("io.{}: unknown function", func))
    }
}
