use crate::value::Value;
use std::path::Path;

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "join" => {
            let parts: Vec<String> = args.iter().map(|v| v.to_string()).collect();
            let mut p = std::path::PathBuf::new();
            for part in &parts { p.push(part); }
            Ok(Value::Str(p.to_string_lossy().to_string()))
        }
        "dir" => match args.first() {
            Some(Value::Str(s)) => {
                Ok(Value::Str(Path::new(s).parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()))
            }
            _ => Err("path.dir: expected str".into())
        },
        "file" => match args.first() {
            Some(Value::Str(s)) => {
                Ok(Value::Str(Path::new(s).file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default()))
            }
            _ => Err("path.file: expected str".into())
        },
        "ext" => match args.first() {
            Some(Value::Str(s)) => {
                Ok(Value::Str(Path::new(s).extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default()))
            }
            _ => Err("path.ext: expected str".into())
        },
        "stem" => match args.first() {
            Some(Value::Str(s)) => {
                Ok(Value::Str(Path::new(s).file_stem().map(|e| e.to_string_lossy().to_string()).unwrap_or_default()))
            }
            _ => Err("path.stem: expected str".into())
        },
        "abs" => match args.first() {
            Some(Value::Str(s)) => {
                let p = std::fs::canonicalize(s).map_err(|e| e.to_string())?;
                Ok(Value::Str(p.to_string_lossy().to_string()))
            }
            _ => Err("path.abs: expected str".into())
        },
        _ => Err(format!("path.{}: unknown function", func))
    }
}
