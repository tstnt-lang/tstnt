use crate::value::Value;
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    let s = args.first().map(|v| v.to_string()).unwrap_or_default();
    let wrap = |code: &str| -> String { let mut r = String::from("\x1b["); r.push_str(code); r.push('m'); r.push_str(&s); r.push_str("\x1b[0m"); r };
    match func {
        "red" => Ok(Value::Str(wrap("31"))), "green" => Ok(Value::Str(wrap("32"))),
        "yellow" => Ok(Value::Str(wrap("33"))), "blue" => Ok(Value::Str(wrap("34"))),
        "magenta" => Ok(Value::Str(wrap("35"))), "cyan" => Ok(Value::Str(wrap("36"))),
        "white" => Ok(Value::Str(wrap("37"))), "bold" => Ok(Value::Str(wrap("1"))),
        "dim" => Ok(Value::Str(wrap("2"))), "underline" => Ok(Value::Str(wrap("4"))),
        "blink" => Ok(Value::Str(wrap("5"))), "reset" => Ok(Value::Str("\x1b[0m".into())),
        "bg_red" => Ok(Value::Str(wrap("41"))), "bg_green" => Ok(Value::Str(wrap("42"))),
        "bg_blue" => Ok(Value::Str(wrap("44"))), "bg_yellow" => Ok(Value::Str(wrap("43"))),
        "strip" => {
            let mut r = String::new(); let mut in_esc = false;
            for c in s.chars() { if c == '\x1b' { in_esc = true; } else if in_esc && c == 'm' { in_esc = false; } else if !in_esc { r.push(c); } }
            Ok(Value::Str(r))
        }
        _ => Err(format!("color.{}: unknown", func))
    }
}
