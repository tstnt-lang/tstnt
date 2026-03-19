use crate::value::Value;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn ws_handshake(stream: &mut TcpStream) -> bool {
    let mut buf = [0u8; 2048];
    let n = stream.read(&mut buf).unwrap_or(0);
    let req = String::from_utf8_lossy(&buf[..n]);
    let key = req.lines()
        .find(|l| l.to_lowercase().starts_with("sec-websocket-key:"))
        .and_then(|l| l.splitn(2, ':').nth(1))
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if key.is_empty() { return false; }
    let accept = ws_accept_key(&key);
    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n",
        accept
    );
    stream.write_all(response.as_bytes()).is_ok()
}

fn ws_accept_key(key: &str) -> String {
    let combined = format!("{}258EAFA5-E914-47DA-95CA-C5AB0DC85B11", key);
    let hash = sha1_simple(combined.as_bytes());
    base64_encode(&hash)
}

fn sha1_simple(data: &[u8]) -> [u8; 20] {
    let mut h: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
    let mut msg = data.to_vec();
    let orig_len = (data.len() as u64) * 8;
    msg.push(0x80);
    while msg.len() % 64 != 56 { msg.push(0); }
    msg.extend_from_slice(&orig_len.to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 80];
        for i in 0..16 { w[i] = u32::from_be_bytes(chunk[i*4..i*4+4].try_into().unwrap()); }
        for i in 16..80 { w[i] = (w[i-3]^w[i-8]^w[i-14]^w[i-16]).rotate_left(1); }
        let (mut a,mut b,mut c,mut d,mut e) = (h[0],h[1],h[2],h[3],h[4]);
        for i in 0..80 {
            let (f, k) = match i {
                0..=19  => ((b&c)|(!b&d), 0x5A827999u32),
                20..=39 => (b^c^d,        0x6ED9EBA1u32),
                40..=59 => ((b&c)|(b&d)|(c&d), 0x8F1BBCDCu32),
                _       => (b^c^d,        0xCA62C1D6u32),
            };
            let temp = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(w[i]);
            e=d; d=c; c=b.rotate_left(30); b=a; a=temp;
        }
        h[0]=h[0].wrapping_add(a); h[1]=h[1].wrapping_add(b);
        h[2]=h[2].wrapping_add(c); h[3]=h[3].wrapping_add(d); h[4]=h[4].wrapping_add(e);
    }
    let mut out = [0u8; 20];
    for (i, &v) in h.iter().enumerate() { out[i*4..i*4+4].copy_from_slice(&v.to_be_bytes()); }
    out
}

fn base64_encode(data: &[u8]) -> String {
    const B64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let b = match chunk.len() {
            3 => ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | chunk[2] as u32,
            2 => ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8),
            _ => (chunk[0] as u32) << 16,
        };
        out.push(B64[((b >> 18) & 63) as usize] as char);
        out.push(B64[((b >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 { B64[((b >> 6) & 63) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { B64[(b & 63) as usize] as char } else { '=' });
    }
    out
}

fn ws_read_frame(stream: &mut TcpStream) -> Option<String> {
    let mut header = [0u8; 2];
    stream.read_exact(&mut header).ok()?;
    let masked = (header[1] & 0x80) != 0;
    let mut len = (header[1] & 0x7F) as usize;
    if len == 126 {
        let mut b = [0u8; 2]; stream.read_exact(&mut b).ok()?;
        len = u16::from_be_bytes(b) as usize;
    }
    let mask = if masked {
        let mut m = [0u8; 4]; stream.read_exact(&mut m).ok()?; Some(m)
    } else { None };
    let mut data = vec![0u8; len];
    stream.read_exact(&mut data).ok()?;
    if let Some(m) = mask {
        for (i, b) in data.iter_mut().enumerate() { *b ^= m[i % 4]; }
    }
    String::from_utf8(data).ok()
}

fn ws_send_frame(stream: &mut TcpStream, msg: &str) -> bool {
    let data = msg.as_bytes();
    let mut frame = vec![0x81u8];
    if data.len() < 126 {
        frame.push(data.len() as u8);
    } else {
        frame.push(126);
        frame.extend_from_slice(&(data.len() as u16).to_be_bytes());
    }
    frame.extend_from_slice(data);
    stream.write_all(&frame).is_ok()
}

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "listen" => {
            let port = match args.first() { Some(Value::Int(n)) => *n as u16, _ => 8080 };
            let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).map_err(|e| e.to_string())?;
            println!("\x1b[32mws\x1b[0m listening on ws://localhost:{}", port);
            for stream in listener.incoming() {
                if let Ok(mut stream) = stream {
                    if ws_handshake(&mut stream) {
                        println!("\x1b[36mws\x1b[0m client connected");
                        loop {
                            match ws_read_frame(&mut stream) {
                                Some(msg) => {
                                    println!("\x1b[90mws recv:\x1b[0m {}", msg);
                                    ws_send_frame(&mut stream, &format!("echo: {}", msg));
                                }
                                None => { println!("\x1b[90mws client disconnected\x1b[0m"); break; }
                            }
                        }
                    }
                }
            }
            Ok(Value::Null)
        }
        "send" => {
            let addr = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("ws.send: (addr, msg)".into()) };
            let msg = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => return Err("ws.send: msg".into()) };
            let mut stream = TcpStream::connect(&addr).map_err(|e| e.to_string())?;
            ws_send_frame(&mut stream, &msg);
            Ok(Value::Null)
        }
        "frame" => {
            let msg = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("ws.frame: str".into()) };
            let data = msg.as_bytes();
            let mut frame = vec![0x81u8];
            frame.push(data.len().min(125) as u8);
            frame.extend_from_slice(&data[..data.len().min(125)]);
            Ok(Value::Str(String::from_utf8_lossy(&frame).to_string()))
        }
        _ => Err(format!("ws.{}: unknown", func))
    }
}
