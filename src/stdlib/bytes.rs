use crate::value::Value;

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "from_str" => match args.first() {
            Some(Value::Str(s)) => Ok(Value::Array(s.bytes().map(|b| Value::Int(b as i64)).collect())),
            _ => Err("bytes.from_str: str".into())
        }
        "to_str" => match args.first() {
            Some(Value::Array(a)) => {
                let bytes: Vec<u8> = a.iter().filter_map(|v| if let Value::Int(n) = v { Some(*n as u8) } else { None }).collect();
                Ok(Value::Str(String::from_utf8_lossy(&bytes).to_string()))
            }
            _ => Err("bytes.to_str: array".into())
        }
        "from_hex" => match args.first() {
            Some(Value::Str(s)) => {
                let s = s.replace(" ", "").replace("0x", "");
                let bytes: Result<Vec<_>, _> = (0..s.len()).step_by(2)
                    .map(|i| u8::from_str_radix(&s[i..i+2], 16).map(|b| Value::Int(b as i64)))
                    .collect();
                bytes.map(Value::Array).map_err(|_| "bytes.from_hex: invalid hex".into())
            }
            _ => Err("bytes.from_hex: str".into())
        }
        "to_hex" => match args.first() {
            Some(Value::Array(a)) => {
                let hex: String = a.iter().filter_map(|v| if let Value::Int(n) = v { Some(format!("{:02x}", n)) } else { None }).collect();
                Ok(Value::Str(hex))
            }
            _ => Err("bytes.to_hex: array".into())
        }
        "len" => match args.first() {
            Some(Value::Array(a)) => Ok(Value::Int(a.len() as i64)),
            Some(Value::Str(s)) => Ok(Value::Int(s.len() as i64)),
            _ => Err("bytes.len: array".into())
        }
        "at" => match (args.first(), args.get(1)) {
            (Some(Value::Array(a)), Some(Value::Int(i))) => Ok(a.get(*i as usize).cloned().unwrap_or(Value::Null)),
            (Some(Value::Str(s)), Some(Value::Int(i))) => Ok(s.as_bytes().get(*i as usize).map(|b| Value::Int(*b as i64)).unwrap_or(Value::Null)),
            _ => Err("bytes.at: (bytes, index)".into())
        }
        "slice" => match (args.first(), args.get(1), args.get(2)) {
            (Some(Value::Array(a)), Some(Value::Int(from)), Some(Value::Int(to))) => {
                let f = (*from as usize).min(a.len());
                let t = (*to as usize).min(a.len());
                Ok(Value::Array(a[f..t].to_vec()))
            }
            _ => Err("bytes.slice: (bytes, from, to)".into())
        }
        "concat" => {
            let mut r = Vec::new();
            for arg in &args { if let Value::Array(a) = arg { r.extend(a.clone()); } }
            Ok(Value::Array(r))
        }
        "repeat" => match (args.first(), args.get(1)) {
            (Some(Value::Array(a)), Some(Value::Int(n))) => {
                Ok(Value::Array(a.iter().cycle().take(a.len() * *n as usize).cloned().collect()))
            }
            _ => Err("bytes.repeat: (bytes, n)".into())
        }
        "xor" => match (args.first(), args.get(1)) {
            (Some(Value::Array(a)), Some(Value::Array(b))) => Ok(Value::Array(
                a.iter().zip(b.iter()).map(|(x, y)| match (x, y) {
                    (Value::Int(a), Value::Int(b)) => Value::Int(a ^ b),
                    _ => Value::Null
                }).collect()
            )),
            _ => Err("bytes.xor: (bytes, bytes)".into())
        }
        _ => Err(format!("bytes.{}: unknown", func))
    }
}
