use crate::value::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn db_path(name: &str) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".tstnt").join("db").join(format!("{}.tsdb", name))
}

fn load_db(name: &str) -> HashMap<String, String> {
    let path = db_path(name);
    if !path.exists() { return HashMap::new(); }
    let content = fs::read_to_string(&path).unwrap_or_default();
    let mut map = HashMap::new();
    for line in content.lines() {
        if let Some(i) = line.find('\t') {
            map.insert(line[..i].to_string(), line[i+1..].to_string());
        }
    }
    map
}

fn save_db(name: &str, map: &HashMap<String, String>) {
    let path = db_path(name);
    fs::create_dir_all(path.parent().unwrap()).ok();
    let content: String = map.iter().map(|(k, v)| format!("{}\t{}\n", k, v)).collect();
    fs::write(&path, content).ok();
}

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "open" => {
            let name = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("db.open: str".into()) };
            let map = load_db(&name);
            println!("[db] opened '{}' ({} records)", name, map.len());
            Ok(Value::Str(name))
        }
        "set" => {
            let db = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("db.set: (db, key, val)".into()) };
            let key = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("db.set: key must be str".into()) };
            let val = args.get(2).map(|v| v.to_string()).unwrap_or_default();
            let mut map = load_db(&db);
            map.insert(key, val);
            save_db(&db, &map);
            Ok(Value::Null)
        }
        "get" => {
            let db = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("db.get: (db, key)".into()) };
            let key = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("db.get: key".into()) };
            let map = load_db(&db);
            Ok(map.get(&key).map(|v| Value::Str(v.clone())).unwrap_or(Value::Null))
        }
        "delete" => {
            let db = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("db.delete: (db, key)".into()) };
            let key = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("db.delete: key".into()) };
            let mut map = load_db(&db);
            map.remove(&key);
            save_db(&db, &map);
            Ok(Value::Null)
        }
        "has" => {
            let db = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("db.has: (db, key)".into()) };
            let key = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("db.has: key".into()) };
            Ok(Value::Bool(load_db(&db).contains_key(&key)))
        }
        "keys" => {
            let db = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("db.keys: db".into()) };
            Ok(Value::Array(load_db(&db).keys().map(|k| Value::Str(k.clone())).collect()))
        }
        "all" => {
            let db = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("db.all: db".into()) };
            let map = load_db(&db);
            Ok(Value::Struct("DB".into(), map.into_iter().map(|(k, v)| (k, Value::Str(v))).collect()))
        }
        "clear" => {
            let db = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("db.clear: db".into()) };
            save_db(&db, &HashMap::new());
            Ok(Value::Null)
        }
        "count" => {
            let db = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("db.count: db".into()) };
            Ok(Value::Int(load_db(&db).len() as i64))
        }
        "incr" => {
            let db = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("db.incr: (db, key)".into()) };
            let key = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("db.incr: key".into()) };
            let by = match args.get(2) { Some(Value::Int(n)) => *n, _ => 1 };
            let mut map = load_db(&db);
            let cur: i64 = map.get(&key).and_then(|v| v.parse().ok()).unwrap_or(0);
            map.insert(key, (cur + by).to_string());
            save_db(&db, &map);
            Ok(Value::Int(cur + by))
        }
        _ => Err(format!("db.{}: unknown", func))
    }
}
