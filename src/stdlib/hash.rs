use crate::value::Value;

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    let s = args.first().map(|v| v.to_string()).unwrap_or_default();
    match func {
        "fnv32" => {
            let mut h: u32 = 2166136261;
            for b in s.bytes() { h ^= b as u32; h = h.wrapping_mul(16777619); }
            Ok(Value::Int(h as i64))
        }
        "fnv64" => {
            let mut h: u64 = 14695981039346656037;
            for b in s.bytes() { h ^= b as u64; h = h.wrapping_mul(1099511628211); }
            Ok(Value::Int(h as i64))
        }
        "djb2" => {
            let mut h: u32 = 5381;
            for b in s.bytes() { h = h.wrapping_shl(5).wrapping_add(h).wrapping_add(b as u32); }
            Ok(Value::Int(h as i64))
        }
        "sdbm" => {
            let mut h: u32 = 0;
            for b in s.bytes() { h = (b as u32).wrapping_add(h.wrapping_shl(6)).wrapping_add(h.wrapping_shl(16)).wrapping_sub(h); }
            Ok(Value::Int(h as i64))
        }
        "crc32" => {
            let mut crc: u32 = 0xffffffff;
            for b in s.bytes() {
                let mut byte = b;
                for _ in 0..8 { let bit = (byte ^ crc as u8) & 1; crc >>= 1; if bit != 0 { crc ^= 0xedb88320; } byte >>= 1; }
            }
            Ok(Value::Int((!crc) as i64))
        }
        _ => Err(format!("hash.{}: unknown", func))
    }
}
