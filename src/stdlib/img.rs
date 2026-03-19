use crate::value::Value;

const CHARS_DARK_TO_LIGHT: &[char] = &[' ','.',':','-','=','+','*','#','@','%'];

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "ascii_box" => {
            let w = match args.first() { Some(Value::Int(n)) => *n as usize, _ => 20 };
            let h = match args.get(1) { Some(Value::Int(n)) => *n as usize, _ => 10 };
            let fill = args.get(2).map(|v| v.to_string()).unwrap_or_else(|| " ".into()).chars().next().unwrap_or(' ');
            let mut out = String::new();
            out.push_str(&format!("┌{}┐\n", "─".repeat(w)));
            for _ in 0..h { out.push_str(&format!("│{}│\n", fill.to_string().repeat(w))); }
            out.push_str(&format!("└{}┘", "─".repeat(w)));
            Ok(Value::Str(out))
        }
        "pixel" => {
            let brightness = match args.first() { Some(Value::Int(n)) => (*n as usize).min(9), _ => 0 };
            Ok(Value::Str(CHARS_DARK_TO_LIGHT[brightness].to_string()))
        }
        "gradient_block" => {
            let w = match args.first() { Some(Value::Int(n)) => *n as usize, _ => 40 };
            let h = match args.get(1) { Some(Value::Int(n)) => *n as usize, _ => 10 };
            let mut out = String::new();
            for y in 0..h {
                for x in 0..w {
                    let b = (x * 9 / w.max(1)).min(9);
                    out.push(CHARS_DARK_TO_LIGHT[b]);
                }
                out.push('\n');
            }
            Ok(Value::Str(out))
        }
        "big_text" => {
            let text = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("img.big_text: str".into()) };
            let mut out = String::new();
            for c in text.chars() {
                match c {
                    'A' => out.push_str(" /\\ \n/--\\\n/  \\\n"),
                    'B' => out.push_str("|B \n|--\n|B_\n"),
                    'C' => out.push_str(" C\n|\n C\n"),
                    'T' => out.push_str("TTT\n T \n T \n"),
                    'S' => out.push_str(" S \n S \n S_\n"),
                    'N' => out.push_str("N N\nNN \nN N\n"),
                    ' ' => out.push_str("   \n   \n   \n"),
                    _ => out.push_str(&format!(" {} \n[ ]\n   \n", c)),
                }
            }
            Ok(Value::Str(out))
        }
        "spinner" => {
            let frame = match args.first() { Some(Value::Int(n)) => *n as usize, _ => 0 };
            let frames = ["⣾","⣽","⣻","⢿","⡿","⣟","⣯","⣷"];
            Ok(Value::Str(frames[frame % frames.len()].to_string()))
        }
        "progress_block" => {
            let pct = match args.first() { Some(Value::Int(n)) => (*n as usize).min(100), _ => 0 };
            let w = match args.get(1) { Some(Value::Int(n)) => *n as usize, _ => 20 };
            let filled = pct * w / 100;
            let bar: String = (0..w).map(|i| if i < filled { '█' } else { '░' }).collect();
            Ok(Value::Str(format!("[{}] {}%", bar, pct)))
        }
        "frame" => {
            let content = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("img.frame: str".into()) };
            let lines: Vec<&str> = content.lines().collect();
            let max_w = lines.iter().map(|l| l.len()).max().unwrap_or(0);
            let mut out = format!("╔{}╗\n", "═".repeat(max_w + 2));
            for line in &lines {
                out.push_str(&format!("║ {:width$} ║\n", line, width = max_w));
            }
            out.push_str(&format!("╚{}╝", "═".repeat(max_w + 2)));
            Ok(Value::Str(out))
        }
        "hline" => {
            let w = match args.first() { Some(Value::Int(n)) => *n as usize, _ => 40 };
            let ch = args.get(1).map(|v| v.to_string()).unwrap_or_else(|| "─".into());
            Ok(Value::Str(ch.repeat(w)))
        }
        "center_text" => {
            let text = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("img.center_text: (str, width)".into()) };
            let w = match args.get(1) { Some(Value::Int(n)) => *n as usize, _ => 40 };
            let len = text.len();
            if len >= w { return Ok(Value::Str(text)); }
            let pad = (w - len) / 2;
            Ok(Value::Str(format!("{}{}{}", " ".repeat(pad), text, " ".repeat(w - len - pad))))
        }
        _ => Err(format!("img.{}: unknown", func))
    }
}
