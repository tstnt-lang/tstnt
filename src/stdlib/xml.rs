use crate::value::Value;
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "tag" => {
            let name = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("xml.tag: (name, content)".into()) };
            let content = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            Ok(Value::Str(format!("<{}>{}</{}>", name, content, name)))
        }
        "attr_tag" => {
            let name = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("xml.attr_tag".into()) };
            let attrs = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => String::new() };
            let content = args.get(2).map(|v| v.to_string()).unwrap_or_default();
            Ok(Value::Str(format!("<{} {}>{}</{}>", name, attrs, content, name)))
        }
        "self_close" => {
            let name = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("xml.self_close".into()) };
            let attrs = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            if attrs.is_empty() { Ok(Value::Str(format!("<{}/>", name))) }
            else { Ok(Value::Str(format!("<{} {}/>", name, attrs))) }
        }
        "escape" => {
            let s = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("xml.escape: str".into()) };
            Ok(Value::Str(s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")))
        }
        "wrap" => {
            let tag = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("xml.wrap".into()) };
            let items = match args.get(1) { Some(Value::Array(a)) => a.clone(), _ => return Err("xml.wrap: array".into()) };
            let inner: String = items.iter().map(|v| v.to_string()).collect::<Vec<_>>().join("\n  ");
            Ok(Value::Str(format!("<{}>\n  {}\n</{}>", tag, inner, tag)))
        }
        "header" => Ok(Value::Str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>".into())),
        _ => Err(format!("xml.{}: unknown", func))
    }
}
