use crate::value::Value;

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "format" => {
            let template = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("fmt2.format: (template, ...)".into()) };
            let mut result = template.clone();
            for (i, arg) in args.iter().skip(1).enumerate() {
                result = result.replacen("{}", &arg.to_string(), 1);
                result = result.replace(&format!("{{{}}}", i), &arg.to_string());
            }
            Ok(Value::Str(result))
        }
        "pad_left" => {
            let s = match args.first() { Some(Value::Str(s)) => s.clone(), _ => args.first().map(|v| v.to_string()).unwrap_or_default() };
            let width = match args.get(1) { Some(Value::Int(n)) => *n as usize, _ => return Err("fmt2.pad_left: (str, width)".into()) };
            let pad_char = args.get(2).map(|v| v.to_string()).unwrap_or_else(|| " ".into()).chars().next().unwrap_or(' ');
            Ok(Value::Str(format!("{:>width$}", s, width = width).replace(' ', &pad_char.to_string())))
        }
        "pad_right" => {
            let s = match args.first() { Some(Value::Str(s)) => s.clone(), _ => args.first().map(|v| v.to_string()).unwrap_or_default() };
            let width = match args.get(1) { Some(Value::Int(n)) => *n as usize, _ => return Err("fmt2.pad_right: (str, width)".into()) };
            let chars: usize = s.chars().count();
            if chars >= width { return Ok(Value::Str(s)); }
            let pad = " ".repeat(width - chars);
            Ok(Value::Str(format!("{}{}", s, pad)))
        }
        "pad_center" => {
            let s = match args.first() { Some(Value::Str(s)) => s.clone(), _ => args.first().map(|v| v.to_string()).unwrap_or_default() };
            let width = match args.get(1) { Some(Value::Int(n)) => *n as usize, _ => return Err("fmt2.pad_center: (str, width)".into()) };
            let chars = s.chars().count();
            if chars >= width { return Ok(Value::Str(s)); }
            let total_pad = width - chars;
            let left = total_pad / 2;
            let right = total_pad - left;
            Ok(Value::Str(format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))))
        }
        "truncate" => {
            let s = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("fmt2.truncate: (str, len)".into()) };
            let len = match args.get(1) { Some(Value::Int(n)) => *n as usize, _ => return Err("fmt2.truncate: len".into()) };
            let suffix = args.get(2).map(|v| v.to_string()).unwrap_or_else(|| "...".into());
            let chars: Vec<char> = s.chars().collect();
            if chars.len() <= len { return Ok(Value::Str(s)); }
            let cut = len.saturating_sub(suffix.len());
            Ok(Value::Str(format!("{}{}", chars[..cut].iter().collect::<String>(), suffix)))
        }
        "number" => {
            let n = match args.first() { Some(Value::Int(n)) => *n as f64, Some(Value::Float(f)) => *f, _ => return Err("fmt2.number: number".into()) };
            let decimals = match args.get(1) { Some(Value::Int(n)) => *n as usize, _ => 2 };
            Ok(Value::Str(format!("{:.prec$}", n, prec = decimals)))
        }
        "thousands" => {
            let n = match args.first() { Some(Value::Int(n)) => *n, _ => return Err("fmt2.thousands: int".into()) };
            let s = n.abs().to_string();
            let chars: Vec<char> = s.chars().collect();
            let mut result = String::new();
            for (i, c) in chars.iter().rev().enumerate() {
                if i > 0 && i % 3 == 0 { result.push(','); }
                result.push(*c);
            }
            if n < 0 { result.push('-'); }
            Ok(Value::Str(result.chars().rev().collect()))
        }
        "bytes" => {
            let n = match args.first() { Some(Value::Int(n)) => *n as f64, _ => return Err("fmt2.bytes: int".into()) };
            let units = ["B", "KB", "MB", "GB", "TB"];
            let mut size = n;
            let mut unit = units[0];
            for u in &units[1..] {
                if size < 1024.0 { break; }
                size /= 1024.0;
                unit = u;
            }
            if size.fract() == 0.0 { Ok(Value::Str(format!("{} {}", size as i64, unit))) }
            else { Ok(Value::Str(format!("{:.1} {}", size, unit))) }
        }
        "duration" => {
            let ms = match args.first() { Some(Value::Int(n)) => *n, _ => return Err("fmt2.duration: ms".into()) };
            if ms < 1000 { return Ok(Value::Str(format!("{}ms", ms))); }
            let secs = ms / 1000;
            if secs < 60 { return Ok(Value::Str(format!("{}s", secs))); }
            let mins = secs / 60;
            if mins < 60 { return Ok(Value::Str(format!("{}m {}s", mins, secs % 60))); }
            let hrs = mins / 60;
            Ok(Value::Str(format!("{}h {}m", hrs, mins % 60)))
        }
        "plural" => {
            let n = match args.first() { Some(Value::Int(n)) => *n, _ => return Err("fmt2.plural: (n, singular, plural)".into()) };
            let singular = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("fmt2.plural: singular".into()) };
            let plural = match args.get(2) { Some(Value::Str(s)) => s.clone(), _ => format!("{}s", singular) };
            Ok(Value::Str(format!("{} {}", n, if n == 1 { singular } else { plural })))
        }
        "table" => {
            let rows = match args.first() { Some(Value::Array(a)) => a.clone(), _ => return Err("fmt2.table: ([[rows]], [headers])".into()) };
            let headers: Vec<String> = match args.get(1) { Some(Value::Array(a)) => a.iter().map(|v| v.to_string()).collect(), _ => vec![] };
            let row_data: Vec<Vec<String>> = rows.iter().map(|r| match r {
                Value::Array(cells) => cells.iter().map(|v| v.to_string()).collect(),
                v => vec![v.to_string()],
            }).collect();
            let cols = headers.len().max(row_data.iter().map(|r| r.len()).max().unwrap_or(0));
            let col_widths: Vec<usize> = (0..cols).map(|i| {
                let hlen = headers.get(i).map(|h| h.len()).unwrap_or(0);
                let rlen = row_data.iter().map(|r| r.get(i).map(|v| v.len()).unwrap_or(0)).max().unwrap_or(0);
                hlen.max(rlen) + 2
            }).collect();
            let mut out = String::new();
            if !headers.is_empty() {
                out.push_str(&headers.iter().enumerate().map(|(i, h)| format!("{:width$}", h, width = col_widths.get(i).copied().unwrap_or(10))).collect::<Vec<_>>().join(" | "));
                out.push('\n');
                out.push_str(&col_widths.iter().map(|w| "-".repeat(*w)).collect::<Vec<_>>().join("-+-"));
                out.push('\n');
            }
            for row in &row_data {
                out.push_str(&(0..cols).map(|i| format!("{:width$}", row.get(i).cloned().unwrap_or_default(), width = col_widths.get(i).copied().unwrap_or(10))).collect::<Vec<_>>().join(" | "));
                out.push('\n');
            }
            Ok(Value::Str(out))
        }
        _ => Err(format!("fmt2.{}: unknown", func))
    }
}
