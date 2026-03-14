use crate::value::Value;
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    let f = |v: &Value| -> f64 { match v { Value::Float(f) => *f, Value::Int(n) => *n as f64, _ => 0.0 } };
    match func {
        "sin" => Ok(Value::Float(f(args.first().unwrap_or(&Value::Int(0))).sin())),
        "cos" => Ok(Value::Float(f(args.first().unwrap_or(&Value::Int(0))).cos())),
        "tan" => Ok(Value::Float(f(args.first().unwrap_or(&Value::Int(0))).tan())),
        "atan2" => { let y = f(args.first().unwrap_or(&Value::Int(0))); let x = f(args.get(1).unwrap_or(&Value::Int(1))); Ok(Value::Float(y.atan2(x))) }
        "log" => Ok(Value::Float(f(args.first().unwrap_or(&Value::Int(1))).ln())),
        "log2" => Ok(Value::Float(f(args.first().unwrap_or(&Value::Int(1))).log2())),
        "log10" => Ok(Value::Float(f(args.first().unwrap_or(&Value::Int(1))).log10())),
        "exp" => Ok(Value::Float(f(args.first().unwrap_or(&Value::Int(0))).exp())),
        "pi" => Ok(Value::Float(std::f64::consts::PI)),
        "tau" => Ok(Value::Float(std::f64::consts::TAU)),
        "e" => Ok(Value::Float(std::f64::consts::E)),
        "inf" => Ok(Value::Float(f64::INFINITY)),
        "round" => {
            let v = f(args.first().unwrap_or(&Value::Int(0)));
            let d = match args.get(1) { Some(Value::Int(n)) => *n, _ => 0 };
            let factor = 10f64.powi(d as i32);
            Ok(Value::Float((v * factor).round() / factor))
        }
        "hypot" => { let a = f(args.first().unwrap_or(&Value::Int(0))); let b = f(args.get(1).unwrap_or(&Value::Int(0))); Ok(Value::Float(a.hypot(b))) }
        _ => Err(format!("math2.{}: unknown", func))
    }
}
