use crate::value::Value;
use std::io::{self, Write};
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "print_table" => {
            let arr = match args.first() { Some(Value::Array(a)) => a.clone(), _ => return Err("io2.print_table: array".into()) };
            let headers = match args.get(1) { Some(Value::Array(h)) => h.iter().map(|v| v.to_string()).collect::<Vec<_>>(), _ => vec![] };
            if !headers.is_empty() {
                let row = headers.iter().map(|h| format!("{:15}", h)).collect::<Vec<_>>().join(" | ");
                println!("{}", row);
                println!("{}", "-".repeat(row.len()));
            }
            for item in &arr {
                match item {
                    Value::Array(row) => println!("{}", row.iter().map(|v| format!("{:15}", v)).collect::<Vec<_>>().join(" | ")),
                    Value::Struct(_, fields) => println!("{}", fields.values().map(|v| format!("{:15}", v)).collect::<Vec<_>>().join(" | ")),
                    v => println!("{}", v),
                }
            }
            Ok(Value::Null)
        }
        "print_json" => {
            let v = args.first().cloned().unwrap_or(Value::Null);
            fn fmt(v: &Value, indent: usize) -> String {
                let pad = "  ".repeat(indent);
                match v {
                    Value::Str(s) => format!("\"{}\"", s),
                    Value::Int(n) => n.to_string(),
                    Value::Float(f) => f.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => "null".into(),
                    Value::Array(a) => {
                        if a.is_empty() { return "[]".into(); }
                        let items: Vec<String> = a.iter().map(|x| format!("{}  {}", pad, fmt(x, indent+1))).collect();
                        format!("[\n{}\n{}]", items.join(",\n"), pad)
                    }
                    Value::Struct(_, fields) => {
                        let items: Vec<String> = fields.iter().map(|(k, v)| format!("{}  \"{}\": {}", pad, k, fmt(v, indent+1))).collect();
                        format!("{{\n{}\n{}}}", items.join(",\n"), pad)
                    }
                    _ => format!("\"{}\"", v)
                }
            }
            println!("{}", fmt(&v, 0));
            Ok(Value::Null)
        }
        "progress" => {
            let cur = match args.first() { Some(Value::Int(n)) => *n, _ => 0 };
            let total = match args.get(1) { Some(Value::Int(n)) => *n, _ => 100 };
            let width = 30i64;
            let filled = (cur * width / total).min(width);
            let bar: String = (0..width).map(|i| if i < filled { '█' } else { '░' }).collect();
            print!("\r[{}] {}%", bar, cur * 100 / total);
            io::stdout().flush().ok();
            if cur >= total { println!(); }
            Ok(Value::Null)
        }
        "clear" => { print!("\x1b[2J\x1b[H"); io::stdout().flush().ok(); Ok(Value::Null) }
        "bell" => { print!("\x07"); io::stdout().flush().ok(); Ok(Value::Null) }
        _ => Err(format!("io2.{}: unknown", func))
    }
}
