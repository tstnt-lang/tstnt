use crate::value::Value;
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    let s = || args.first().map(|v| v.to_string()).unwrap_or_default();
    match func {
        "rgb" => {
            let r = match args.first() { Some(Value::Int(n)) => *n, _ => 0 };
            let g = match args.get(1) { Some(Value::Int(n)) => *n, _ => 0 };
            let b = match args.get(2) { Some(Value::Int(n)) => *n, _ => 0 };
            let text = args.get(3).map(|v| v.to_string()).unwrap_or_default();
            Ok(Value::Str(format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, text)))
        }
        "bg" => {
            let r = match args.first() { Some(Value::Int(n)) => *n, _ => 0 };
            let g = match args.get(1) { Some(Value::Int(n)) => *n, _ => 0 };
            let b = match args.get(2) { Some(Value::Int(n)) => *n, _ => 0 };
            let text = args.get(3).map(|v| v.to_string()).unwrap_or_default();
            Ok(Value::Str(format!("\x1b[48;2;{};{};{}m{}\x1b[0m", r, g, b, text)))
        }
        "hex_to_rgb" => {
            let h = s().trim_start_matches('#').to_string();
            if h.len() < 6 { return Err("color2.hex_to_rgb: invalid hex".into()); }
            let r = i64::from_str_radix(&h[0..2], 16).unwrap_or(0);
            let g = i64::from_str_radix(&h[2..4], 16).unwrap_or(0);
            let b = i64::from_str_radix(&h[4..6], 16).unwrap_or(0);
            Ok(Value::Array(vec![Value::Int(r), Value::Int(g), Value::Int(b)]))
        }
        "mix" => {
            match (args.first(), args.get(1)) {
                (Some(Value::Array(a)), Some(Value::Array(b))) => {
                    let mix_comp = |i: usize| match (a.get(i), b.get(i)) {
                        (Some(Value::Int(x)), Some(Value::Int(y))) => Value::Int((x + y) / 2),
                        _ => Value::Int(0)
                    };
                    Ok(Value::Array(vec![mix_comp(0), mix_comp(1), mix_comp(2)]))
                }
                _ => Err("color2.mix: ([r,g,b], [r,g,b])".into())
            }
        }
        "to_hex" => {
            match args.first() {
                Some(Value::Array(rgb)) if rgb.len() >= 3 => {
                    let r = match &rgb[0] { Value::Int(n) => *n, _ => 0 };
                    let g = match &rgb[1] { Value::Int(n) => *n, _ => 0 };
                    let b = match &rgb[2] { Value::Int(n) => *n, _ => 0 };
                    Ok(Value::Str(format!("#{:02x}{:02x}{:02x}", r, g, b)))
                }
                _ => Err("color2.to_hex: [r,g,b]".into())
            }
        }
        _ => Err(format!("color2.{}: unknown", func))
    }
}
