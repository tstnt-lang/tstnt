use crate::value::Value;
use std::fs;
use std::path::Path;

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "read" => match args.first() {
            Some(Value::Str(p)) => Ok(Value::Str(fs::read_to_string(p).map_err(|e| e.to_string())?)),
            _ => Err("fs.read: expected str".into())
        },
        "write" => match (args.first(), args.get(1)) {
            (Some(Value::Str(p)), Some(Value::Str(c))) => {
                fs::write(p, c).map_err(|e| e.to_string())?;
                Ok(Value::Null)
            }
            _ => Err("fs.write: expected (str, str)".into())
        },
        "append" => match (args.first(), args.get(1)) {
            (Some(Value::Str(p)), Some(Value::Str(c))) => {
                use std::io::Write;
                let mut f = fs::OpenOptions::new().append(true).create(true).open(p).map_err(|e| e.to_string())?;
                f.write_all(c.as_bytes()).map_err(|e| e.to_string())?;
                Ok(Value::Null)
            }
            _ => Err("fs.append: expected (str, str)".into())
        },
        "exists" => match args.first() {
            Some(Value::Str(p)) => Ok(Value::Bool(Path::new(p).exists())),
            _ => Err("fs.exists: expected str".into())
        },
        "delete" => match args.first() {
            Some(Value::Str(p)) => {
                fs::remove_file(p).map_err(|e| e.to_string())?;
                Ok(Value::Null)
            }
            _ => Err("fs.delete: expected str".into())
        },
        "mkdir" => match args.first() {
            Some(Value::Str(p)) => {
                fs::create_dir_all(p).map_err(|e| e.to_string())?;
                Ok(Value::Null)
            }
            _ => Err("fs.mkdir: expected str".into())
        },
        "ls" => match args.first() {
            Some(Value::Str(p)) => {
                let entries: Vec<Value> = fs::read_dir(p).map_err(|e| e.to_string())?
                    .filter_map(|e| e.ok())
                    .map(|e| Value::Str(e.file_name().to_string_lossy().to_string()))
                    .collect();
                Ok(Value::Array(entries))
            }
            _ => Err("fs.ls: expected str".into())
        },
        "is_dir" => match args.first() {
            Some(Value::Str(p)) => Ok(Value::Bool(Path::new(p).is_dir())),
            _ => Err("fs.is_dir: expected str".into())
        },
        "is_file" => match args.first() {
            Some(Value::Str(p)) => Ok(Value::Bool(Path::new(p).is_file())),
            _ => Err("fs.is_file: expected str".into())
        },
        "copy" => match (args.first(), args.get(1)) {
            (Some(Value::Str(src)), Some(Value::Str(dst))) => {
                fs::copy(src, dst).map_err(|e| e.to_string())?;
                Ok(Value::Null)
            }
            _ => Err("fs.copy: expected (str, str)".into())
        },
        "rename" => match (args.first(), args.get(1)) {
            (Some(Value::Str(src)), Some(Value::Str(dst))) => {
                fs::rename(src, dst).map_err(|e| e.to_string())?;
                Ok(Value::Null)
            }
            _ => Err("fs.rename: expected (str, str)".into())
        },
        "size" => match args.first() {
            Some(Value::Str(p)) => {
                let meta = fs::metadata(p).map_err(|e| e.to_string())?;
                Ok(Value::Int(meta.len() as i64))
            }
            _ => Err("fs.size: expected str".into())
        },
        _ => Err(format!("fs.{}: unknown function", func))
    }
}
