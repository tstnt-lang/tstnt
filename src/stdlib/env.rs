use crate::value::Value;
use std::env;

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "get" => match args.first() {
            Some(Value::Str(key)) => {
                Ok(Value::Str(env::var(key).unwrap_or_default()))
            }
            _ => Err("env.get: expected str key".into())
        },
        "set" => match (args.first(), args.get(1)) {
            (Some(Value::Str(key)), Some(Value::Str(val))) => {
                env::set_var(key, val);
                Ok(Value::Null)
            }
            _ => Err("env.set: expected (str, str)".into())
        },
        "args" => {
            let a: Vec<Value> = env::args().map(|s| Value::Str(s)).collect();
            Ok(Value::Array(a))
        }
        _ => Err(format!("env.{}: unknown function", func))
    }
}
