use crate::value::Value;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn call(func: &str, _args: Vec<Value>) -> Result<Value, String> {
    match func {
        "v4" => {
            let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().subsec_nanos();
            let a = t ^ 0xdeadbeef;
            let b = t.wrapping_mul(0x9e3779b9);
            let c = t.wrapping_add(0x6c62272e);
            let d = t.wrapping_mul(0x517cc1b727220a95u64 as u32);
            Ok(Value::Str(format!("{:08x}-{:04x}-4{:03x}-{:04x}-{:08x}{:04x}",
                a, b >> 16, c >> 20, (d >> 16) | 0x8000,
                a ^ b, c >> 16)))
        }
        "nil" => Ok(Value::Str("00000000-0000-0000-0000-000000000000".into())),
        _ => Err(format!("uuid.{}: unknown", func))
    }
}
