use crate::value::Value;
use std::time::{SystemTime, UNIX_EPOCH};

fn ts() -> String {
    let s = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    format!("{}", s)
}

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    let msg = args.first().map(|v| v.to_string()).unwrap_or_default();
    match func {
        "info"  => { eprintln!("[INFO  {}] {}", ts(), msg); Ok(Value::Null) }
        "warn"  => { eprintln!("[WARN  {}] {}", ts(), msg); Ok(Value::Null) }
        "error" => { eprintln!("[ERROR {}] {}", ts(), msg); Ok(Value::Null) }
        "debug" => { eprintln!("[DEBUG {}] {}", ts(), msg); Ok(Value::Null) }
        _ => Err(format!("log.{}: unknown", func))
    }
}
