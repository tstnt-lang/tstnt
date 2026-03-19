use crate::value::Value;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Mutex;

static ROUTES: std::sync::OnceLock<Mutex<HashMap<String, String>>> = std::sync::OnceLock::new();
static RESPONSES: std::sync::OnceLock<Mutex<HashMap<String, String>>> = std::sync::OnceLock::new();
static PORT_STORE: std::sync::OnceLock<Mutex<u16>> = std::sync::OnceLock::new();

fn routes() -> &'static Mutex<HashMap<String, String>> { ROUTES.get_or_init(|| Mutex::new(HashMap::new())) }
fn responses() -> &'static Mutex<HashMap<String, String>> { RESPONSES.get_or_init(|| Mutex::new(HashMap::new())) }

fn parse_request(stream: &mut TcpStream) -> (String, String, String) {
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf).unwrap_or(0);
    let req = String::from_utf8_lossy(&buf[..n]).to_string();
    let mut lines = req.lines();
    let first = lines.next().unwrap_or("");
    let parts: Vec<&str> = first.split_whitespace().collect();
    let method = parts.first().copied().unwrap_or("GET").to_string();
    let path = parts.get(1).copied().unwrap_or("/").to_string();
    let body = req.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
    (method, path, body)
}

fn send_response(stream: &mut TcpStream, status: u16, content_type: &str, body: &str) {
    let status_text = match status { 200 => "OK", 404 => "Not Found", 500 => "Internal Server Error", _ => "OK" };
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{}",
        status, status_text, content_type, body.len(), body
    );
    stream.write_all(response.as_bytes()).ok();
}

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "route" => {
            let path = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("server.route: (path, body)".into()) };
            let body = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => String::new() };
            responses().lock().unwrap().insert(path.clone(), body);
            routes().lock().unwrap().insert(path, "GET".into());
            Ok(Value::Null)
        }
        "route_post" => {
            let path = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("server.route_post: (path, body)".into()) };
            let body = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => String::new() };
            responses().lock().unwrap().insert(format!("POST:{}", path), body);
            routes().lock().unwrap().insert(path, "POST".into());
            Ok(Value::Null)
        }
        "listen" => {
            let port = match args.first() { Some(Value::Int(n)) => *n as u16, _ => 8080 };
            let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).map_err(|e| e.to_string())?;
            println!("\x1b[32mserver\x1b[0m listening on http://localhost:{}", port);
            for stream in listener.incoming() {
                if let Ok(mut stream) = stream {
                    let (method, path, _body) = parse_request(&mut stream);
                    let resp_map = responses().lock().unwrap();
                    let key = if method == "POST" { format!("POST:{}", path) } else { path.clone() };
                    if let Some(body) = resp_map.get(&key) {
                        let ct = if body.trim_start().starts_with('{') || body.trim_start().starts_with('[') { "application/json" } else { "text/html; charset=utf-8" };
                        send_response(&mut stream, 200, ct, body);
                    } else {
                        send_response(&mut stream, 404, "text/plain", &format!("404 Not Found: {}", path));
                    }
                }
            }
            Ok(Value::Null)
        }
        "respond" => {
            let status = match args.first() { Some(Value::Int(n)) => *n as u16, _ => 200 };
            let body = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => String::new() };
            Ok(Value::Str(format!("{}:{}", status, body)))
        }
        "json_response" => {
            let val = args.first().cloned().unwrap_or(Value::Null);
            let json = crate::stdlib::json::to_json_pub(&val);
            Ok(Value::Str(json))
        }
        _ => Err(format!("server.{}: unknown", func))
    }
}
