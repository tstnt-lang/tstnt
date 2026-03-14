pub mod opcode;
pub mod chunk;
pub mod codegen;

use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use crate::vm::opcode::Op;
use crate::vm::chunk::{Chunk, Const};
use crate::vm::codegen::{Codegen, FuncProto};
use crate::value::Value;
use crate::parser::Node;
use crate::stdlib;

#[allow(dead_code)]
struct CallFrame {
    ip: usize,
    stack_base: usize,
    func_name: String,
}

#[allow(dead_code)]
pub struct VM {
    stack: Vec<Value>,
    globals: Vec<Value>,
    locals: Vec<Value>,
    frames: Vec<CallFrame>,
    funcs: HashMap<String, FuncProto>,
    chunk: Chunk,
    ip: usize,
}

impl VM {
    pub fn new(chunk: Chunk, funcs: Vec<FuncProto>) -> Self {
        let mut func_map = HashMap::new();
        for f in funcs { func_map.insert(f.name.clone(), f); }
        VM {
            stack: Vec::new(),
            globals: vec![Value::Null; 1024],
            locals: Vec::new(),
            frames: Vec::new(),
            funcs: func_map,
            chunk,
            ip: 0,
        }
    }

    fn push(&mut self, v: Value) { self.stack.push(v); }

    fn pop(&mut self) -> Value { self.stack.pop().unwrap_or(Value::Null) }

    fn peek(&self) -> &Value { self.stack.last().unwrap_or(&Value::Null) }

    pub fn run(&mut self) -> Result<(), String> {
        loop {
            if self.ip >= self.chunk.code.len() { break; }
            let byte = self.chunk.code[self.ip];
            self.ip += 1;

            let op = Op::from_u8(byte).ok_or_else(|| format!("Unknown opcode: 0x{:02X}", byte))?;

            match op {
                Op::PushInt => {
                    let idx = self.read_u16();
                    if let Const::Int(n) = &self.chunk.consts[idx as usize] {
                        self.push(Value::Int(*n));
                    }
                }
                Op::PushFloat => {
                    let idx = self.read_u16();
                    if let Const::Float(f) = &self.chunk.consts[idx as usize] {
                        self.push(Value::Float(*f));
                    }
                }
                Op::PushStr => {
                    let idx = self.read_u16();
                    if let Const::Str(s) = &self.chunk.consts[idx as usize] {
                        self.push(Value::Str(s.clone()));
                    }
                }
                Op::PushBool => {
                    let b = self.chunk.code[self.ip] != 0;
                    self.ip += 1;
                    self.push(Value::Bool(b));
                }
                Op::PushNull => self.push(Value::Null),

                Op::Pop => { self.pop(); }
                Op::Dup => {
                    let v = self.peek().clone();
                    self.push(v);
                }
                Op::Swap => {
                    let a = self.pop();
                    let b = self.pop();
                    self.push(a);
                    self.push(b);
                }

                Op::LoadGlobal => {
                    let idx = self.read_u16() as usize;
                    self.push(self.globals[idx].clone());
                }
                Op::StoreGlobal => {
                    let idx = self.read_u16() as usize;
                    let v = self.pop();
                    if idx >= self.globals.len() { self.globals.resize(idx + 1, Value::Null); }
                    self.globals[idx] = v;
                }
                Op::LoadLocal => {
                    let idx = self.read_u16() as usize;
                    let base = self.frames.last().map(|f| f.stack_base).unwrap_or(0);
                    let v = self.stack.get(base + idx).cloned().unwrap_or(Value::Null);
                    self.push(v);
                }
                Op::StoreLocal => {
                    let idx = self.read_u16() as usize;
                    let base = self.frames.last().map(|f| f.stack_base).unwrap_or(0);
                    let v = self.pop();
                    let target = base + idx;
                    while self.stack.len() <= target { self.stack.push(Value::Null); }
                    self.stack[target] = v;
                }
                Op::Load => {
                    let idx = self.read_u16();
                    if let Const::Str(name) = &self.chunk.consts[idx as usize].clone() {
                        self.push(Value::Str(name.clone()));
                    }
                }
                Op::Store => {
                    self.read_u16();
                    self.pop();
                }

                Op::Add => { let r = self.pop(); let l = self.pop(); self.push(self.binop(l, "+", r)?); }
                Op::Sub => { let r = self.pop(); let l = self.pop(); self.push(self.binop(l, "-", r)?); }
                Op::Mul => { let r = self.pop(); let l = self.pop(); self.push(self.binop(l, "*", r)?); }
                Op::Div => { let r = self.pop(); let l = self.pop(); self.push(self.binop(l, "/", r)?); }
                Op::Mod => { let r = self.pop(); let l = self.pop(); self.push(self.binop(l, "%", r)?); }
                Op::Pow => { let r = self.pop(); let l = self.pop(); self.push(self.binop(l, "**", r)?); }
                Op::Neg => {
                    let v = self.pop();
                    self.push(match v {
                        Value::Int(n) => Value::Int(-n),
                        Value::Float(f) => Value::Float(-f),
                        _ => return Err("Cannot negate".into())
                    });
                }

                Op::Eq    => { let r = self.pop(); let l = self.pop(); self.push(self.binop(l, "==", r)?); }
                Op::Neq   => { let r = self.pop(); let l = self.pop(); self.push(self.binop(l, "!=", r)?); }
                Op::Lt    => { let r = self.pop(); let l = self.pop(); self.push(self.binop(l, "<", r)?); }
                Op::Gt    => { let r = self.pop(); let l = self.pop(); self.push(self.binop(l, ">", r)?); }
                Op::LtEq  => { let r = self.pop(); let l = self.pop(); self.push(self.binop(l, "<=", r)?); }
                Op::GtEq  => { let r = self.pop(); let l = self.pop(); self.push(self.binop(l, ">=", r)?); }
                Op::And   => { let r = self.pop(); let l = self.pop(); self.push(Value::Bool(self.truthy(&l) && self.truthy(&r))); }
                Op::Or    => { let r = self.pop(); let l = self.pop(); self.push(Value::Bool(self.truthy(&l) || self.truthy(&r))); }
                Op::Not   => { let v = self.pop(); self.push(Value::Bool(!self.truthy(&v))); }

                Op::Jump => {
                    let target = self.read_u16() as usize;
                    self.ip = target;
                }
                Op::JumpIf => {
                    let target = self.read_u16() as usize;
                    let v = self.pop();
                    if self.truthy(&v) { self.ip = target; }
                }
                Op::JumpIfNot => {
                    let target = self.read_u16() as usize;
                    let v = self.pop();
                    if !self.truthy(&v) { self.ip = target; }
                }

                Op::CallNative => {
                    let idx = self.read_u16() as usize;
                    let argc = self.chunk.code[self.ip] as usize;
                    self.ip += 1;

                    let name = if let Const::Str(s) = &self.chunk.consts[idx] {
                        s.clone()
                    } else {
                        return Err("CallNative: expected string name".into());
                    };

                    let args: Vec<Value> = self.stack.drain(self.stack.len().saturating_sub(argc)..).collect();
                    let result = self.call_named(&name, args)?;
                    self.push(result);
                }

                Op::Return => {
                    let ret = self.pop();
                    if let Some(frame) = self.frames.pop() {
                        self.stack.truncate(frame.stack_base);
                        self.ip = frame.ip;
                        self.push(ret);
                    } else {
                        break;
                    }
                }

                Op::MakeArray => {
                    let count = self.read_u16() as usize;
                    let elements: Vec<Value> = self.stack.drain(self.stack.len().saturating_sub(count)..).collect();
                    self.push(Value::Array(elements));
                }
                Op::MakeTuple => {
                    let count = self.read_u16() as usize;
                    let elements: Vec<Value> = self.stack.drain(self.stack.len().saturating_sub(count)..).collect();
                    self.push(Value::Tuple(elements));
                }
                Op::MakeStruct => {
                    let name_idx = self.read_u16() as usize;
                    let fields_idx = self.read_u16() as usize;
                    let count = self.read_u16() as usize;
                    let name = if let Const::Str(s) = &self.chunk.consts[name_idx] { s.clone() } else { "".into() };
                    let field_names: Vec<String> = if let Const::Str(s) = &self.chunk.consts[fields_idx] {
                        s.split(',').map(|s: &str| s.to_string()).collect()
                    } else { vec![] };
                    let values: Vec<Value> = self.stack.drain(self.stack.len().saturating_sub(count)..).collect();
                    let mut map = HashMap::new();
                    for (k, v) in field_names.into_iter().zip(values.into_iter()) { map.insert(k, v); }
                    self.push(Value::Struct(name, map));
                }
                Op::ArrayGet => {
                    let idx = self.pop();
                    let arr = self.pop();
                    match (arr, idx) {
                        (Value::Array(a), Value::Int(i)) => self.push(a.get(i as usize).cloned().unwrap_or(Value::Null)),
                        _ => self.push(Value::Null)
                    }
                }
                Op::ArraySet => {
                    let val = self.pop();
                    let idx = self.pop();
                    let arr = self.pop();
                    match (arr, idx) {
                        (Value::Array(mut a), Value::Int(i)) => {
                            if (i as usize) < a.len() { a[i as usize] = val; }
                            self.push(Value::Array(a));
                        }
                        _ => self.push(Value::Null)
                    }
                }
                Op::ArrayLen => {
                    let v = self.pop();
                    match v {
                        Value::Array(a) => self.push(Value::Int(a.len() as i64)),
                        Value::Str(s) => self.push(Value::Int(s.len() as i64)),
                        _ => self.push(Value::Int(0))
                    }
                }
                Op::ArrayPush => {
                    let val = self.pop();
                    let arr = self.pop();
                    match arr {
                        Value::Array(mut a) => { a.push(val); self.push(Value::Array(a)); }
                        _ => self.push(Value::Null)
                    }
                }
                Op::ArrayPop => {
                    let arr = self.pop();
                    match arr {
                        Value::Array(mut a) => { a.pop(); self.push(Value::Array(a)); }
                        _ => self.push(Value::Null)
                    }
                }
                Op::FieldGet => {
                    let idx = self.read_u16() as usize;
                    let field = if let Const::Str(s) = &self.chunk.consts[idx] { s.clone() } else { "".into() };
                    let obj = self.pop();
                    match obj {
                        Value::Struct(_, fields) => self.push(fields.get(&field).cloned().unwrap_or(Value::Null)),
                        _ => self.push(Value::Null)
                    }
                }
                Op::FieldSet => {
                    let idx = self.read_u16() as usize;
                    let field = if let Const::Str(s) = &self.chunk.consts[idx] { s.clone() } else { "".into() };
                    let val = self.pop();
                    let obj = self.pop();
                    match obj {
                        Value::Struct(name, mut fields) => {
                            fields.insert(field, val);
                            self.push(Value::Struct(name, fields));
                        }
                        _ => self.push(Value::Null)
                    }
                }

                Op::ToInt => {
                    let v = self.pop();
                    self.push(match v {
                        Value::Int(n) => Value::Int(n),
                        Value::Float(f) => Value::Int(f as i64),
                        Value::Str(s) => Value::Int(s.parse().unwrap_or(0)),
                        Value::Bool(b) => Value::Int(if b { 1 } else { 0 }),
                        _ => Value::Int(0)
                    });
                }
                Op::ToFloat => {
                    let v = self.pop();
                    self.push(match v {
                        Value::Float(f) => Value::Float(f),
                        Value::Int(n) => Value::Float(n as f64),
                        Value::Str(s) => Value::Float(s.parse().unwrap_or(0.0)),
                        _ => Value::Float(0.0)
                    });
                }
                Op::ToStr => {
                    let v = self.pop();
                    self.push(Value::Str(v.to_string()));
                }
                Op::ToBool => {
                    let v = self.pop();
                    let b = self.truthy(&v);
                    self.push(Value::Bool(b));
                }
                Op::TypeOf => {
                    let v = self.pop();
                    let t = match v {
                        Value::Int(_) => "int",
                        Value::Float(_) => "float",
                        Value::Str(_) => "str",
                        Value::Bool(_) => "bool",
                        Value::Array(_) => "array",
                        Value::Tuple(_) => "tuple",
                        Value::Struct(_, _) => "struct",
                        Value::Lambda(_, _) => "lambda",
                        Value::Null => "null",
                    };
                    self.push(Value::Str(t.into()));
                }

                Op::Print => {
                    let v = self.pop();
                    print!("{}", v);
                    io::stdout().flush().ok();
                    self.push(Value::Null);
                }
                Op::PrintLn => {
                    let v = self.pop();
                    println!("{}", v);
                    self.push(Value::Null);
                }
                Op::Input => {
                    let stdin = io::stdin();
                    let mut line = String::new();
                    stdin.lock().read_line(&mut line).ok();
                    self.push(Value::Str(line.trim_end().to_string()));
                }
                Op::Assert => {
                    let v = self.pop();
                    if !self.truthy(&v) {
                        return Err("Assertion failed".into());
                    }
                    self.push(Value::Null);
                }
                Op::Panic => {
                    let v = self.pop();
                    return Err(format!("panic: {}", v));
                }

                Op::Halt => break,
                _ => return Err(format!("Unimplemented op: {:?}", op))
            }
        }
        Ok(())
    }

    fn read_u16(&mut self) -> u16 {
        let v = u16::from_le_bytes([self.chunk.code[self.ip], self.chunk.code[self.ip + 1]]);
        self.ip += 2;
        v
    }

    fn binop(&self, l: Value, op: &str, r: Value) -> Result<Value, String> {
        match (&l, &r) {
            (Value::Int(a), Value::Int(b)) => match op {
                "+" => Ok(Value::Int(a + b)),
                "-" => Ok(Value::Int(a - b)),
                "*" => Ok(Value::Int(a * b)),
                "/" => if *b == 0 { Err("Division by zero".into()) } else { Ok(Value::Int(a / b)) },
                "%" => Ok(Value::Int(a % b)),
                "**" => Ok(Value::Float((*a as f64).powf(*b as f64))),
                "==" => Ok(Value::Bool(a == b)),
                "!=" => Ok(Value::Bool(a != b)),
                "<" => Ok(Value::Bool(a < b)),
                ">" => Ok(Value::Bool(a > b)),
                "<=" => Ok(Value::Bool(a <= b)),
                ">=" => Ok(Value::Bool(a >= b)),
                _ => Err(format!("Unknown op: {}", op))
            },
            (Value::Float(a), Value::Float(b)) => match op {
                "+" => Ok(Value::Float(a + b)),
                "-" => Ok(Value::Float(a - b)),
                "*" => Ok(Value::Float(a * b)),
                "/" => Ok(Value::Float(a / b)),
                "**" => Ok(Value::Float(a.powf(*b))),
                "==" => Ok(Value::Bool(a == b)),
                "!=" => Ok(Value::Bool(a != b)),
                "<" => Ok(Value::Bool(a < b)),
                ">" => Ok(Value::Bool(a > b)),
                "<=" => Ok(Value::Bool(a <= b)),
                ">=" => Ok(Value::Bool(a >= b)),
                _ => Err(format!("Unknown op: {}", op))
            },
            (Value::Int(a), Value::Float(b)) => self.binop(Value::Float(*a as f64), op, Value::Float(*b)),
            (Value::Float(a), Value::Int(b)) => self.binop(Value::Float(*a), op, Value::Float(*b as f64)),
            (Value::Str(a), Value::Str(b)) => match op {
                "+" => Ok(Value::Str(a.clone() + b)),
                "==" => Ok(Value::Bool(a == b)),
                "!=" => Ok(Value::Bool(a != b)),
                _ => Err(format!("Unknown str op: {}", op))
            },
            _ => Err(format!("Type mismatch: {} {} {}", l, op, r))
        }
    }

    fn truthy(&self, v: &Value) -> bool {
        match v {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Null => false,
            _ => true,
        }
    }

    fn call_named(&mut self, name: &str, args: Vec<Value>) -> Result<Value, String> {
        if name.contains('.') {
            let parts: Vec<&str> = name.splitn(2, '.').collect();
            return stdlib::call(parts[0], parts[1], args);
        }

        match name {
            "print" => {
                let parts: Vec<String> = args.iter().map(|v| v.to_string()).collect();
                println!("{}", parts.join(" "));
                Ok(Value::Null)
            }
            "len" => match args.first() {
                Some(Value::Array(a)) => Ok(Value::Int(a.len() as i64)),
                Some(Value::Str(s)) => Ok(Value::Int(s.len() as i64)),
                _ => Ok(Value::Int(0))
            },
            "str" => Ok(Value::Str(args.first().map(|v| v.to_string()).unwrap_or_default())),
            "int" => match args.first() {
                Some(Value::Float(f)) => Ok(Value::Int(*f as i64)),
                Some(Value::Str(s)) => Ok(Value::Int(s.parse().unwrap_or(0))),
                Some(Value::Int(n)) => Ok(Value::Int(*n)),
                _ => Ok(Value::Int(0))
            },
            "float" => match args.first() {
                Some(Value::Int(n)) => Ok(Value::Float(*n as f64)),
                Some(Value::Str(s)) => Ok(Value::Float(s.parse().unwrap_or(0.0))),
                Some(Value::Float(f)) => Ok(Value::Float(*f)),
                _ => Ok(Value::Float(0.0))
            },
            "main" => {
                if let Some(proto) = self.funcs.get("main").map(|f| f.chunk.clone()) {
                    let saved_chunk = std::mem::replace(&mut self.chunk, proto);
                    let saved_ip = self.ip;
                    self.ip = 0;
                    self.frames.push(CallFrame { ip: saved_ip, stack_base: self.stack.len(), func_name: "main".into() });
                    self.chunk = self.funcs.get("main").unwrap().chunk.clone();
                    self.ip = 0;
                    self.run()?;
                    if let Some(frame) = self.frames.pop() {
                        self.chunk = saved_chunk;
                        self.ip = frame.ip;
                    }
                }
                Ok(Value::Null)
            }
            _ => {
                if let Some(proto) = self.funcs.get(name) {
                    let func_chunk = proto.chunk.clone();
                    let param_count = proto.params.len();
                    let saved_ip = self.ip;
                    let saved_chunk = self.chunk.clone();
                    let stack_base = self.stack.len();

                    for arg in args.into_iter().take(param_count) {
                        self.stack.push(arg);
                    }

                    self.frames.push(CallFrame { ip: saved_ip, stack_base, func_name: name.into() });
                    self.chunk = func_chunk;
                    self.ip = 0;
                    self.run()?;

                    let ret = self.pop();
                    if let Some(frame) = self.frames.pop() {
                        self.stack.truncate(frame.stack_base);
                        self.chunk = saved_chunk;
                        self.ip = frame.ip;
                    }
                    Ok(ret)
                } else {
                    Err(format!("Unknown function: {}", name))
                }
            }
        }
    }
}

#[allow(dead_code)]
pub fn run_from_ast(ast: &[Node]) -> Result<(), String> {
    let mut cg = Codegen::new();
    cg.compile(ast)?;
    let funcs = cg.funcs;
    let chunk = cg.chunk;
    let mut vm = VM::new(chunk, funcs);
    vm.run()
}

pub fn run_tst_file(path: &str) -> Result<(), String> {
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    let chunk = Chunk::deserialize(&data)?;
    let mut vm = VM::new(chunk, vec![]);
    vm.run()
}
