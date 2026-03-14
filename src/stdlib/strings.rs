use crate::value::Value;

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "len" => match args.first() {
            Some(Value::Str(s)) => Ok(Value::Int(s.len() as i64)),
            _ => Err("str.len: expected str".into())
        },
        "split" => match (args.first(), args.get(1)) {
            (Some(Value::Str(s)), Some(Value::Str(sep))) => {
                let parts: Vec<Value> = s.split(sep.as_str()).map(|p| Value::Str(p.to_string())).collect();
                Ok(Value::Array(parts))
            }
            _ => Err("str.split: expected (str, str)".into())
        },
        "join" => match (args.first(), args.get(1)) {
            (Some(Value::Array(arr)), Some(Value::Str(sep))) => {
                let parts: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                Ok(Value::Str(parts.join(sep)))
            }
            _ => Err("str.join: expected (arr, str)".into())
        },
        "trim" => match args.first() {
            Some(Value::Str(s)) => Ok(Value::Str(s.trim().to_string())),
            _ => Err("str.trim: expected str".into())
        },
        "upper" => match args.first() {
            Some(Value::Str(s)) => Ok(Value::Str(s.to_uppercase())),
            _ => Err("str.upper: expected str".into())
        },
        "lower" => match args.first() {
            Some(Value::Str(s)) => Ok(Value::Str(s.to_lowercase())),
            _ => Err("str.lower: expected str".into())
        },
        "contains" => match (args.first(), args.get(1)) {
            (Some(Value::Str(s)), Some(Value::Str(pat))) => Ok(Value::Bool(s.contains(pat.as_str()))),
            _ => Err("str.contains: expected (str, str)".into())
        },
        "replace" => match (args.first(), args.get(1), args.get(2)) {
            (Some(Value::Str(s)), Some(Value::Str(from)), Some(Value::Str(to))) => {
                Ok(Value::Str(s.replace(from.as_str(), to)))
            }
            _ => Err("str.replace: expected (str, str, str)".into())
        },
        "starts_with" => match (args.first(), args.get(1)) {
            (Some(Value::Str(s)), Some(Value::Str(pat))) => Ok(Value::Bool(s.starts_with(pat.as_str()))),
            _ => Err("str.starts_with: expected (str, str)".into())
        },
        "ends_with" => match (args.first(), args.get(1)) {
            (Some(Value::Str(s)), Some(Value::Str(pat))) => Ok(Value::Bool(s.ends_with(pat.as_str()))),
            _ => Err("str.ends_with: expected (str, str)".into())
        },
        _ => Err(format!("str.{}: unknown function", func))
    }
}
