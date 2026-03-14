use crate::value::Value;
use std::fs;
use std::path::PathBuf;

fn pickle_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".tstnt").join("pickle")
}

fn val_to_str(v: &Value) -> String {
    match v {
        Value::Int(n) => format!("i:{}", n),
        Value::Float(f) => format!("f:{}", f),
        Value::Str(s) => format!("s:{}", s.replace('\\', "\\\\").replace('\n', "\\n")),
        Value::Bool(b) => format!("b:{}", b),
        Value::Null => "n:".to_string(),
        Value::Array(a) => {
            let items: Vec<String> = a.iter().map(val_to_str).collect();
            format!("a:{}:{}", items.len(), items.join("\x01"))
        }
        Value::Struct(name, fields) => {
            let items: Vec<String> = fields.iter().map(|(k, v)| format!("{}={}", k, val_to_str(v))).collect();
            format!("S:{}:{}", name, items.join("\x01"))
        }
        _ => format!("s:{}", v)
    }
}

fn str_to_val(s: &str) -> Value {
    if s.is_empty() { return Value::Null; }
    let (prefix, rest) = s.split_at(2);
    match prefix {
        "i:" => Value::Int(rest.parse().unwrap_or(0)),
        "f:" => Value::Float(rest.parse().unwrap_or(0.0)),
        "s:" => Value::Str(rest.replace("\\n", "\n").replace("\\\\", "\\")),
        "b:" => Value::Bool(rest == "true"),
        "n:" => Value::Null,
        "a:" => {
            let mut parts = rest.splitn(2, ':');
            let n: usize = parts.next().unwrap_or("0").parse().unwrap_or(0);
            let data = parts.next().unwrap_or("");
            if n == 0 || data.is_empty() { return Value::Array(vec![]); }
            let items: Vec<Value> = data.split('\x01').map(str_to_val).collect();
            Value::Array(items)
        }
        "S:" => {
            let mut parts = rest.splitn(2, ':');
            let name = parts.next().unwrap_or("Obj").to_string();
            let data = parts.next().unwrap_or("");
            let mut map = std::collections::HashMap::new();
            if !data.is_empty() {
                for item in data.split('\x01') {
                    if let Some(eq) = item.find('=') {
                        map.insert(item[..eq].to_string(), str_to_val(&item[eq+1..]));
                    }
                }
            }
            Value::Struct(name, map)
        }
        _ => Value::Str(s.to_string())
    }
}

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "save" => {
            let name = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("pickle.save: (name, value)".into()) };
            let val = args.get(1).cloned().unwrap_or(Value::Null);
            let dir = pickle_dir();
            fs::create_dir_all(&dir).ok();
            let path = dir.join(format!("{}.pkl", name));
            fs::write(&path, val_to_str(&val)).map_err(|e| e.to_string())?;
            Ok(Value::Null)
        }
        "load" => {
            let name = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("pickle.load: name".into()) };
            let path = pickle_dir().join(format!("{}.pkl", name));
            match fs::read_to_string(&path) {
                Ok(data) => Ok(str_to_val(&data)),
                Err(_) => Ok(Value::Null)
            }
        }
        "exists" => {
            let name = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("pickle.exists: name".into()) };
            let path = pickle_dir().join(format!("{}.pkl", name));
            Ok(Value::Bool(path.exists()))
        }
        "delete" => {
            let name = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("pickle.delete: name".into()) };
            let path = pickle_dir().join(format!("{}.pkl", name));
            fs::remove_file(&path).ok();
            Ok(Value::Null)
        }
        "list" => {
            let dir = pickle_dir();
            if !dir.exists() { return Ok(Value::Array(vec![])); }
            let entries: Vec<Value> = fs::read_dir(&dir).map_err(|e| e.to_string())?
                .flatten()
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.ends_with(".pkl") { Some(Value::Str(name.trim_end_matches(".pkl").to_string())) }
                    else { None }
                })
                .collect();
            Ok(Value::Array(entries))
        }
        "save_file" => {
            let path = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("pickle.save_file: (path, value)".into()) };
            let val = args.get(1).cloned().unwrap_or(Value::Null);
            fs::write(&path, val_to_str(&val)).map_err(|e| e.to_string())?;
            Ok(Value::Null)
        }
        "load_file" => {
            let path = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("pickle.load_file: path".into()) };
            match fs::read_to_string(&path) {
                Ok(data) => Ok(str_to_val(&data)),
                Err(_) => Ok(Value::Null)
            }
        }
        _ => Err(format!("pickle.{}: unknown", func))
    }
}
