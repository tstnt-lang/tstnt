use crate::value::Value;
use std::process::Command;
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "exec" => {
            let cmd = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("os.exec: str".into()) };
            let out = Command::new("sh").args(["-c", &cmd]).output().map_err(|e| e.to_string())?;
            Ok(Value::Str(String::from_utf8_lossy(&out.stdout).trim().to_string()))
        }
        "exec_err" => {
            let cmd = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("os.exec_err: str".into()) };
            let out = Command::new("sh").args(["-c", &cmd]).output().map_err(|e| e.to_string())?;
            Ok(Value::Str(String::from_utf8_lossy(&out.stderr).trim().to_string()))
        }
        "which" => {
            let bin = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("os.which: str".into()) };
            let out = Command::new("which").arg(&bin).output().map_err(|e| e.to_string())?;
            Ok(Value::Str(String::from_utf8_lossy(&out.stdout).trim().to_string()))
        }
        "getenv" => {
            let key = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("os.getenv: str".into()) };
            Ok(Value::Str(std::env::var(&key).unwrap_or_default()))
        }
        "setenv" => {
            let key = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("os.setenv: (key, val)".into()) };
            let val = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => String::new() };
            std::env::set_var(&key, &val); Ok(Value::Null)
        }
        "platform" => Ok(Value::Str(std::env::consts::OS.into())),
        "arch" => Ok(Value::Str(std::env::consts::ARCH.into())),
        _ => Err(format!("os.{}: unknown", func))
    }
}
