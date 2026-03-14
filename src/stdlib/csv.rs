use crate::value::Value;

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "parse" => {
            let s = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("csv.parse: str".into()) };
            let rows: Vec<Value> = s.lines().map(|line| {
                Value::Array(line.split(',').map(|c| Value::Str(c.trim().to_string())).collect())
            }).collect();
            Ok(Value::Array(rows))
        }
        "stringify" => {
            let arr = match args.first() { Some(Value::Array(a)) => a.clone(), _ => return Err("csv.stringify: array".into()) };
            let lines: Vec<String> = arr.iter().map(|row| {
                match row { Value::Array(cols) => cols.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(","), v => v.to_string() }
            }).collect();
            Ok(Value::Str(lines.join("\n")))
        }
        "headers" => {
            let s = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("csv.headers: str".into()) };
            let first = s.lines().next().unwrap_or("");
            Ok(Value::Array(first.split(',').map(|c| Value::Str(c.trim().to_string())).collect()))
        }
        _ => Err(format!("csv.{}: unknown", func))
    }
}
