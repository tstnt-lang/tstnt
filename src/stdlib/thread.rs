use crate::value::Value;
use std::sync::{Arc, Mutex};

// Глобальный хранилище мьютексов по имени
use std::collections::HashMap;
static MUTEXES: std::sync::OnceLock<Mutex<HashMap<String, Arc<Mutex<Value>>>>> = std::sync::OnceLock::new();

fn get_store() -> &'static Mutex<HashMap<String, Arc<Mutex<Value>>>> {
    MUTEXES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "sleep" => {
            let ms = match args.first() { Some(Value::Int(n)) => *n as u64, _ => 0 };
            std::thread::sleep(std::time::Duration::from_millis(ms));
            Ok(Value::Null)
        }
        "cpu_count" => Ok(Value::Int(std::thread::available_parallelism().map(|n| n.get() as i64).unwrap_or(1))),
        "mutex_new" => {
            let name = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("mutex_new: str".into()) };
            let val = args.get(1).cloned().unwrap_or(Value::Null);
            let mut store = get_store().lock().unwrap();
            store.insert(name, Arc::new(Mutex::new(val)));
            Ok(Value::Null)
        }
        "mutex_get" => {
            let name = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("mutex_get: str".into()) };
            let store = get_store().lock().unwrap();
            if let Some(m) = store.get(&name) { Ok(m.lock().unwrap().clone()) } else { Err(format!("mutex '{}' not found", name)) }
        }
        "mutex_set" => {
            let name = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("mutex_set: str".into()) };
            let val = args.get(1).cloned().unwrap_or(Value::Null);
            let store = get_store().lock().unwrap();
            if let Some(m) = store.get(&name) { *m.lock().unwrap() = val; Ok(Value::Null) } else { Err(format!("mutex '{}' not found", name)) }
        }
        _ => Err(format!("thread.{}: unknown", func))
    }
}
