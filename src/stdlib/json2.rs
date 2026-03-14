use crate::value::Value;
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "pretty" => {
            fn fmt(v: &Value, d: usize) -> String {
                let pad = "  ".repeat(d);
                match v {
                    Value::Str(s) => format!("\"{}\"", s.replace('"', "\\\"")),
                    Value::Int(n) => n.to_string(), Value::Float(f) => f.to_string(),
                    Value::Bool(b) => b.to_string(), Value::Null => "null".into(),
                    Value::Array(a) => { if a.is_empty() { return "[]".into(); } let items: Vec<_> = a.iter().map(|x| format!("{}  {}", pad, fmt(x, d+1))).collect(); format!("[\n{}\n{}]", items.join(",\n"), pad) }
                    Value::Struct(_, f) => { let items: Vec<_> = f.iter().map(|(k, v)| format!("{}  \"{}\": {}", pad, k, fmt(v, d+1))).collect(); format!("{{\n{}\n{}}}", items.join(",\n"), pad) }
                    _ => format!("\"{}\"", v)
                }
            }
            Ok(Value::Str(fmt(args.first().unwrap_or(&Value::Null), 0)))
        }
        "minify" => {
            let s = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("json2.minify: str".into()) };
            Ok(Value::Str(s.split_whitespace().collect::<Vec<_>>().join("")))
        }
        _ => Err(format!("json2.{}: unknown", func))
    }
}
