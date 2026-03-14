use crate::value::Value;
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    let s1 = || args.first().map(|v| v.to_string()).unwrap_or_default();
    let s2 = || args.get(1).map(|v| v.to_string()).unwrap_or_default();
    match func {
        "tag" => Ok(Value::Str(format!("<{}>{}</{}>", s1(), s2(), s1()))),
        "p" => Ok(Value::Str(format!("<p>{}</p>", s1()))),
        "h1" => Ok(Value::Str(format!("<h1>{}</h1>", s1()))),
        "h2" => Ok(Value::Str(format!("<h2>{}</h2>", s1()))),
        "h3" => Ok(Value::Str(format!("<h3>{}</h3>", s1()))),
        "a" => Ok(Value::Str(format!("<a href=\"{}\">{}</a>", s1(), s2()))),
        "img" => Ok(Value::Str(format!("<img src=\"{}\" alt=\"{}\">", s1(), s2()))),
        "div" => Ok(Value::Str(format!("<div>{}</div>", s1()))),
        "span" => Ok(Value::Str(format!("<span>{}</span>", s1()))),
        "bold" => Ok(Value::Str(format!("<strong>{}</strong>", s1()))),
        "italic" => Ok(Value::Str(format!("<em>{}</em>", s1()))),
        "code" => Ok(Value::Str(format!("<code>{}</code>", s1()))),
        "ul" => {
            let items = match args.first() { Some(Value::Array(a)) => a.clone(), _ => return Err("html.ul: array".into()) };
            let li: String = items.iter().map(|v| format!("  <li>{}</li>", v)).collect::<Vec<_>>().join("\n");
            Ok(Value::Str(format!("<ul>\n{}\n</ul>", li)))
        }
        "ol" => {
            let items = match args.first() { Some(Value::Array(a)) => a.clone(), _ => return Err("html.ol: array".into()) };
            let li: String = items.iter().map(|v| format!("  <li>{}</li>", v)).collect::<Vec<_>>().join("\n");
            Ok(Value::Str(format!("<ol>\n{}\n</ol>", li)))
        }
        "table" => {
            let rows = match args.first() { Some(Value::Array(a)) => a.clone(), _ => return Err("html.table: array of arrays".into()) };
            let trs: String = rows.iter().map(|row| {
                match row {
                    Value::Array(cells) => {
                        let tds: String = cells.iter().map(|c| format!("<td>{}</td>", c)).collect();
                        format!("<tr>{}</tr>", tds)
                    }
                    _ => format!("<tr><td>{}</td></tr>", row)
                }
            }).collect::<Vec<_>>().join("\n");
            Ok(Value::Str(format!("<table>\n{}\n</table>", trs)))
        }
        "escape" => Ok(Value::Str(s1().replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;"))),
        "page" => Ok(Value::Str(format!("<!DOCTYPE html><html><head><meta charset=\"UTF-8\"><title>{}</title></head><body>{}</body></html>", s1(), s2()))),
        _ => Err(format!("html.{}: unknown", func))
    }
}
