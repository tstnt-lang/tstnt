use crate::value::Value;
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "to_bin" => match args.first() { Some(Value::Int(n)) => Ok(Value::Str(format!("{:b}", n))), _ => Err("num.to_bin: int".into()) }
        "to_hex" => match args.first() { Some(Value::Int(n)) => Ok(Value::Str(format!("{:x}", n))), _ => Err("num.to_hex: int".into()) }
        "to_oct" => match args.first() { Some(Value::Int(n)) => Ok(Value::Str(format!("{:o}", n))), _ => Err("num.to_oct: int".into()) }
        "from_bin" => match args.first() { Some(Value::Str(s)) => Ok(Value::Int(i64::from_str_radix(s, 2).unwrap_or(0))), _ => Err("num.from_bin: str".into()) }
        "from_hex" => match args.first() { Some(Value::Str(s)) => Ok(Value::Int(i64::from_str_radix(s, 16).unwrap_or(0))), _ => Err("num.from_hex: str".into()) }
        "from_oct" => match args.first() { Some(Value::Str(s)) => Ok(Value::Int(i64::from_str_radix(s, 8).unwrap_or(0))), _ => Err("num.from_oct: str".into()) }
        "is_nan" => match args.first() { Some(Value::Float(f)) => Ok(Value::Bool(f.is_nan())), _ => Ok(Value::Bool(false)) }
        "is_inf" => match args.first() { Some(Value::Float(f)) => Ok(Value::Bool(f.is_infinite())), _ => Ok(Value::Bool(false)) }
        "max_int" => Ok(Value::Int(i64::MAX)),
        "min_int" => Ok(Value::Int(i64::MIN)),
        _ => Err(format!("num.{}: unknown", func))
    }
}
