use crate::value::Value;

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    let s = args.first().map(|v| v.to_string()).unwrap_or_default();
    match func {
        "red"     => Ok(Value::Str(format!("\x1b[31m{}\x1b[0m", s))),
        "green"   => Ok(Value::Str(format!("\x1b[32m{}\x1b[0m", s))),
        "yellow"  => Ok(Value::Str(format!("\x1b[33m{}\x1b[0m", s))),
        "blue"    => Ok(Value::Str(format!("\x1b[34m{}\x1b[0m", s))),
        "bold"    => Ok(Value::Str(format!("\x1b[1m{}\x1b[0m", s))),
        "dim"     => Ok(Value::Str(format!("\x1b[2m{}\x1b[0m", s))),
        "reset"   => Ok(Value::Str("\x1b[0m".into())),
        "clear"   => { print!("\x1b[2J\x1b[H"); Ok(Value::Null) }
        "width"   => Ok(Value::Int(80)),
        "height"  => Ok(Value::Int(24)),
        _ => Err(format!("term.{}: unknown", func))
    }
}
