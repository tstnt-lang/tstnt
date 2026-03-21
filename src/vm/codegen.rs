use std::collections::{HashMap, HashSet};
use crate::parser::{Node, MatchPattern, InterpPart};
use crate::vm::chunk::{Chunk, Const};
use crate::vm::opcode::Op;

pub struct FuncProto {
    pub name: String,
    pub params: Vec<String>,
    pub chunk: Chunk,
    pub is_cached: bool,
    pub is_timed: bool,
}

struct Scope {
    locals: HashMap<String, u16>,
    count: u16,
}

impl Scope {
    fn new() -> Self { Scope { locals: HashMap::new(), count: 0 } }
    fn def(&mut self, name: &str) -> u16 {
        if let Some(&i) = self.locals.get(name) { return i; }
        let i = self.count; self.locals.insert(name.to_string(), i); self.count += 1; i
    }
    fn get(&self, name: &str) -> Option<u16> { self.locals.get(name).copied() }
}

pub struct Codegen {
    pub chunk: Chunk,
    pub funcs: Vec<FuncProto>,
    pub known_funcs: HashSet<String>,
    cached_funcs: HashSet<String>,
    timed_funcs: HashSet<String>,
    scopes: Vec<Scope>,
    globals: HashMap<String, String>,
    global_indices: HashMap<String, u16>,
    global_count: u16,
    break_patches: Vec<Vec<usize>>,
    continue_patches: Vec<Vec<usize>>,
    loop_starts: Vec<usize>,
    try_stack: Vec<usize>,
    pub line: usize,
}

impl Codegen {
    pub fn new() -> Self {
        Codegen {
            chunk: Chunk::new(),
            funcs: Vec::new(),
            known_funcs: HashSet::new(),
            cached_funcs: HashSet::new(),
            timed_funcs: HashSet::new(),
            scopes: Vec::new(),
            globals: HashMap::new(),
            global_indices: HashMap::new(),
            global_count: 0,
            break_patches: Vec::new(),
            continue_patches: Vec::new(),
            loop_starts: Vec::new(),
            try_stack: Vec::new(),
            line: 1,
        }
    }

    pub fn compile(&mut self, ast: &[Node]) -> Result<(), String> {
        for node in ast {
            match node {
                Node::FuncDef { name, .. } => { self.known_funcs.insert(name.clone()); }
                Node::Decorator { name, target } => {
                    if let Node::FuncDef { name: fname, .. } = target.as_ref() {
                        match name.as_str() {
                            "cache" => { self.cached_funcs.insert(fname.clone()); self.known_funcs.insert(fname.clone()); }
                            "timer" => { self.timed_funcs.insert(fname.clone()); self.known_funcs.insert(fname.clone()); }
                            _ => { self.known_funcs.insert(fname.clone()); }
                        }
                    }
                }
                _ => {}
            }
        }
        for node in ast {
            match node {
                Node::FuncDef { name, params, body, .. } => {
                    self.compile_func(name, params, body, false, false)?;
                }
                Node::Decorator { name, target } => {
                    if let Node::FuncDef { name: fname, params, body, .. } = target.as_ref() {
                        let is_cached = name == "cache";
                        let is_timed = name == "timer";
                        self.compile_func(fname, params, body, is_cached, is_timed)?;
                    }
                }
                _ => {}
            }
        }
        let main_idx = self.chunk.add_const(Const::Str("main".into()));
        self.chunk.write_op(Op::Call, 0);
        self.chunk.write_u16(main_idx, 0);
        self.chunk.write(0, 0);
        self.chunk.write_op(Op::Halt, 0);
        Ok(())
    }

    fn compile_func(&mut self, name: &str, params: &[(String, String)], body: &[Node], is_cached: bool, is_timed: bool) -> Result<(), String> {
        let mut sub = Codegen::new();
        sub.known_funcs = self.known_funcs.clone();
        sub.cached_funcs = self.cached_funcs.clone();
        sub.timed_funcs = self.timed_funcs.clone();
        sub.global_indices = self.global_indices.clone();
        sub.global_count = self.global_count;
        sub.scopes.push(Scope::new());
        for (pname, _) in params {
            sub.scopes.last_mut().unwrap().def(pname);
        }
        if is_cached {
            let cache_key_idx = sub.chunk.add_const(Const::Str(name.to_string()));
            sub.chunk.write_op(Op::CacheCheck, sub.line);
            sub.chunk.write_u16(cache_key_idx, sub.line);
            sub.chunk.write(params.len() as u8, sub.line);
            let _hit_jump = sub.chunk.len();
            sub.chunk.write_u16(0, sub.line);
        }
        if is_timed {
            sub.chunk.write_op(Op::TimerStart, sub.line);
        }
        for node in body {
            sub.compile_node(node)?;
        }
        if is_timed {
            let name_idx = sub.chunk.add_const(Const::Str(name.to_string()));
            sub.chunk.write_op(Op::TimerEnd, sub.line);
            sub.chunk.write_u16(name_idx, sub.line);
        }
        sub.chunk.write_op(Op::PushNull, sub.line);
        sub.chunk.write_op(Op::Return, sub.line);
        self.global_indices = sub.global_indices;
        self.global_count = sub.global_count;
        self.funcs.push(FuncProto {
            name: name.into(),
            params: params.iter().map(|(p, _)| p.clone()).collect(),
            chunk: sub.chunk,
            is_cached,
            is_timed,
        });
        Ok(())
    }

    fn lookup_local(&self, name: &str) -> Option<u16> {
        for scope in self.scopes.iter().rev() {
            if let Some(idx) = scope.get(name) { return Some(idx); }
        }
        None
    }

    fn def_local(&mut self, name: &str) -> u16 {
        if let Some(scope) = self.scopes.last_mut() { scope.def(name) } else { 0 }
    }

    fn emit_load(&mut self, name: &str) {
        if let Some(idx) = self.lookup_local(name) {
            self.chunk.write_op(Op::LoadLocal, self.line);
            self.chunk.write_u16(idx, self.line);
        } else {
            let idx = self.get_or_def_global(name);
            self.chunk.write_op(Op::LoadGlobal, self.line);
            self.chunk.write_u16(idx, self.line);
        }
    }

    fn get_or_def_global(&mut self, name: &str) -> u16 {
        if let Some(&i) = self.global_indices.get(name) { return i; }
        let i = self.global_count;
        self.global_indices.insert(name.to_string(), i);
        self.global_count += 1;
        i
    }
    fn emit_store(&mut self, name: &str) {
        if let Some(idx) = self.lookup_local(name) {
            self.chunk.write_op(Op::StoreLocal, self.line);
            self.chunk.write_u16(idx, self.line);
        } else if !self.scopes.is_empty() {
            let idx = self.def_local(name);
            self.chunk.write_op(Op::StoreLocal, self.line);
            self.chunk.write_u16(idx, self.line);
        } else {
            let idx = self.get_or_def_global(name);
            self.chunk.write_op(Op::StoreGlobal, self.line);
            self.chunk.write_u16(idx, self.line);
        }
    }

    pub fn compile_node(&mut self, node: &Node) -> Result<(), String> {
        match node {
            Node::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    let ni = *n as i64;
                    if ni >= -32768 && ni <= 32767 {
                        self.chunk.write_op(Op::PushIntInline, self.line);
                        self.chunk.write_u16((ni + 32768) as u16, self.line);
                    } else {
                        let idx = self.chunk.add_const(Const::Int(ni));
                        self.chunk.write_op(Op::PushInt, self.line);
                        self.chunk.write_u16(idx, self.line);
                    }
                } else {
                    let idx = self.chunk.add_const(Const::Float(*n));
                    self.chunk.write_op(Op::PushFloat, self.line);
                    self.chunk.write_u16(idx, self.line);
                }
            }
            Node::Str(s) => {
                let idx = self.chunk.add_const(Const::Str(s.clone()));
                self.chunk.write_op(Op::PushStr, self.line);
                self.chunk.write_u16(idx, self.line);
            }
            Node::Interp(parts) => {
                let mut count = 0u16;
                for part in parts {
                    match part {
                        InterpPart::Literal(s) => {
                            let idx = self.chunk.add_const(Const::Str(s.clone()));
                            self.chunk.write_op(Op::PushStr, self.line);
                            self.chunk.write_u16(idx, self.line);
                        }
                        InterpPart::Expr(e) => {
                            self.compile_node(e)?;
                            self.chunk.write_op(Op::ToStr, self.line);
                        }
                    }
                    count += 1;
                }
                self.chunk.write_op(Op::Interp, self.line);
                self.chunk.write_u16(count, self.line);
            }
            Node::Bool(b) => {
                self.chunk.write_op(Op::PushBool, self.line);
                self.chunk.write(if *b { 1 } else { 0 }, self.line);
            }
            Node::Null => { self.chunk.write_op(Op::PushNull, self.line); }
            Node::Ident(name) => { self.emit_load(name); }

            Node::Assign { name, value, .. } => {
                self.compile_node(value)?;
                self.emit_store(name);
            }
            Node::MultiAssign { names, value } => {
                self.compile_node(value)?;
                let tmp_idx = self.get_or_def_global("__multi__");
                self.chunk.write_op(Op::StoreGlobal, self.line);
                self.chunk.write_u16(tmp_idx, self.line);
                for (i, name) in names.iter().enumerate() {
                    self.chunk.write_op(Op::LoadGlobal, self.line);
                    self.chunk.write_u16(tmp_idx, self.line);
                    let idx_c = self.chunk.add_const(Const::Int(i as i64));
                    self.chunk.write_op(Op::PushInt, self.line);
                    self.chunk.write_u16(idx_c, self.line);
                    self.chunk.write_op(Op::ArrayGet, self.line);
                    self.emit_store(name);
                }
            }
            Node::AssignOp { name, op, value } => {
                self.emit_load(name);
                self.compile_node(value)?;
                let opcode = match op.as_str() {
                    "+=" => Op::Add, "-=" => Op::Sub, "*=" => Op::Mul,
                    "/=" => Op::Div, "%=" => Op::Mod, _ => Op::Add
                };
                self.chunk.write_op(opcode, self.line);
                self.emit_store(name);
            }

            Node::BinOp(l, op, r) => {
                self.compile_node(l)?;
                self.compile_node(r)?;
                let opcode = match op.as_str() {
                    "+" => Op::Add, "-" => Op::Sub, "*" => Op::Mul,
                    "/" => Op::Div, "%" => Op::Mod, "**" => Op::Pow,
                    "==" => Op::Eq, "!=" => Op::Neq,
                    "<" => Op::Lt, ">" => Op::Gt, "<=" => Op::LtEq, ">=" => Op::GtEq,
                    "&&" => Op::And, "||" => Op::Or,
                    _ => return Err(format!("vm: unknown op: {}", op))
                };
                self.chunk.write_op(opcode, self.line);
            }
            Node::Unary(op, expr) => {
                self.compile_node(expr)?;
                match op.as_str() {
                    "-" => self.chunk.write_op(Op::Neg, self.line),
                    "!" => self.chunk.write_op(Op::Not, self.line),
                    _ => {}
                }
            }
            Node::Ternary(cond, then, else_) => {
                self.compile_node(cond)?;
                self.chunk.write_op(Op::JumpIfNot, self.line);
                let jp = self.chunk.len(); self.chunk.write_u16(0, self.line);
                self.compile_node(then)?;
                self.chunk.write_op(Op::Jump, self.line);
                let ej = self.chunk.len(); self.chunk.write_u16(0, self.line);
                self.chunk.patch_u16(jp, self.chunk.len() as u16);
                self.compile_node(else_)?;
                self.chunk.patch_u16(ej, self.chunk.len() as u16);
            }

            Node::Return(val) => {
                self.compile_node(val)?;
                self.chunk.write_op(Op::Return, self.line);
            }
            Node::Throw(val) => {
                self.compile_node(val)?;
                self.chunk.write_op(Op::Throw, self.line);
            }
            Node::Break => {
                self.chunk.write_op(Op::Break, self.line);
                let p = self.chunk.len(); self.chunk.write_u16(0, self.line);
                if let Some(patches) = self.break_patches.last_mut() { patches.push(p); }
            }
            Node::Continue => {
                self.chunk.write_op(Op::Continue, self.line);
                let p = self.chunk.len(); self.chunk.write_u16(0, self.line);
                if let Some(patches) = self.continue_patches.last_mut() { patches.push(p); }
            }

            Node::If { cond, body, else_body } => {
                self.compile_node(cond)?;
                self.chunk.write_op(Op::JumpIfNot, self.line);
                let jp = self.chunk.len(); self.chunk.write_u16(0, self.line);
                self.scopes.push(Scope::new());
                for s in body { self.compile_node(s)?; }
                self.scopes.pop();
                if let Some(eb) = else_body {
                    self.chunk.write_op(Op::Jump, self.line);
                    let ej = self.chunk.len(); self.chunk.write_u16(0, self.line);
                    self.chunk.patch_u16(jp, self.chunk.len() as u16);
                    self.scopes.push(Scope::new());
                    for s in eb { self.compile_node(s)?; }
                    self.scopes.pop();
                    self.chunk.patch_u16(ej, self.chunk.len() as u16);
                } else {
                    self.chunk.patch_u16(jp, self.chunk.len() as u16);
                }
            }

            Node::While { cond, body } => {
                let loop_start = self.chunk.len() as u16;
                self.loop_starts.push(loop_start as usize);
                self.break_patches.push(vec![]);
                self.continue_patches.push(vec![]);
                self.compile_node(cond)?;
                self.chunk.write_op(Op::JumpIfNot, self.line);
                let exit = self.chunk.len(); self.chunk.write_u16(0, self.line);
                self.scopes.push(Scope::new());
                for s in body { self.compile_node(s)?; }
                self.scopes.pop();
                self.chunk.write_op(Op::Jump, self.line);
                self.chunk.write_u16(loop_start, self.line);
                let after = self.chunk.len() as u16;
                self.chunk.patch_u16(exit, after);
                self.loop_starts.pop();
                for p in self.break_patches.pop().unwrap_or_default() { self.chunk.patch_u16(p, after); }
                for p in self.continue_patches.pop().unwrap_or_default() { self.chunk.patch_u16(p, loop_start); }
            }

            Node::Loop { var, start, end, body } => {
                self.scopes.push(Scope::new());
                self.compile_node(start)?;
                let var_slot = self.def_local(var);
                self.chunk.write_op(Op::StoreLocal, self.line);
                self.chunk.write_u16(var_slot, self.line);
                let loop_start = self.chunk.len();
                self.loop_starts.push(loop_start);
                self.break_patches.push(vec![]);
                self.continue_patches.push(vec![]);
                self.chunk.write_op(Op::LoadLocal, self.line);
                self.chunk.write_u16(var_slot, self.line);
                self.compile_node(end)?;
                self.chunk.write_op(Op::Lt, self.line);
                self.chunk.write_op(Op::JumpIfNot, self.line);
                let exit = self.chunk.len(); self.chunk.write_u16(0, self.line);
                for s in body { self.compile_node(s)?; }
                let cont_target = self.chunk.len() as u16;
                self.chunk.write_op(Op::LoadLocal, self.line);
                self.chunk.write_u16(var_slot, self.line);
                self.chunk.write_op(Op::PushIntInline, self.line);
                self.chunk.write_u16(1 + 32768, self.line);
                self.chunk.write_op(Op::Add, self.line);
                self.chunk.write_op(Op::StoreLocal, self.line);
                self.chunk.write_u16(var_slot, self.line);
                self.chunk.write_op(Op::Jump, self.line);
                self.chunk.write_u16(loop_start as u16, self.line);
                let after = self.chunk.len() as u16;
                self.chunk.patch_u16(exit, after);
                self.scopes.pop();
                self.loop_starts.pop();
                for p in self.break_patches.pop().unwrap_or_default() { self.chunk.patch_u16(p, after); }
                for p in self.continue_patches.pop().unwrap_or_default() { self.chunk.patch_u16(p, cont_target as u16); }
            }

            Node::LoopIn { var, iter, body } => {
                self.compile_node(iter)?;
                let iter_key = self.get_or_def_global(&format!("__iter_{}", var));
                self.chunk.write_op(Op::StoreGlobal, self.line);
                self.chunk.write_u16(iter_key, self.line);
                let idx_key = self.get_or_def_global(&format!("__idx_{}", var));
                self.chunk.write_op(Op::PushIntInline, self.line);
                self.chunk.write_u16(32768, self.line);
                self.chunk.write_op(Op::StoreGlobal, self.line);
                self.chunk.write_u16(idx_key, self.line);
                let loop_start = self.chunk.len();
                self.loop_starts.push(loop_start);
                self.break_patches.push(vec![]);
                self.continue_patches.push(vec![]);
                self.chunk.write_op(Op::LoadGlobal, self.line);
                self.chunk.write_u16(idx_key, self.line);
                self.chunk.write_op(Op::LoadGlobal, self.line);
                self.chunk.write_u16(iter_key, self.line);
                self.chunk.write_op(Op::ArrayLen, self.line);
                self.chunk.write_op(Op::Lt, self.line);
                self.chunk.write_op(Op::JumpIfNot, self.line);
                let exit = self.chunk.len(); self.chunk.write_u16(0, self.line);
                self.chunk.write_op(Op::LoadGlobal, self.line);
                self.chunk.write_u16(iter_key, self.line);
                self.chunk.write_op(Op::LoadGlobal, self.line);
                self.chunk.write_u16(idx_key, self.line);
                self.chunk.write_op(Op::ArrayGet, self.line);
                self.scopes.push(Scope::new());
                let var_slot = self.def_local(var);
                self.chunk.write_op(Op::StoreLocal, self.line);
                self.chunk.write_u16(var_slot, self.line);
                for s in body { self.compile_node(s)?; }
                self.scopes.pop();
                let cont_target = self.chunk.len() as u16;
                self.chunk.write_op(Op::LoadGlobal, self.line);
                self.chunk.write_u16(idx_key, self.line);
                let one = self.chunk.add_const(Const::Int(1));
                self.chunk.write_op(Op::PushInt, self.line);
                self.chunk.write_u16(one, self.line);
                self.chunk.write_op(Op::Add, self.line);
                self.chunk.write_op(Op::StoreGlobal, self.line);
                self.chunk.write_u16(idx_key, self.line);
                self.chunk.write_op(Op::Jump, self.line);
                self.chunk.write_u16(loop_start as u16, self.line);
                let after = self.chunk.len() as u16;
                self.chunk.patch_u16(exit, after);
                self.loop_starts.pop();
                for p in self.break_patches.pop().unwrap_or_default() { self.chunk.patch_u16(p, after); }
                for p in self.continue_patches.pop().unwrap_or_default() { self.chunk.patch_u16(p, cont_target as u16); }
            }

            Node::LoopEnum { idx, var, iter, body } => {
                self.compile_node(iter)?;
                let iter_key = self.chunk.add_const(Const::Str(format!("__eiter_{}", var)));
                self.chunk.write_op(Op::StoreGlobal, self.line);
                self.chunk.write_u16(iter_key, self.line);
                let idx_key = self.chunk.add_const(Const::Str(format!("__eidx_{}", var)));
                let zero = self.chunk.add_const(Const::Int(0));
                self.chunk.write_op(Op::PushInt, self.line); self.chunk.write_u16(zero, self.line);
                self.chunk.write_op(Op::StoreGlobal, self.line); self.chunk.write_u16(idx_key, self.line);
                let loop_start = self.chunk.len();
                self.loop_starts.push(loop_start);
                self.break_patches.push(vec![]);
                self.continue_patches.push(vec![]);
                self.chunk.write_op(Op::LoadGlobal, self.line); self.chunk.write_u16(idx_key, self.line);
                self.chunk.write_op(Op::LoadGlobal, self.line); self.chunk.write_u16(iter_key, self.line);
                self.chunk.write_op(Op::ArrayLen, self.line);
                self.chunk.write_op(Op::Lt, self.line);
                self.chunk.write_op(Op::JumpIfNot, self.line);
                let exit = self.chunk.len(); self.chunk.write_u16(0, self.line);
                self.chunk.write_op(Op::LoadGlobal, self.line); self.chunk.write_u16(iter_key, self.line);
                self.chunk.write_op(Op::LoadGlobal, self.line); self.chunk.write_u16(idx_key, self.line);
                self.chunk.write_op(Op::ArrayGet, self.line);
                self.scopes.push(Scope::new());
                let var_slot = self.def_local(var);
                self.chunk.write_op(Op::StoreLocal, self.line); self.chunk.write_u16(var_slot, self.line);
                let idx_slot = self.def_local(idx);
                self.chunk.write_op(Op::LoadGlobal, self.line); self.chunk.write_u16(idx_key, self.line);
                self.chunk.write_op(Op::StoreLocal, self.line); self.chunk.write_u16(idx_slot, self.line);
                for s in body { self.compile_node(s)?; }
                self.scopes.pop();
                let cont_target = self.chunk.len() as u16;
                self.chunk.write_op(Op::LoadGlobal, self.line); self.chunk.write_u16(idx_key, self.line);
                let one = self.chunk.add_const(Const::Int(1));
                self.chunk.write_op(Op::PushInt, self.line); self.chunk.write_u16(one, self.line);
                self.chunk.write_op(Op::Add, self.line);
                self.chunk.write_op(Op::StoreGlobal, self.line); self.chunk.write_u16(idx_key, self.line);
                self.chunk.write_op(Op::Jump, self.line); self.chunk.write_u16(loop_start as u16, self.line);
                let after = self.chunk.len() as u16;
                self.chunk.patch_u16(exit, after);
                self.loop_starts.pop();
                for p in self.break_patches.pop().unwrap_or_default() { self.chunk.patch_u16(p, after); }
                for p in self.continue_patches.pop().unwrap_or_default() { self.chunk.patch_u16(p, cont_target as u16); }
            }

            Node::Repeat { count, body } => {
                self.compile_node(count)?;
                let cnt_key = self.get_or_def_global("__repeat_cnt__");
                self.chunk.write_op(Op::StoreGlobal, self.line); self.chunk.write_u16(cnt_key, self.line);
                let i_key = self.get_or_def_global("__repeat_i__");
                let zero = self.chunk.add_const(Const::Int(0));
                self.chunk.write_op(Op::PushInt, self.line); self.chunk.write_u16(zero, self.line);
                self.chunk.write_op(Op::StoreGlobal, self.line); self.chunk.write_u16(i_key, self.line);
                let loop_start = self.chunk.len() as u16;
                self.chunk.write_op(Op::LoadGlobal, self.line); self.chunk.write_u16(i_key, self.line);
                self.chunk.write_op(Op::LoadGlobal, self.line); self.chunk.write_u16(cnt_key, self.line);
                self.chunk.write_op(Op::Lt, self.line);
                self.chunk.write_op(Op::JumpIfNot, self.line);
                let exit = self.chunk.len(); self.chunk.write_u16(0, self.line);
                for s in body { self.compile_node(s)?; }
                self.chunk.write_op(Op::LoadGlobal, self.line); self.chunk.write_u16(i_key, self.line);
                let one = self.chunk.add_const(Const::Int(1));
                self.chunk.write_op(Op::PushInt, self.line); self.chunk.write_u16(one, self.line);
                self.chunk.write_op(Op::Add, self.line);
                self.chunk.write_op(Op::StoreGlobal, self.line); self.chunk.write_u16(i_key, self.line);
                self.chunk.write_op(Op::Jump, self.line); self.chunk.write_u16(loop_start, self.line);
                self.chunk.patch_u16(exit, self.chunk.len() as u16);
            }

            Node::Match { expr, arms } => {
                self.compile_node(expr)?;
                let match_key = self.get_or_def_global("__match__");
                self.chunk.write_op(Op::StoreGlobal, self.line);
                self.chunk.write_u16(match_key, self.line);
                let mut end_jumps = vec![];
                for arm in arms {
                    self.chunk.write_op(Op::LoadGlobal, self.line);
                    self.chunk.write_u16(match_key, self.line);
                    match &arm.pattern {
                        MatchPattern::Literal(lit) => {
                            self.compile_node(lit)?;
                            self.chunk.write_op(Op::Eq, self.line);
                        }
                        MatchPattern::Wildcard => {
                            let _t = self.chunk.add_const(Const::Bool(true));
                            self.chunk.write_op(Op::Pop, self.line);
                            self.chunk.write_op(Op::PushBool, self.line);
                            self.chunk.write(1, self.line);
                        }
                        MatchPattern::Ident(name) => {
                            self.emit_store(name);
                            self.chunk.write_op(Op::PushBool, self.line);
                            self.chunk.write(1, self.line);
                        }
                        MatchPattern::Range(lo, hi) => {
                            self.chunk.write_op(Op::Dup, self.line);
                            self.compile_node(lo)?;
                            self.chunk.write_op(Op::GtEq, self.line);
                            self.chunk.write_op(Op::Swap, self.line);
                            self.compile_node(hi)?;
                            self.chunk.write_op(Op::LtEq, self.line);
                            self.chunk.write_op(Op::And, self.line);
                        }
                    }
                    if let Some(guard) = &arm.guard {
                        self.chunk.write_op(Op::JumpIfNot, self.line);
                        let skip = self.chunk.len(); self.chunk.write_u16(0, self.line);
                        self.compile_node(guard)?;
                        self.chunk.write_op(Op::And, self.line);
                        self.chunk.patch_u16(skip, self.chunk.len() as u16);
                    }
                    self.chunk.write_op(Op::JumpIfNot, self.line);
                    let no_match = self.chunk.len(); self.chunk.write_u16(0, self.line);
                    self.scopes.push(Scope::new());
                    for s in &arm.body { self.compile_node(s)?; }
                    self.scopes.pop();
                    self.chunk.write_op(Op::Jump, self.line);
                    let end_jump = self.chunk.len(); self.chunk.write_u16(0, self.line);
                    end_jumps.push(end_jump);
                    self.chunk.patch_u16(no_match, self.chunk.len() as u16);
                }
                let after = self.chunk.len() as u16;
                for p in end_jumps { self.chunk.patch_u16(p, after); }
            }

            Node::Try { body, catch_var, catch_body } => {
                self.chunk.write_op(Op::EnterTry, self.line);
                let catch_patch = self.chunk.len(); self.chunk.write_u16(0, self.line);
                self.scopes.push(Scope::new());
                for s in body { self.compile_node(s)?; }
                self.scopes.pop();
                self.chunk.write_op(Op::LeaveTry, self.line);
                self.chunk.write_op(Op::Jump, self.line);
                let end_patch = self.chunk.len(); self.chunk.write_u16(0, self.line);
                self.chunk.patch_u16(catch_patch, self.chunk.len() as u16);
                self.scopes.push(Scope::new());
                let cv_slot = self.def_local(catch_var);
                self.chunk.write_op(Op::StoreLocal, self.line);
                self.chunk.write_u16(cv_slot, self.line);
                for s in catch_body { self.compile_node(s)?; }
                self.scopes.pop();
                self.chunk.patch_u16(end_patch, self.chunk.len() as u16);
            }

            Node::Lambda { params, body } => {
                let lambda_name = format!("__lambda_{}", self.funcs.len() + self.chunk.len());
                self.known_funcs.insert(lambda_name.clone());
                let _params_typed: Vec<(String, String)> = params.iter().map(|p| (p.clone(), "any".into())).collect();
                let body_nodes = match body.as_ref() {
                    Node::Array(nodes) => nodes.clone(),
                    other => vec![Node::Return(Box::new(other.clone()))],
                };
                let mut sub = Codegen::new();
                sub.known_funcs = self.known_funcs.clone();
                sub.cached_funcs = self.cached_funcs.clone();
                sub.timed_funcs = self.timed_funcs.clone();
                sub.scopes.push(Scope::new());
                for p in params { sub.scopes.last_mut().unwrap().def(p); }
                for n in &body_nodes { sub.compile_node(n)?; }
                sub.chunk.write_op(Op::PushNull, sub.line);
                sub.chunk.write_op(Op::Return, sub.line);
                self.funcs.push(FuncProto { name: lambda_name.clone(), params: params.clone(), chunk: sub.chunk, is_cached: false, is_timed: false });
                let idx = self.chunk.add_const(Const::Str(lambda_name));
                self.chunk.write_op(Op::MakeLambda, self.line);
                self.chunk.write_u16(idx, self.line);
            }

            Node::FuncCall { name, args } => {
                let filtered: Vec<&Node> = args.iter().filter(|a| !matches!(a, Node::NamedArg { .. })).collect();
                let named: Vec<(&str, &Node)> = args.iter().filter_map(|a| if let Node::NamedArg { name, value } = a { Some((name.as_str(), value.as_ref())) } else { None }).collect();
                for arg in &filtered { self.compile_node(arg)?; }
                for (_, val) in &named { self.compile_node(val)?; }
                let total = filtered.len() + named.len();
                let idx = self.chunk.add_const(Const::Str(name.clone()));
                if self.known_funcs.contains(name.as_str()) {
                    self.chunk.write_op(Op::Call, self.line);
                } else {
                    self.chunk.write_op(Op::CallNative, self.line);
                }
                self.chunk.write_u16(idx, self.line);
                self.chunk.write(total as u8, self.line);
            }

            Node::MethodCall { obj, method, args } => {
                if let Node::Ident(mod_name) = obj.as_ref() {
                    if self.lookup_local(mod_name).is_none() && !self.known_funcs.contains(mod_name.as_str()) {
                        let full = format!("{}.{}", mod_name, method);
                        for arg in args { self.compile_node(arg)?; }
                        let idx = self.chunk.add_const(Const::Str(full));
                        self.chunk.write_op(Op::CallNative, self.line);
                        self.chunk.write_u16(idx, self.line);
                        self.chunk.write(args.len() as u8, self.line);
                        return Ok(());
                    }
                }
                self.compile_node(obj)?;
                for arg in args { self.compile_node(arg)?; }
                let idx = self.chunk.add_const(Const::Str(method.clone()));
                self.chunk.write_op(Op::CallMethod, self.line);
                self.chunk.write_u16(idx, self.line);
                self.chunk.write(args.len() as u8, self.line);
            }

            Node::FieldAccess { obj, field } => {
                self.compile_node(obj)?;
                let idx = self.chunk.add_const(Const::Str(field.clone()));
                self.chunk.write_op(Op::FieldGet, self.line);
                self.chunk.write_u16(idx, self.line);
            }
            Node::OptionalAccess { obj, field } => {
                self.compile_node(obj)?;
                let idx = self.chunk.add_const(Const::Str(field.clone()));
                self.chunk.write_op(Op::OptField, self.line);
                self.chunk.write_u16(idx, self.line);
            }
            Node::OptionalCall { obj, method, args } => {
                self.compile_node(obj)?;
                for arg in args { self.compile_node(arg)?; }
                let idx = self.chunk.add_const(Const::Str(method.clone()));
                self.chunk.write_op(Op::OptCall, self.line);
                self.chunk.write_u16(idx, self.line);
                self.chunk.write(args.len() as u8, self.line);
            }
            Node::Index { obj, idx } => {
                self.compile_node(obj)?;
                self.compile_node(idx)?;
                self.chunk.write_op(Op::ArrayGet, self.line);
            }

            Node::Array(elements) => {
                for el in elements { self.compile_node(el)?; }
                self.chunk.write_op(Op::MakeArray, self.line);
                self.chunk.write_u16(elements.len() as u16, self.line);
            }
            Node::Tuple(elements) => {
                for el in elements { self.compile_node(el)?; }
                self.chunk.write_op(Op::MakeTuple, self.line);
                self.chunk.write_u16(elements.len() as u16, self.line);
            }
            Node::Spread(inner) => {
                self.compile_node(inner)?;
                self.chunk.write_op(Op::Spread, self.line);
            }
            Node::StructInit { name, fields } => {
                for (_, val) in fields { self.compile_node(val)?; }
                let ni = self.chunk.add_const(Const::Str(name.clone()));
                let fi = self.chunk.add_const(Const::Str(fields.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>().join(",")));
                self.chunk.write_op(Op::MakeStruct, self.line);
                self.chunk.write_u16(ni, self.line);
                self.chunk.write_u16(fi, self.line);
                self.chunk.write_u16(fields.len() as u16, self.line);
            }
            Node::EnumVariantInit { enum_name, variant, args } => {
                for arg in args { self.compile_node(arg)?; }
                let idx = self.chunk.add_const(Const::Str(format!("{}::{}", enum_name, variant)));
                self.chunk.write_op(Op::MakeEnum, self.line);
                self.chunk.write_u16(idx, self.line);
                self.chunk.write(args.len() as u8, self.line);
            }
            Node::In { value, container } => {
                self.compile_node(value)?;
                self.compile_node(container)?;
                self.chunk.write_op(Op::In, self.line);
            }
            Node::Ref(inner) => self.compile_node(inner)?,
            Node::NamedArg { value, .. } => self.compile_node(value)?,

            Node::ForIn { var, iter, body } => {
                self.compile_node(iter)?;
                let iter_key = self.get_or_def_global(&format!("__for_{}", var));
                self.chunk.write_op(Op::StoreGlobal, self.line);
                self.chunk.write_u16(iter_key, self.line);
                let idx_key = self.get_or_def_global(&format!("__fori_{}", var));
                self.chunk.write_op(Op::PushIntInline, self.line);
                self.chunk.write_u16(32768, self.line);
                self.chunk.write_op(Op::StoreGlobal, self.line);
                self.chunk.write_u16(idx_key, self.line);
                let loop_start = self.chunk.len();
                self.loop_starts.push(loop_start);
                self.break_patches.push(vec![]);
                self.continue_patches.push(vec![]);
                self.chunk.write_op(Op::LoadGlobal, self.line); self.chunk.write_u16(idx_key, self.line);
                self.chunk.write_op(Op::LoadGlobal, self.line); self.chunk.write_u16(iter_key, self.line);
                self.chunk.write_op(Op::ArrayLen, self.line);
                self.chunk.write_op(Op::Lt, self.line);
                self.chunk.write_op(Op::JumpIfNot, self.line);
                let exit = self.chunk.len(); self.chunk.write_u16(0, self.line);
                self.chunk.write_op(Op::LoadGlobal, self.line); self.chunk.write_u16(iter_key, self.line);
                self.chunk.write_op(Op::LoadGlobal, self.line); self.chunk.write_u16(idx_key, self.line);
                self.chunk.write_op(Op::ArrayGet, self.line);
                self.scopes.push(Scope::new());
                let var_slot = self.def_local(var);
                self.chunk.write_op(Op::StoreLocal, self.line); self.chunk.write_u16(var_slot, self.line);
                for s2 in body { self.compile_node(s2)?; }
                self.scopes.pop();
                let cont = self.chunk.len() as u16;
                self.chunk.write_op(Op::LoadGlobal, self.line); self.chunk.write_u16(idx_key, self.line);
                self.chunk.write_op(Op::PushIntInline, self.line); self.chunk.write_u16(1 + 32768, self.line);
                self.chunk.write_op(Op::Add, self.line);
                self.chunk.write_op(Op::StoreGlobal, self.line); self.chunk.write_u16(idx_key, self.line);
                self.chunk.write_op(Op::Jump, self.line); self.chunk.write_u16(loop_start as u16, self.line);
                let after = self.chunk.len() as u16;
                self.chunk.patch_u16(exit, after);
                self.loop_starts.pop();
                for p in self.break_patches.pop().unwrap_or_default() { self.chunk.patch_u16(p, after); }
                for p in self.continue_patches.pop().unwrap_or_default() { self.chunk.patch_u16(p, cont); }
            }
            Node::Const { name, value, .. } => {
                self.compile_node(value)?;
                self.emit_store(name);
            }
            Node::FuncDef { .. } | Node::StructDef { .. } | Node::ImplBlock { .. } |
            Node::Use(_) | Node::Mod(_) | Node::EnumDef { .. } | Node::TestDef { .. } |
            Node::InterfaceDef { .. } | Node::UnitDef { .. } | Node::Decorator { .. } |
            Node::Breakpoint => {}

            Node::Pipe(nodes) => {
                if nodes.is_empty() { return Ok(()); }
                self.compile_node(&nodes[0])?;
                for n in &nodes[1..] {
                    match n {
                        Node::Ident(name) => {
                            let idx = self.chunk.add_const(Const::Str(name.clone()));
                            if self.known_funcs.contains(name.as_str()) {
                                self.chunk.write_op(Op::Call, self.line);
                            } else {
                                self.chunk.write_op(Op::CallNative, self.line);
                            }
                            self.chunk.write_u16(idx, self.line);
                            self.chunk.write(1, self.line);
                        }
                        other => {
                            self.compile_node(other)?;
                        }
                    }
                }
            }

            Node::Await(inner) => { self.compile_node(inner)?; }
            Node::Spawn(inner) => { self.compile_node(inner)?; }

            _ => return Err(format!("vm: unsupported node {:?}", std::mem::discriminant(node)))
        }
        Ok(())
    }
}
