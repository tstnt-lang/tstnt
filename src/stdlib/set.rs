use crate::value::Value;
use std::collections::HashSet;

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "new" => Ok(Value::Array(vec![])),
        "from" => {
            match args.first() {
                Some(Value::Array(a)) => {
                    let mut seen = HashSet::new();
                    Ok(Value::Array(a.iter().filter(|v| seen.insert(v.to_string())).cloned().collect()))
                }
                _ => Err("set.from: array".into())
            }
        }
        "add" => {
            match (args.first(), args.get(1)) {
                (Some(Value::Array(a)), Some(v)) => {
                    if a.iter().any(|x| x.to_string() == v.to_string()) { return Ok(Value::Array(a.clone())); }
                    let mut new = a.clone(); new.push(v.clone()); Ok(Value::Array(new))
                }
                _ => Err("set.add: (set, val)".into())
            }
        }
        "remove" => {
            match (args.first(), args.get(1)) {
                (Some(Value::Array(a)), Some(v)) => {
                    Ok(Value::Array(a.iter().filter(|x| x.to_string() != v.to_string()).cloned().collect()))
                }
                _ => Err("set.remove: (set, val)".into())
            }
        }
        "has" => {
            match (args.first(), args.get(1)) {
                (Some(Value::Array(a)), Some(v)) => Ok(Value::Bool(a.iter().any(|x| x.to_string() == v.to_string()))),
                _ => Err("set.has: (set, val)".into())
            }
        }
        "union" => {
            match (args.first(), args.get(1)) {
                (Some(Value::Array(a)), Some(Value::Array(b))) => {
                    let mut seen = HashSet::new();
                    let mut result = vec![];
                    for v in a.iter().chain(b.iter()) {
                        if seen.insert(v.to_string()) { result.push(v.clone()); }
                    }
                    Ok(Value::Array(result))
                }
                _ => Err("set.union: (set, set)".into())
            }
        }
        "intersect" => {
            match (args.first(), args.get(1)) {
                (Some(Value::Array(a)), Some(Value::Array(b))) => {
                    let b_set: HashSet<String> = b.iter().map(|v| v.to_string()).collect();
                    Ok(Value::Array(a.iter().filter(|v| b_set.contains(&v.to_string())).cloned().collect()))
                }
                _ => Err("set.intersect: (set, set)".into())
            }
        }
        "difference" => {
            match (args.first(), args.get(1)) {
                (Some(Value::Array(a)), Some(Value::Array(b))) => {
                    let b_set: HashSet<String> = b.iter().map(|v| v.to_string()).collect();
                    Ok(Value::Array(a.iter().filter(|v| !b_set.contains(&v.to_string())).cloned().collect()))
                }
                _ => Err("set.difference: (set, set)".into())
            }
        }
        "size" => match args.first() { Some(Value::Array(a)) => Ok(Value::Int(a.len() as i64)), _ => Ok(Value::Int(0)) }
        "is_empty" => match args.first() { Some(Value::Array(a)) => Ok(Value::Bool(a.is_empty())), _ => Ok(Value::Bool(true)) }
        _ => Err(format!("set.{}: unknown", func))
    }
}
