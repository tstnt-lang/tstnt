use crate::value::Value;
use std::process;

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "exit" => {
            let code = match args.first() {
                Some(Value::Int(n)) => *n as i32,
                _ => 0,
            };
            process::exit(code);
        }
        "run" => match args.first() {
            Some(Value::Str(cmd)) => {
                let parts: Vec<&str> = cmd.split_whitespace().collect();
                if parts.is_empty() { return Err("process.run: empty command".into()); }
                let out = process::Command::new(parts[0])
                    .args(&parts[1..])
                    .output()
                    .map_err(|e| e.to_string())?;
                Ok(Value::Str(String::from_utf8_lossy(&out.stdout).to_string()))
            }
            _ => Err("process.run: expected str".into())
        },
        "run_status" => match args.first() {
            Some(Value::Str(cmd)) => {
                let parts: Vec<&str> = cmd.split_whitespace().collect();
                if parts.is_empty() { return Err("process.run_status: empty command".into()); }
                let status = process::Command::new(parts[0])
                    .args(&parts[1..])
                    .status()
                    .map_err(|e| e.to_string())?;
                Ok(Value::Int(status.code().unwrap_or(-1) as i64))
            }
            _ => Err("process.run_status: expected str".into())
        },
        "pid" => Ok(Value::Int(process::id() as i64)),
        _ => Err(format!("process.{}: unknown function", func))
    }
}
