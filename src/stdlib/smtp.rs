use crate::value::Value;
use std::process::Command;
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "send" => {
            let to = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("smtp.send: (to, subject, body)".into()) };
            let subject = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("smtp.send: subject".into()) };
            let body = match args.get(2) { Some(Value::Str(s)) => s.clone(), _ => return Err("smtp.send: body".into()) };
            // Uses local sendmail or msmtp if available
            let cmd = format!("echo '{}' | mail -s '{}' '{}'", body, subject, to);
            let ok = Command::new("sh").args(["-c", &cmd]).status().map(|s| s.success()).unwrap_or(false);
            Ok(Value::Bool(ok))
        }
        "format" => {
            let to = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("smtp.format".into()) };
            let subject = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("smtp.format".into()) };
            let body = match args.get(2) { Some(Value::Str(s)) => s.clone(), _ => return Err("smtp.format".into()) };
            Ok(Value::Str(format!("To: {}\nSubject: {}\n\n{}", to, subject, body)))
        }
        _ => Err(format!("smtp.{}: unknown", func))
    }
}
