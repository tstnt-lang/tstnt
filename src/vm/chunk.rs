use crate::vm::opcode::Op;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Const {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub consts: Vec<Const>,
    pub lines: Vec<usize>,
}

#[allow(dead_code)]
impl Chunk {
    pub fn new() -> Self {
        Chunk { code: Vec::new(), consts: Vec::new(), lines: Vec::new() }
    }

    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn write_op(&mut self, op: Op, line: usize) {
        self.write(op as u8, line);
    }

    pub fn write_u16(&mut self, val: u16, line: usize) {
        let bytes = val.to_le_bytes();
        self.write(bytes[0], line);
        self.write(bytes[1], line);
    }

    pub fn write_u32(&mut self, val: u32, line: usize) {
        let bytes = val.to_le_bytes();
        for b in bytes { self.write(b, line); }
    }

    pub fn write_i64(&mut self, val: i64, line: usize) {
        let bytes = val.to_le_bytes();
        for b in bytes { self.write(b, line); }
    }

    pub fn write_f64(&mut self, val: f64, line: usize) {
        let bytes = val.to_bits().to_le_bytes();
        for b in bytes { self.write(b, line); }
    }

    pub fn add_const(&mut self, c: Const) -> u16 {
        self.consts.push(c);
        (self.consts.len() - 1) as u16
    }

    pub fn patch_u16(&mut self, offset: usize, val: u16) {
        let bytes = val.to_le_bytes();
        self.code[offset] = bytes[0];
        self.code[offset + 1] = bytes[1];
    }

    pub fn len(&self) -> usize { self.code.len() }

    pub fn read_u16(&self, offset: usize) -> u16 {
        u16::from_le_bytes([self.code[offset], self.code[offset + 1]])
    }

    pub fn read_u32(&self, offset: usize) -> u32 {
        u32::from_le_bytes([
            self.code[offset], self.code[offset+1],
            self.code[offset+2], self.code[offset+3]
        ])
    }

    pub fn read_i64(&self, offset: usize) -> i64 {
        i64::from_le_bytes([
            self.code[offset], self.code[offset+1], self.code[offset+2], self.code[offset+3],
            self.code[offset+4], self.code[offset+5], self.code[offset+6], self.code[offset+7],
        ])
    }

    pub fn read_f64(&self, offset: usize) -> f64 {
        f64::from_bits(u64::from_le_bytes([
            self.code[offset], self.code[offset+1], self.code[offset+2], self.code[offset+3],
            self.code[offset+4], self.code[offset+5], self.code[offset+6], self.code[offset+7],
        ]))
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"TST\x02");

        let const_count = self.consts.len() as u32;
        out.extend_from_slice(&const_count.to_le_bytes());
        for c in &self.consts {
            match c {
                Const::Int(n) => {
                    out.push(0x01);
                    out.extend_from_slice(&n.to_le_bytes());
                }
                Const::Float(f) => {
                    out.push(0x02);
                    out.extend_from_slice(&f.to_bits().to_le_bytes());
                }
                Const::Str(s) => {
                    out.push(0x03);
                    let bytes = s.as_bytes();
                    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                    out.extend_from_slice(bytes);
                }
                Const::Bool(b) => {
                    out.push(0x04);
                    out.push(if *b { 1 } else { 0 });
                }
                Const::Null => out.push(0x05),
            }
        }

        let code_len = self.code.len() as u32;
        out.extend_from_slice(&code_len.to_le_bytes());
        out.extend_from_slice(&self.code);
        out
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, String> {
        if &data[..4] != b"TST\x02" {
            return Err("Invalid .tst file".into());
        }
        let mut pos = 4;

        let const_count = u32::from_le_bytes(data[pos..pos+4].try_into().unwrap()) as usize;
        pos += 4;

        let mut consts = Vec::new();
        for _ in 0..const_count {
            let tag = data[pos]; pos += 1;
            match tag {
                0x01 => {
                    let n = i64::from_le_bytes(data[pos..pos+8].try_into().unwrap());
                    pos += 8;
                    consts.push(Const::Int(n));
                }
                0x02 => {
                    let f = f64::from_bits(u64::from_le_bytes(data[pos..pos+8].try_into().unwrap()));
                    pos += 8;
                    consts.push(Const::Float(f));
                }
                0x03 => {
                    let len = u32::from_le_bytes(data[pos..pos+4].try_into().unwrap()) as usize;
                    pos += 4;
                    let s = std::str::from_utf8(&data[pos..pos+len]).map_err(|e| e.to_string())?;
                    pos += len;
                    consts.push(Const::Str(s.to_string()));
                }
                0x04 => {
                    let b = data[pos] != 0; pos += 1;
                    consts.push(Const::Bool(b));
                }
                0x05 => consts.push(Const::Null),
                _ => return Err(format!("Unknown const tag: {}", tag))
            }
        }

        let code_len = u32::from_le_bytes(data[pos..pos+4].try_into().unwrap()) as usize;
        pos += 4;
        let code = data[pos..pos+code_len].to_vec();

        Ok(Chunk { code, consts, lines: vec![0; code_len] })
    }
}
