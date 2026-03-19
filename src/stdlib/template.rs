use crate::value::Value;
use std::collections::HashMap;

fn render(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (k, v) in vars {
        result = result.replace(&format!("{{{{{}}}}}", k), v);
        result = result.replace(&format!("{{{{ {} }}}}", k), v);
    }
    result
}

fn render_list(template: &str, items: &[Value], var: &str) -> String {
    let mut out = String::new();
    for item in items {
        let rendered = template.replace(&format!("{{{{{}}}}}", var), &item.to_string());
        out.push_str(&rendered);
    }
    out
}

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "render" => {
            let tmpl = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("template.render: (str, struct)".into()) };
            let vars: HashMap<String, String> = match args.get(1) {
                Some(Value::Struct(_, fields)) => fields.iter().map(|(k, v)| (k.clone(), v.to_string())).collect(),
                _ => HashMap::new(),
            };
            Ok(Value::Str(render(&tmpl, &vars)))
        }
        "render_str" => {
            let tmpl = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("template.render_str: (tmpl, key, val)".into()) };
            let key = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("template.render_str: key".into()) };
            let val = args.get(2).map(|v| v.to_string()).unwrap_or_default();
            let mut map = HashMap::new();
            map.insert(key, val);
            Ok(Value::Str(render(&tmpl, &map)))
        }
        "render_list" => {
            let tmpl = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("template.render_list: (tmpl, arr, var)".into()) };
            let items = match args.get(1) { Some(Value::Array(a)) => a.clone(), _ => return Err("template.render_list: array".into()) };
            let var = match args.get(2) { Some(Value::Str(s)) => s.clone(), _ => "item".into() };
            Ok(Value::Str(render_list(&tmpl, &items, &var)))
        }
        "html_escape" => {
            let s = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("template.html_escape: str".into()) };
            Ok(Value::Str(s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")))
        }
        "strip_html" => {
            let s = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("template.strip_html: str".into()) };
            let mut out = String::new();
            let mut in_tag = false;
            for c in s.chars() {
                if c == '<' { in_tag = true; }
                else if c == '>' { in_tag = false; }
                else if !in_tag { out.push(c); }
            }
            Ok(Value::Str(out))
        }
        "fill" => {
            let tmpl = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("template.fill: (tmpl, ...)".into()) };
            let mut result = tmpl.clone();
            for arg in args.iter().skip(1) {
                result = result.replacen("{}", &arg.to_string(), 1);
            }
            Ok(Value::Str(result))
        }
        "repeat" => {
            let tmpl = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("template.repeat: (tmpl, n)".into()) };
            let n = match args.get(1) { Some(Value::Int(n)) => *n as usize, _ => return Err("template.repeat: n".into()) };
            Ok(Value::Str(tmpl.repeat(n)))
        }
        "count_vars" => {
            let tmpl = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("template.count_vars: str".into()) };
            let count = tmpl.matches("{{").count();
            Ok(Value::Int(count as i64))
        }
        _ => Err(format!("template.{}: unknown", func))
    }
}
