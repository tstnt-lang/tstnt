use crate::value::Value;

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    let s = args.first().map(|v| v.to_string()).unwrap_or_default();
    let wrap = |code: &str, text: &str| -> String { format!("\x1b[{}m{}\x1b[0m", code, text) };
    match func {
        "red"     => Ok(Value::Str(wrap("31", &s))),
        "green"   => Ok(Value::Str(wrap("32", &s))),
        "yellow"  => Ok(Value::Str(wrap("33", &s))),
        "blue"    => Ok(Value::Str(wrap("34", &s))),
        "magenta" => Ok(Value::Str(wrap("35", &s))),
        "cyan"    => Ok(Value::Str(wrap("36", &s))),
        "white"   => Ok(Value::Str(wrap("37", &s))),
        "bold"    => Ok(Value::Str(wrap("1", &s))),
        "dim"     => Ok(Value::Str(wrap("2", &s))),
        "italic"  => Ok(Value::Str(wrap("3", &s))),
        "underline" => Ok(Value::Str(wrap("4", &s))),
        "blink"   => Ok(Value::Str(wrap("5", &s))),
        "strikethrough" => Ok(Value::Str(wrap("9", &s))),
        "reset"   => Ok(Value::Str("\x1b[0m".into())),
        "clear"   => { print!("\x1b[2J\x1b[H"); Ok(Value::Null) }
        "clear_line" => { print!("\x1b[2K\r"); Ok(Value::Null) }
        "bg_red"     => Ok(Value::Str(wrap("41", &s))),
        "bg_green"   => Ok(Value::Str(wrap("42", &s))),
        "bg_yellow"  => Ok(Value::Str(wrap("43", &s))),
        "bg_blue"    => Ok(Value::Str(wrap("44", &s))),
        "bg_magenta" => Ok(Value::Str(wrap("45", &s))),
        "bg_cyan"    => Ok(Value::Str(wrap("46", &s))),
        "bg_white"   => Ok(Value::Str(wrap("47", &s))),
        "width"   => {
            if let Ok(output) = std::process::Command::new("tput").arg("cols").output() {
                let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                Ok(Value::Int(s.parse().unwrap_or(80)))
            } else { Ok(Value::Int(80)) }
        }
        "height"  => {
            if let Ok(output) = std::process::Command::new("tput").arg("lines").output() {
                let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                Ok(Value::Int(s.parse().unwrap_or(24)))
            } else { Ok(Value::Int(24)) }
        }
        "strip" => {
            let mut r = String::new(); let mut in_esc = false;
            for c in s.chars() { if c == '\x1b' { in_esc = true; } else if in_esc && c == 'm' { in_esc = false; } else if !in_esc { r.push(c); } }
            Ok(Value::Str(r))
        }
        _ => Err(format!("term.{}: unknown", func))
    }
}

