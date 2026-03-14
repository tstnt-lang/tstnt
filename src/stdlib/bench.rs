use crate::value::Value;
use std::time::{SystemTime, UNIX_EPOCH};

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

pub fn call(func: &str, _args: Vec<Value>) -> Result<Value, String> {
    match func {
        "now_ms" => Ok(Value::Int(now_ms() as i64)),
        "now_us" => {
            let us = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_micros() as i64;
            Ok(Value::Int(us))
        }
        "elapsed" => {
            let v = _args.first().map(|v| match v { Value::Int(n) => *n, _ => 0 }).unwrap_or(0);
            Ok(Value::Int(now_ms() as i64 - v))
        }
        _ => Err(format!("bench.{}: unknown", func))
    }
}
