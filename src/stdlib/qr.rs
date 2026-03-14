use crate::value::Value;
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "ascii" => {
            let text = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("qr.ascii: str".into()) };
            let hash = text.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
            let size = 9usize;
            let mut grid = vec![vec![false; size]; size];
            // Finder patterns
            for i in 0..3 { for j in 0..3 { grid[i][j] = true; grid[i][size-1-j] = true; grid[size-1-i][j] = true; } }
            // Data bits from hash
            for i in 3..size-3 {
                for j in 3..size-3 {
                    let bit_idx = (i * size + j) as u64;
                    grid[i][j] = (hash >> (bit_idx % 64)) & 1 == 1;
                }
            }
            let mut result = String::new();
            for row in &grid {
                for &cell in row { result.push_str(if cell { "██" } else { "  " }); }
                result.push('\n');
            }
            Ok(Value::Str(result))
        }
        "url" => {
            let text = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("qr.url: str".into()) };
            Ok(Value::Str(format!("https://api.qrserver.com/v1/create-qr-code/?size=200x200&data={}", text.replace(' ', "+"))))
        }
        _ => Err(format!("qr.{}: unknown", func))
    }
}
