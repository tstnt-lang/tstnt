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
        _ => Err(format!("math.{}: unknown function", func))
    }
}
