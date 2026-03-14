use crate::value::Value;
use std::collections::HashMap;
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "parse" => {
            let s = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("ini.parse: str".into()) };
            let mut sections: HashMap<String, Value> = HashMap::new();
            let mut current = "default".to_string();
            for line in s.lines() {
                let t = line.trim();
                if t.is_empty() || t.starts_with(';') || t.starts_with('#') { continue; }
                if t.starts_with('[') && t.ends_with(']') {
                    current = t[1..t.len()-1].to_string();
                    sections.entry(current.clone()).or_insert(Value::Struct(current.clone(), HashMap::new()));
                } else if let Some(eq) = t.find('=') {
                    let key = t[..eq].trim().to_string();
                    let val = t[eq+1..].trim().to_string();
                    if let Some(Value::Struct(_, fields)) = sections.get_mut(&current) {
                        fields.insert(key, Value::Str(val));
                    } else {
                        let mut f = HashMap::new();
                        f.insert(key, Value::Str(val));
                        sections.insert(current.clone(), Value::Struct(current.clone(), f));
                    }
                }
            }
            Ok(Value::Struct("INI".into(), sections))
        }
        "get" => {
            match (args.first(), args.get(1), args.get(2)) {
                (Some(Value::Struct(_, sections)), Some(Value::Str(section)), Some(Value::Str(key))) => {
                    if let Some(Value::Struct(_, fields)) = sections.get(section) {
                        Ok(fields.get(key).cloned().unwrap_or(Value::Null))
                    } else { Ok(Value::Null) }
                }
                _ => Err("ini.get: (ini, section, key)".into())
            }
        }
        "stringify" => {
            match args.first() {
                Some(Value::Struct(_, sections)) => {
                    let mut result = String::new();
                    for (sec, val) in sections {
                        result.push_str(&format!("[{}]\n", sec));
                        if let Value::Struct(_, fields) = val {
                            for (k, v) in fields { result.push_str(&format!("{}={}\n", k, v)); }
                        }
                        result.push('\n');
                    }
                    Ok(Value::Str(result))
                }
                _ => Err("ini.stringify: ini object".into())
            }
        }
        _ => Err(format!("ini.{}: unknown", func))
    }
}
