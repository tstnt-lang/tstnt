use crate::value::Value;
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "pack_i32" => {
            let n = match args.first() { Some(Value::Int(n)) => *n as i32, _ => return Err("pack.pack_i32".into()) };
            let bytes = n.to_le_bytes();
            Ok(Value::Array(bytes.iter().map(|&b| Value::Int(b as i64)).collect()))
        }
        "unpack_i32" => {
            let arr = match args.first() { Some(Value::Array(a)) => a.clone(), _ => return Err("pack.unpack_i32".into()) };
            if arr.len() < 4 { return Err("pack.unpack_i32: need 4 bytes".into()); }
            let bytes: [u8; 4] = [
                match &arr[0] { Value::Int(n) => *n as u8, _ => 0 },
                match &arr[1] { Value::Int(n) => *n as u8, _ => 0 },
                match &arr[2] { Value::Int(n) => *n as u8, _ => 0 },
                match &arr[3] { Value::Int(n) => *n as u8, _ => 0 },
            ];
            Ok(Value::Int(i32::from_le_bytes(bytes) as i64))
        }
        "pack_f64" => {
            let f = match args.first() { Some(Value::Float(f)) => *f, Some(Value::Int(n)) => *n as f64, _ => return Err("pack.pack_f64".into()) };
            let bytes = f.to_le_bytes();
            Ok(Value::Array(bytes.iter().map(|&b| Value::Int(b as i64)).collect()))
        }
        "bytes_to_str" => {
            let arr = match args.first() { Some(Value::Array(a)) => a.clone(), _ => return Err("pack.bytes_to_str".into()) };
            let bytes: Vec<u8> = arr.iter().map(|v| match v { Value::Int(n) => *n as u8, _ => 0 }).collect();
            Ok(Value::Str(String::from_utf8_lossy(&bytes).to_string()))
        }
        "str_to_bytes" => {
            let s = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("pack.str_to_bytes".into()) };
            Ok(Value::Array(s.bytes().map(|b| Value::Int(b as i64)).collect()))
        }
        _ => Err(format!("pack.{}: unknown", func))
    }
}
