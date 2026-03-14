use crate::value::Value;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::thread;

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "now" => {
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_secs();
            Ok(Value::Int(ts as i64))
        }
        "sleep" => {
            match args.first() {
                Some(Value::Int(ms)) => {
                    thread::sleep(Duration::from_millis(*ms as u64));
                    Ok(Value::Null)
                }
                Some(Value::Float(ms)) => {
                    thread::sleep(Duration::from_millis(*ms as u64));
                    Ok(Value::Null)
                }
                _ => Err("time.sleep: expected int (ms)".into())
            }
        }
        _ => Err(format!("time.{}: unknown function", func))
    }
}
