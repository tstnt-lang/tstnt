use crate::value::Value;
use std::io::{Read, Write};
use std::net::TcpStream;

pub fn call(module: &str, func: &str, args: Vec<Value>) -> Result<Value, String> {
    match module {
        "http" => match func {
            "get" => {
                match args.first() {
                    Some(Value::Str(url)) => http_get(url),
                    _ => Err("net.http.get: expected str url".into())
                }
            }
            "post" => {
                match (args.first(), args.get(1)) {
                    (Some(Value::Str(url)), Some(Value::Str(body))) => http_post(url, body),
                    _ => Err("net.http.post: expected (str, str)".into())
                }
            }
            _ => Err(format!("net.http.{}: unknown function", func))
        },
        "tcp" => match func {
            "connect" => {
                match args.first() {
                    Some(Value::Str(addr)) => {
                        TcpStream::connect(addr).map_err(|e| e.to_string())?;
                        Ok(Value::Bool(true))
                    }
                    _ => Err("net.tcp.connect: expected str addr".into())
                }
            }
            _ => Err(format!("net.tcp.{}: unknown function", func))
        },
        _ => Err(format!("net.{}: unknown module", module))
    }
}

fn http_get(url: &str) -> Result<Value, String> {
    let (host, path) = parse_url(url)?;
    let mut stream = TcpStream::connect(format!("{}:80", host)).map_err(|e| e.to_string())?;
    let req = format!("GET {} HTTP/1.0\r\nHost: {}\r\nConnection: close\r\n\r\n", path, host);
    stream.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
    let mut response = String::new();
    stream.read_to_string(&mut response).map_err(|e| e.to_string())?;
    let body = response.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
    Ok(Value::Str(body))
}

fn http_post(url: &str, body: &str) -> Result<Value, String> {
    let (host, path) = parse_url(url)?;
    let mut stream = TcpStream::connect(format!("{}:80", host)).map_err(|e| e.to_string())?;
    let req = format!(
        "POST {} HTTP/1.0\r\nHost: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        path, host, body.len(), body
    );
    stream.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
    let mut response = String::new();
    stream.read_to_string(&mut response).map_err(|e| e.to_string())?;
    let res_body = response.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
    Ok(Value::Str(res_body))
}

fn parse_url(url: &str) -> Result<(String, String), String> {
    let url = url.trim_start_matches("http://").trim_start_matches("https://");
    let (host, path) = if let Some(idx) = url.find('/') {
        (&url[..idx], &url[idx..])
    } else {
        (url, "/")
    };
    Ok((host.to_string(), path.to_string()))
}
