use crate::value::Value;

pub fn call(func: &str, _args: Vec<Value>) -> Result<Value, String> {
    match func {
        "os" => Ok(Value::Str(std::env::consts::OS.to_string())),
        "arch" => Ok(Value::Str(std::env::consts::ARCH.to_string())),
        "family" => Ok(Value::Str(std::env::consts::FAMILY.to_string())),
        "cwd" => {
            let p = std::env::current_dir().map_err(|e| e.to_string())?;
            Ok(Value::Str(p.to_string_lossy().to_string()))
        }
        "home" => {
            Ok(Value::Str(std::env::var("HOME").unwrap_or_else(|_| "/".into())))
        }
        "hostname" => {
            Ok(Value::Str(std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".into())))
        }
        "cpu_count" => {
            Ok(Value::Int(std::thread::available_parallelism().map(|n| n.get() as i64).unwrap_or(1)))
        }
        _ => Err(format!("sys.{}: unknown function", func))
    }
}
