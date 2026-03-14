use std::io::{self, BufRead, Write};
use std::collections::HashMap;

fn read_message(stdin: &mut impl BufRead) -> Option<String> {
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        stdin.read_line(&mut line).ok()?;
        let line = line.trim_end_matches('\n').trim_end_matches('\r');
        if line.is_empty() { break; }
        if let Some(val) = line.strip_prefix("Content-Length: ") {
            content_length = val.trim().parse().unwrap_or(0);
        }
    }
    let mut buf = vec![0u8; content_length];
    use std::io::Read;
    io::stdin().read_exact(&mut buf).ok()?;
    String::from_utf8(buf).ok()
}

fn send_message(stdout: &mut impl Write, msg: &str) {
    let header = format!("Content-Length: {}\r\n\r\n", msg.len());
    stdout.write_all(header.as_bytes()).ok();
    stdout.write_all(msg.as_bytes()).ok();
    stdout.flush().ok();
}

fn extract_str<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{}\":", key);
    let start = json.find(&needle)? + needle.len();
    let rest = json[start..].trim_start();
    if rest.starts_with('"') { let inner = &rest[1..]; let end = inner.find('"')?; Some(&inner[..end]) }
    else { let end = rest.find(|c: char| c == ',' || c == '}').unwrap_or(rest.len()); Some(rest[..end].trim()) }
}

fn extract_int(json: &str, key: &str) -> Option<i64> {
    extract_str(json, key).and_then(|s| s.parse().ok())
}

const KEYWORDS: &[&str] = &["do","let","mut","return","if","else","while","loop","match","use","struct","impl","test","null","repeat","in","break","continue","try","catch","throw","async","await","thread","true","false"];
const BUILTINS: &[&str] = &["print","len","str","int","float","bool","type_of","assert","assert_eq","assert_ne","panic","range","map","filter","reduce","apply","zip","flatten","sort","unique","keys","values","max","min","sum","abs","is_null","input"];
const MODULES: &[&str] = &["io","math","strings","arr","json","time","env","fs","process","crypto","rand","fmt","sys","path","buf","regex","net","log","uuid","term","bench","csv","hash","thread","tg"];

#[allow(dead_code)]
fn completions_for(prefix: &str) -> Vec<&'static str> {
    let mut items: Vec<&str> = Vec::new();
    for &kw in KEYWORDS { if kw.starts_with(prefix) { items.push(kw); } }
    for &b in BUILTINS { if b.starts_with(prefix) { items.push(b); } }
    for &m in MODULES { if m.starts_with(prefix) { items.push(m); } }
    items
}

fn module_methods(module: &str) -> Vec<&'static str> {
    match module {
        "tg" => vec!["token","send","send_md","get_updates","me","delete_webhook"],
        "math" => vec!["sqrt","pow","abs","floor","ceil","min","max","log","sin","cos","tan","pi"],
        "io" => vec!["print","input","read_file","write_file"],
        "fs" => vec!["read","write","append","exists","delete","mkdir","ls","copy","rename","size"],
        "rand" => vec!["int","float","bool","choice","shuffle","seed"],
        "crypto" => vec!["sha256","md5","base64_encode","base64_decode","hex_encode","xor"],
        "time" => vec!["now","sleep"],
        "env" => vec!["get","set","args"],
        "json" => vec!["parse","stringify"],
        "strings" => vec!["len","split","join","trim","upper","lower","contains","replace","starts_with","ends_with"],
        "arr" => vec!["len","push","pop","first","last","reverse","contains","slice"],
        "thread" => vec!["sleep","cpu_count","mutex_new","mutex_get","mutex_set"],
        "log" => vec!["info","warn","error","debug"],
        "uuid" => vec!["v4","nil"],
        "term" => vec!["red","green","yellow","blue","bold","dim","reset","clear"],
        "bench" => vec!["now_ms","now_us","elapsed"],
        "csv" => vec!["parse","stringify","headers"],
        "hash" => vec!["fnv32","fnv64","djb2","sdbm","crc32"],
        "sys" => vec!["os","arch","family","cwd","home","hostname","cpu_count"],
        "path" => vec!["join","dir","file","ext","stem","abs"],
        "fmt" => vec!["format","pad_left","pad_right","repeat","int_hex","int_bin","float_fixed"],
        _ => vec![],
    }
}

fn make_completion_item(label: &str, kind: u8, detail: &str) -> String {
    format!(r#"{{"label":"{}","kind":{},"detail":"{}"}}"#, label, kind, detail)
}

fn hover_docs(word: &str) -> Option<&'static str> {
    match word {
        "do" => Some("define a function: `do name(params) -> RetType { }`"),
        "let" => Some("declare variable: `let x: int = 10`"),
        "match" => Some("pattern matching: `match x { 0 -> ... _ -> ... }`"),
        "loop" => Some("iterate: `loop i in 0..10 { }` or `loop x in arr { }`"),
        "repeat" => Some("repeat N times: `repeat 3 { }`"),
        "thread" => Some("thread module: `thread.spawn`, `thread.mutex_new`, `thread.sleep`"),
        "tg" => Some("Telegram bot API: `tg.token(tok)`, `tg.send(chat_id, text)`, `tg.get_updates(offset)`"),
        "map" => Some("map(array, |x| expr) -> array"),
        "filter" => Some("filter(array, |x| condition) -> array"),
        "reduce" => Some("reduce(array, |acc x| expr, init) -> value"),
        "assert_eq" => Some("assert_eq(a, b) — fails test if a != b"),
        "test" => Some("define a test block: `test name { assert_eq(...) }`"),
        _ => None,
    }
}

fn main() {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut initialized = false;
    let mut _doc_text: HashMap<String, String> = HashMap::new();

    loop {
        let msg = match read_message(&mut stdin) { Some(m) => m, None => break };
        let method = extract_str(&msg, "method").unwrap_or("").to_string();
        let id = extract_int(&msg, "id");

        match method.as_str() {
            "initialize" => {
                initialized = true;
                let resp = format!(r#"{{"jsonrpc":"2.0","id":{},"result":{{"capabilities":{{"textDocumentSync":1,"completionProvider":{{"triggerCharacters":[".","("]}},"hoverProvider":true}}}}}}"#, id.unwrap_or(0));
                send_message(&mut stdout, &resp);
            }
            "initialized" => {}
            "textDocument/didOpen" | "textDocument/didChange" => {
                if let Some(uri) = extract_str(&msg, "uri") {
                    if let Some(text) = extract_str(&msg, "text") {
                        _doc_text.insert(uri.to_string(), text.to_string());
                    }
                }
            }
            "textDocument/completion" => {
                if !initialized { continue; }
                // Проверяем trigger character
                let trigger = extract_str(&msg, "triggerCharacter").unwrap_or("");
                let items = if trigger == "." {
                    // Нужно определить модуль — берём из контекста (упрощённо)
                    MODULES.iter().map(|m| make_completion_item(m, 9, "module")).collect::<Vec<_>>().join(",")
                } else {
                    // Получаем prefix из позиции (упрощённо — все builtin + keywords)
                    let all: Vec<String> = KEYWORDS.iter().map(|k| make_completion_item(k, 14, "keyword"))
                        .chain(BUILTINS.iter().map(|b| make_completion_item(b, 3, "function")))
                        .chain(MODULES.iter().map(|m| make_completion_item(m, 9, "module")))
                        .collect();
                    all.join(",")
                };
                let resp = format!(r#"{{"jsonrpc":"2.0","id":{},"result":{{"isIncomplete":false,"items":[{}]}}}}"#, id.unwrap_or(0), items);
                send_message(&mut stdout, &resp);
            }
            "textDocument/completion_dot" => {
                // При вводе "tg." показываем методы модуля
                if let Some(module) = extract_str(&msg, "module") {
                    let methods = module_methods(module);
                    let items: Vec<String> = methods.iter().map(|m| make_completion_item(m, 2, "method")).collect();
                    let resp = format!(r#"{{"jsonrpc":"2.0","id":{},"result":{{"isIncomplete":false,"items":[{}]}}}}"#, id.unwrap_or(0), items.join(","));
                    send_message(&mut stdout, &resp);
                }
            }
            "textDocument/hover" => {
                if !initialized { continue; }
                // Простой hover — ищем слово в документе по позиции
                let resp = if let Some(word) = extract_str(&msg, "word") {
                    if let Some(doc) = hover_docs(word) {
                        format!(r#"{{"jsonrpc":"2.0","id":{},"result":{{"contents":{{"kind":"markdown","value":"{}"}}}}}}"#, id.unwrap_or(0), doc)
                    } else {
                        format!(r#"{{"jsonrpc":"2.0","id":{},"result":null}}"#, id.unwrap_or(0))
                    }
                } else {
                    format!(r#"{{"jsonrpc":"2.0","id":{},"result":null}}"#, id.unwrap_or(0))
                };
                send_message(&mut stdout, &resp);
            }
            "shutdown" => {
                let resp = format!(r#"{{"jsonrpc":"2.0","id":{},"result":null}}"#, id.unwrap_or(0));
                send_message(&mut stdout, &resp);
            }
            "exit" => break,
            _ => {
                if let Some(i) = id {
                    let resp = format!(r#"{{"jsonrpc":"2.0","id":{},"error":{{"code":-32601,"message":"method not found"}}}}"#, i);
                    send_message(&mut stdout, &resp);
                }
            }
        }
    }
}
