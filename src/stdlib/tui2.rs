use crate::value::Value;
use std::io::{self, Write};

fn move_to(row: u16, col: u16) -> String { format!("\x1b[{};{}H", row, col) }
fn clear_screen() -> String { "\x1b[2J\x1b[H".to_string() }
fn hide_cursor() -> String { "\x1b[?25l".to_string() }
fn show_cursor() -> String { "\x1b[?25h".to_string() }

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    let stdout = io::stdout();
    match func {
        "clear" => { print!("{}", clear_screen()); io::stdout().flush().ok(); Ok(Value::Null) }
        "hide_cursor" => { print!("{}", hide_cursor()); io::stdout().flush().ok(); Ok(Value::Null) }
        "show_cursor" => { print!("{}", show_cursor()); io::stdout().flush().ok(); Ok(Value::Null) }
        "move_to" => {
            let row = match args.first() { Some(Value::Int(n)) => *n as u16, _ => 1 };
            let col = match args.get(1) { Some(Value::Int(n)) => *n as u16, _ => 1 };
            print!("{}", move_to(row, col)); io::stdout().flush().ok();
            Ok(Value::Null)
        }
        "print_at" => {
            let row = match args.first() { Some(Value::Int(n)) => *n as u16, _ => 1 };
            let col = match args.get(1) { Some(Value::Int(n)) => *n as u16, _ => 1 };
            let text = args.get(2).map(|v| v.to_string()).unwrap_or_default();
            print!("{}{}", move_to(row, col), text); io::stdout().flush().ok();
            Ok(Value::Null)
        }
        "box" => {
            let row = match args.first() { Some(Value::Int(n)) => *n as u16, _ => 1 };
            let col = match args.get(1) { Some(Value::Int(n)) => *n as u16, _ => 1 };
            let width = match args.get(2) { Some(Value::Int(n)) => *n as usize, _ => 20 };
            let height = match args.get(3) { Some(Value::Int(n)) => *n as usize, _ => 5 };
            let title = args.get(4).map(|v| v.to_string()).unwrap_or_default();
            let top_border = if title.is_empty() {
                format!("┌{}┐", "─".repeat(width))
            } else {
                let t = format!(" {} ", title);
                let rem = width.saturating_sub(t.len());
                format!("┌{}{}{}┐", "─".repeat(rem/2), t, "─".repeat(rem - rem/2))
            };
            print!("{}{}", move_to(row, col), top_border);
            for i in 1..height { print!("{}│{}│", move_to(row + i as u16, col), " ".repeat(width)); }
            print!("{}└{}┘", move_to(row + height as u16, col), "─".repeat(width));
            io::stdout().flush().ok();
            Ok(Value::Null)
        }
        "progress_bar" => {
            let cur = match args.first() { Some(Value::Int(n)) => *n, _ => 0 };
            let total = match args.get(1) { Some(Value::Int(n)) => *n, _ => 100 };
            let width = match args.get(2) { Some(Value::Int(n)) => *n as usize, _ => 40 };
            let label = args.get(3).map(|v| v.to_string()).unwrap_or_default();
            let filled = ((cur as f64 / total as f64) * width as f64) as usize;
            let pct = cur * 100 / total.max(1);
            let bar: String = (0..width).map(|i| if i < filled { '█' } else { '░' }).collect();
            print!("\r\x1b[36m{}\x1b[0m [{}] \x1b[32m{}%\x1b[0m", label, bar, pct);
            if cur >= total { println!(); }
            io::stdout().flush().ok();
            Ok(Value::Null)
        }
        "table" => {
            let headers = match args.first() { Some(Value::Array(a)) => a.iter().map(|v| v.to_string()).collect::<Vec<_>>(), _ => return Err("tui.table: ([headers], [[rows]])".into()) };
            let rows = match args.get(1) { Some(Value::Array(a)) => a.clone(), _ => return Err("tui.table: rows array".into()) };
            let widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
            let row_data: Vec<Vec<String>> = rows.iter().map(|r| match r {
                Value::Array(cells) => cells.iter().map(|v| v.to_string()).collect(),
                Value::Struct(_, fields) => headers.iter().map(|h| fields.get(h).map(|v| v.to_string()).unwrap_or_default()).collect(),
                v => vec![v.to_string()],
            }).collect();
            let col_widths: Vec<usize> = (0..headers.len()).map(|i| {
                let max_row = row_data.iter().map(|r| r.get(i).map(|v| v.len()).unwrap_or(0)).max().unwrap_or(0);
                widths.get(i).copied().unwrap_or(0).max(max_row) + 2
            }).collect();
            let sep: String = col_widths.iter().map(|w| "─".repeat(*w)).collect::<Vec<_>>().join("┼");
            let header_row: String = headers.iter().enumerate().map(|(i, h)| format!(" {:width$}", h, width = col_widths[i]-1)).collect::<Vec<_>>().join("│");
            println!("┌{}┐", col_widths.iter().map(|w| "─".repeat(*w)).collect::<Vec<_>>().join("┬"));
            println!("│{}│", header_row);
            println!("├{}┤", sep);
            for row in &row_data {
                let cells: String = (0..headers.len()).map(|i| format!(" {:width$}", row.get(i).cloned().unwrap_or_default(), width = col_widths[i]-1)).collect::<Vec<_>>().join("│");
                println!("│{}│", cells);
            }
            println!("└{}┘", col_widths.iter().map(|w| "─".repeat(*w)).collect::<Vec<_>>().join("┴"));
            Ok(Value::Null)
        }
        "menu" => {
            let items = match args.first() { Some(Value::Array(a)) => a.iter().map(|v| v.to_string()).collect::<Vec<_>>(), _ => return Err("tui.menu: (items, title)".into()) };
            let title = args.get(1).map(|v| v.to_string()).unwrap_or_else(|| "Menu".into());
            println!("\x1b[1m{}\x1b[0m", title);
            println!("{}", "─".repeat(title.len().max(20)));
            for (i, item) in items.iter().enumerate() {
                println!("  \x1b[36m{}\x1b[0m) {}", i + 1, item);
            }
            print!("\x1b[90mChoice: \x1b[0m");
            io::stdout().flush().ok();
            let mut input = String::new();
            io::stdin().read_line(&mut input).ok();
            let choice: i64 = input.trim().parse().unwrap_or(0) - 1;
            if choice >= 0 && (choice as usize) < items.len() {
                Ok(Value::Str(items[choice as usize].clone()))
            } else {
                Ok(Value::Null)
            }
        }
        "confirm" => {
            let msg = args.first().map(|v| v.to_string()).unwrap_or_else(|| "Continue?".into());
            print!("{} \x1b[90m[y/n]:\x1b[0m ", msg);
            io::stdout().flush().ok();
            let mut input = String::new();
            io::stdin().read_line(&mut input).ok();
            let ans = input.trim().to_lowercase();
            Ok(Value::Bool(ans == "y" || ans == "yes"))
        }
        "spinner_frame" => {
            let idx = match args.first() { Some(Value::Int(n)) => *n as usize, _ => 0 };
            let frames = ["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"];
            let msg = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            print!("\r\x1b[36m{}\x1b[0m {}", frames[idx % frames.len()], msg);
            io::stdout().flush().ok();
            Ok(Value::Null)
        }
        "alert" => {
            let msg = args.first().map(|v| v.to_string()).unwrap_or_default();
            let kind = args.get(1).map(|v| v.to_string()).unwrap_or_else(|| "info".into());
            let (icon, color) = match kind.as_str() {
                "error" => ("✗", "\x1b[31m"),
                "warn" | "warning" => ("⚠", "\x1b[33m"),
                "success" | "ok" => ("✓", "\x1b[32m"),
                _ => ("ℹ", "\x1b[34m"),
            };
            println!("{}{} {}\x1b[0m", color, icon, msg);
            Ok(Value::Null)
        }
        "input" => {
            let prompt = args.first().map(|v| v.to_string()).unwrap_or_default();
            print!("\x1b[36m{}\x1b[0m", prompt);
            io::stdout().flush().ok();
            let mut s = String::new();
            io::stdin().read_line(&mut s).ok();
            Ok(Value::Str(s.trim().to_string()))
        }
        "rule" => {
            let width = match args.first() { Some(Value::Int(n)) => *n as usize, _ => 40 };
            let label = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            if label.is_empty() {
                println!("{}", "─".repeat(width));
            } else {
                let pad = width.saturating_sub(label.len() + 2);
                println!("{} \x1b[1m{}\x1b[0m {}", "─".repeat(pad/2), label, "─".repeat(pad - pad/2));
            }
            Ok(Value::Null)
        }
        _ => Err(format!("tui.{}: unknown", func))
    }
}
