use crate::value::Value;
use std::collections::HashMap;

fn ok_val(v: Value) -> Value {
    let mut fields = HashMap::new();
    fields.insert("ok".into(), Value::Bool(true));
    fields.insert("value".into(), v);
    fields.insert("err".into(), Value::Null);
    Value::Struct("Result".into(), fields)
}

fn err_val(msg: String) -> Value {
    let mut fields = HashMap::new();
    fields.insert("ok".into(), Value::Bool(false));
    fields.insert("value".into(), Value::Null);
    fields.insert("err".into(), Value::Str(msg));
    Value::Struct("Result".into(), fields)
}

fn some_val(v: Value) -> Value {
    let mut fields = HashMap::new();
    fields.insert("some".into(), Value::Bool(true));
    fields.insert("value".into(), v);
    Value::Struct("Option".into(), fields)
}

fn none_val() -> Value {
    let mut fields = HashMap::new();
    fields.insert("some".into(), Value::Bool(false));
    fields.insert("value".into(), Value::Null);
    Value::Struct("Option".into(), fields)
}

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "ok" => Ok(ok_val(args.into_iter().next().unwrap_or(Value::Null))),
        "err" => {
            let msg = args.first().map(|v| v.to_string()).unwrap_or_else(|| "error".into());
            Ok(err_val(msg))
        }
        "some" => Ok(some_val(args.into_iter().next().unwrap_or(Value::Null))),
        "none" => Ok(none_val()),
        "is_ok" => match args.first() {
            Some(Value::Struct(_, fields)) => Ok(Value::Bool(fields.get("ok").map(|v| matches!(v, Value::Bool(true))).unwrap_or(false))),
            _ => Ok(Value::Bool(false))
        }
        "is_err" => match args.first() {
            Some(Value::Struct(_, fields)) => Ok(Value::Bool(fields.get("ok").map(|v| matches!(v, Value::Bool(false))).unwrap_or(false))),
            _ => Ok(Value::Bool(false))
        }
        "is_some" => match args.first() {
            Some(Value::Struct(_, fields)) => Ok(Value::Bool(fields.get("some").map(|v| matches!(v, Value::Bool(true))).unwrap_or(false))),
            _ => Ok(Value::Bool(false))
        }
        "is_none" => match args.first() {
            Some(Value::Struct(_, fields)) => Ok(Value::Bool(fields.get("some").map(|v| matches!(v, Value::Bool(false))).unwrap_or(true))),
            _ => Ok(Value::Bool(true))
        }
        "unwrap" => match args.first() {
            Some(Value::Struct(_, fields)) => {
                if let Some(Value::Bool(false)) = fields.get("ok") {
                    let msg = fields.get("err").map(|v| v.to_string()).unwrap_or_else(|| "unwrap on Err".into());
                    return Err(msg);
                }
                Ok(fields.get("value").cloned().unwrap_or(Value::Null))
            }
            Some(v) => Ok(v.clone()),
            None => Err("result.unwrap: no value".into())
        }
        "unwrap_or" => match (args.first(), args.get(1)) {
            (Some(Value::Struct(_, fields)), Some(fallback)) => {
                if let Some(Value::Bool(false)) = fields.get("ok") { return Ok(fallback.clone()); }
                if let Some(Value::Bool(false)) = fields.get("some") { return Ok(fallback.clone()); }
                Ok(fields.get("value").cloned().unwrap_or_else(|| fallback.clone()))
            }
            (Some(Value::Null), Some(fallback)) => Ok(fallback.clone()),
            (Some(v), _) => Ok(v.clone()),
            _ => Ok(Value::Null)
        }
        "map" => match (args.first(), args.get(1)) {
            (Some(Value::Struct(name, fields)), Some(Value::Lambda(params, body))) => {
                if let Some(Value::Bool(false)) = fields.get("ok") { return Ok(Value::Struct(name.clone(), fields.clone())); }
                if let Some(Value::Bool(false)) = fields.get("some") { return Ok(none_val()); }
                let val = fields.get("value").cloned().unwrap_or(Value::Null);
                Ok(some_val(val))
            }
            _ => Ok(args.into_iter().next().unwrap_or(Value::Null))
        }
        "get_err" => match args.first() {
            Some(Value::Struct(_, fields)) => Ok(fields.get("err").cloned().unwrap_or(Value::Null)),
            _ => Ok(Value::Null)
        }
        _ => Err(format!("result.{}: unknown", func))
    }
}
