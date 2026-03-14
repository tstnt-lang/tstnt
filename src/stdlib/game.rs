use crate::value::Value;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static STATE: std::sync::OnceLock<Mutex<HashMap<String, Value>>> = std::sync::OnceLock::new();
fn store() -> &'static Mutex<HashMap<String, Value>> { STATE.get_or_init(|| Mutex::new(HashMap::new())) }

fn now_ms() -> i64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as i64 }

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        // Состояние игры
        "set" => {
            let key = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("game.set: key".into()) };
            let val = args.get(1).cloned().unwrap_or(Value::Null);
            store().lock().unwrap().insert(key, val);
            Ok(Value::Null)
        }
        "get" => {
            let key = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("game.get: key".into()) };
            Ok(store().lock().unwrap().get(&key).cloned().unwrap_or(Value::Null))
        }
        "has" => {
            let key = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("game.has: key".into()) };
            Ok(Value::Bool(store().lock().unwrap().contains_key(&key)))
        }
        "del" => {
            let key = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("game.del: key".into()) };
            store().lock().unwrap().remove(&key);
            Ok(Value::Null)
        }
        "all" => {
            let map = store().lock().unwrap().clone();
            Ok(Value::Struct("GameState".into(), map))
        }
        "reset" => { store().lock().unwrap().clear(); Ok(Value::Null) }

        // RNG для игр
        "roll" => {
            let sides = match args.first() { Some(Value::Int(n)) => *n, _ => 6 };
            let seed = now_ms() as u64 ^ 0xdeadbeef;
            Ok(Value::Int((seed % sides as u64) as i64 + 1))
        }
        "roll_adv" => {
            // Roll with advantage (2d, take higher)
            let sides = match args.first() { Some(Value::Int(n)) => *n, _ => 20 };
            let t = now_ms() as u64;
            let a = (t % sides as u64) as i64 + 1;
            let b = ((t ^ 0xabcd1234) % sides as u64) as i64 + 1;
            Ok(Value::Int(a.max(b)))
        }
        "chance" => {
            let pct = match args.first() { Some(Value::Int(n)) => *n, _ => 50 };
            let r = (now_ms() as u64 % 100) as i64;
            Ok(Value::Bool(r < pct))
        }

        // Математика для игр
        "clamp" => {
            match (args.first(), args.get(1), args.get(2)) {
                (Some(Value::Int(v)), Some(Value::Int(lo)), Some(Value::Int(hi))) => Ok(Value::Int((*v).max(*lo).min(*hi))),
                (Some(Value::Float(v)), Some(Value::Float(lo)), Some(Value::Float(hi))) => Ok(Value::Float(v.max(*lo).min(*hi))),
                _ => Err("game.clamp: (val, min, max)".into())
            }
        }
        "lerp" => {
            match (args.first(), args.get(1), args.get(2)) {
                (Some(Value::Float(a)), Some(Value::Float(b)), Some(Value::Float(t))) => Ok(Value::Float(a + (b - a) * t)),
                (Some(Value::Int(a)), Some(Value::Int(b)), Some(Value::Float(t))) => Ok(Value::Float(*a as f64 + (*b - *a) as f64 * t)),
                _ => Err("game.lerp: (a, b, t)".into())
            }
        }
        "distance" => {
            match (args.first(), args.get(1), args.get(2), args.get(3)) {
                (Some(Value::Int(x1)), Some(Value::Int(y1)), Some(Value::Int(x2)), Some(Value::Int(y2))) => {
                    let dx = (x2 - x1) as f64; let dy = (y2 - y1) as f64;
                    Ok(Value::Float((dx*dx + dy*dy).sqrt()))
                }
                _ => Err("game.distance: (x1,y1,x2,y2)".into())
            }
        }

        // Таймер
        "timer_start" => { store().lock().unwrap().insert("__timer__".into(), Value::Int(now_ms())); Ok(Value::Null) }
        "timer_elapsed" => {
            let start = match store().lock().unwrap().get("__timer__").cloned() { Some(Value::Int(n)) => n, _ => now_ms() };
            Ok(Value::Int(now_ms() - start))
        }

        // Инвентарь
        "inv_add" => {
            let item = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("game.inv_add: str".into()) };
            let mut st = store().lock().unwrap();
            let inv = st.entry("__inv__".into()).or_insert(Value::Array(vec![]));
            if let Value::Array(arr) = inv { arr.push(Value::Str(item)); }
            Ok(Value::Null)
        }
        "inv_remove" => {
            let item = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("game.inv_remove: str".into()) };
            let mut st = store().lock().unwrap();
            if let Some(Value::Array(arr)) = st.get_mut("__inv__") { arr.retain(|v| v.to_string() != item); }
            Ok(Value::Null)
        }
        "inv_has" => {
            let item = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("game.inv_has: str".into()) };
            let st = store().lock().unwrap();
            let found = st.get("__inv__").map(|v| if let Value::Array(arr) = v { arr.iter().any(|x| x.to_string() == item) } else { false }).unwrap_or(false);
            Ok(Value::Bool(found))
        }
        "inv_list" => {
            let st = store().lock().unwrap();
            Ok(st.get("__inv__").cloned().unwrap_or(Value::Array(vec![])))
        }

        // Карта/уровень
        "map_new" => {
            let w = match args.first() { Some(Value::Int(n)) => *n, _ => 10 };
            let h = match args.get(1) { Some(Value::Int(n)) => *n, _ => 10 };
            let fill = args.get(2).cloned().unwrap_or(Value::Int(0));
            let grid: Vec<Value> = (0..h).map(|_| Value::Array((0..w).map(|_| fill.clone()).collect())).collect();
            Ok(Value::Array(grid))
        }
        "map_get" => {
            match (args.first(), args.get(1), args.get(2)) {
                (Some(Value::Array(grid)), Some(Value::Int(y)), Some(Value::Int(x))) => {
                    if let Some(Value::Array(row)) = grid.get(*y as usize) { Ok(row.get(*x as usize).cloned().unwrap_or(Value::Null)) }
                    else { Ok(Value::Null) }
                }
                _ => Err("game.map_get: (grid, y, x)".into())
            }
        }
        "map_set" => {
            match (args.first(), args.get(1), args.get(2), args.get(3)) {
                (Some(Value::Array(grid)), Some(Value::Int(y)), Some(Value::Int(x)), Some(val)) => {
                    let mut grid = grid.clone();
                    if let Some(Value::Array(row)) = grid.get_mut(*y as usize) {
                        if (*x as usize) < row.len() { row[*x as usize] = val.clone(); }
                    }
                    Ok(Value::Array(grid))
                }
                _ => Err("game.map_set: (grid, y, x, val)".into())
            }
        }

        _ => Err(format!("game.{}: unknown", func))
    }
}
