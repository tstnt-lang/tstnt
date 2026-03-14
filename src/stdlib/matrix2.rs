use crate::value::Value;
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "add" => match (args.first(), args.get(1)) {
            (Some(Value::Array(a)), Some(Value::Array(b))) => {
                let result: Vec<Value> = a.iter().zip(b.iter()).map(|(ra, rb)| {
                    match (ra, rb) {
                        (Value::Array(r1), Value::Array(r2)) => Value::Array(r1.iter().zip(r2.iter()).map(|(x, y)| {
                            match (x, y) { (Value::Int(a), Value::Int(b)) => Value::Int(a+b), _ => Value::Int(0) }
                        }).collect()),
                        _ => Value::Null
                    }
                }).collect();
                Ok(Value::Array(result))
            }
            _ => Err("matrix2.add: (matrix, matrix)".into())
        }
        "scale" => match (args.first(), args.get(1)) {
            (Some(Value::Array(m)), Some(Value::Int(s))) => {
                let result: Vec<Value> = m.iter().map(|row| match row {
                    Value::Array(r) => Value::Array(r.iter().map(|x| match x { Value::Int(n) => Value::Int(n*s), _ => x.clone() }).collect()),
                    _ => row.clone()
                }).collect();
                Ok(Value::Array(result))
            }
            _ => Err("matrix2.scale: (matrix, int)".into())
        }
        "zeros" => {
            let n = match args.first() { Some(Value::Int(n)) => *n as usize, _ => 3 };
            let m = match args.get(1) { Some(Value::Int(m)) => *m as usize, _ => n };
            Ok(Value::Array((0..n).map(|_| Value::Array(vec![Value::Int(0); m])).collect()))
        }
        "ones" => {
            let n = match args.first() { Some(Value::Int(n)) => *n as usize, _ => 3 };
            Ok(Value::Array((0..n).map(|_| Value::Array(vec![Value::Int(1); n])).collect()))
        }
        "dot" => match (args.first(), args.get(1)) {
            (Some(Value::Array(a)), Some(Value::Array(b))) => {
                if a.is_empty() { return Ok(Value::Array(vec![])); }
                let rows = a.len();
                let cols = match &b[0] { Value::Array(r) => r.len(), _ => 0 };
                let inner = match &a[0] { Value::Array(r) => r.len(), _ => 0 };
                let mut result = vec![vec![0i64; cols]; rows];
                for i in 0..rows {
                    for j in 0..cols {
                        for k in 0..inner {
                            let av = match &a[i] { Value::Array(r) => match &r[k] { Value::Int(n) => *n, _ => 0 }, _ => 0 };
                            let bv = match &b[k] { Value::Array(r) => match &r[j] { Value::Int(n) => *n, _ => 0 }, _ => 0 };
                            result[i][j] += av * bv;
                        }
                    }
                }
                Ok(Value::Array(result.into_iter().map(|r| Value::Array(r.into_iter().map(Value::Int).collect())).collect()))
            }
            _ => Err("matrix2.dot: (matrix, matrix)".into())
        }
        _ => Err(format!("matrix2.{}: unknown", func))
    }
}
