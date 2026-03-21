use crate::value::Value;

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "type_of" => match args.first() {
            Some(Value::Int(_)) => Ok(Value::Str("int".into())),
            Some(Value::Float(_)) => Ok(Value::Str("float".into())),
            Some(Value::Str(_)) => Ok(Value::Str("str".into())),
            Some(Value::Bool(_)) => Ok(Value::Str("bool".into())),
            Some(Value::Array(_)) => Ok(Value::Str("array".into())),
            Some(Value::Tuple(_)) => Ok(Value::Str("tuple".into())),
            Some(Value::Struct(n, _)) => Ok(Value::Str(n.clone())),
            Some(Value::Lambda(_, _)) => Ok(Value::Str("lambda".into())),
            Some(Value::EnumVariant(e, v, _)) => Ok(Value::Str(format!("{}::{}", e, v))),
            Some(Value::Null) | None => Ok(Value::Str("null".into())),
        }
        "is_int" => Ok(Value::Bool(matches!(args.first(), Some(Value::Int(_))))),
        "is_float" => Ok(Value::Bool(matches!(args.first(), Some(Value::Float(_))))),
        "is_str" => Ok(Value::Bool(matches!(args.first(), Some(Value::Str(_))))),
        "is_bool" => Ok(Value::Bool(matches!(args.first(), Some(Value::Bool(_))))),
        "is_array" => Ok(Value::Bool(matches!(args.first(), Some(Value::Array(_))))),
        "is_null" => Ok(Value::Bool(matches!(args.first(), Some(Value::Null) | None))),
        "is_lambda" => Ok(Value::Bool(matches!(args.first(), Some(Value::Lambda(_, _))))),
        "is_struct" => Ok(Value::Bool(matches!(args.first(), Some(Value::Struct(_, _))))),
        "fields" => match args.first() {
            Some(Value::Struct(_, fields)) => Ok(Value::Array(fields.keys().map(|k| Value::Str(k.clone())).collect())),
            _ => Ok(Value::Array(vec![]))
        }
        "get_field" => match (args.first(), args.get(1)) {
            (Some(Value::Struct(_, fields)), Some(Value::Str(k))) => Ok(fields.get(k).cloned().unwrap_or(Value::Null)),
            _ => Ok(Value::Null)
        }
        "has_field" => match (args.first(), args.get(1)) {
            (Some(Value::Struct(_, fields)), Some(Value::Str(k))) => Ok(Value::Bool(fields.contains_key(k))),
            _ => Ok(Value::Bool(false))
        }
        "size_of" => match args.first() {
            Some(Value::Array(a)) => Ok(Value::Int(a.len() as i64)),
            Some(Value::Str(s)) => Ok(Value::Int(s.len() as i64)),
            Some(Value::Struct(_, f)) => Ok(Value::Int(f.len() as i64)),
            Some(Value::Tuple(t)) => Ok(Value::Int(t.len() as i64)),
            _ => Ok(Value::Int(1))
        }
        "equal" => match (args.first(), args.get(1)) {
            (Some(a), Some(b)) => Ok(Value::Bool(a.to_string() == b.to_string())),
            _ => Ok(Value::Bool(false))
        }
        "to_array" => match args.first() {
            Some(Value::Tuple(t)) => Ok(Value::Array(t.clone())),
            Some(Value::Str(s)) => Ok(Value::Array(s.chars().map(|c| Value::Str(c.to_string())).collect())),
            Some(v) => Ok(Value::Array(vec![v.clone()])),
            None => Ok(Value::Array(vec![]))
        }
        "to_tuple" => match args.first() {
            Some(Value::Array(a)) => Ok(Value::Tuple(a.clone())),
            Some(v) => Ok(Value::Tuple(vec![v.clone()])),
            None => Ok(Value::Tuple(vec![]))
        }
        _ => Err(format!("reflect.{}: unknown", func))
    }
}
