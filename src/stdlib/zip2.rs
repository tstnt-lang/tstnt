use crate::value::Value;
use std::process::Command;
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "compress" => {
            let src = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("zip2.compress: (src, dst)".into()) };
            let dst = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("zip2.compress: dst".into()) };
            let ok = Command::new("zip").args(["-r", &dst, &src]).status().map(|s| s.success()).unwrap_or(false);
            Ok(Value::Bool(ok))
        }
        "extract" => {
            let src = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("zip2.extract: (src, dst)".into()) };
            let dst = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => ".".to_string() };
            let ok = Command::new("unzip").args(["-o", &src, "-d", &dst]).status().map(|s| s.success()).unwrap_or(false);
            Ok(Value::Bool(ok))
        }
        "list" => {
            let src = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("zip2.list: path".into()) };
            let out = Command::new("unzip").args(["-l", &src]).output().map_err(|e| e.to_string())?;
            Ok(Value::Str(String::from_utf8_lossy(&out.stdout).to_string()))
        }
        _ => Err(format!("zip2.{}: unknown", func))
    }
}
