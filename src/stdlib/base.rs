use crate::value::Value;
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "to_base" => {
            let n = match args.first() { Some(Value::Int(n)) => *n, _ => return Err("base.to_base: (int, base)".into()) };
            let base = match args.get(1) { Some(Value::Int(b)) => *b as u32, _ => 10 };
            if base < 2 || base > 36 { return Err("base: base must be 2-36".into()); }
            let chars = "0123456789abcdefghijklmnopqrstuvwxyz";
            if n == 0 { return Ok(Value::Str("0".into())); }
            let mut result = String::new();
            let mut num = n.unsigned_abs();
            while num > 0 {
                let digit = (num % base as u64) as usize;
                result.push(chars.chars().nth(digit).unwrap_or('0'));
                num /= base as u64;
            }
            if n < 0 { result.push('-'); }
            Ok(Value::Str(result.chars().rev().collect()))
        }
        "from_base" => {
            let s = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("base.from_base: (str, base)".into()) };
            let base = match args.get(1) { Some(Value::Int(b)) => *b as u32, _ => 10 };
            Ok(Value::Int(i64::from_str_radix(&s, base).unwrap_or(0)))
        }
        "to_bin" => match args.first() { Some(Value::Int(n)) => Ok(Value::Str(format!("{:b}", n))), _ => Err("base.to_bin".into()) }
        "to_hex" => match args.first() { Some(Value::Int(n)) => Ok(Value::Str(format!("{:x}", n))), _ => Err("base.to_hex".into()) }
        "to_oct" => match args.first() { Some(Value::Int(n)) => Ok(Value::Str(format!("{:o}", n))), _ => Err("base.to_oct".into()) }
        "from_bin" => match args.first() { Some(Value::Str(s)) => Ok(Value::Int(i64::from_str_radix(s, 2).unwrap_or(0))), _ => Err("base.from_bin".into()) }
        "from_hex" => match args.first() { Some(Value::Str(s)) => Ok(Value::Int(i64::from_str_radix(s, 16).unwrap_or(0))), _ => Err("base.from_hex".into()) }
        "from_oct" => match args.first() { Some(Value::Str(s)) => Ok(Value::Int(i64::from_str_radix(s, 8).unwrap_or(0))), _ => Err("base.from_oct".into()) }
        _ => Err(format!("base.{}: unknown", func))
    }
}
