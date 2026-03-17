use crate::value::Value;

fn to_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Int(n) => Some(*n as f64),
        Value::Float(f) => Some(*f),
        _ => None,
    }
}

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "sqrt" => {
            let n = to_f64(args.first().ok_or("sqrt: expected arg")?).ok_or("sqrt: expected number")?;
            Ok(Value::Float(n.sqrt()))
        }
        "pow" => {
            let base = to_f64(args.first().ok_or("pow: expected 2 args")?).ok_or("pow: expected number")?;
            let exp = to_f64(args.get(1).ok_or("pow: expected 2 args")?).ok_or("pow: expected number")?;
            Ok(Value::Float(base.powf(exp)))
        }
        "abs" => match args.first() {
            Some(Value::Int(n)) => Ok(Value::Int(n.abs())),
            Some(Value::Float(f)) => Ok(Value::Float(f.abs())),
            _ => Err("abs: expected number".into())
        },
        "floor" => {
            let n = to_f64(args.first().ok_or("floor: expected arg")?).ok_or("floor: expected number")?;
            Ok(Value::Int(n.floor() as i64))
        }
        "ceil" => {
            let n = to_f64(args.first().ok_or("ceil: expected arg")?).ok_or("ceil: expected number")?;
            Ok(Value::Int(n.ceil() as i64))
        }
        "min" => {
            let a = to_f64(args.first().ok_or("min: expected 2 args")?).ok_or("min: expected number")?;
            let b = to_f64(args.get(1).ok_or("min: expected 2 args")?).ok_or("min: expected number")?;
            Ok(Value::Float(a.min(b)))
        }
        "max" => {
            let a = to_f64(args.first().ok_or("max: expected 2 args")?).ok_or("max: expected number")?;
            let b = to_f64(args.get(1).ok_or("max: expected 2 args")?).ok_or("max: expected number")?;
            Ok(Value::Float(a.max(b)))
        }
        "round" => {
            let n = to_f64(args.first().ok_or("round: expected arg")?).ok_or("round: expected number")?;
            let d = match args.get(1) { Some(Value::Int(n)) => *n, _ => 0 };
            let factor = 10f64.powi(d as i32);
            Ok(Value::Float((n * factor).round() / factor))
        }
        "sign" => match args.first() {
            Some(Value::Int(n)) => Ok(Value::Int(if *n > 0 { 1 } else if *n < 0 { -1 } else { 0 })),
            Some(Value::Float(f)) => Ok(Value::Float(if *f > 0.0 { 1.0 } else if *f < 0.0 { -1.0 } else { 0.0 })),
            _ => Err("sign: expected number".into())
        }
        "trunc" => {
            let n = to_f64(args.first().ok_or("trunc: expected arg")?).ok_or("trunc: expected number")?;
            Ok(Value::Int(n.trunc() as i64))
        }
        "sum" => match args.first() {
            Some(Value::Array(a)) => {
                let mut s = 0f64;
                for v in a { if let Some(f) = to_f64(v) { s += f; } }
                if s.fract() == 0.0 { Ok(Value::Int(s as i64)) } else { Ok(Value::Float(s)) }
            }
            _ => Err("math.sum: array".into())
        }
        "avg" => match args.first() {
            Some(Value::Array(a)) if !a.is_empty() => {
                let s: f64 = a.iter().filter_map(|v| to_f64(v)).sum();
                Ok(Value::Float(s / a.len() as f64))
            }
            _ => Err("math.avg: non-empty array".into())
        }
        _ => Err(format!("math.{}: unknown function", func))
    }
}
