pub mod opcode;
pub mod chunk;
pub mod codegen;
pub mod jit;

use std::collections::HashMap;
use std::rc::Rc;
use std::io::{self, BufRead, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::vm::opcode::Op;
use crate::vm::chunk::{Chunk, Const};
use crate::vm::codegen::{Codegen, FuncProto};
use crate::vm::jit::{JitCache, try_jit_compile, jit_call};
use crate::value::Value;
use crate::parser::Node;
use crate::stdlib;

#[derive(Clone)]
struct CallFrame {
    chunk: Rc<Chunk>,
    ip: usize,
    stack_base: usize,
    is_cached: bool,
    try_stack: Vec<(usize, usize)>,
}

pub struct VM {
    stack: Vec<Value>,
    globals: Vec<Value>,
    frames: Vec<CallFrame>,
    funcs: HashMap<String, (Rc<Chunk>, usize, bool, bool)>,
    cache: HashMap<String, Value>,
    timer_start: u64,
    jit: JitCache,
}

impl VM {
    pub fn new(main_chunk: Chunk, funcs: Vec<FuncProto>) -> Self {
        let mut func_map = HashMap::new();
        for f in funcs { func_map.insert(f.name.clone(), (Rc::new(f.chunk), f.params.len(), f.is_cached, f.is_timed)); }
        VM {
            stack: Vec::with_capacity(512),
            globals: Vec::new(),
            frames: vec![CallFrame { chunk: Rc::new(main_chunk), ip: 0, stack_base: 0, is_cached: false, try_stack: vec![] }],
            funcs: func_map,
            cache: HashMap::new(),
            timer_start: 0,
            jit: JitCache::new(),
        }
    }

    #[inline] fn push(&mut self, v: Value) { self.stack.push(v); }
    #[inline] fn pop(&mut self) -> Value { self.stack.pop().unwrap_or(Value::Null) }
    #[inline] fn peek(&self) -> &Value { self.stack.last().unwrap_or(&Value::Null) }

    #[inline] fn read_u8(&mut self) -> u8 {
        let f = unsafe { self.frames.last_mut().unwrap_unchecked() };
        let b = f.chunk.code[f.ip]; f.ip += 1; b
    }
    #[inline] fn read_u16(&mut self) -> u16 {
        let f = unsafe { self.frames.last_mut().unwrap_unchecked() };
        let v = u16::from_le_bytes([f.chunk.code[f.ip], f.chunk.code[f.ip+1]]);
        f.ip += 2; v
    }
    #[inline] fn get_const(&self, idx: usize) -> &Const {
        unsafe { &self.frames.last().unwrap_unchecked().chunk.consts[idx] }
    }
    fn now_ms() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
    }

    pub fn run(&mut self) -> Result<(), String> {
        loop {
            let (frame_ip, code_len) = match self.frames.last() {
                Some(f) => (f.ip, f.chunk.code.len()),
                None => break,
            };
            if frame_ip >= code_len {
                let frame = self.frames.pop().unwrap();
                if self.frames.is_empty() { break; }
                self.stack.truncate(frame.stack_base);
                self.push(Value::Null);
                continue;
            }
            let byte = { let f = unsafe { self.frames.last_mut().unwrap_unchecked() }; let b = f.chunk.code[f.ip]; f.ip += 1; b };
            let op = match Op::from_u8(byte) {
                Some(o) => o,
                None => return Err(format!("unknown opcode 0x{:02X}", byte))
            };

            match op {
                Op::PushInt => { let i = self.read_u16() as usize; let v = if let Const::Int(n) = unsafe { self.get_const(i) } { Value::Int(*n) } else { Value::Null }; self.push(v); }
                Op::PushIntInline => { let n = self.read_u16() as i64 - 32768; self.push(Value::Int(n)); }
                Op::PushFloat => { let i = self.read_u16() as usize; let v = if let Const::Float(f) = self.get_const(i) { Value::Float(*f) } else { Value::Null }; self.push(v); }
                Op::PushStr => { let i = self.read_u16() as usize; let v = if let Const::Str(s) = self.get_const(i) { Value::Str(s.clone()) } else { Value::Null }; self.push(v); }
                Op::PushBool => { let b = self.read_u8() != 0; self.push(Value::Bool(b)); }
                Op::PushNull => self.push(Value::Null),
                Op::Pop => { self.pop(); }
                Op::Dup => { let v = self.peek().clone(); self.push(v); }
                Op::Swap => { let a = self.pop(); let b = self.pop(); self.push(a); self.push(b); }

                Op::LoadGlobal => {
                    let i = self.read_u16() as usize;
                    let v = self.globals.get(i).cloned().unwrap_or(Value::Null);
                    self.push(v);
                }
                Op::StoreGlobal => {
                    let i = self.read_u16() as usize;
                    let v = self.pop();
                    while self.globals.len() <= i { self.globals.push(Value::Null); }
                    self.globals[i] = v;
                }
                Op::LoadLocal => {
                    let slot = self.read_u16() as usize;
                    let base = unsafe { self.frames.last().unwrap_unchecked() }.stack_base;
                    let v = self.stack.get(base + slot).cloned().unwrap_or(Value::Null);
                    self.push(v);
                }
                Op::StoreLocal => {
                    let slot = self.read_u16() as usize;
                    let base = unsafe { self.frames.last().unwrap_unchecked() }.stack_base;
                    let v = self.pop();
                    let idx = base + slot;
                    while self.stack.len() <= idx { self.stack.push(Value::Null); }
                    self.stack[idx] = v;
                }
                Op::Load => { let i = self.read_u16() as usize; let v = if let Const::Str(s) = self.get_const(i) { Value::Str(s.clone()) } else { Value::Null }; self.push(v); }
                Op::Store => { self.read_u16(); self.pop(); }

                Op::Add => { let r = self.pop(); let l = self.pop(); self.push(self.binop(l, "+", r)?); }
                Op::Sub => { let r = self.pop(); let l = self.pop(); self.push(self.binop(l, "-", r)?); }
                Op::Mul => { let r = self.pop(); let l = self.pop(); self.push(self.binop(l, "*", r)?); }
                Op::Div => { let r = self.pop(); let l = self.pop(); self.push(self.binop(l, "/", r)?); }
                Op::Mod => { let r = self.pop(); let l = self.pop(); self.push(self.binop(l, "%", r)?); }
                Op::Pow => { let r = self.pop(); let l = self.pop(); self.push(self.binop(l, "**", r)?); }
                Op::Neg => { let v = self.pop(); self.push(match v { Value::Int(n) => Value::Int(-n), Value::Float(f) => Value::Float(-f), _ => return Err("neg: number required".into()) }); }
                Op::Concat => {
                    let count = self.read_u16() as usize;
                    let parts: Vec<Value> = self.stack.drain(self.stack.len().saturating_sub(count)..).collect();
                    let s: String = parts.iter().map(|v| v.to_string()).collect();
                    self.push(Value::Str(s));
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
                Op::In    => {
                    let container = self.pop(); let val = self.pop();
                    let found = match &container {
                        Value::Array(a) => a.iter().any(|x| x.to_string() == val.to_string()),
                        Value::Str(s) => s.contains(&val.to_string()),
                        _ => false,
                    };
                    self.push(Value::Bool(found));
                }

                Op::Jump => { let t = self.read_u16() as usize; self.frames.last_mut().unwrap().ip = t; }
                Op::JumpIf => { let t = self.read_u16() as usize; let v = self.pop(); if self.truthy(&v) { self.frames.last_mut().unwrap().ip = t; } }
                Op::JumpIfNot => { let t = self.read_u16() as usize; let v = self.pop(); if !self.truthy(&v) { self.frames.last_mut().unwrap().ip = t; } }
                Op::Break => {
                    let t = self.read_u16() as usize;
                    self.frames.last_mut().unwrap().ip = t;
                }
                Op::Continue => {
                    let t = self.read_u16() as usize;
                    self.frames.last_mut().unwrap().ip = t;
                }

                Op::Call => {
                    let i = self.read_u16() as usize;
                    let argc = self.read_u8() as usize;
                    let name = if let Const::Str(s) = self.get_const(i) { s.clone() } else { return Err("Call: expected name".into()); };
                    if let Some((arc_chunk, param_count, is_cached)) = self.funcs.get(&name).map(|(c, p, ic, _)| (c.clone(), *p, *ic)) {
                        let args_start = self.stack.len().saturating_sub(argc);
                        if is_cached {
                            let cache_key = format!("{}__{}", name, self.stack[args_start..].iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","));
                            if let Some(cached) = self.cache.get(&cache_key).cloned() {
                                self.stack.truncate(args_start);
                                self.push(cached); continue;
                            }
                        }
                        self.stack.truncate(args_start + param_count.min(argc as usize));
                        self.frames.push(CallFrame { chunk: arc_chunk, ip: 0, stack_base: args_start, is_cached, try_stack: vec![] });
                    } else {
                        let args: Vec<Value> = self.stack.drain(self.stack.len().saturating_sub(argc)..).collect();
                        let r = self.call_native(&name, args)?;
                        self.push(r);
                    }
                }
                Op::CallNative => {
                    let i = self.read_u16() as usize;
                    let argc = self.read_u8() as usize;
                    let name = if let Const::Str(s) = self.get_const(i) { s.clone() } else { return Err("CallNative: expected name".into()); };
                    let args: Vec<Value> = self.stack.drain(self.stack.len().saturating_sub(argc)..).collect();
                    let r = self.call_native(&name, args)?;
                    self.push(r);
                }
                Op::CallMethod => {
                    let i = self.read_u16() as usize;
                    let argc = self.read_u8() as usize;
                    let method = if let Const::Str(s) = self.get_const(i) { s.clone() } else { return Err("CallMethod: expected name".into()); };
                    let args: Vec<Value> = self.stack.drain(self.stack.len().saturating_sub(argc)..).collect();
                    let obj = self.pop();
                    let full_args = std::iter::once(obj.clone()).chain(args.into_iter()).collect();
                    let r = match &obj {
                        Value::Str(_) => stdlib::call("strings", &method, full_args)?,
                        Value::Array(_) => stdlib::call("arr", &method, full_args)?,
                        _ => return Err(format!("no method {} on {:?}", method, obj))
                    };
                    self.push(r);
                }
                Op::MakeLambda => {
                    let i = self.read_u16() as usize;
                    let name = if let Const::Str(s) = self.get_const(i) { s.clone() } else { return Err("MakeLambda".into()); };
                    self.push(Value::Str(format!("__lambda:{}", name)));
                }

                Op::Return => {
                    let base = unsafe { self.frames.last().unwrap_unchecked() }.stack_base;
                    let ret = if self.stack.len() > base { self.pop() } else { Value::Null };
                    let frame = unsafe { self.frames.pop().unwrap_unchecked() };
                    if self.frames.is_empty() { break; }
                    self.stack.truncate(frame.stack_base);
                    self.push(ret);
                }

                Op::Throw => {
                    let v = self.pop();
                    let msg = v.to_string();
                    let mut caught = false;
                    loop {
                        let result = self.frames.last_mut().and_then(|f| f.try_stack.pop());
                        if let Some((catch_ip, stack_base)) = result {
                            self.stack.truncate(stack_base);
                            self.push(Value::Str(msg.clone()));
                            self.frames.last_mut().unwrap().ip = catch_ip;
                            caught = true;
                            break;
                        }
                        if self.frames.pop().is_none() { break; }
                        if self.frames.is_empty() { break; }
                    }
                    if !caught { return Err(msg); }
                }
                Op::EnterTry => {
                    let catch_ip = self.read_u16() as usize;
                    let base = self.stack.len();
                    self.frames.last_mut().unwrap().try_stack.push((catch_ip, base));
                }
                Op::LeaveTry => {
                    self.frames.last_mut().unwrap().try_stack.pop();
                }

                Op::CacheCheck => {
                    let i = self.read_u16() as usize;
                    let argc = self.read_u8() as usize;
                    let _hit_jump = self.read_u16() as usize;
                    let fname = if let Const::Str(s) = self.get_const(i) { s.clone() } else { "".into() };
                    let base = self.frames.last().map(|f| f.stack_base).unwrap_or(0);
                    let args: Vec<String> = (0..argc).map(|i| self.stack.get(base + i).map(|v| v.to_string()).unwrap_or_default()).collect();
                    let key = format!("{}__{}", fname, args.join(","));
                    if let Some(cached) = self.cache.get(&key).cloned() {
                        self.push(cached);
                        self.chunk_return()?;
                    }
                }
                Op::CacheStore => {
                    let i = self.read_u16() as usize;
                    let fname = if let Const::Str(s) = self.get_const(i) { s.clone() } else { "".into() };
                    let val = self.peek().clone();
                    self.cache.insert(fname, val);
                }
                Op::TimerStart => { self.timer_start = Self::now_ms(); }
                Op::TimerEnd => {
                    let i = self.read_u16() as usize;
                    let fname = if let Const::Str(s) = self.get_const(i) { s.clone() } else { "".into() };
                    let elapsed = Self::now_ms() - self.timer_start;
                    eprintln!("\x1b[36m@timer\x1b[0m: {}() took {}ms", fname, elapsed);
                }

                Op::MakeArray => {
                    let count = self.read_u16() as usize;
                    let els: Vec<Value> = self.stack.drain(self.stack.len().saturating_sub(count)..).collect();
                    self.push(Value::Array(els));
                }
                Op::MakeTuple => {
                    let count = self.read_u16() as usize;
                    let els: Vec<Value> = self.stack.drain(self.stack.len().saturating_sub(count)..).collect();
                    self.push(Value::Tuple(els));
                }
                Op::MakeStruct => {
                    let ni = self.read_u16() as usize;
                    let fi = self.read_u16() as usize;
                    let count = self.read_u16() as usize;
                    let name = if let Const::Str(s) = self.get_const(ni) { s.clone() } else { "".into() };
                    let fnames: Vec<String> = if let Const::Str(s) = self.get_const(fi) { s.split(',').map(|s| s.to_string()).collect() } else { vec![] };
                    let vals: Vec<Value> = self.stack.drain(self.stack.len().saturating_sub(count)..).collect();
                    let mut map = HashMap::new();
                    for (k, v) in fnames.into_iter().zip(vals) { map.insert(k, v); }
                    self.push(Value::Struct(name, map));
                }
                Op::MakeEnum => {
                    let i = self.read_u16() as usize;
                    let argc = self.read_u8() as usize;
                    let name = if let Const::Str(s) = self.get_const(i) { s.clone() } else { "".into() };
                    let parts: Vec<&str> = name.splitn(2, "::").collect();
                    let (en, vn) = if parts.len() == 2 { (parts[0].to_string(), parts[1].to_string()) } else { (name.clone(), name.clone()) };
                    let args: Vec<Value> = self.stack.drain(self.stack.len().saturating_sub(argc)..).collect();
                    self.push(Value::EnumVariant(en, vn, args));
                }
                Op::Spread => {
                    let v = self.pop();
                    match v {
                        Value::Array(a) => { for item in a { self.push(item); } }
                        other => self.push(other)
                    }
                }
                Op::ArrayGet => { let i = self.pop(); let a = self.pop(); self.push(match (a, i) { (Value::Array(a), Value::Int(i)) => a.get(i as usize).cloned().unwrap_or(Value::Null), (Value::Str(s), Value::Int(i)) => s.chars().nth(i as usize).map(|c| Value::Str(c.to_string())).unwrap_or(Value::Null), _ => Value::Null }); }
                Op::ArraySet => { let v = self.pop(); let i = self.pop(); let a = self.pop(); match (a, i) { (Value::Array(mut a), Value::Int(i)) => { if (i as usize) < a.len() { a[i as usize] = v; } self.push(Value::Array(a)); } _ => self.push(Value::Null) } }
                Op::ArrayLen => { let v = self.pop(); self.push(match v { Value::Array(a) => Value::Int(a.len() as i64), Value::Str(s) => Value::Int(s.chars().count() as i64), _ => Value::Int(0) }); }
                Op::ArrayPush => { let v = self.pop(); let a = self.pop(); match a { Value::Array(mut a) => { a.push(v); self.push(Value::Array(a)); } _ => self.push(Value::Null) } }
                Op::ArrayPop => { let a = self.pop(); match a { Value::Array(mut a) => { a.pop(); self.push(Value::Array(a)); } _ => self.push(Value::Null) } }
                Op::FieldGet => {
                    let i = self.read_u16() as usize;
                    let f = if let Const::Str(s) = self.get_const(i) { s.clone() } else { "".into() };
                    let obj = self.pop();
                    self.push(match obj {
                        Value::Struct(_, fields) => fields.get(&f).cloned().unwrap_or(Value::Null),
                        Value::EnumVariant(_, _, data) => {
                            match f.as_str() {
                                "0" => data.get(0).cloned().unwrap_or(Value::Null),
                                "1" => data.get(1).cloned().unwrap_or(Value::Null),
                                _ => Value::Null
                            }
                        }
                        _ => Value::Null
                    });
                }
                Op::FieldSet => {
                    let i = self.read_u16() as usize;
                    let f = if let Const::Str(s) = self.get_const(i) { s.clone() } else { "".into() };
                    let v = self.pop(); let obj = self.pop();
                    match obj { Value::Struct(n, mut fields) => { fields.insert(f, v); self.push(Value::Struct(n, fields)); } _ => self.push(Value::Null) }
                }
                Op::OptField => {
                    let i = self.read_u16() as usize;
                    let f = if let Const::Str(s) = self.get_const(i) { s.clone() } else { "".into() };
                    let obj = self.pop();
                    self.push(match obj {
                        Value::Null => Value::Null,
                        Value::Struct(_, fields) => fields.get(&f).cloned().unwrap_or(Value::Null),
                        _ => Value::Null
                    });
                }
                Op::OptCall => {
                    let i = self.read_u16() as usize;
                    let argc = self.read_u8() as usize;
                    let _method = if let Const::Str(s) = self.get_const(i) { s.clone() } else { "".into() };
                    let _args: Vec<Value> = self.stack.drain(self.stack.len().saturating_sub(argc)..).collect();
                    let obj = self.pop();
                    if matches!(obj, Value::Null) { self.push(Value::Null); } else { self.push(obj); }
                }
                Op::Interp => {
                    let count = self.read_u16() as usize;
                    let parts: Vec<Value> = self.stack.drain(self.stack.len().saturating_sub(count)..).collect();
                    let s: String = parts.iter().map(|v| v.to_string()).collect();
                    self.push(Value::Str(s));
                }

                Op::ToInt => { let v = self.pop(); self.push(match v { Value::Int(n) => Value::Int(n), Value::Float(f) => Value::Int(f as i64), Value::Str(s) => Value::Int(s.trim().parse().unwrap_or(0)), Value::Bool(b) => Value::Int(if b { 1 } else { 0 }), _ => Value::Int(0) }); }
                Op::ToFloat => { let v = self.pop(); self.push(match v { Value::Float(f) => Value::Float(f), Value::Int(n) => Value::Float(n as f64), Value::Str(s) => Value::Float(s.trim().parse().unwrap_or(0.0)), _ => Value::Float(0.0) }); }
                Op::ToStr => { let v = self.pop(); self.push(Value::Str(v.to_string())); }
                Op::ToBool => { let v = self.pop(); let b = self.truthy(&v); self.push(Value::Bool(b)); }
                Op::TypeOf => { let v = self.pop(); let t = match v { Value::Int(_) => "int", Value::Float(_) => "float", Value::Str(_) => "str", Value::Bool(_) => "bool", Value::Array(_) => "array", Value::Tuple(_) => "tuple", Value::Struct(_, _) => "struct", Value::Lambda(_, _) => "lambda", Value::Null => "null", Value::EnumVariant(_, _, _) => "enum" }; self.push(Value::Str(t.into())); }
                Op::IsNull => { let v = self.pop(); self.push(Value::Bool(matches!(v, Value::Null))); }
                Op::NotNull => { let v = self.pop(); self.push(Value::Bool(!matches!(v, Value::Null))); }

                Op::Print => { let v = self.pop(); print!("{}", v); io::stdout().flush().ok(); self.push(Value::Null); }
                Op::PrintLn => { let v = self.pop(); println!("{}", v); self.push(Value::Null); }
                Op::Input => { let stdin = io::stdin(); let mut line = String::new(); stdin.lock().read_line(&mut line).ok(); self.push(Value::Str(line.trim_end().to_string())); }
                Op::Assert => { let v = self.pop(); if !self.truthy(&v) { return Err("Assertion failed".into()); } self.push(Value::Null); }
                Op::Panic => { let v = self.pop(); return Err(format!("panic: {}", v)); }
                Op::Halt => break,
                _ => return Err(format!("vm: unimplemented op {:?}", op))
            }
        }
        Ok(())
    }

    fn chunk_return(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn truthy(&self, v: &Value) -> bool {
        match v { Value::Bool(b) => *b, Value::Int(n) => *n != 0, Value::Null => false, Value::Str(s) => !s.is_empty(), Value::Array(a) => !a.is_empty(), _ => true }
    }

    fn binop(&self, l: Value, op: &str, r: Value) -> Result<Value, String> {
        match (&l, &r) {
            (Value::Int(a), Value::Int(b)) => match op {
                "+" => Ok(Value::Int(a.wrapping_add(*b))),
                "-" => Ok(Value::Int(a.wrapping_sub(*b))),
                "*" => Ok(Value::Int(a.wrapping_mul(*b))),
                "/" => if *b == 0 { Err("division by zero".into()) } else { Ok(Value::Int(a / b)) },
                "%" => if *b == 0 { Err("division by zero".into()) } else { Ok(Value::Int(a % b)) },
                "**" => Ok(Value::Float((*a as f64).powf(*b as f64))),
                "==" => Ok(Value::Bool(a == b)), "!=" => Ok(Value::Bool(a != b)),
                "<" => Ok(Value::Bool(a < b)), ">" => Ok(Value::Bool(a > b)),
                "<=" => Ok(Value::Bool(a <= b)), ">=" => Ok(Value::Bool(a >= b)),
                _ => Err(format!("unknown op: {}", op))
            },
            (Value::Float(a), Value::Float(b)) => match op {
                "+" => Ok(Value::Float(a + b)), "-" => Ok(Value::Float(a - b)),
                "*" => Ok(Value::Float(a * b)), "/" => Ok(Value::Float(a / b)),
                "**" => Ok(Value::Float(a.powf(*b))),
                "==" => Ok(Value::Bool(a == b)), "!=" => Ok(Value::Bool(a != b)),
                "<" => Ok(Value::Bool(a < b)), ">" => Ok(Value::Bool(a > b)),
                "<=" => Ok(Value::Bool(a <= b)), ">=" => Ok(Value::Bool(a >= b)),
                _ => Err(format!("unknown op: {}", op))
            },
            (Value::Int(a), Value::Float(b)) => self.binop(Value::Float(*a as f64), op, Value::Float(*b)),
            (Value::Float(a), Value::Int(b)) => self.binop(Value::Float(*a), op, Value::Float(*b as f64)),
            (Value::Str(a), _) => match op {
                "+" => Ok(Value::Str(a.clone() + &r.to_string())),
                "==" => Ok(Value::Bool(a.as_str() == r.to_string().as_str())),
                "!=" => Ok(Value::Bool(a.as_str() != r.to_string().as_str())),
                _ => Err(format!("str op: {}", op))
            },
            _ => Err(format!("type mismatch: {} {} {}", l, op, r))
        }
    }

    fn call_native(&mut self, name: &str, args: Vec<Value>) -> Result<Value, String> {
        if name.contains('.') {
            let p: Vec<&str> = name.splitn(2, '.').collect();
            return stdlib::call(p[0], p[1], args);
        }
        match name {
            "print" => {
                let msg = args.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ");
                match msg.as_str() {
                    "hello world" | "Hello, World!" => println!("{}  \x1b[90m<-- your first TSTNT program!\x1b[0m", msg),
                    "42" => println!("42  \x1b[90m<-- the answer to everything\x1b[0m"),
                    _ => println!("{}", msg),
                }
                Ok(Value::Null)
            }
            "print_no_ln" => { print!("{}", args.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ")); io::stdout().flush().ok(); Ok(Value::Null) }
            "len" => match args.first() { Some(Value::Array(a)) => Ok(Value::Int(a.len() as i64)), Some(Value::Str(s)) => Ok(Value::Int(s.chars().count() as i64)), _ => Ok(Value::Int(0)) }
            "str" => Ok(Value::Str(args.first().map(|v| v.to_string()).unwrap_or_default())),
            "int" => match args.first() { Some(Value::Float(f)) => Ok(Value::Int(*f as i64)), Some(Value::Str(s)) => Ok(Value::Int(s.trim().parse().unwrap_or(0))), Some(Value::Int(n)) => Ok(Value::Int(*n)), Some(Value::Bool(b)) => Ok(Value::Int(if *b { 1 } else { 0 })), _ => Ok(Value::Int(0)) }
            "float" => match args.first() { Some(Value::Int(n)) => Ok(Value::Float(*n as f64)), Some(Value::Str(s)) => Ok(Value::Float(s.trim().parse().unwrap_or(0.0))), Some(Value::Float(f)) => Ok(Value::Float(*f)), _ => Ok(Value::Float(0.0)) }
            "bool" => Ok(Value::Bool(args.first().map(|v| self.truthy(v)).unwrap_or(false))),
            "type_of" => match args.first() {
                Some(Value::Int(_)) => Ok(Value::Str("int".into())), Some(Value::Float(_)) => Ok(Value::Str("float".into())),
                Some(Value::Str(_)) => Ok(Value::Str("str".into())), Some(Value::Bool(_)) => Ok(Value::Str("bool".into())),
                Some(Value::Array(_)) => Ok(Value::Str("array".into())), Some(Value::Null) | None => Ok(Value::Str("null".into())),
                Some(Value::EnumVariant(en, vn, _)) => Ok(Value::Str(format!("{}::{}", en, vn))),
                _ => Ok(Value::Str("any".into()))
            }
            "is_null" => Ok(Value::Bool(matches!(args.first(), Some(Value::Null) | None))),
            "not_null" => Ok(Value::Bool(!matches!(args.first(), Some(Value::Null) | None))),
            "default" => match (args.first(), args.get(1)) { (Some(Value::Null), Some(fb)) | (None, Some(fb)) => Ok(fb.clone()), (Some(v), _) => Ok(v.clone()), _ => Ok(Value::Null) }
            "coalesce" => { for a in &args { if !matches!(a, Value::Null) { return Ok(a.clone()); } } Ok(Value::Null) }
            "tap" => { let v = args.into_iter().next().unwrap_or(Value::Null); println!("[tap] {}", v); Ok(v) }
            "range" => match (args.first(), args.get(1)) { (Some(Value::Int(s)), Some(Value::Int(e))) => Ok(Value::Array((*s..*e).map(Value::Int).collect())), _ => Err("range: (int,int)".into()) }
            "assert" => { let ok = args.first().map(|v| self.truthy(v)).unwrap_or(false); if !ok { return Err(args.get(1).map(|v| v.to_string()).unwrap_or_else(|| "Assertion failed".into())); } Ok(Value::Null) }
            "assert_eq" => match (args.first(), args.get(1)) { (Some(a), Some(b)) => if a.to_string() == b.to_string() { Ok(Value::Null) } else { Err(format!("assert_eq: {} != {}", a, b)) } _ => Err("assert_eq".into()) }
            "assert_ne" => match (args.first(), args.get(1)) { (Some(a), Some(b)) => if a.to_string() != b.to_string() { Ok(Value::Null) } else { Err(format!("assert_ne: {} == {}", a, b)) } _ => Err("assert_ne".into()) }
            "panic" => Err(format!("panic: {}", args.first().map(|v| v.to_string()).unwrap_or_else(|| "panic".into()))),
            "keys" => match args.first() { Some(Value::Struct(_, f)) => Ok(Value::Array(f.keys().map(|k| Value::Str(k.clone())).collect())), _ => Err("keys: struct".into()) }
            "values" => match args.first() { Some(Value::Struct(_, f)) => Ok(Value::Array(f.values().cloned().collect())), _ => Err("values: struct".into()) }
            "max" => match args.first() { Some(Value::Array(a)) if !a.is_empty() => { let mut m = a[0].clone(); for v in a.iter().skip(1) { if let (Value::Int(a), Value::Int(b)) = (&m, v) { if b > a { m = v.clone(); } } } Ok(m) } _ => Err("max: array".into()) }
            "min" => match args.first() { Some(Value::Array(a)) if !a.is_empty() => { let mut m = a[0].clone(); for v in a.iter().skip(1) { if let (Value::Int(a), Value::Int(b)) = (&m, v) { if b < a { m = v.clone(); } } } Ok(m) } _ => Err("min: array".into()) }
            "sum" => match args.first() { Some(Value::Array(a)) => { let mut s = 0i64; for v in a { if let Value::Int(n) = v { s += n; } } Ok(Value::Int(s)) } _ => Err("sum: array".into()) }
            "abs" => match args.first() { Some(Value::Int(n)) => Ok(Value::Int(n.abs())), Some(Value::Float(f)) => Ok(Value::Float(f.abs())), _ => Err("abs: number".into()) }
            "clamp" => match (args.first(), args.get(1), args.get(2)) { (Some(Value::Int(v)), Some(Value::Int(lo)), Some(Value::Int(hi))) => Ok(Value::Int((*v).max(*lo).min(*hi))), _ => Err("clamp".into()) }
            "repeat_str" => match (args.first(), args.get(1)) { (Some(Value::Str(s)), Some(Value::Int(n))) => Ok(Value::Str(s.repeat(*n as usize))), _ => Err("repeat_str".into()) }
            "sprintf" => { let t = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("sprintf".into()) }; let mut r = t; for a in args.iter().skip(1) { r = r.replacen("{}", &a.to_string(), 1); } Ok(Value::Str(r)) }
            "find" => match (args.first(), args.get(1)) {
                (Some(Value::Array(arr)), Some(Value::Lambda(params, body))) => {
                    let arr = arr.clone(); let _params = params.clone(); let _body = body.clone();
                    for _item in arr {
                        if let Value::Lambda(ref _p, ref _b) = args[1] { }
                    }
                    Ok(Value::Null)
                }
                _ => Ok(Value::Null)
            }
            "sort" => match args.first() { Some(Value::Array(a)) => { let mut v = a.clone(); v.sort_by(|a, b| a.to_string().cmp(&b.to_string())); Ok(Value::Array(v)) } _ => Err("sort".into()) }
            "unique" => match args.first() { Some(Value::Array(a)) => { let mut seen = std::collections::HashSet::new(); Ok(Value::Array(a.iter().filter(|v| seen.insert(v.to_string())).cloned().collect())) } _ => Err("unique".into()) }
            "zip" => match (args.first(), args.get(1)) { (Some(Value::Array(a)), Some(Value::Array(b))) => Ok(Value::Array(a.iter().zip(b.iter()).map(|(x, y)| Value::Tuple(vec![x.clone(), y.clone()])).collect())), _ => Err("zip".into()) }
            "flatten" => match args.first() { Some(Value::Array(a)) => { let mut r = Vec::new(); for v in a { match v { Value::Array(inner) => r.extend(inner.clone()), x => r.push(x.clone()) } } Ok(Value::Array(r)) } _ => Err("flatten".into()) }
            "take" => match (args.first(), args.get(1)) { (Some(Value::Array(a)), Some(Value::Int(n))) => Ok(Value::Array(a.iter().take(*n as usize).cloned().collect())), _ => Err("take".into()) }
            "drop" => match (args.first(), args.get(1)) { (Some(Value::Array(a)), Some(Value::Int(n))) => Ok(Value::Array(a.iter().skip(*n as usize).cloned().collect())), _ => Err("drop".into()) }
            "chunks" => match (args.first(), args.get(1)) { (Some(Value::Array(a)), Some(Value::Int(n))) => Ok(Value::Array(a.chunks(*n as usize).map(|c| Value::Array(c.to_vec())).collect())), _ => Err("chunks".into()) }
            "concat" => { let mut r = Vec::new(); for a in &args { match a { Value::Array(v) => r.extend(v.clone()), v => r.push(v.clone()) } } Ok(Value::Array(r)) }
            "reverse" => match args.first() { Some(Value::Array(a)) => { let mut v = a.clone(); v.reverse(); Ok(Value::Array(v)) } Some(Value::Str(s)) => Ok(Value::Str(s.chars().rev().collect())), _ => Err("reverse".into()) }
            "index_of" => match (args.first(), args.get(1)) { (Some(Value::Array(a)), Some(v)) => Ok(a.iter().position(|x| x.to_string() == v.to_string()).map(|i| Value::Int(i as i64)).unwrap_or(Value::Int(-1))), _ => Err("index_of".into()) }
            "slice" => match args.first() { Some(Value::Array(a)) => { let from = match args.get(1) { Some(Value::Int(n)) => *n as usize, _ => 0 }; let to = match args.get(2) { Some(Value::Int(n)) => *n as usize, _ => a.len() }; Ok(Value::Array(a[from.min(a.len())..to.min(a.len())].to_vec())) } _ => Err("slice".into()) }
            "input" => { let mut s = String::new(); if let Some(p) = args.first() { print!("{}", p); io::stdout().flush().ok(); } io::stdin().read_line(&mut s).ok(); Ok(Value::Str(s.trim().to_string())) }
            _ => Err(format!("vm: unknown function: {}", name))
        }
    }
}



pub fn run_from_ast(ast: &[Node]) -> Result<(), String> {
    let mut cg = Codegen::new();
    cg.compile(ast).map_err(|e| format!("compile: {}", e))?;
    let main_chunk = cg.chunk;
    let funcs = cg.funcs;
    let mut vm = VM::new(main_chunk, funcs);
    vm.run().map_err(|e| format!("vm: {}", e))
}

pub fn run_tst_file(path: &str) -> Result<(), String> {
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    let chunk = Chunk::deserialize(&data)?;
    let mut vm = VM::new(chunk, vec![]);
    vm.run()
}
