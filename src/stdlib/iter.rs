use crate::value::Value;

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "range" => match (args.first(), args.get(1), args.get(2)) {
            (Some(Value::Int(s)), Some(Value::Int(e)), Some(Value::Int(step))) => {
                let mut v = Vec::new(); let mut i = *s;
                if *step > 0 { while i < *e { v.push(Value::Int(i)); i += step; } }
                else if *step < 0 { while i > *e { v.push(Value::Int(i)); i += step; } }
                Ok(Value::Array(v))
            }
            (Some(Value::Int(s)), Some(Value::Int(e)), None) => {
                Ok(Value::Array((*s..*e).map(Value::Int).collect()))
            }
            (Some(Value::Int(e)), None, None) => {
                Ok(Value::Array((0..*e).map(Value::Int).collect()))
            }
            _ => Err("iter.range: (end) or (start, end) or (start, end, step)".into())
        }
        "enumerate" => match args.first() {
            Some(Value::Array(a)) => Ok(Value::Array(
                a.iter().enumerate().map(|(i, v)| Value::Tuple(vec![Value::Int(i as i64), v.clone()])).collect()
            )),
            Some(Value::Str(s)) => Ok(Value::Array(
                s.chars().enumerate().map(|(i, c)| Value::Tuple(vec![Value::Int(i as i64), Value::Str(c.to_string())])).collect()
            )),
            _ => Err("iter.enumerate: array".into())
        }
        "zip" => match (args.first(), args.get(1)) {
            (Some(Value::Array(a)), Some(Value::Array(b))) => Ok(Value::Array(
                a.iter().zip(b.iter()).map(|(x, y)| Value::Tuple(vec![x.clone(), y.clone()])).collect()
            )),
            _ => Err("iter.zip: (array, array)".into())
        }
        "chain" => {
            let mut result = Vec::new();
            for arg in &args {
                match arg {
                    Value::Array(a) => result.extend(a.clone()),
                    _ => return Err("iter.chain: arrays".into())
                }
            }
            Ok(Value::Array(result))
        }
        "take" => match (args.first(), args.get(1)) {
            (Some(Value::Array(a)), Some(Value::Int(n))) => Ok(Value::Array(a.iter().take(*n as usize).cloned().collect())),
            _ => Err("iter.take: (array, n)".into())
        }
        "skip" => match (args.first(), args.get(1)) {
            (Some(Value::Array(a)), Some(Value::Int(n))) => Ok(Value::Array(a.iter().skip(*n as usize).cloned().collect())),
            _ => Err("iter.skip: (array, n)".into())
        }
        "step_by" => match (args.first(), args.get(1)) {
            (Some(Value::Array(a)), Some(Value::Int(n))) => Ok(Value::Array(a.iter().step_by(*n as usize).cloned().collect())),
            _ => Err("iter.step_by: (array, n)".into())
        }
        "flatten" => match args.first() {
            Some(Value::Array(a)) => {
                let mut r = Vec::new();
                for v in a { match v { Value::Array(inner) => r.extend(inner.clone()), x => r.push(x.clone()) } }
                Ok(Value::Array(r))
            }
            _ => Err("iter.flatten: array".into())
        }
        "chunks" => match (args.first(), args.get(1)) {
            (Some(Value::Array(a)), Some(Value::Int(n))) => Ok(Value::Array(
                a.chunks(*n as usize).map(|c| Value::Array(c.to_vec())).collect()
            )),
            _ => Err("iter.chunks: (array, size)".into())
        }
        "windows" => match (args.first(), args.get(1)) {
            (Some(Value::Array(a)), Some(Value::Int(n))) => Ok(Value::Array(
                a.windows(*n as usize).map(|w| Value::Array(w.to_vec())).collect()
            )),
            _ => Err("iter.windows: (array, size)".into())
        }
        "product" => match (args.first(), args.get(1)) {
            (Some(Value::Array(a)), Some(Value::Array(b))) => {
                let mut r = Vec::new();
                for x in a { for y in b { r.push(Value::Tuple(vec![x.clone(), y.clone()])); } }
                Ok(Value::Array(r))
            }
            _ => Err("iter.product: (array, array)".into())
        }
        "count" => match args.first() {
            Some(Value::Array(a)) => Ok(Value::Int(a.len() as i64)),
            Some(Value::Str(s)) => Ok(Value::Int(s.chars().count() as i64)),
            _ => Err("iter.count: array".into())
        }
        "sum" => match args.first() {
            Some(Value::Array(a)) => {
                let mut s = 0i64;
                for v in a { if let Value::Int(n) = v { s += n; } }
                Ok(Value::Int(s))
            }
            _ => Err("iter.sum: array".into())
        }
        "min" => match args.first() {
            Some(Value::Array(a)) if !a.is_empty() => {
                let mut m = &a[0];
                for v in a { if let (Value::Int(a), Value::Int(b)) = (v, m) { if a < b { m = v; } } }
                Ok(m.clone())
            }
            _ => Err("iter.min: non-empty array".into())
        }
        "max" => match args.first() {
            Some(Value::Array(a)) if !a.is_empty() => {
                let mut m = &a[0];
                for v in a { if let (Value::Int(a), Value::Int(b)) = (v, m) { if a > b { m = v; } } }
                Ok(m.clone())
            }
            _ => Err("iter.max: non-empty array".into())
        }
        "all_equal" => match args.first() {
            Some(Value::Array(a)) => Ok(Value::Bool(a.windows(2).all(|w| w[0].to_string() == w[1].to_string()))),
            _ => Err("iter.all_equal: array".into())
        }
        "unique" => match args.first() {
            Some(Value::Array(a)) => {
                let mut seen = std::collections::HashSet::new();
                Ok(Value::Array(a.iter().filter(|v| seen.insert(v.to_string())).cloned().collect()))
            }
            _ => Err("iter.unique: array".into())
        }
        "sorted" => match args.first() {
            Some(Value::Array(a)) => {
                let mut v = a.clone();
                v.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
                Ok(Value::Array(v))
            }
            _ => Err("iter.sorted: array".into())
        }
        "reversed" => match args.first() {
            Some(Value::Array(a)) => { let mut v = a.clone(); v.reverse(); Ok(Value::Array(v)) }
            Some(Value::Str(s)) => Ok(Value::Str(s.chars().rev().collect())),
            _ => Err("iter.reversed: array or str".into())
        }
        _ => Err(format!("iter.{}: unknown", func))
    }
}
