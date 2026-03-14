use crate::value::Value;

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "len" => match args.first() {
            Some(Value::Array(a)) => Ok(Value::Int(a.len() as i64)),
            _ => Err("arr.len: expected array".into())
        },
        "push" => match (args.first(), args.get(1)) {
            (Some(Value::Array(a)), Some(v)) => {
                let mut arr = a.clone();
                arr.push(v.clone());
                Ok(Value::Array(arr))
            }
            _ => Err("arr.push: expected (array, value)".into())
        },
        "pop" => match args.first() {
            Some(Value::Array(a)) => {
                let mut arr = a.clone();
                arr.pop();
                Ok(Value::Array(arr))
            }
            _ => Err("arr.pop: expected array".into())
        },
        "first" => match args.first() {
            Some(Value::Array(a)) => Ok(a.first().cloned().unwrap_or(Value::Null)),
            _ => Err("arr.first: expected array".into())
        },
        "last" => match args.first() {
            Some(Value::Array(a)) => Ok(a.last().cloned().unwrap_or(Value::Null)),
            _ => Err("arr.last: expected array".into())
        },
        "reverse" => match args.first() {
            Some(Value::Array(a)) => {
                let mut arr = a.clone();
                arr.reverse();
                Ok(Value::Array(arr))
            }
            _ => Err("arr.reverse: expected array".into())
        },
        "contains" => match (args.first(), args.get(1)) {
            (Some(Value::Array(a)), Some(v)) => {
                let found = a.iter().any(|x| x.to_string() == v.to_string());
                Ok(Value::Bool(found))
            }
            _ => Err("arr.contains: expected (array, value)".into())
        },
        _ => Err(format!("arr.{}: unknown function", func))
    }
}
