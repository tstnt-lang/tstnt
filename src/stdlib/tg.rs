use crate::value::Value;
use std::collections::HashMap;
use std::process::Command;

fn curl_get(url: &str) -> Result<String, String> {
    let out = Command::new("curl").args(["-s", "--max-time", "35", url]).output().map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn curl_post(url: &str, body: &str) -> Result<String, String> {
    let out = Command::new("curl").args(["-s", "--max-time", "10", "-X", "POST", "-H", "Content-Type: application/json", "-d", body, url]).output().map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn jval(v: &Value) -> String {
    match v {
        Value::Str(s) => {
            let escaped = s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
            let mut r = String::from("\""); r.push_str(&escaped); r.push('"'); r
        }
        Value::Int(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::from("null"),
        _ => { let mut r = String::from("\""); r.push_str(&v.to_string()); r.push('"'); r }
    }
}

fn decode_unicode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('u') => {
                    let hex: String = (0..4).filter_map(|_| chars.next()).collect();
                    if let Ok(n) = u32::from_str_radix(&hex, 16) {
                        if let Some(ch) = char::from_u32(n) { result.push(ch); continue; }
                    }
                    result.push_str("\\u"); result.push_str(&hex);
                }
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some(other) => { result.push('\\'); result.push(other); }
                None => {}
            }
        } else { result.push(c); }
    }
    result
}

fn jget<'a>(s: &'a str, key: &str) -> Option<&'a str> {
    let mut needle = String::from("\""); needle.push_str(key); needle.push_str("\":");
    let i = s.find(&needle)? + needle.len();
    let rest = s[i..].trim_start();
    if rest.starts_with('"') {
        let r = &rest[1..]; Some(&r[..r.find('"')?])
    } else {
        let end = rest.find(|c: char| ",}]".contains(c)).unwrap_or(rest.len());
        Some(rest[..end].trim())
    }
}

fn jget_owned(s: &str, key: &str) -> Option<String> {
    let mut needle = String::from("\""); needle.push_str(key); needle.push_str("\":");
    let i = s.find(&needle)? + needle.len();
    let rest = s[i..].trim_start();
    if rest.starts_with('"') {
        let r = &rest[1..];
        let mut end = 0;
        let rb: Vec<char> = r.chars().collect();
        while end < rb.len() {
            if rb[end] == '"' && (end == 0 || rb[end-1] != '\\') { break; }
            end += 1;
        }
        Some(decode_unicode(&r[..end]))
    } else {
        let end = rest.find(|c: char| ",}]".contains(c)).unwrap_or(rest.len());
        Some(rest[..end].trim().to_string())
    }
}

fn extract_obj(s: &str, start: usize) -> Option<&str> {
    let bytes = s.as_bytes();
    if bytes.get(start) != Some(&b'{') { return None; }
    let mut depth = 0usize;
    let mut in_str = false;
    let mut escape = false;
    for (i, &b) in bytes[start..].iter().enumerate() {
        if escape { escape = false; continue; }
        if b == b'\\' && in_str { escape = true; continue; }
        if b == b'"' { in_str = !in_str; continue; }
        if in_str { continue; }
        if b == b'{' { depth += 1; }
        if b == b'}' { depth -= 1; if depth == 0 { return Some(&s[start..start+i+1]); } }
    }
    None
}

fn parse_updates(json: &str) -> Vec<HashMap<String, Value>> {
    let mut results = Vec::new();
    let result_tag = "\"result\":[";
    let start = match json.find(result_tag) { Some(i) => i + result_tag.len(), None => return results };
    let arr = &json[start..];
    let mut pos = 0;
    loop {
        let rel = match arr[pos..].find('{') { Some(i) => pos + i, None => break };
        let obj_str = match extract_obj(arr, rel) { Some(s) => s, None => break };
        pos = rel + obj_str.len();
        let mut map = HashMap::new();
        if let Some(uid) = jget(obj_str, "update_id") { if let Ok(n) = uid.parse::<i64>() { map.insert("update_id".into(), Value::Int(n)); } }
        // callback_query
        if obj_str.contains("\"callback_query\":{") {
            if let Some(cq_pos) = obj_str.find("\"callback_query\":{") {
                if let Some(cq_str) = extract_obj(obj_str, cq_pos + "\"callback_query\":".len()) {
                    map.insert("callback".into(), Value::Bool(true));
                    if let Some(v) = jget_owned(cq_str, "data") { map.insert("text".into(), Value::Str(v.clone())); map.insert("callback_data".into(), Value::Str(v)); }
                    if let Some(v) = jget(cq_str, "id") { map.insert("callback_id".into(), Value::Str(v.to_string())); }
                    if let Some(msg_pos) = cq_str.find("\"message\":{") {
                        if let Some(msg_str) = extract_obj(cq_str, msg_pos + "\"message\":".len()) {
                            if let Some(chat_pos) = msg_str.find("\"chat\":{") {
                                if let Some(chat_str) = extract_obj(msg_str, chat_pos + "\"chat\":".len()) {
                                    if let Some(cid) = jget(chat_str, "id") { if let Ok(n) = cid.parse::<i64>() { map.insert("chat_id".into(), Value::Int(n)); } }
                                }
                            }
                        }
                    }
                }
            }
        }
        if obj_str.contains("\"message\":{") {
            if let Some(msg_pos) = obj_str.find("\"message\":{") {
                if let Some(msg_str) = extract_obj(obj_str, msg_pos + "\"message\":".len()) {
                    if let Some(t) = jget_owned(msg_str, "text") { map.insert("text".into(), Value::Str(t)); }
                    else if let Some(t) = jget_owned(msg_str, "caption") { map.insert("text".into(), Value::Str(t)); }
                    if let Some(chat_pos) = msg_str.find("\"chat\":{") {
                        if let Some(chat_str) = extract_obj(msg_str, chat_pos + "\"chat\":".len()) {
                            if let Some(cid) = jget(chat_str, "id") { if let Ok(n) = cid.parse::<i64>() { map.insert("chat_id".into(), Value::Int(n)); } }
                        }
                    }
                    if let Some(from_pos) = msg_str.find("\"from\":{") {
                        if let Some(from_str) = extract_obj(msg_str, from_pos + "\"from\":".len()) {
                            if let Some(v) = jget_owned(from_str, "first_name") { map.insert("from_name".into(), Value::Str(v)); }
                            if let Some(v) = jget_owned(from_str, "username") { map.insert("username".into(), Value::Str(v)); }
                            if let Some(v) = jget(from_str, "id") { if let Ok(n) = v.parse::<i64>() { map.insert("from_id".into(), Value::Int(n)); } }
                        }
                    }
                }
            }
        }
        if !map.contains_key("text") { map.insert("text".into(), Value::Str(String::new())); }
        if map.contains_key("update_id") { results.push(map); }
    }
    results
}

fn kv(key: &str, val: &str) -> String {
    let mut r = String::from("\""); r.push_str(key); r.push_str("\":"); r.push_str(val); r
}

fn obj(pairs: &[(&str, &str)]) -> String {
    let mut r = String::from("{");
    for (i, (k, v)) in pairs.iter().enumerate() { if i > 0 { r.push(','); } r.push_str(&kv(k, v)); }
    r.push('}'); r
}

static TOKEN: std::sync::OnceLock<std::sync::Mutex<String>> = std::sync::OnceLock::new();
fn tok() -> String { TOKEN.get_or_init(|| std::sync::Mutex::new(String::new())).lock().unwrap().clone() }
fn set_tok(t: &str) { *TOKEN.get_or_init(|| std::sync::Mutex::new(String::new())).lock().unwrap() = t.to_string(); }
fn api(m: &str) -> String { let mut r = String::from("https://api.telegram.org/bot"); r.push_str(&tok()); r.push('/'); r.push_str(m); r }

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "token" => { if let Some(Value::Str(s)) = args.first() { set_tok(s); } Ok(Value::Null) }
        "send" | "answer" => {
            let chat_id = args.first().map(|v| v.to_string()).ok_or("tg.send: need chat_id")?;
            let text = args.get(1).map(|v| v.to_string()).ok_or("tg.send: need text")?;
            let body = obj(&[("chat_id", &chat_id), ("text", &jval(&Value::Str(text)))]);
            curl_post(&api("sendMessage"), &body).map(|_| Value::Null)
        }
        "send_md" => {
            let chat_id = args.first().map(|v| v.to_string()).ok_or("tg.send_md")?;
            let text = args.get(1).map(|v| v.to_string()).ok_or("tg.send_md")?;
            let body = obj(&[("chat_id", &chat_id), ("text", &jval(&Value::Str(text))), ("parse_mode", "\"Markdown\"")]);
            curl_post(&api("sendMessage"), &body).map(|_| Value::Null)
        }
        "send_keyboard" => {
            let chat_id = args.first().map(|v| v.to_string()).ok_or("tg.send_keyboard")?;
            let text = args.get(1).map(|v| v.to_string()).ok_or("tg.send_keyboard")?;
            let keyboard = match args.get(2) {
                Some(Value::Array(rows)) => {
                    let json_rows: Vec<String> = rows.iter().map(|row| {
                        let btns: Vec<String> = match row {
                            Value::Array(b) => b.iter().map(|x| obj(&[("text", &jval(x))])).collect(),
                            x => vec![obj(&[("text", &jval(x))])],
                        };
                        let mut r = String::from("["); r.push_str(&btns.join(",")); r.push(']'); r
                    }).collect();
                    let mut r = String::from("{\"keyboard\":["); r.push_str(&json_rows.join(",")); r.push_str("],\"resize_keyboard\":true,\"one_time_keyboard\":false}"); r
                }
                _ => String::from("{\"remove_keyboard\":true}")
            };
            let body = obj(&[("chat_id", &chat_id), ("text", &jval(&Value::Str(text))), ("reply_markup", &keyboard)]);
            curl_post(&api("sendMessage"), &body).map(|_| Value::Null)
        }
        "send_inline" => {
            let chat_id = args.first().map(|v| v.to_string()).ok_or("tg.send_inline")?;
            let text = args.get(1).map(|v| v.to_string()).ok_or("tg.send_inline")?;
            let keyboard = match args.get(2) {
                Some(Value::Array(rows)) => {
                    let json_rows: Vec<String> = rows.iter().map(|row| {
                        let btns: Vec<String> = match row {
                            Value::Array(b) => b.iter().map(|x| match x {
                                Value::Array(pair) if pair.len() >= 2 => obj(&[("text", &jval(&pair[0])), ("callback_data", &jval(&pair[1]))]),
                                x => obj(&[("text", &jval(x)), ("callback_data", &jval(x))]),
                            }).collect(),
                            x => vec![obj(&[("text", &jval(x)), ("callback_data", &jval(x))])],
                        };
                        let mut r = String::from("["); r.push_str(&btns.join(",")); r.push(']'); r
                    }).collect();
                    let mut r = String::from("{\"inline_keyboard\":["); r.push_str(&json_rows.join(",")); r.push(']'); r.push('}'); r
                }
                _ => String::from("{}")
            };
            let body = obj(&[("chat_id", &chat_id), ("text", &jval(&Value::Str(text))), ("reply_markup", &keyboard)]);
            curl_post(&api("sendMessage"), &body).map(|_| Value::Null)
        }
        "answer_callback" => {
            let cid = args.first().map(|v| v.to_string()).ok_or("tg.answer_callback")?;
            let text = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            let body = obj(&[("callback_query_id", &cid), ("text", &jval(&Value::Str(text)))]);
            curl_post(&api("answerCallbackQuery"), &body).map(|_| Value::Null)
        }
        "get_updates" => {
            let offset = match args.first() { Some(Value::Int(n)) => *n, _ => 0 };
            let mut url = api("getUpdates"); url.push_str("?offset="); url.push_str(&offset.to_string()); url.push_str("&timeout=30");
            let resp = curl_get(&url)?;
            Ok(Value::Array(parse_updates(&resp).into_iter().map(|m| Value::Struct("Update".into(), m)).collect()))
        }
        "me" => {
            let resp = curl_get(&api("getMe"))?;
            let mut map = HashMap::new();
            for key in &["username", "first_name", "id"] { if let Some(v) = jget(&resp, key) { map.insert(key.to_string(), Value::Str(v.to_string())); } }
            Ok(Value::Struct("Bot".into(), map))
        }
        "delete_webhook" => curl_get(&api("deleteWebhook")).map(|_| Value::Null),
        "raw_updates" => {
            let offset = match args.first() { Some(Value::Int(n)) => *n, _ => 0 };
            let mut url = api("getUpdates"); url.push_str("?offset="); url.push_str(&offset.to_string()); url.push_str("&timeout=10");
            curl_get(&url).map(Value::Str)
        }
        "ping" => curl_get(&api("getMe")).map(Value::Str),
        _ => Err(format!("tg.{}: unknown", func))
    }
}
