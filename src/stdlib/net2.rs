use crate::value::Value;
use std::process::Command;
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "get" => {
            let url = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("net2.get: url".into()) };
            let out = Command::new("curl").args(["-s", "--max-time", "10", &url]).output().map_err(|e| e.to_string())?;
            Ok(Value::Str(String::from_utf8_lossy(&out.stdout).to_string()))
        }
        "post" => {
            let url = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("net2.post: url".into()) };
            let body = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => String::new() };
            let out = Command::new("curl").args(["-s", "-X", "POST", "-d", &body, &url]).output().map_err(|e| e.to_string())?;
            Ok(Value::Str(String::from_utf8_lossy(&out.stdout).to_string()))
        }
        "download" => {
            let url = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("net2.download: (url, path)".into()) };
            let path = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("net2.download: path".into()) };
            let status = Command::new("curl").args(["-s", "-o", &path, &url]).status().map_err(|e| e.to_string())?;
            Ok(Value::Bool(status.success()))
        }
        "ping" => {
            let host = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("net2.ping: host".into()) };
            let out = Command::new("ping").args(["-c", "1", "-W", "2", &host]).status().map_err(|e| e.to_string())?;
            Ok(Value::Bool(out.success()))
        }
        _ => Err(format!("net2.{}: unknown", func))
    }
}
