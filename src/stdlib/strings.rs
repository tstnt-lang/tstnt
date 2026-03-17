use crate::value::Value;

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "len" => match args.first() {
            Some(Value::Str(s)) => Ok(Value::Int(s.len() as i64)),
            _ => Err("str.len: expected str".into())
        },
        "split" => match (args.first(), args.get(1)) {
            (Some(Value::Str(s)), Some(Value::Str(sep))) => {
                let parts: Vec<Value> = s.split(sep.as_str()).map(|p| Value::Str(p.to_string())).collect();
                Ok(Value::Array(parts))
            }
            _ => Err("str.split: expected (str, str)".into())
        },
        "join" => match (args.first(), args.get(1)) {
            (Some(Value::Array(arr)), Some(Value::Str(sep))) => {
                let parts: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                Ok(Value::Str(parts.join(sep)))
            }
            _ => Err("str.join: expected (arr, str)".into())
        },
        "trim" => match args.first() {
            Some(Value::Str(s)) => Ok(Value::Str(s.trim().to_string())),
            _ => Err("str.trim: expected str".into())
        },
        "upper" => match args.first() {
            Some(Value::Str(s)) => Ok(Value::Str(s.to_uppercase())),
            _ => Err("str.upper: expected str".into())
        },
        "lower" => match args.first() {
            Some(Value::Str(s)) => Ok(Value::Str(s.to_lowercase())),
            _ => Err("str.lower: expected str".into())
        },
        "contains" => match (args.first(), args.get(1)) {
            (Some(Value::Str(s)), Some(Value::Str(pat))) => Ok(Value::Bool(s.contains(pat.as_str()))),
            _ => Err("str.contains: expected (str, str)".into())
        },
        "replace" => match (args.first(), args.get(1), args.get(2)) {
            (Some(Value::Str(s)), Some(Value::Str(from)), Some(Value::Str(to))) => {
                Ok(Value::Str(s.replace(from.as_str(), to)))
            }
            _ => Err("str.replace: expected (str, str, str)".into())
        },
        "starts_with" => match (args.first(), args.get(1)) {
            (Some(Value::Str(s)), Some(Value::Str(pat))) => Ok(Value::Bool(s.starts_with(pat.as_str()))),
            _ => Err("str.starts_with: expected (str, str)".into())
        },
        "ends_with" => match (args.first(), args.get(1)) {
            (Some(Value::Str(s)), Some(Value::Str(pat))) => Ok(Value::Bool(s.ends_with(pat.as_str()))),
            _ => Err("str.ends_with: expected (str, str)".into())
        },
        "repeat" => match (args.first(), args.get(1)) {
            (Some(Value::Str(s)), Some(Value::Int(n))) => Ok(Value::Str(s.repeat(*n as usize))),
            _ => Err("str.repeat: (str, int)".into())
        },
        "reverse" => match args.first() {
            Some(Value::Str(s)) => Ok(Value::Str(s.chars().rev().collect())),
            _ => Err("str.reverse: str".into())
        },
        "pad_left" => match (args.first(), args.get(1)) {
            (Some(Value::Str(s)), Some(Value::Int(n))) => {
                let width = *n as usize;
                let chars = s.chars().count();
                if chars >= width { return Ok(Value::Str(s.clone())); }
                Ok(Value::Str(format!("{}{}", " ".repeat(width - chars), s)))
            }
            _ => Err("str.pad_left: (str, width)".into())
        },
        "pad_right" => match (args.first(), args.get(1)) {
            (Some(Value::Str(s)), Some(Value::Int(n))) => {
                let width = *n as usize;
                let chars = s.chars().count();
                if chars >= width { return Ok(Value::Str(s.clone())); }
                Ok(Value::Str(format!("{}{}", s, " ".repeat(width - chars))))
            }
            _ => Err("str.pad_right: (str, width)".into())
        },
        "title" => match args.first() {
            Some(Value::Str(s)) => Ok(Value::Str(s.split_whitespace().map(|w| {
                let mut c = w.chars();
                match c.next() { None => String::new(), Some(f) => f.to_uppercase().to_string() + c.as_str() }
            }).collect::<Vec<_>>().join(" "))),
            _ => Err("str.title: str".into())
        },
        "snake" => match args.first() {
            Some(Value::Str(s)) => {
                let mut result = String::new();
                for (i, c) in s.chars().enumerate() {
                    if c.is_uppercase() && i > 0 { result.push('_'); }
                    result.push(c.to_lowercase().next().unwrap_or(c));
                }
                Ok(Value::Str(result.replace(' ', "_").replace('-', "_")))
            }
            _ => Err("str.snake: str".into())
        },
        "camel" => match args.first() {
            Some(Value::Str(s)) => {
                let words: Vec<&str> = s.split(|c: char| c == '_' || c == '-' || c == ' ').collect();
                let result = words.iter().enumerate().map(|(i, w)| {
                    if i == 0 { w.to_lowercase() } else {
                        let mut c = w.chars();
                        match c.next() { None => String::new(), Some(f) => f.to_uppercase().to_string() + &c.as_str().to_lowercase() }
                    }
                }).collect();
                Ok(Value::Str(result))
            }
            _ => Err("str.camel: str".into())
        },
        "count" => match (args.first(), args.get(1)) {
            (Some(Value::Str(s)), Some(Value::Str(pat))) => Ok(Value::Int(s.matches(pat.as_str()).count() as i64)),
            _ => Err("str.count: (str, pattern)".into())
        },
        "index_of" => match (args.first(), args.get(1)) {
            (Some(Value::Str(s)), Some(Value::Str(pat))) => Ok(s.find(pat.as_str()).map(|i| Value::Int(i as i64)).unwrap_or(Value::Int(-1))),
            _ => Err("str.index_of: (str, str)".into())
        },
        "slice" => match args.first() {
            Some(Value::Str(s)) => {
                let chars: Vec<char> = s.chars().collect();
                let from = match args.get(1) { Some(Value::Int(n)) => *n as usize, _ => 0 };
                let to = match args.get(2) { Some(Value::Int(n)) => *n as usize, _ => chars.len() };
                Ok(Value::Str(chars[from.min(chars.len())..to.min(chars.len())].iter().collect()))
            }
            _ => Err("str.slice: (str, from, to)".into())
        },
        "trim_left" => match args.first() {
            Some(Value::Str(s)) => Ok(Value::Str(s.trim_start().to_string())),
            _ => Err("str.trim_left: str".into())
        },
        "trim_right" => match args.first() {
            Some(Value::Str(s)) => Ok(Value::Str(s.trim_end().to_string())),
            _ => Err("str.trim_right: str".into())
        },
        _ => Err(format!("str.{}: unknown function", func))
    }
}
