use crate::value::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicU64, Ordering};

static SEED: AtomicU64 = AtomicU64::new(0);

fn next() -> u64 {
    let mut s = SEED.load(Ordering::Relaxed);
    if s == 0 {
        s = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(12345);
    }
    s ^= s << 13;
    s ^= s >> 7;
    s ^= s << 17;
    SEED.store(s, Ordering::Relaxed);
    s
}

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "int" => match (args.first(), args.get(1)) {
            (Some(Value::Int(lo)), Some(Value::Int(hi))) => {
                if hi <= lo { return Err("rand.int: hi must be > lo".into()); }
                Ok(Value::Int(lo + (next() as i64).abs() % (hi - lo)))
            }
            _ => Err("rand.int: expected (int, int)".into())
        },
        "float" => Ok(Value::Float((next() as f64) / (u64::MAX as f64))),
        "bool" => Ok(Value::Bool(next() % 2 == 0)),
        "seed" => match args.first() {
            Some(Value::Int(n)) => { SEED.store(*n as u64, Ordering::Relaxed); Ok(Value::Null) }
            _ => Err("rand.seed: expected int".into())
        },
        "choice" => match args.first() {
            Some(Value::Array(a)) if !a.is_empty() => {
                Ok(a[(next() as usize) % a.len()].clone())
            }
            _ => Err("rand.choice: expected non-empty array".into())
        },
        "shuffle" => match args.first() {
            Some(Value::Array(a)) => {
                let mut v = a.clone();
                for i in (1..v.len()).rev() {
                    let j = (next() as usize) % (i + 1);
                    v.swap(i, j);
                }
                Ok(Value::Array(v))
            }
            _ => Err("rand.shuffle: expected array".into())
        },
        _ => Err(format!("rand.{}: unknown function", func))
    }
}
