use crate::value::Value;
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    let f = |v: &Value| -> f64 { match v { Value::Float(f) => *f, Value::Int(n) => *n as f64, _ => 0.0 } };
    let f1 = || f(args.first().unwrap_or(&Value::Int(0)));
    let f2 = || f(args.get(1).unwrap_or(&Value::Int(0)));
    match func {
        "factorial" => {
            let n = match args.first() { Some(Value::Int(n)) => *n, _ => 0 };
            let mut result = 1i64;
            for i in 2..=n { result = result.saturating_mul(i); }
            Ok(Value::Int(result))
        }
        "fib" => {
            let n = match args.first() { Some(Value::Int(n)) => *n, _ => 0 };
            let (mut a, mut b) = (0i64, 1i64);
            for _ in 0..n { let c = a.wrapping_add(b); a = b; b = c; }
            Ok(Value::Int(a))
        }
        "gcd" => {
            let mut a = match args.first() { Some(Value::Int(n)) => n.unsigned_abs(), _ => 0 };
            let mut b = match args.get(1) { Some(Value::Int(n)) => n.unsigned_abs(), _ => 0 };
            while b != 0 { let t = b; b = a % b; a = t; }
            Ok(Value::Int(a as i64))
        }
        "lcm" => {
            let a = match args.first() { Some(Value::Int(n)) => *n, _ => 0 };
            let b = match args.get(1) { Some(Value::Int(n)) => *n, _ => 0 };
            if a == 0 || b == 0 { return Ok(Value::Int(0)); }
            let mut ga = a.unsigned_abs(); let mut gb = b.unsigned_abs();
            while gb != 0 { let t = gb; gb = ga % gb; ga = t; }
            Ok(Value::Int((a.abs() / ga as i64) * b.abs()))
        }
        "is_prime" => {
            let n = match args.first() { Some(Value::Int(n)) => *n, _ => 0 };
            if n < 2 { return Ok(Value::Bool(false)); }
            let mut i = 2i64;
            while i * i <= n { if n % i == 0 { return Ok(Value::Bool(false)); } i += 1; }
            Ok(Value::Bool(true))
        }
        "clamp" => {
            let v = f1(); let lo = f2(); let hi = f(args.get(2).unwrap_or(&Value::Int(0)));
            Ok(Value::Float(v.max(lo).min(hi)))
        }
        "lerp" => Ok(Value::Float(f1() + (f2() - f1()) * f(args.get(2).unwrap_or(&Value::Float(0.5))))),
        "map_range" => {
            let v = f1(); let in_min = f2();
            let in_max = f(args.get(2).unwrap_or(&Value::Int(1)));
            let out_min = f(args.get(3).unwrap_or(&Value::Int(0)));
            let out_max = f(args.get(4).unwrap_or(&Value::Int(1)));
            Ok(Value::Float((v - in_min) / (in_max - in_min) * (out_max - out_min) + out_min))
        }
        _ => Err(format!("math3.{}: unknown", func))
    }
}
