use crate::value::Value;
use std::collections::HashMap;
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "parse" => {
            let s = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("toml.parse: str".into()) };
            let mut map: HashMap<String, Value> = HashMap::new();
            let mut section = String::new();
            for line in s.lines() {
                let t = line.trim();
                if t.is_empty() || t.starts_with('#') { continue; }
                if t.starts_with('[') && t.ends_with(']') {
                    section = t[1..t.len()-1].to_string();
                } else if let Some(eq) = t.find('=') {
                    let key = t[..eq].trim().to_string();
                    let raw = t[eq+1..].trim().to_string();
                    let val = if raw.starts_with('"') { Value::Str(raw.trim_matches('"').to_string()) }
                        else if raw == "true" { Value::Bool(true) }
                        else if raw == "false" { Value::Bool(false) }
                        else if let Ok(n) = raw.parse::<i64>() { Value::Int(n) }
                        else if let Ok(f) = raw.parse::<f64>() { Value::Float(f) }
                        else { Value::Str(raw) };
                    let full_key = if section.is_empty() { key } else { format!("{}.{}", section, key) };
                    map.insert(full_key, val);
                }
            }
            Ok(Value::Struct("TOML".into(), map))
        }
        "get" => {
            match (args.first(), args.get(1)) {
                (Some(Value::Struct(_, fields)), Some(Value::Str(k))) => Ok(fields.get(k).cloned().unwrap_or(Value::Null)),
                _ => Err("toml.get: (toml, key)".into())
            }
        }
        _ => Err(format!("toml.{}: unknown", func))
    }
}
