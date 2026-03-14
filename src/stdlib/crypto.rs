use crate::value::Value;

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "sha256" => match args.first() {
            Some(Value::Str(s)) => Ok(Value::Str(sha256(s.as_bytes()))),
            _ => Err("crypto.sha256: expected str".into())
        },
        "md5" => match args.first() {
            Some(Value::Str(s)) => Ok(Value::Str(md5(s.as_bytes()))),
            _ => Err("crypto.md5: expected str".into())
        },
        "base64_encode" => match args.first() {
            Some(Value::Str(s)) => Ok(Value::Str(base64_encode(s.as_bytes()))),
            _ => Err("crypto.base64_encode: expected str".into())
        },
        "base64_decode" => match args.first() {
            Some(Value::Str(s)) => Ok(Value::Str(
                String::from_utf8(base64_decode(s)?).map_err(|e| e.to_string())?
            )),
            _ => Err("crypto.base64_decode: expected str".into())
        },
        "xor" => match (args.first(), args.get(1)) {
            (Some(Value::Str(data)), Some(Value::Str(key))) => {
                let result: Vec<u8> = data.bytes().zip(key.bytes().cycle()).map(|(a, b)| a ^ b).collect();
                Ok(Value::Str(hex_encode(&result)))
            }
            _ => Err("crypto.xor: expected (str, str)".into())
        },
        "hex_encode" => match args.first() {
            Some(Value::Str(s)) => Ok(Value::Str(hex_encode(s.as_bytes()))),
            _ => Err("crypto.hex_encode: expected str".into())
        },
        _ => Err(format!("crypto.{}: unknown function", func))
    }
}

fn sha256(data: &[u8]) -> String {
    let mut h = [
        0x6a09e667u32, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    let k = [
        0x428a2f98u32,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
        0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
        0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
        0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
        0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
        0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
        0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
        0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2,
    ];

    let mut msg = data.to_vec();
    let orig_len = data.len() as u64 * 8;
    msg.push(0x80);
    while msg.len() % 64 != 56 { msg.push(0); }
    msg.extend_from_slice(&orig_len.to_be_bytes());

    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes(chunk[i*4..i*4+4].try_into().unwrap());
        }
        for i in 16..64 {
            let s0 = w[i-15].rotate_right(7) ^ w[i-15].rotate_right(18) ^ (w[i-15] >> 3);
            let s1 = w[i-2].rotate_right(17) ^ w[i-2].rotate_right(19) ^ (w[i-2] >> 10);
            w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
        }
        let (mut a,mut b,mut c,mut d,mut e,mut f,mut g,mut hh) = (h[0],h[1],h[2],h[3],h[4],h[5],h[6],h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(k[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            hh=g; g=f; f=e; e=d.wrapping_add(temp1); d=c; c=b; b=a; a=temp1.wrapping_add(temp2);
        }
        h[0]=h[0].wrapping_add(a); h[1]=h[1].wrapping_add(b); h[2]=h[2].wrapping_add(c); h[3]=h[3].wrapping_add(d);
        h[4]=h[4].wrapping_add(e); h[5]=h[5].wrapping_add(f); h[6]=h[6].wrapping_add(g); h[7]=h[7].wrapping_add(hh);
    }
    h.iter().map(|x| format!("{:08x}", x)).collect()
}

fn md5(data: &[u8]) -> String {
    let s = [
        7u32,12,17,22,7,12,17,22,7,12,17,22,7,12,17,22,
        5,9,14,20,5,9,14,20,5,9,14,20,5,9,14,20,
        4,11,16,23,4,11,16,23,4,11,16,23,4,11,16,23,
        6,10,15,21,6,10,15,21,6,10,15,21,6,10,15,21,
    ];
    let t: Vec<u32> = (1..=64u64).map(|i| ((i as f64).sin().abs() * 4294967296.0) as u32).collect();
    let mut msg = data.to_vec();
    let orig_len = data.len() as u64 * 8;
    msg.push(0x80);
    while msg.len() % 64 != 56 { msg.push(0); }
    msg.extend_from_slice(&orig_len.to_le_bytes());

    let (mut a0,mut b0,mut c0,mut d0) = (0x67452301u32,0xefcdab89,0x98badcfe,0x10325476);

    for chunk in msg.chunks(64) {
        let mut m = [0u32; 16];
        for i in 0..16 { m[i] = u32::from_le_bytes(chunk[i*4..i*4+4].try_into().unwrap()); }
        let (mut a,mut b,mut c,mut d) = (a0,b0,c0,d0);
        for i in 0..64usize {
            let (f, g) = match i {
                0..=15 => ((b & c) | ((!b) & d), i),
                16..=31 => ((d & b) | ((!d) & c), (5*i+1)%16),
                32..=47 => (b ^ c ^ d, (3*i+5)%16),
                _ => (c ^ (b | (!d)), (7*i)%16),
            };
            let temp = a.wrapping_add(f).wrapping_add(t[i]).wrapping_add(m[g]);
            a = d; d = c; c = b;
            b = b.wrapping_add(temp.rotate_left(s[i]));
        }
        a0=a0.wrapping_add(a); b0=b0.wrapping_add(b); c0=c0.wrapping_add(c); d0=d0.wrapping_add(d);
    }
    let r: Vec<u8> = [a0,b0,c0,d0].iter().flat_map(|x| x.to_le_bytes()).collect();
    hex_encode(&r)
}

fn hex_encode(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02x}", b)).collect()
}

const B64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(data: &[u8]) -> String {
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

fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let chars: Vec<u8> = s.bytes().filter(|&c| c != b'=').collect();
    for chunk in chars.chunks(4) {
        let vals: Vec<u8> = chunk.iter().map(|&c| {
            B64.iter().position(|&b| b == c).unwrap_or(0) as u8
        }).collect();
        if vals.len() >= 2 { out.push((vals[0] << 2) | (vals[1] >> 4)); }
        if vals.len() >= 3 { out.push((vals[1] << 4) | (vals[2] >> 2)); }
        if vals.len() >= 4 { out.push((vals[2] << 6) | vals[3]); }
    }
    Ok(out)
}
