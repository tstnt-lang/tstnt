use crate::value::Value;

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "stringify" => match args.first() {
            Some(v) => Ok(Value::Str(to_json(v))),
            _ => Err("json.stringify: expected value".into())
        },
        "parse" => match args.first() {
            Some(Value::Str(s)) => parse_json(s.trim()),
            _ => Err("json.parse: expected str".into())
        },
        _ => Err(format!("json.{}: unknown function", func))
    }
}

fn to_json(v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Int(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Str(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        Value::Array(a) => format!("[{}]", a.iter().map(to_json).collect::<Vec<_>>().join(",")),
        Value::Tuple(t) => format!("[{}]", t.iter().map(to_json).collect::<Vec<_>>().join(",")),
        Value::Struct(_, fields) => {
            let pairs: Vec<String> = fields.iter().map(|(k, v)| format!("\"{}\":{}", k, to_json(v))).collect();
            format!("{{{}}}", pairs.join(","))
        }
        Value::Lambda(_, _) => "\"<lambda>\"".into(),
    }
}

fn parse_json(s: &str) -> Result<Value, String> {
    let s = s.trim();
    if s == "null" { return Ok(Value::Null); }
    if s == "true" { return Ok(Value::Bool(true)); }
    if s == "false" { return Ok(Value::Bool(false)); }
    if s.starts_with('"') && s.ends_with('"') {
        return Ok(Value::Str(s[1..s.len()-1].to_string()));
    }
    if s.starts_with('[') && s.ends_with(']') {
        let inner = &s[1..s.len()-1];
        if inner.trim().is_empty() { return Ok(Value::Array(vec![])); }
        let items = split_json_array(inner);
        let vals: Vec<Value> = items.iter().map(|i| parse_json(i.trim())).collect::<Result<_, _>>()?;
        return Ok(Value::Array(vals));
    }
    if let Ok(n) = s.parse::<i64>() { return Ok(Value::Int(n)); }
    if let Ok(f) = s.parse::<f64>() { return Ok(Value::Float(f)); }
    Err(format!("json.parse: cannot parse: {}", s))
}

fn split_json_array(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut current = String::new();
    let mut in_str = false;
    for c in s.chars() {
        if c == '"' { in_str = !in_str; }
        if !in_str {
            if c == '[' || c == '{' { depth += 1; }
            else if c == ']' || c == '}' { depth -= 1; }
            else if c == ',' && depth == 0 { parts.push(current.clone()); current.clear(); continue; }
        }
        current.push(c);
    }
    if !current.trim().is_empty() { parts.push(current); }
    parts
}
