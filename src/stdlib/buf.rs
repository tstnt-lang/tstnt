use crate::value::Value;

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "from_str" => match args.first() {
            Some(Value::Str(s)) => {
                Ok(Value::Array(s.bytes().map(|b| Value::Int(b as i64)).collect()))
            }
            _ => Err("buf.from_str: expected str".into())
        },
        "to_str" => match args.first() {
            Some(Value::Array(a)) => {
                let bytes: Vec<u8> = a.iter().filter_map(|v| match v {
                    Value::Int(n) => Some(*n as u8),
                    _ => None
                }).collect();
                Ok(Value::Str(String::from_utf8_lossy(&bytes).to_string()))
            }
            _ => Err("buf.to_str: expected array".into())
        },
        "from_hex" => match args.first() {
            Some(Value::Str(s)) => {
                let bytes: Vec<Value> = (0..s.len()).step_by(2)
                    .map(|i| u8::from_str_radix(&s[i..i+2], 16).map(|b| Value::Int(b as i64)).map_err(|e| e.to_string()))
                    .collect::<Result<_, _>>()?;
                Ok(Value::Array(bytes))
            }
            _ => Err("buf.from_hex: expected str".into())
        },
        "to_hex" => match args.first() {
            Some(Value::Array(a)) => {
                let hex: String = a.iter().filter_map(|v| match v {
                    Value::Int(n) => Some(format!("{:02x}", n)),
                    _ => None
                }).collect();
                Ok(Value::Str(hex))
            }
            _ => Err("buf.to_hex: expected array".into())
        },
        "slice" => match (args.first(), args.get(1), args.get(2)) {
            (Some(Value::Array(a)), Some(Value::Int(start)), Some(Value::Int(end))) => {
                let s = *start as usize;
                let e = (*end as usize).min(a.len());
                Ok(Value::Array(a[s..e].to_vec()))
            }
            _ => Err("buf.slice: expected (array, int, int)".into())
        },
        "concat" => {
            let mut result = Vec::new();
            for arg in &args {
                if let Value::Array(a) = arg { result.extend(a.clone()); }
            }
            Ok(Value::Array(result))
        },
        _ => Err(format!("buf.{}: unknown function", func))
    }
}
