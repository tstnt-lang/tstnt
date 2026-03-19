use crate::value::Value;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

fn u16_le(n: u16) -> [u8; 2] { n.to_le_bytes() }
fn u32_le(n: u32) -> [u8; 4] { n.to_le_bytes() }

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            if crc & 1 != 0 { crc = (crc >> 1) ^ 0xEDB88320; } else { crc >>= 1; }
        }
    }
    !crc
}

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "create" => {
            let out_path = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("zip.create: (output, [files])".into()) };
            let files = match args.get(1) { Some(Value::Array(a)) => a.iter().map(|v| v.to_string()).collect::<Vec<_>>(), _ => return Err("zip.create: files array".into()) };
            let mut zip_data: Vec<u8> = Vec::new();
            let mut central_dir: Vec<u8> = Vec::new();
            let mut num_entries = 0u16;
            for file_path in &files {
                let data = fs::read(file_path).map_err(|e| format!("{}: {}", file_path, e))?;
                let name = Path::new(file_path).file_name().unwrap_or_default().to_string_lossy().to_string();
                let crc = crc32(&data);
                let offset = zip_data.len() as u32;
                let name_bytes = name.as_bytes();
                zip_data.extend_from_slice(&[0x50, 0x4B, 0x03, 0x04]);
                zip_data.extend_from_slice(&u16_le(20));
                zip_data.extend_from_slice(&u16_le(0));
                zip_data.extend_from_slice(&u16_le(0));
                zip_data.extend_from_slice(&u16_le(0));
                zip_data.extend_from_slice(&u16_le(0));
                zip_data.extend_from_slice(&u32_le(crc));
                zip_data.extend_from_slice(&u32_le(data.len() as u32));
                zip_data.extend_from_slice(&u32_le(data.len() as u32));
                zip_data.extend_from_slice(&u16_le(name_bytes.len() as u16));
                zip_data.extend_from_slice(&u16_le(0));
                zip_data.extend_from_slice(name_bytes);
                zip_data.extend_from_slice(&data);
                central_dir.extend_from_slice(&[0x50, 0x4B, 0x01, 0x02]);
                central_dir.extend_from_slice(&u16_le(20));
                central_dir.extend_from_slice(&u16_le(20));
                central_dir.extend_from_slice(&u16_le(0));
                central_dir.extend_from_slice(&u16_le(0));
                central_dir.extend_from_slice(&u16_le(0));
                central_dir.extend_from_slice(&u16_le(0));
                central_dir.extend_from_slice(&u32_le(crc));
                central_dir.extend_from_slice(&u32_le(data.len() as u32));
                central_dir.extend_from_slice(&u32_le(data.len() as u32));
                central_dir.extend_from_slice(&u16_le(name_bytes.len() as u16));
                central_dir.extend_from_slice(&u16_le(0));
                central_dir.extend_from_slice(&u16_le(0));
                central_dir.extend_from_slice(&u16_le(0));
                central_dir.extend_from_slice(&u16_le(0));
                central_dir.extend_from_slice(&u32_le(0));
                central_dir.extend_from_slice(&u32_le(offset));
                central_dir.extend_from_slice(name_bytes);
                num_entries += 1;
            }
            let cd_offset = zip_data.len() as u32;
            let cd_size = central_dir.len() as u32;
            zip_data.extend_from_slice(&central_dir);
            zip_data.extend_from_slice(&[0x50, 0x4B, 0x05, 0x06]);
            zip_data.extend_from_slice(&u16_le(0));
            zip_data.extend_from_slice(&u16_le(0));
            zip_data.extend_from_slice(&u16_le(num_entries));
            zip_data.extend_from_slice(&u16_le(num_entries));
            zip_data.extend_from_slice(&u32_le(cd_size));
            zip_data.extend_from_slice(&u32_le(cd_offset));
            zip_data.extend_from_slice(&u16_le(0));
            fs::write(&out_path, &zip_data).map_err(|e| e.to_string())?;
            Ok(Value::Int(num_entries as i64))
        }
        "list" => {
            let path = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("zip.list: str".into()) };
            let data = fs::read(&path).map_err(|e| e.to_string())?;
            let mut files = Vec::new();
            let mut i = 0;
            while i + 30 < data.len() {
                if data[i..i+4] == [0x50, 0x4B, 0x03, 0x04] {
                    let name_len = u16::from_le_bytes([data[i+26], data[i+27]]) as usize;
                    let extra_len = u16::from_le_bytes([data[i+28], data[i+29]]) as usize;
                    let comp_size = u32::from_le_bytes([data[i+18], data[i+19], data[i+20], data[i+21]]) as usize;
                    if i + 30 + name_len <= data.len() {
                        let name = String::from_utf8_lossy(&data[i+30..i+30+name_len]).to_string();
                        files.push(Value::Str(name));
                    }
                    i += 30 + name_len + extra_len + comp_size;
                } else { i += 1; }
            }
            Ok(Value::Array(files))
        }
        "extract" => {
            let path = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("zip.extract: (path, outdir)".into()) };
            let outdir = match args.get(1) { Some(Value::Str(s)) => s.clone(), _ => ".".into() };
            let data = fs::read(&path).map_err(|e| e.to_string())?;
            fs::create_dir_all(&outdir).ok();
            let mut count = 0;
            let mut i = 0;
            while i + 30 < data.len() {
                if data[i..i+4] == [0x50, 0x4B, 0x03, 0x04] {
                    let name_len = u16::from_le_bytes([data[i+26], data[i+27]]) as usize;
                    let extra_len = u16::from_le_bytes([data[i+28], data[i+29]]) as usize;
                    let comp_size = u32::from_le_bytes([data[i+18], data[i+19], data[i+20], data[i+21]]) as usize;
                    if i + 30 + name_len + extra_len + comp_size <= data.len() {
                        let name = String::from_utf8_lossy(&data[i+30..i+30+name_len]).to_string();
                        let file_data = &data[i+30+name_len+extra_len..i+30+name_len+extra_len+comp_size];
                        let out_path = format!("{}/{}", outdir, name);
                        fs::write(&out_path, file_data).ok();
                        count += 1;
                    }
                    i += 30 + name_len + extra_len + comp_size;
                } else { i += 1; }
            }
            Ok(Value::Int(count))
        }
        "size" => {
            let path = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("zip.size: str".into()) };
            let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
            Ok(Value::Int(meta.len() as i64))
        }
        _ => Err(format!("zip.{}: unknown", func))
    }
}
