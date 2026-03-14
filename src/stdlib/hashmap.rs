use crate::value::Value;
use std::collections::HashMap;

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "new" => Ok(Value::Struct("HashMap".into(), HashMap::new())),
        "set" => {
            match (args.first(), args.get(1), args.get(2)) {
                (Some(Value::Struct(_, fields)), Some(Value::Str(k)), Some(v)) => {
                    let mut f = fields.clone();
                    f.insert(k.clone(), v.clone());
                    Ok(Value::Struct("HashMap".into(), f))
                }
                _ => Err("hashmap.set: (map, key, val)".into())
            }
        }
        "get" => {
            match (args.first(), args.get(1)) {
                (Some(Value::Struct(_, fields)), Some(Value::Str(k))) => {
                    Ok(fields.get(k).cloned().unwrap_or(Value::Null))
                }
                _ => Err("hashmap.get: (map, key)".into())
            }
        }
        "has" => {
            match (args.first(), args.get(1)) {
                (Some(Value::Struct(_, fields)), Some(Value::Str(k))) => Ok(Value::Bool(fields.contains_key(k))),
                _ => Err("hashmap.has: (map, key)".into())
            }
        }
        "delete" => {
            match (args.first(), args.get(1)) {
                (Some(Value::Struct(_, fields)), Some(Value::Str(k))) => {
                    let mut f = fields.clone();
                    f.remove(k);
                    Ok(Value::Struct("HashMap".into(), f))
                }
                _ => Err("hashmap.delete: (map, key)".into())
            }
        }
        "keys" => {
            match args.first() {
                Some(Value::Struct(_, fields)) => Ok(Value::Array(fields.keys().map(|k| Value::Str(k.clone())).collect())),
                _ => Err("hashmap.keys: map".into())
            }
        }
        "values" => {
            match args.first() {
                Some(Value::Struct(_, fields)) => Ok(Value::Array(fields.values().cloned().collect())),
                _ => Err("hashmap.values: map".into())
            }
        }
        "size" => {
            match args.first() {
                Some(Value::Struct(_, fields)) => Ok(Value::Int(fields.len() as i64)),
                _ => Ok(Value::Int(0))
            }
        }
        "merge" => {
            match (args.first(), args.get(1)) {
                (Some(Value::Struct(_, a)), Some(Value::Struct(_, b))) => {
                    let mut merged = a.clone();
                    merged.extend(b.clone());
                    Ok(Value::Struct("HashMap".into(), merged))
                }
                _ => Err("hashmap.merge: (map, map)".into())
            }
        }
        "from_keys" => {
            match (args.first(), args.get(1)) {
                (Some(Value::Array(keys)), Some(default)) => {
                    let mut map = HashMap::new();
                    for k in keys { map.insert(k.to_string(), default.clone()); }
                    Ok(Value::Struct("HashMap".into(), map))
                }
                _ => Err("hashmap.from_keys: (keys, default)".into())
            }
        }
        "to_array" => {
            match args.first() {
                Some(Value::Struct(_, fields)) => {
                    Ok(Value::Array(fields.iter().map(|(k, v)| Value::Array(vec![Value::Str(k.clone()), v.clone()])).collect()))
                }
                _ => Err("hashmap.to_array: map".into())
            }
        }
        _ => Err(format!("hashmap.{}: unknown", func))
    }
}
