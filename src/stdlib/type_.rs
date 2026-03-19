use crate::value::Value;
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "of" => match args.first() {
            Some(Value::Int(_)) => Ok(Value::Str("int".into())),
            Some(Value::Float(_)) => Ok(Value::Str("float".into())),
            Some(Value::Str(_)) => Ok(Value::Str("str".into())),
            Some(Value::Bool(_)) => Ok(Value::Str("bool".into())),
            Some(Value::Array(_)) => Ok(Value::Str("array".into())),
            Some(Value::Tuple(_)) => Ok(Value::Str("tuple".into())),
            Some(Value::Lambda(_, _)) => Ok(Value::Str("lambda".into())),
            Some(Value::Struct(n, _)) => Ok(Value::Str(n.clone())),
            Some(Value::Null) | None => Ok(Value::Str("null".into())),
            Some(Value::EnumVariant(en, vn, _)) => Ok(Value::Str(format!("{}::{}", en, vn))),
        }
        "is_int" => Ok(Value::Bool(matches!(args.first(), Some(Value::Int(_))))),
        "is_str" => Ok(Value::Bool(matches!(args.first(), Some(Value::Str(_))))),
        "is_float" => Ok(Value::Bool(matches!(args.first(), Some(Value::Float(_))))),
        "is_bool" => Ok(Value::Bool(matches!(args.first(), Some(Value::Bool(_))))),
        "is_null" => Ok(Value::Bool(matches!(args.first(), Some(Value::Null) | None))),
        "is_array" => Ok(Value::Bool(matches!(args.first(), Some(Value::Array(_))))),
        "is_func" => Ok(Value::Bool(matches!(args.first(), Some(Value::Lambda(_, _))))),
        "cast_int" => match args.first() {
            Some(Value::Float(f)) => Ok(Value::Int(*f as i64)),
            Some(Value::Str(s)) => Ok(Value::Int(s.trim().parse().unwrap_or(0))),
            Some(Value::Bool(b)) => Ok(Value::Int(if *b { 1 } else { 0 })),
            Some(v) => Ok(v.clone()),
            None => Ok(Value::Int(0))
        }
        "cast_str" => Ok(Value::Str(args.first().map(|v| v.to_string()).unwrap_or_default())),
        "cast_float" => match args.first() {
            Some(Value::Int(n)) => Ok(Value::Float(*n as f64)),
            Some(Value::Str(s)) => Ok(Value::Float(s.trim().parse().unwrap_or(0.0))),
            Some(v) => Ok(v.clone()),
            None => Ok(Value::Float(0.0))
        }
        _ => Err(format!("type.{}: unknown", func))
    }
}
