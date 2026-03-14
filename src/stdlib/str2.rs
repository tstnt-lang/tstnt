use crate::value::Value;
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    let s = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err(format!("str2.{}: str", func)) };
    match func {
        "chars" => Ok(Value::Array(s.chars().map(|c| Value::Str(c.to_string())).collect())),
        "bytes" => Ok(Value::Array(s.bytes().map(|b| Value::Int(b as i64)).collect())),
        "is_alpha" => Ok(Value::Bool(s.chars().all(|c| c.is_alphabetic()))),
        "is_digit" => Ok(Value::Bool(s.chars().all(|c| c.is_ascii_digit()))),
        "is_alnum" => Ok(Value::Bool(s.chars().all(|c| c.is_alphanumeric()))),
        "is_upper" => Ok(Value::Bool(s.chars().all(|c| c.is_uppercase()))),
        "is_lower" => Ok(Value::Bool(s.chars().all(|c| c.is_lowercase()))),
        "is_empty" => Ok(Value::Bool(s.trim().is_empty())),
        "reverse" => Ok(Value::Str(s.chars().rev().collect())),
        "count" => {
            let pat = match args.get(1) { Some(Value::Str(p)) => p.clone(), _ => return Err("str2.count: (str, pat)".into()) };
            Ok(Value::Int(s.matches(pat.as_str()).count() as i64))
        }
        "index_of" => {
            let pat = match args.get(1) { Some(Value::Str(p)) => p.clone(), _ => return Err("str2.index_of: (str, pat)".into()) };
            Ok(s.find(pat.as_str()).map(|i| Value::Int(i as i64)).unwrap_or(Value::Int(-1)))
        }
        "starts_with" => {
            let pat = match args.get(1) { Some(Value::Str(p)) => p.clone(), _ => return Err("str2.starts_with: (str, pat)".into()) };
            Ok(Value::Bool(s.starts_with(pat.as_str())))
        }
        "ends_with" => {
            let pat = match args.get(1) { Some(Value::Str(p)) => p.clone(), _ => return Err("str2.ends_with: (str, pat)".into()) };
            Ok(Value::Bool(s.ends_with(pat.as_str())))
        }
        "slice" => {
            let from = match args.get(1) { Some(Value::Int(n)) => *n as usize, _ => 0 };
            let to = match args.get(2) { Some(Value::Int(n)) => *n as usize, _ => s.len() };
            let chars: Vec<char> = s.chars().collect();
            Ok(Value::Str(chars[from.min(chars.len())..to.min(chars.len())].iter().collect()))
        }
        _ => Err(format!("str2.{}: unknown", func))
    }
}
