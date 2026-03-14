use crate::value::Value;
use std::process::Command;
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "kill" => {
            let pid = match args.first() { Some(Value::Int(n)) => *n, _ => return Err("signal2.kill: pid".into()) };
            let sig = match args.get(1) { Some(Value::Int(n)) => *n, _ => 15 };
            let ok = Command::new("kill").args([&format!("-{}", sig), &pid.to_string()]).status().map(|s| s.success()).unwrap_or(false);
            Ok(Value::Bool(ok))
        }
        "terminate" => {
            let pid = match args.first() { Some(Value::Int(n)) => *n, _ => return Err("signal2.terminate: pid".into()) };
            let ok = Command::new("kill").args(["-15", &pid.to_string()]).status().map(|s| s.success()).unwrap_or(false);
            Ok(Value::Bool(ok))
        }
        "interrupt" => {
            let pid = match args.first() { Some(Value::Int(n)) => *n, _ => return Err("signal2.interrupt: pid".into()) };
            let ok = Command::new("kill").args(["-2", &pid.to_string()]).status().map(|s| s.success()).unwrap_or(false);
            Ok(Value::Bool(ok))
        }
        "names" => Ok(Value::Array(["HUP","INT","QUIT","ILL","TRAP","ABRT","FPE","KILL","SEGV","PIPE","ALRM","TERM"].iter().map(|s| Value::Str(s.to_string())).collect())),
        _ => Err(format!("signal2.{}: unknown", func))
    }
}
