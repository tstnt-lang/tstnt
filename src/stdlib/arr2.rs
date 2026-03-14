use crate::value::Value;
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "zip" => match (args.first(), args.get(1)) {
            (Some(Value::Array(a)), Some(Value::Array(b))) => Ok(Value::Array(a.iter().zip(b.iter()).map(|(x, y)| Value::Tuple(vec![x.clone(), y.clone()])).collect())),
            _ => Err("arr2.zip: (arr, arr)".into())
        }
        "flatten" => match args.first() {
            Some(Value::Array(a)) => { let mut r = Vec::new(); for v in a { match v { Value::Array(inner) => r.extend(inner.clone()), x => r.push(x.clone()) } } Ok(Value::Array(r)) }
            _ => Err("arr2.flatten: array".into())
        }
        "chunk" => match (args.first(), args.get(1)) {
            (Some(Value::Array(a)), Some(Value::Int(size))) => {
                let size = *size as usize;
                Ok(Value::Array(a.chunks(size).map(|c| Value::Array(c.to_vec())).collect()))
            }
            _ => Err("arr2.chunk: (arr, size)".into())
        }
        "take" => match (args.first(), args.get(1)) {
            (Some(Value::Array(a)), Some(Value::Int(n))) => Ok(Value::Array(a.iter().take(*n as usize).cloned().collect())),
            _ => Err("arr2.take: (arr, n)".into())
        }
        "drop" => match (args.first(), args.get(1)) {
            (Some(Value::Array(a)), Some(Value::Int(n))) => Ok(Value::Array(a.iter().skip(*n as usize).cloned().collect())),
            _ => Err("arr2.drop: (arr, n)".into())
        }
        "count" => match (args.first(), args.get(1)) {
            (Some(Value::Array(a)), Some(v)) => Ok(Value::Int(a.iter().filter(|x| x.to_string() == v.to_string()).count() as i64)),
            _ => Err("arr2.count: (arr, val)".into())
        }
        "index_of" => match (args.first(), args.get(1)) {
            (Some(Value::Array(a)), Some(v)) => Ok(a.iter().position(|x| x.to_string() == v.to_string()).map(|i| Value::Int(i as i64)).unwrap_or(Value::Int(-1))),
            _ => Err("arr2.index_of: (arr, val)".into())
        }
        "rotate" => match (args.first(), args.get(1)) {
            (Some(Value::Array(a)), Some(Value::Int(n))) => {
                let mut v = a.clone();
                let n = ((*n % v.len() as i64 + v.len() as i64) % v.len() as i64) as usize;
                v.rotate_left(n);
                Ok(Value::Array(v))
            }
            _ => Err("arr2.rotate: (arr, n)".into())
        }
        "dedupe" => match args.first() {
            Some(Value::Array(a)) => {
                let mut seen = std::collections::HashSet::new();
                Ok(Value::Array(a.iter().filter(|v| seen.insert(v.to_string())).cloned().collect()))
            }
            _ => Err("arr2.dedupe: array".into())
        }
        _ => Err(format!("arr2.{}: unknown", func))
    }
}
