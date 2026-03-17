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
        "sort" => match args.first() {
            Some(Value::Array(a)) => {
                let mut v = a.clone();
                v.sort_by(|a, b| {
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => x.cmp(y),
                        (Value::Float(x), Value::Float(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                        _ => a.to_string().cmp(&b.to_string())
                    }
                });
                Ok(Value::Array(v))
            }
            _ => Err("arr.sort: array".into())
        },
        "join" => match (args.first(), args.get(1)) {
            (Some(Value::Array(a)), Some(Value::Str(sep))) => Ok(Value::Str(a.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(sep))),
            _ => Err("arr.join: (array, str)".into())
        },
        "slice" => match args.first() {
            Some(Value::Array(a)) => {
                let from = match args.get(1) { Some(Value::Int(n)) => *n as usize, _ => 0 };
                let to = match args.get(2) { Some(Value::Int(n)) => *n as usize, _ => a.len() };
                Ok(Value::Array(a[from.min(a.len())..to.min(a.len())].to_vec()))
            }
            _ => Err("arr.slice: (array, from, to)".into())
        },
        "flatten" => match args.first() {
            Some(Value::Array(a)) => {
                let mut result = Vec::new();
                for v in a { match v { Value::Array(inner) => result.extend(inner.clone()), x => result.push(x.clone()) } }
                Ok(Value::Array(result))
            }
            _ => Err("arr.flatten: array".into())
        },
        "index_of" => match (args.first(), args.get(1)) {
            (Some(Value::Array(a)), Some(v)) => Ok(a.iter().position(|x| x.to_string() == v.to_string()).map(|i| Value::Int(i as i64)).unwrap_or(Value::Int(-1))),
            _ => Err("arr.index_of: (array, val)".into())
        },
        "unique" => match args.first() {
            Some(Value::Array(a)) => {
                let mut seen = std::collections::HashSet::new();
                Ok(Value::Array(a.iter().filter(|v| seen.insert(v.to_string())).cloned().collect()))
            }
            _ => Err("arr.unique: array".into())
        },
        "count" => match (args.first(), args.get(1)) {
            (Some(Value::Array(a)), Some(v)) => Ok(Value::Int(a.iter().filter(|x| x.to_string() == v.to_string()).count() as i64)),
            (Some(Value::Array(a)), None) => Ok(Value::Int(a.len() as i64)),
            _ => Err("arr.count: (array, val)".into())
        },
        _ => Err(format!("arr.{}: unknown function", func))
    }
}
