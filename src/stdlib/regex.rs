use crate::value::Value;

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "match" => match (args.first(), args.get(1)) {
            (Some(Value::Str(text)), Some(Value::Str(pattern))) => {
                Ok(Value::Bool(simple_match(text, pattern)))
            }
            _ => Err("regex.match: expected (str, str)".into())
        },
        "find_all" => match (args.first(), args.get(1)) {
            (Some(Value::Str(text)), Some(Value::Str(pattern))) => {
                let results = simple_find_all(text, pattern);
                Ok(Value::Array(results.into_iter().map(Value::Str).collect()))
            }
            _ => Err("regex.find_all: expected (str, str)".into())
        },
        "replace" => match (args.first(), args.get(1), args.get(2)) {
            (Some(Value::Str(text)), Some(Value::Str(pattern)), Some(Value::Str(repl))) => {
                Ok(Value::Str(simple_replace(text, pattern, repl)))
            }
            _ => Err("regex.replace: expected (str, str, str)".into())
        },
        "split" => match (args.first(), args.get(1)) {
            (Some(Value::Str(text)), Some(Value::Str(pattern))) => {
                let parts: Vec<Value> = text.split(pattern.as_str()).map(|s| Value::Str(s.to_string())).collect();
                Ok(Value::Array(parts))
            }
            _ => Err("regex.split: expected (str, str)".into())
        },
        "count" => match (args.first(), args.get(1)) {
            (Some(Value::Str(text)), Some(Value::Str(pattern))) => {
                Ok(Value::Int(text.matches(pattern.as_str()).count() as i64))
            }
            _ => Err("regex.count: expected (str, str)".into())
        },
        _ => Err(format!("regex.{}: unknown function", func))
    }
}

fn simple_match(text: &str, pattern: &str) -> bool {
    if pattern == "*" { return true; }
    if pattern.starts_with('*') && pattern.ends_with('*') {
        return text.contains(&pattern[1..pattern.len()-1]);
    }
    if pattern.starts_with('*') { return text.ends_with(&pattern[1..]); }
    if pattern.ends_with('*') { return text.starts_with(&pattern[..pattern.len()-1]); }
    text == pattern
}

fn simple_find_all(text: &str, pattern: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut start = 0;
    while let Some(pos) = text[start..].find(pattern) {
        results.push(pattern.to_string());
        start += pos + pattern.len();
        if start >= text.len() { break; }
    }
    results
}

fn simple_replace(text: &str, pattern: &str, repl: &str) -> String {
    text.replace(pattern, repl)
}
