use crate::value::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn db_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".tstnt").join("sql")
}

#[derive(Clone, Debug)]
struct Table {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
}

fn load_table(db: &str, table: &str) -> Table {
    let path = db_dir().join(db).join(format!("{}.csv", table));
    if !path.exists() { return Table { columns: vec![], rows: vec![] }; }
    let content = fs::read_to_string(&path).unwrap_or_default();
    let mut lines = content.lines();
    let columns = lines.next().unwrap_or("").split('\t').map(|s| s.to_string()).collect();
    let rows = lines.map(|l| l.split('\t').map(|s| s.to_string()).collect()).collect();
    Table { columns, rows }
}

fn save_table(db: &str, table: &str, t: &Table) {
    let dir = db_dir().join(db);
    fs::create_dir_all(&dir).ok();
    let path = dir.join(format!("{}.csv", table));
    let mut content = t.columns.join("\t") + "\n";
    for row in &t.rows { content.push_str(&row.join("\t")); content.push('\n'); }
    fs::write(path, content).ok();
}

fn row_to_struct(table: &Table, row: &[String]) -> Value {
    let mut fields = HashMap::new();
    for (i, col) in table.columns.iter().enumerate() {
        let v = row.get(i).cloned().unwrap_or_default();
        let val = if let Ok(n) = v.parse::<i64>() { Value::Int(n) }
                  else if let Ok(f) = v.parse::<f64>() { Value::Float(f) }
                  else if v == "true" { Value::Bool(true) }
                  else if v == "false" { Value::Bool(false) }
                  else if v == "null" { Value::Null }
                  else { Value::Str(v) };
        fields.insert(col.clone(), val);
    }
    Value::Struct("Row".into(), fields)
}

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "open" => {
            let name = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("sql.open: str".into()) };
            fs::create_dir_all(db_dir().join(&name)).ok();
            Ok(Value::Str(name))
        }
        "create_table" => {
            let db = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("sql.create_table: (db, table, cols)".into()) };
            let table = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("sql.create_table: table name".into()) };
            let cols = match args.get(2) { Some(Value::Array(a)) => a.iter().map(|v| v.to_string()).collect::<Vec<_>>(), _ => return Err("sql.create_table: columns array".into()) };
            let t = Table { columns: cols, rows: vec![] };
            save_table(&db, &table, &t);
            Ok(Value::Null)
        }
        "insert" => {
            let db = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("sql.insert: (db, table, struct)".into()) };
            let table = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("sql.insert: table".into()) };
            let mut t = load_table(&db, &table);
            match args.get(2) {
                Some(Value::Struct(_, fields)) => {
                    let row: Vec<String> = t.columns.iter().map(|col| fields.get(col).map(|v| v.to_string()).unwrap_or_default()).collect();
                    t.rows.push(row);
                }
                Some(Value::Array(vals)) => {
                    let row: Vec<String> = vals.iter().map(|v| v.to_string()).collect();
                    t.rows.push(row);
                }
                _ => return Err("sql.insert: expected struct or array".into())
            }
            save_table(&db, &table, &t);
            Ok(Value::Int(t.rows.len() as i64))
        }
        "select" => {
            let db = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("sql.select: (db, table)".into()) };
            let table = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("sql.select: table".into()) };
            let t = load_table(&db, &table);
            Ok(Value::Array(t.rows.iter().map(|row| row_to_struct(&t, row)).collect()))
        }
        "where" => {
            let db = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("sql.where: (db, table, col, val)".into()) };
            let table = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("sql.where: table".into()) };
            let col = match args.get(2) { Some(Value::Str(s)) => s.clone(), _ => return Err("sql.where: column".into()) };
            let val = args.get(3).map(|v| v.to_string()).unwrap_or_default();
            let t = load_table(&db, &table);
            let col_idx = t.columns.iter().position(|c| c == &col).ok_or_else(|| format!("sql.where: column '{}' not found", col))?;
            let rows: Vec<Value> = t.rows.iter().filter(|row| row.get(col_idx).map(|v| v == &val).unwrap_or(false)).map(|row| row_to_struct(&t, row)).collect();
            Ok(Value::Array(rows))
        }
        "delete_where" => {
            let db = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("sql.delete_where: (db, table, col, val)".into()) };
            let table = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("sql.delete_where: table".into()) };
            let col = match args.get(2) { Some(Value::Str(s)) => s.clone(), _ => return Err("sql.delete_where: column".into()) };
            let val = args.get(3).map(|v| v.to_string()).unwrap_or_default();
            let mut t = load_table(&db, &table);
            let col_idx = t.columns.iter().position(|c| c == &col).ok_or_else(|| format!("sql.delete_where: column '{}' not found", col))?;
            let before = t.rows.len();
            t.rows.retain(|row| row.get(col_idx).map(|v| v != &val).unwrap_or(true));
            save_table(&db, &table, &t);
            Ok(Value::Int((before - t.rows.len()) as i64))
        }
        "update" => {
            let db = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("sql.update: (db, table, where_col, where_val, set_col, set_val)".into()) };
            let table = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("sql.update: table".into()) };
            let where_col = match args.get(2) { Some(Value::Str(s)) => s.clone(), _ => return Err("sql.update: where_col".into()) };
            let where_val = args.get(3).map(|v| v.to_string()).unwrap_or_default();
            let set_col = match args.get(4) { Some(Value::Str(s)) => s.clone(), _ => return Err("sql.update: set_col".into()) };
            let set_val = args.get(5).map(|v| v.to_string()).unwrap_or_default();
            let mut t = load_table(&db, &table);
            let wi = t.columns.iter().position(|c| c == &where_col).ok_or_else(|| format!("column '{}' not found", where_col))?;
            let si = t.columns.iter().position(|c| c == &set_col).ok_or_else(|| format!("column '{}' not found", set_col))?;
            let mut count = 0i64;
            for row in &mut t.rows {
                if row.get(wi).map(|v| v == &where_val).unwrap_or(false) {
                    if si < row.len() { row[si] = set_val.clone(); count += 1; }
                }
            }
            save_table(&db, &table, &t);
            Ok(Value::Int(count))
        }
        "count" => {
            let db = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("sql.count: (db, table)".into()) };
            let table = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("sql.count: table".into()) };
            Ok(Value::Int(load_table(&db, &table).rows.len() as i64))
        }
        "drop" => {
            let db = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("sql.drop: (db, table)".into()) };
            let table = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("sql.drop: table".into()) };
            let path = db_dir().join(&db).join(format!("{}.csv", table));
            fs::remove_file(path).ok();
            Ok(Value::Null)
        }
        "tables" => {
            let db = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("sql.tables: db".into()) };
            let dir = db_dir().join(&db);
            let tables: Vec<Value> = fs::read_dir(&dir).map(|rd| rd.filter_map(|e| e.ok()).filter_map(|e| {
                let n = e.file_name().to_string_lossy().to_string();
                if n.ends_with(".csv") { Some(Value::Str(n.trim_end_matches(".csv").to_string())) } else { None }
            }).collect()).unwrap_or_default();
            Ok(Value::Array(tables))
        }
        _ => Err(format!("sql.{}: unknown", func))
    }
}
