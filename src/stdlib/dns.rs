use crate::value::Value;
use std::process::Command;
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "lookup" => {
            let host = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("dns.lookup: host".into()) };
            let out = Command::new("nslookup").arg(&host).output().map_err(|e| e.to_string())?;
            Ok(Value::Str(String::from_utf8_lossy(&out.stdout).to_string()))
        }
        "resolve" => {
            let host = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("dns.resolve: host".into()) };
            let out = Command::new("host").arg(&host).output().map_err(|e| e.to_string())?;
            Ok(Value::Str(String::from_utf8_lossy(&out.stdout).trim().to_string()))
        }
        "mx" => {
            let domain = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("dns.mx: domain".into()) };
            let out = Command::new("dig").args(["+short", "MX", &domain]).output().map_err(|e| e.to_string())?;
            Ok(Value::Str(String::from_utf8_lossy(&out.stdout).trim().to_string()))
        }
        "is_valid_domain" => {
            let s = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("dns.is_valid_domain".into()) };
            let valid = s.contains('.') && s.len() > 3 && !s.starts_with('.') && !s.ends_with('.');
            Ok(Value::Bool(valid))
        }
        _ => Err(format!("dns.{}: unknown", func))
    }
}
