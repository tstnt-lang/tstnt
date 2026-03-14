use crate::value::Value;
use std::collections::HashMap;
use std::sync::Mutex;

static HANDLERS: std::sync::OnceLock<Mutex<HashMap<String, Vec<String>>>> = std::sync::OnceLock::new();
fn store() -> &'static Mutex<HashMap<String, Vec<String>>> { HANDLERS.get_or_init(|| Mutex::new(HashMap::new())) }

// Простая система событий — храним имена функций-хендлеров
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "on" => {
            let event = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("event.on: (event, handler_name)".into()) };
            let handler = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("event.on: handler must be str name".into()) };
            store().lock().unwrap().entry(event).or_default().push(handler);
            Ok(Value::Null)
        }
        "handlers" => {
            let event = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("event.handlers: event".into()) };
            let map = store().lock().unwrap();
            let handlers = map.get(&event).cloned().unwrap_or_default();
            Ok(Value::Array(handlers.into_iter().map(Value::Str).collect()))
        }
        "off" => {
            let event = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("event.off: event".into()) };
            store().lock().unwrap().remove(&event);
            Ok(Value::Null)
        }
        "events" => {
            let map = store().lock().unwrap();
            Ok(Value::Array(map.keys().map(|k| Value::Str(k.clone())).collect()))
        }
        _ => Err(format!("event.{}: unknown", func))
    }
}
