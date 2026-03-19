use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::parser::{Node, MatchPattern, InterpPart};
use crate::value::Value;
use crate::stdlib;

pub enum Signal { Return(Value), Break, Continue, Throw(String) }

pub struct Interpreter {
    env: Vec<HashMap<String, Value>>,
    pub functions: HashMap<String, (Vec<(String, String)>, Vec<Node>, bool)>,
    pub impls: HashMap<String, HashMap<String, (Vec<(String, String)>, Vec<Node>)>>,
    pub test_results: Vec<(String, bool, String)>,
    pub debug: bool,
    cache_store: HashMap<String, Value>,
    timer_funcs: std::collections::HashSet<String>,
    deprecated_funcs: std::collections::HashSet<String>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter { env: vec![HashMap::new()], functions: HashMap::new(), impls: HashMap::new(), test_results: Vec::new(), debug: false, cache_store: HashMap::new(), timer_funcs: std::collections::HashSet::new(), deprecated_funcs: std::collections::HashSet::new() }
    }
    fn push_scope(&mut self) { self.env.push(HashMap::new()); }
    fn pop_scope(&mut self) { self.env.pop(); }
    fn get(&self, name: &str) -> Option<Value> { for scope in self.env.iter().rev() { if let Some(v) = scope.get(name) { return Some(v.clone()); } } None }
    fn set(&mut self, name: &str, val: Value) {
        for scope in self.env.iter_mut().rev() { if scope.contains_key(name) { scope.insert(name.to_string(), val); return; } }
        if let Some(scope) = self.env.last_mut() { scope.insert(name.to_string(), val); }
    }
    fn set_local(&mut self, name: &str, val: Value) { if let Some(scope) = self.env.last_mut() { scope.insert(name.to_string(), val); } }
    fn truthy(&self, val: &Value) -> bool {
        match val { Value::Bool(b) => *b, Value::Int(n) => *n != 0, Value::Null => false, Value::Str(s) => !s.is_empty(), Value::Array(a) => !a.is_empty(), _ => true }
    }
    pub fn run(&mut self, nodes: &[Node]) -> Result<(), String> {
        for node in nodes {
            match node {
                Node::FuncDef { name, params, body, is_async, .. } => { self.functions.insert(name.clone(), (params.clone(), body.clone(), *is_async)); }
                Node::ImplBlock { target, methods, .. } => {
                    let entry = self.impls.entry(target.clone()).or_default();
                    for m in methods { if let Node::FuncDef { name, params, body, .. } = m { entry.insert(name.clone(), (params.clone(), body.clone())); } }
                }
                Node::Use(path) if path.starts_with("./") || path.starts_with("/") || path.ends_with(".tstnt") => {
                    let filepath = if path.ends_with(".tstnt") { path.clone() } else { format!("{}.tstnt", path) };
                    if let Ok(src) = std::fs::read_to_string(&filepath) {
                        let tokens = crate::lexer::Lexer::new(&src).tokenize();
                        if let Ok(ast) = crate::parser::Parser::new(tokens).parse() {
                            self.run(&ast).ok();
                        }
                    } else { eprintln!("use: file not found: {}", filepath); }
                }
                Node::Decorator { name, target } => {
                    match target.as_ref() {
                        Node::FuncDef { name: fname, params, body, is_async, .. } => {
                            let fname = fname.clone();
                            let params = params.clone();
                            let body = body.clone();
                            let is_async = *is_async;
                            let dname = name.clone();
                            match dname.as_str() {
                                "cache" => {
                                    self.cache_store.insert(format!("__cache_registered_{}__", fname), Value::Bool(true));
                                    self.functions.insert(fname.clone(), (params.clone(), body.clone(), is_async));
                                }
                                "timer" => {
                                    self.timer_funcs.insert(fname.clone());
                                    self.functions.insert(fname.clone(), (params.clone(), body.clone(), is_async));
                                }
                                "deprecated" => {
                                    self.deprecated_funcs.insert(fname.clone());
                                    self.functions.insert(fname.clone(), (params.clone(), body.clone(), is_async));
                                }
                                _ => {
                                    self.functions.insert(fname, (params, body, is_async));
                                }
                            }
                        }
                        _ => { self.run(std::slice::from_ref(target))?; }
                    }
                }
                Node::EnumDef { name, variants } => {
                    for v in variants {
                        let ev = Value::EnumVariant(name.clone(), v.clone(), vec![]);
                        self.set_local(v, ev);
                    }
                }
                Node::Use(pkg) => {
                    // Пакет из ~/.tstnt/packages/<pkg>/main.tstnt
                    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
                    let pkg_path = format!("{}/.tstnt/packages/{}/main.tstnt", home, pkg);
                    if let Ok(src) = std::fs::read_to_string(&pkg_path) {
                        let tokens = crate::lexer::Lexer::new(&src).tokenize();
                        if let Ok(ast) = crate::parser::Parser::new(tokens).parse() {
                            self.run(&ast).ok();
                        }
                    }
                    // stdlib пакеты игнорируем (io, math и т.д.)
                }
                _ => {}
            }
        }
        if let Some((_, body, _)) = self.functions.get("main").cloned() {
            self.push_scope(); self.exec_block(&body)?; self.pop_scope();
        }
        Ok(())
    }
    pub fn run_tests(&mut self, nodes: &[Node]) -> Result<(), String> {
        for node in nodes {
            match node {
                Node::FuncDef { name, params, body, is_async, .. } => { self.functions.insert(name.clone(), (params.clone(), body.clone(), *is_async)); }
                Node::ImplBlock { target, methods, .. } => {
                    let entry = self.impls.entry(target.clone()).or_default();
                    for m in methods { if let Node::FuncDef { name, params, body, .. } = m { entry.insert(name.clone(), (params.clone(), body.clone())); } }
                }
                Node::TestDef { name, body } => {
                    let name = name.clone(); let body = body.clone();
                    self.push_scope();
                    match self.exec_block(&body) {
                        Ok(_) => { println!("✓ {}", name); self.test_results.push((name, true, String::new())); }
                        Err(e) => { println!("✗ {} — {}", name, e); self.test_results.push((name, false, e)); }
                    }
                    self.pop_scope();
                }
                _ => {}
            }
        }
        let passed = self.test_results.iter().filter(|r| r.1).count();
        let total = self.test_results.len();
        println!("\n{}/{} passed", passed, total);
        Ok(())
    }
    fn exec_block(&mut self, nodes: &[Node]) -> Result<Option<Signal>, String> {
        for node in nodes { if let Some(sig) = self.exec(node)? { return Ok(Some(sig)); } }
        Ok(None)
    }
    fn exec(&mut self, node: &Node) -> Result<Option<Signal>, String> {
        if self.debug { eprintln!("[dbg] {:?}", std::mem::discriminant(node)); }
        match node {
            Node::EnumVariantInit { .. } => { self.eval(node)?; Ok(None) }
            Node::IfLet { var, value, body, else_body } => {
                let val = self.eval(value)?;
                if !matches!(val, Value::Null) {
                    self.push_scope();
                    self.set_local(var, val);
                    let r = self.exec_block(body)?;
                    self.pop_scope();
                    if let Some(sig) = r { return Ok(Some(sig)); }
                } else if let Some(eb) = else_body {
                    self.push_scope();
                    let r = self.exec_block(eb)?;
                    self.pop_scope();
                    if let Some(sig) = r { return Ok(Some(sig)); }
                }
                Ok(None)
            }
            Node::Assign { name, value, .. } => { let val = self.eval(value)?; self.set(name, val); Ok(None) }
            Node::AssignOp { name, op, value } => {
                let cur = self.get(name).unwrap_or(Value::Int(0));
                let rhs = self.eval(value)?;
                let new = self.binop(cur, op, rhs)?;
                self.set(name, new); Ok(None)
            }
            Node::MultiAssign { names, value } => {
                let val = self.eval(value)?;
                match val {
                    Value::Tuple(items) | Value::Array(items) => { for (i, name) in names.iter().enumerate() { self.set(name, items.get(i).cloned().unwrap_or(Value::Null)); } }
                    _ => { if let Some(name) = names.first() { self.set(name, val); } }
                }
                Ok(None)
            }
            Node::Return(val) => Ok(Some(Signal::Return(self.eval(val)?))),
            Node::Break => Ok(Some(Signal::Break)),
            Node::Continue => Ok(Some(Signal::Continue)),
            Node::Throw(val) => { let v = self.eval(val)?; Ok(Some(Signal::Throw(v.to_string()))) }
            Node::Breakpoint => {
                eprintln!("[breakpoint] env vars: {:?}", self.env.last().map(|s| s.keys().collect::<Vec<_>>()));
                Ok(None)
            }
            Node::If { cond, body, else_body } => {
                let c = self.eval(cond)?; let t = self.truthy(&c);
                if t { self.push_scope(); let r = self.exec_block(body)?; self.pop_scope(); Ok(r) }
                else if let Some(eb) = else_body { let eb = eb.clone(); self.push_scope(); let r = self.exec_block(&eb)?; self.pop_scope(); Ok(r) }
                else { Ok(None) }
            }
            Node::While { cond, body } => {
                loop {
                    let c = self.eval(cond)?; if !self.truthy(&c) { break; }
                    self.push_scope(); let r = self.exec_block(body)?; self.pop_scope();
                    match r { Some(Signal::Break) => break, Some(Signal::Continue) => continue, Some(s) => return Ok(Some(s)), None => {} }
                }
                Ok(None)
            }
            Node::Loop { var, start, end, body } => {
                let s = self.eval(start)?; let e = self.eval(end)?;
                let var = var.clone(); let body = body.clone();
                let mut i = match s { Value::Int(n) => n, _ => 0 };
                let end_i = match e { Value::Int(n) => n, _ => 0 };
                while i < end_i {
                    self.push_scope(); self.set_local(&var, Value::Int(i));
                    let r = self.exec_block(&body)?; self.pop_scope();
                    match r { Some(Signal::Break) => break, Some(Signal::Continue) => { i += 1; continue; } Some(s) => return Ok(Some(s)), None => {} }
                    i += 1;
                }
                Ok(None)
            }
            Node::LoopEnum { idx, var, iter, body } => {
                let arr = self.eval(iter)?;
                let idx = idx.clone(); let var = var.clone(); let body = body.clone();
                let items = match arr { Value::Array(a) => a, Value::Str(s) => s.chars().map(|c| Value::Str(c.to_string())).collect(), _ => return Err("loop in: expected array".into()) };
                for (i, item) in items.into_iter().enumerate() {
                    self.push_scope(); self.set_local(&idx, Value::Int(i as i64)); self.set_local(&var, item);
                    let r = self.exec_block(&body)?; self.pop_scope();
                    match r { Some(Signal::Break) => break, Some(Signal::Continue) => continue, Some(s) => return Ok(Some(s)), None => {} }
                }
                Ok(None)
            }
            Node::LoopIn { var, iter, body } => {
                let arr = self.eval(iter)?;
                let var = var.clone(); let body = body.clone();
                let items = match arr { Value::Array(a) => a, Value::Str(s) => s.chars().map(|c| Value::Str(c.to_string())).collect(), _ => return Err("loop in: expected array".into()) };
                for item in items {
                    self.push_scope(); self.set_local(&var, item);
                    let r = self.exec_block(&body)?; self.pop_scope();
                    match r { Some(Signal::Break) => break, Some(Signal::Continue) => continue, Some(s) => return Ok(Some(s)), None => {} }
                }
                Ok(None)
            }
            Node::Repeat { count, body } => {
                let n = match self.eval(count)? { Value::Int(n) => n, _ => 0 };
                let body = body.clone();
                for _ in 0..n {
                    self.push_scope(); let r = self.exec_block(&body)?; self.pop_scope();
                    match r { Some(Signal::Break) => break, Some(Signal::Continue) => continue, Some(s) => return Ok(Some(s)), None => {} }
                }
                Ok(None)
            }
            Node::Match { expr, arms } => {
                let val = self.eval(expr)?; let arms = arms.clone();
                for arm in &arms {
                    let matched = match &arm.pattern {
                        MatchPattern::Wildcard => true,
                        MatchPattern::Ident(name) => {
                            self.push_scope(); self.set_local(name, val.clone());
                            let ok = if let Some(g) = &arm.guard { let gv = self.eval(g)?; self.truthy(&gv) } else { true };
                            if !ok { self.pop_scope(); continue; }
                            let body = arm.body.clone(); let r = self.exec_block(&body)?; self.pop_scope(); return Ok(r);
                        }
                        MatchPattern::Literal(lit) => { let lv = self.eval(lit)?; val.to_string() == lv.to_string() }
                        MatchPattern::Range(lo, hi) => {
                            let lv = self.eval(lo)?; let hv = self.eval(hi)?;
                            match (&val, &lv, &hv) { (Value::Int(v), Value::Int(l), Value::Int(h)) => v >= l && v <= h, (Value::Float(v), Value::Float(l), Value::Float(h)) => v >= l && v <= h, _ => false }
                        }
                    };
                    if matched {
                        let ok = if let Some(g) = &arm.guard { let gv = self.eval(g)?; self.truthy(&gv) } else { true };
                        if !ok { continue; }
                        self.push_scope(); let body = arm.body.clone(); let r = self.exec_block(&body)?; self.pop_scope(); return Ok(r);
                    }
                }
                Ok(None)
            }
            Node::Try { body, catch_var, catch_body } => {
                let body = body.clone(); let catch_var = catch_var.clone(); let catch_body = catch_body.clone();
                self.push_scope(); let r = self.exec_block(&body)?; self.pop_scope();
                match r {
                    Some(Signal::Throw(msg)) => { self.push_scope(); self.set_local(&catch_var, Value::Str(msg)); let cr = self.exec_block(&catch_body)?; self.pop_scope(); Ok(cr) }
                    other => Ok(other)
                }
            }
            Node::Spawn(expr) => {
                let val = self.eval(expr)?;
                match val {
                    Value::Lambda(_, body) => {
                        let funcs = self.functions.clone();
                        let impls = self.impls.clone();
                        let body_clone = body.clone();
                        std::thread::spawn(move || {
                            let mut child = Interpreter::new();
                            child.functions = funcs;
                            child.impls = impls;
                            let _ = child.eval(&body_clone);
                        });
                    }
                    _ => { eprintln!("[thread] spawn: expected lambda"); }
                }
                Ok(None)
            }
            _ => { self.eval(node)?; Ok(None) }
        }
    }
    pub fn eval(&mut self, node: &Node) -> Result<Value, String> {
        match node {
            Node::Number(n) => { if n.fract() == 0.0 && n.abs() < 1e15 { Ok(Value::Int(*n as i64)) } else { Ok(Value::Float(*n)) } }
            Node::Str(s) => Ok(Value::Str(s.clone())),
            Node::Null => Ok(Value::Null),
            Node::Interp(parts) => {
                let mut r = String::new();
                for p in parts { match p { InterpPart::Literal(s) => r.push_str(s), InterpPart::Expr(e) => { let v = self.eval(e)?; r.push_str(&v.to_string()); } } }
                Ok(Value::Str(r))
            }
            Node::Bool(b) => Ok(Value::Bool(*b)),
            Node::Ident(name) => self.get(name).ok_or_else(|| format!("Undefined: {}", name)),
            Node::Ref(inner) | Node::Await(inner) => self.eval(inner),
            Node::Lambda { params, body } => Ok(Value::Lambda(params.clone(), body.as_ref().clone())),
            Node::Unary(op, expr) => {
                let v = self.eval(expr)?;
                match op.as_str() {
                    "-" => match v { Value::Int(n) => Ok(Value::Int(-n)), Value::Float(f) => Ok(Value::Float(-f)), _ => Err("Cannot negate".into()) }
                    "!" => { let t = self.truthy(&v); Ok(Value::Bool(!t)) }
                    _ => Err(format!("Unknown unary: {}", op))
                }
            }
            Node::BinOp(left, op, right) => {
                let op = op.clone();
                if op == "&&" { let l = self.eval(left)?; let t = self.truthy(&l); if !t { return Ok(Value::Bool(false)); } let r = self.eval(right)?; return Ok(Value::Bool(self.truthy(&r))); }
                if op == "||" { let l = self.eval(left)?; let t = self.truthy(&l); if t { return Ok(Value::Bool(true)); } let r = self.eval(right)?; return Ok(Value::Bool(self.truthy(&r))); }
                let l = self.eval(left)?; let r = self.eval(right)?;
                self.binop(l, &op, r)
            }
            Node::Ternary(cond, then, else_) => {
                let c = self.eval(cond)?;
                if self.truthy(&c) { self.eval(then) } else { self.eval(else_) }
            }
            Node::In { value, container } => {
                let v = self.eval(value)?; let c = self.eval(container)?;
                match c {
                    Value::Array(a) => Ok(Value::Bool(a.iter().any(|x| x.to_string() == v.to_string()))),
                    Value::Str(s) => Ok(Value::Bool(s.contains(&v.to_string()))),
                    _ => Err("in: expected array or str".into())
                }
            }
            Node::FuncCall { name, args } => {
                let name = name.clone();
                let vals: Vec<Value> = args.iter().map(|a| self.eval(a)).collect::<Result<_, _>>()?;
                self.call_func(&name, vals)
            }
            Node::MethodCall { obj, method, args } => {
                let method = method.clone();
                if let Node::Ident(mod_name) = obj.as_ref() {
                    let mod_name = mod_name.clone();
                    if self.get(&mod_name).is_none() {
                        let vals: Vec<Value> = args.iter().map(|a| self.eval(a)).collect::<Result<_, _>>()?;
                        return stdlib::call(&mod_name, &method, vals);
                    }
                }
                let obj_val = self.eval(obj)?;
                let mut all_args = vec![obj_val.clone()];
                for a in args { all_args.push(self.eval(a)?); }
                match &obj_val {
                    Value::Str(_) => stdlib::call("strings", &method, all_args),
                    Value::Array(_) => stdlib::call("arr", &method, all_args),
                    Value::EnumVariant(en, vn, _) => {
                        let type_name = format!("{}::{}", en, vn);
                        if let Some(m) = self.impls.get(&type_name).and_then(|ms| ms.get(&method)).cloned() {
                            let (params, body) = m;
                            self.push_scope(); self.set_local("self", obj_val);
                            for ((pname, _), val) in params.iter().zip(all_args.iter().skip(1)) { self.set_local(pname, val.clone()); }
                            let result = self.exec_block(&body)?; self.pop_scope();
                            return Ok(match result { Some(Signal::Return(v)) => v, _ => Value::Null });
                        }
                        stdlib::call(&en, &method, all_args)
                    }
                    Value::Struct(type_name, _) => {
                        let type_name = type_name.clone();
                        if let Some(m) = self.impls.get(&type_name).and_then(|ms| ms.get(&method)).cloned() {
                            let (params, body) = m;
                            self.push_scope(); self.set_local("self", obj_val);
                            for ((pname, _), val) in params.iter().zip(all_args.iter().skip(1)) { self.set_local(pname, val.clone()); }
                            let result = self.exec_block(&body)?; self.pop_scope();
                            return Ok(match result { Some(Signal::Return(v)) => v, _ => Value::Null });
                        }
                        Err(format!("No method {} on {}", method, type_name))
                    }
                    _ => Err(format!("Unknown method: {}", method))
                }
            }
            Node::OptionalAccess { obj, field } => {
                let val = self.eval(obj)?;
                match val {
                    Value::Null => Ok(Value::Null),
                    Value::Struct(_, fields) => Ok(fields.get(field).cloned().unwrap_or(Value::Null)),
                    _ => Ok(Value::Null)
                }
            }
            Node::OptionalCall { obj, method, args } => {
                let obj_val = self.eval(obj)?;
                match &obj_val {
                    Value::Null => Ok(Value::Null),
                    _ => {
                        let method = method.clone();
                        let mut all_args = vec![obj_val.clone()];
                        for a in args { all_args.push(self.eval(a)?); }
                        match &obj_val {
                            Value::Str(_) => stdlib::call("strings", &method, all_args),
                            Value::Array(_) => stdlib::call("arr", &method, all_args),
                            _ => Ok(Value::Null)
                        }
                    }
                }
            }
            Node::FieldAccess { obj, field } => {
                match self.eval(obj)? {
                    Value::Struct(_, fields) => fields.get(field).cloned().ok_or_else(|| format!("No field: {}", field)),
                    _ => Err("Field access on non-struct".into())
                }
            }
            Node::Index { obj, idx } => {
                let o = self.eval(obj)?; let i = self.eval(idx)?;
                match (o, i) {
                    (Value::Array(a), Value::Int(i)) => Ok(a.get(i as usize).cloned().unwrap_or(Value::Null)),
                    (Value::Str(s), Value::Int(i)) => Ok(s.chars().nth(i as usize).map(|c| Value::Str(c.to_string())).unwrap_or(Value::Null)),
                    _ => Err("Invalid index".into())
                }
            }
            Node::Array(elements) => {
                let mut vals = Vec::new();
                for e in elements {
                    match e {
                        Node::Spread(inner) => { if let Value::Array(a) = self.eval(inner)? { vals.extend(a); } }
                        _ => vals.push(self.eval(e)?)
                    }
                }
                Ok(Value::Array(vals))
            }
            Node::Tuple(elements) => { let vals: Vec<Value> = elements.iter().map(|e| self.eval(e)).collect::<Result<_, _>>()?; Ok(Value::Tuple(vals)) }
            Node::EnumVariantInit { enum_name, variant, args } => {
                let vals: Vec<Value> = args.iter().map(|a| self.eval(a)).collect::<Result<_,_>>()?;
                Ok(Value::EnumVariant(enum_name.clone(), variant.clone(), vals))
            }
            Node::NamedArg { value, .. } => self.eval(value),
            Node::IfLet { var, value, body, else_body } => {
                let val = self.eval(value)?;
                if !matches!(val, Value::Null) {
                    self.push_scope();
                    self.set_local(var, val);
                    let r = self.exec_block(body)?;
                    self.pop_scope();
                    if let Some(sig) = r { return Ok(Value::Null); }
                } else if let Some(eb) = else_body {
                    self.push_scope();
                    self.exec_block(eb)?;
                    self.pop_scope();
                }
                Ok(Value::Null)
            }
            Node::StructInit { name, fields } => {
                let mut map = HashMap::new();
                for (fname, fval) in fields { map.insert(fname.clone(), self.eval(fval)?); }
                Ok(Value::Struct(name.clone(), map))
            }
            Node::If { cond, body, else_body } => {
                let c = self.eval(cond)?;
                if self.truthy(&c) {
                    self.push_scope();
                    let mut last = Value::Null;
                    for node in body { last = self.eval(node)?; }
                    self.pop_scope(); Ok(last)
                } else if let Some(eb) = else_body {
                    self.push_scope();
                    let mut last = Value::Null;
                    for node in eb { last = self.eval(node)?; }
                    self.pop_scope(); Ok(last)
                } else { Ok(Value::Null) }
            }
            Node::Assign { name, value, .. } => { let val = self.eval(value)?; self.set(name, val.clone()); Ok(val) }
            Node::Pipe(stages) => {
                let stages = stages.clone(); let mut val = self.eval(&stages[0])?;
                for stage in &stages[1..] {
                    match stage {
                        Node::Ident(name) => { let name = name.clone(); val = self.call_func(&name, vec![val])?; }
                        Node::FuncCall { name, args } => { let name = name.clone(); let mut a: Vec<Value> = args.iter().map(|x| self.eval(x)).collect::<Result<_, _>>()?; a.insert(0, val); val = self.call_func(&name, a)?; }
                        _ => { val = self.eval(stage)?; }
                    }
                }
                Ok(val)
            }
            _ => Ok(Value::Null)
        }
    }
    pub fn binop(&self, l: Value, op: &str, r: Value) -> Result<Value, String> {
        match (&l, &r) {
            (Value::Int(a), Value::Int(b)) => match op {
                "+" => Ok(Value::Int(a.wrapping_add(*b))), "-" => Ok(Value::Int(a.wrapping_sub(*b))),
                "*" => Ok(Value::Int(a.wrapping_mul(*b))), "/" => if *b == 0 { Err("Division by zero".into()) } else { Ok(Value::Int(a / b)) },
                "%" => Ok(Value::Int(a % b)), "==" => Ok(Value::Bool(a == b)), "!=" => Ok(Value::Bool(a != b)),
                "<" => Ok(Value::Bool(a < b)), ">" => Ok(Value::Bool(a > b)), "<=" => Ok(Value::Bool(a <= b)), ">=" => Ok(Value::Bool(a >= b)),
                _ => Err(format!("Unknown op: {}", op))
            },
            (Value::Float(a), Value::Float(b)) => match op {
                "+" => Ok(Value::Float(a + b)), "-" => Ok(Value::Float(a - b)), "*" => Ok(Value::Float(a * b)), "/" => Ok(Value::Float(a / b)),
                "%" => Ok(Value::Float(a % b)), "==" => Ok(Value::Bool(a == b)), "!=" => Ok(Value::Bool(a != b)),
                "<" => Ok(Value::Bool(a < b)), ">" => Ok(Value::Bool(a > b)), "<=" => Ok(Value::Bool(a <= b)), ">=" => Ok(Value::Bool(a >= b)),
                _ => Err(format!("Unknown op: {}", op))
            },
            (Value::Int(a), Value::Float(b)) => self.binop(Value::Float(*a as f64), op, Value::Float(*b)),
            (Value::Float(a), Value::Int(b)) => self.binop(Value::Float(*a), op, Value::Float(*b as f64)),
            (Value::Str(a), Value::Str(b)) => match op { "+" => Ok(Value::Str(a.clone() + b)), "==" => Ok(Value::Bool(a == b)), "!=" => Ok(Value::Bool(a != b)), "<" => Ok(Value::Bool(a < b)), ">" => Ok(Value::Bool(a > b)), "<=" => Ok(Value::Bool(a <= b)), ">=" => Ok(Value::Bool(a >= b)), _ => Err(format!("Unknown str op: {}", op)) },
            (Value::Str(a), _) => match op { "+" => Ok(Value::Str(a.clone() + &r.to_string())), _ => Err(format!("Type mismatch")) },
            _ => Err(format!("Type mismatch: {} {} {}", l, op, r))
        }
    }
    fn call_func(&mut self, name: &str, args: Vec<Value>) -> Result<Value, String> {
        if name.contains('.') { let p: Vec<&str> = name.splitn(2, '.').collect(); return stdlib::call(p[0], p[1], args); }
        match name {
            "print" => {
                let msg = args.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ");
                // Пасхалки
                match msg.as_str() {
                    "hello world" | "Hello, World!" => { println!("{}  [90m<-- your first TSTNT program![0m", msg); }
                    "42" => { println!("42  [90m<-- the answer to everything[0m"); }
                    "tstnt" | "TSTNT" => { println!("[36mTSTNT[0m [90m<-- that's us![0m"); }
                    "meow" | "мяу" => { println!("🐱 meow~"); }
                    "sudo" => { println!("[31mnice try[0m"); }
                    "rm -rf /" | "del /f /s /q c:\\" => { println!("[31mnice try... very nice try[0m"); }
                    "tstnt" | "TSTNT" => { println!("[36mTSTNT[0m [90m<-- that's us! github.com/tstnt-lang[0m"); }
                    "meow" | "мяу" | "nyan" => { println!("🐱 purrr~ meow~"); }
                    "woof" | "гав" => { println!("🐶 woof!"); }
                    "why" | "почему" => { println!("because it's fun 🐉"); }
                    "who made you" | "кто тебя создал" => { println!("A developer on Android with Rust and ☕"); }
                    "1337" | "leet" => { println!("[32ml33t h4x0r d3t3ct3d[0m"); }
                    "todo" => { println!("[33m// TODO: add more easter eggs[0m"); }
                    "null" => { println!("[90mnull pointer goes brrr[0m"); }
                    "recursion" => { println!("recursion: see 'recursion'"); }
                    "nan" => { println!("NaN + NaN = still NaN 🤔"); }
                    "coffee" | "кофе" => { println!("☕ fueling the language..."); }
                    "gg" | "gg ez" => { println!("[32mgg[0m [90m// game good[0m"); }
                    "ok boomer" => { println!("[90m// this syntax was invented in 2024[0m"); }
                    _ => { println!("{}", msg); }
                }
                Ok(Value::Null)
            }
            "print_no_ln" => { print!("{}", args.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ")); Ok(Value::Null) }
            "input" => { let mut s = String::new(); if let Some(prompt) = args.first() { print!("{}", prompt); use std::io::Write; std::io::stdout().flush().ok(); } std::io::stdin().read_line(&mut s).ok(); Ok(Value::Str(s.trim().to_string())) }
            "len" => match args.first() { Some(Value::Array(a)) => Ok(Value::Int(a.len() as i64)), Some(Value::Str(s)) => Ok(Value::Int(s.chars().count() as i64)), _ => Ok(Value::Int(0)) }
            "str" => Ok(Value::Str(args.first().map(|v| v.to_string()).unwrap_or_default())),
            "int" => match args.first() { Some(Value::Float(f)) => Ok(Value::Int(*f as i64)), Some(Value::Str(s)) => Ok(Value::Int(s.trim().parse().unwrap_or(0))), Some(Value::Int(n)) => Ok(Value::Int(*n)), Some(Value::Bool(b)) => Ok(Value::Int(if *b { 1 } else { 0 })), _ => Ok(Value::Int(0)) }
            "float" => match args.first() { Some(Value::Int(n)) => Ok(Value::Float(*n as f64)), Some(Value::Str(s)) => Ok(Value::Float(s.trim().parse().unwrap_or(0.0))), Some(Value::Float(f)) => Ok(Value::Float(*f)), _ => Ok(Value::Float(0.0)) }
            "bool" => match args.first() { Some(v) => { let t = self.truthy(v); Ok(Value::Bool(t)) }, _ => Ok(Value::Bool(false)) }
            "null" => Ok(Value::Null),
            "type_of" => match args.first() {
                Some(Value::Int(_)) => Ok(Value::Str("int".into())), Some(Value::Float(_)) => Ok(Value::Str("float".into())),
                Some(Value::Str(_)) => Ok(Value::Str("str".into())), Some(Value::Bool(_)) => Ok(Value::Str("bool".into())),
                Some(Value::Array(_)) => Ok(Value::Str("array".into())), Some(Value::Tuple(_)) => Ok(Value::Str("tuple".into())),
                Some(Value::Lambda(_, _)) => Ok(Value::Str("lambda".into())), Some(Value::Struct(n, _)) => Ok(Value::Str(n.clone())),
                Some(Value::Null) | None => Ok(Value::Str("null".into())),
            }
            "is_null" => Ok(Value::Bool(matches!(args.first(), Some(Value::Null) | None))),
            "assert" => {
                let cond = args.first().map(|v| self.truthy(v)).unwrap_or(false);
                if !cond { Err(args.get(1).map(|v| v.to_string()).unwrap_or_else(|| "Assertion failed".into())) }
                else { Ok(Value::Null) }
            }
            "assert_eq" => match (args.first(), args.get(1)) {
                (Some(a), Some(b)) => { if a.to_string() == b.to_string() { Ok(Value::Null) } else { Err(format!("assert_eq failed: {} != {}", a, b)) } }
                _ => Err("assert_eq: need 2 args".into())
            }
            "assert_ne" => match (args.first(), args.get(1)) {
                (Some(a), Some(b)) => { if a.to_string() != b.to_string() { Ok(Value::Null) } else { Err(format!("assert_ne failed: {} == {}", a, b)) } }
                _ => Err("assert_ne: need 2 args".into())
            }
            "panic" => Err(format!("panic: {}", args.first().map(|v| v.to_string()).unwrap_or_else(|| "panic".into()))),
            "range" => match (args.first(), args.get(1)) { (Some(Value::Int(s)), Some(Value::Int(e))) => Ok(Value::Array((*s..*e).map(Value::Int).collect())), _ => Err("range: (int,int)".into()) }
            "map" => match (args.first(), args.get(1)) {
                (Some(Value::Array(arr)), Some(Value::Lambda(params, body))) => {
                    let arr = arr.clone(); let params = params.clone(); let body = body.clone(); let mut result = Vec::new();
                    for item in arr { self.push_scope(); if let Some(p) = params.first() { self.set_local(p, item); } let v = self.eval(&body)?; self.pop_scope(); result.push(v); }
                    Ok(Value::Array(result))
                }
                _ => Err("map: (array, lambda)".into())
            }
            "filter" => match (args.first(), args.get(1)) {
                (Some(Value::Array(arr)), Some(Value::Lambda(params, body))) => {
                    let arr = arr.clone(); let params = params.clone(); let body = body.clone(); let mut result = Vec::new();
                    for item in arr { self.push_scope(); if let Some(p) = params.first() { self.set_local(p, item.clone()); } let v = self.eval(&body)?; let t = self.truthy(&v); self.pop_scope(); if t { result.push(item); } }
                    Ok(Value::Array(result))
                }
                _ => Err("filter: (array, lambda)".into())
            }
            "reduce" => match (args.first(), args.get(1), args.get(2)) {
                (Some(Value::Array(arr)), Some(Value::Lambda(params, body)), Some(init)) => {
                    let arr = arr.clone(); let params = params.clone(); let body = body.clone(); let mut acc = init.clone();
                    for item in arr { self.push_scope(); if let Some(p) = params.first() { self.set_local(p, acc); } if let Some(p) = params.get(1) { self.set_local(p, item); } acc = self.eval(&body)?; self.pop_scope(); }
                    Ok(acc)
                }
                _ => Err("reduce: (array, lambda, init)".into())
            }
            "apply" => match args.first() {
                Some(Value::Lambda(params, body)) => {
                    let params = params.clone(); let body = body.clone();
                    self.push_scope(); for (p, v) in params.iter().zip(args.iter().skip(1)) { self.set_local(p, v.clone()); } let v = self.eval(&body)?; self.pop_scope(); Ok(v)
                }
                _ => Err("apply: (lambda, args...)".into())
            }
            "zip" => match (args.first(), args.get(1)) {
                (Some(Value::Array(a)), Some(Value::Array(b))) => {
                    Ok(Value::Array(a.iter().zip(b.iter()).map(|(x, y)| Value::Tuple(vec![x.clone(), y.clone()])).collect()))
                }
                _ => Err("zip: (array, array)".into())
            }
            "flatten" => match args.first() {
                Some(Value::Array(a)) => {
                    let mut result = Vec::new();
                    for item in a { match item { Value::Array(inner) => result.extend(inner.clone()), v => result.push(v.clone()) } }
                    Ok(Value::Array(result))
                }
                _ => Err("flatten: array".into())
            }
            "sort" => match args.first() {
                Some(Value::Array(a)) => {
                    let mut v = a.clone();
                    v.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
                    Ok(Value::Array(v))
                }
                _ => Err("sort: array".into())
            }
            "unique" => match args.first() {
                Some(Value::Array(a)) => {
                    let mut seen = std::collections::HashSet::new();
                    let result: Vec<Value> = a.iter().filter(|v| seen.insert(v.to_string())).cloned().collect();
                    Ok(Value::Array(result))
                }
                _ => Err("unique: array".into())
            }
            "keys" => match args.first() {
                Some(Value::Struct(_, fields)) => Ok(Value::Array(fields.keys().map(|k| Value::Str(k.clone())).collect())),
                _ => Err("keys: struct".into())
            }
            "values" => match args.first() {
                Some(Value::Struct(_, fields)) => Ok(Value::Array(fields.values().cloned().collect())),
                _ => Err("values: struct".into())
            }
            "max" => match args.first() {
                Some(Value::Array(a)) if !a.is_empty() => {
                    let mut m = a[0].clone();
                    for v in a.iter().skip(1) { if let (Value::Int(a), Value::Int(b)) = (&m, v) { if b > a { m = v.clone(); } } else if let (Value::Float(a), Value::Float(b)) = (&m, v) { if b > a { m = v.clone(); } } }
                    Ok(m)
                }
                _ => Err("max: non-empty array".into())
            }
            "min" => match args.first() {
                Some(Value::Array(a)) if !a.is_empty() => {
                    let mut m = a[0].clone();
                    for v in a.iter().skip(1) { if let (Value::Int(a), Value::Int(b)) = (&m, v) { if b < a { m = v.clone(); } } else if let (Value::Float(a), Value::Float(b)) = (&m, v) { if b < a { m = v.clone(); } } }
                    Ok(m)
                }
                _ => Err("min: non-empty array".into())
            }
            "sum" => match args.first() {
                Some(Value::Array(a)) => {
                    let mut s = 0i64; let mut sf = 0.0f64; let mut has_float = false;
                    for v in a { match v { Value::Int(n) => s += n, Value::Float(f) => { has_float = true; sf += f; } _ => {} } }
                    if has_float { Ok(Value::Float(s as f64 + sf)) } else { Ok(Value::Int(s)) }
                }
                _ => Err("sum: array".into())
            }
            "abs" => match args.first() { Some(Value::Int(n)) => Ok(Value::Int(n.abs())), Some(Value::Float(f)) => Ok(Value::Float(f.abs())), _ => Err("abs: number".into()) }
            "not_null" => Ok(Value::Bool(!matches!(args.first(), Some(Value::Null) | None))),
            "default" => match (args.first(), args.get(1)) {
                (Some(Value::Null), Some(fallback)) => Ok(fallback.clone()),
                (None, Some(fallback)) => Ok(fallback.clone()),
                (Some(v), _) => Ok(v.clone()),
                _ => Ok(Value::Null)
            }
            "coalesce" => {
                for arg in &args { if !matches!(arg, Value::Null) { return Ok(arg.clone()); } }
                Ok(Value::Null)
            }
            "tap" => {
                let val = args.first().cloned().unwrap_or(Value::Null);
                println!("[tap] {}", val);
                Ok(val)
            }
            "repeat_str" => match (args.first(), args.get(1)) {
                (Some(Value::Str(s)), Some(Value::Int(n))) => Ok(Value::Str(s.repeat(*n as usize))),
                _ => Err("repeat_str: (str, int)".into())
            }
            "clamp" => match (args.first(), args.get(1), args.get(2)) {
                (Some(Value::Int(v)), Some(Value::Int(lo)), Some(Value::Int(hi))) => Ok(Value::Int((*v).max(*lo).min(*hi))),
                (Some(Value::Float(v)), Some(Value::Float(lo)), Some(Value::Float(hi))) => Ok(Value::Float(v.max(*lo).min(*hi))),
                _ => Err("clamp: (val, min, max)".into())
            }
            "find" => match (args.first(), args.get(1)) {
                (Some(Value::Array(arr)), Some(Value::Lambda(params, body))) => {
                    let arr = arr.clone(); let params = params.clone(); let body = body.clone();
                    for item in arr {
                        self.push_scope();
                        if let Some(p) = params.first() { self.set_local(p, item.clone()); }
                        let v = self.eval(&body)?;
                        self.pop_scope();
                        if self.truthy(&v) { return Ok(item); }
                    }
                    Ok(Value::Null)
                }
                _ => Err("find: (array, lambda)".into())
            }
            "every" => match (args.first(), args.get(1)) {
                (Some(Value::Array(arr)), Some(Value::Lambda(params, body))) => {
                    let arr = arr.clone(); let params = params.clone(); let body = body.clone();
                    for item in arr {
                        self.push_scope();
                        if let Some(p) = params.first() { self.set_local(p, item); }
                        let v = self.eval(&body)?;
                        self.pop_scope();
                        if !self.truthy(&v) { return Ok(Value::Bool(false)); }
                    }
                    Ok(Value::Bool(true))
                }
                _ => Err("every: (array, lambda)".into())
            }
            "any" => match (args.first(), args.get(1)) {
                (Some(Value::Array(arr)), Some(Value::Lambda(params, body))) => {
                    let arr = arr.clone(); let params = params.clone(); let body = body.clone();
                    for item in arr {
                        self.push_scope();
                        if let Some(p) = params.first() { self.set_local(p, item); }
                        let v = self.eval(&body)?;
                        self.pop_scope();
                        if self.truthy(&v) { return Ok(Value::Bool(true)); }
                    }
                    Ok(Value::Bool(false))
                }
                _ => Err("any: (array, lambda)".into())
            }
            "flat_map" => match (args.first(), args.get(1)) {
                (Some(Value::Array(arr)), Some(Value::Lambda(params, body))) => {
                    let arr = arr.clone(); let params = params.clone(); let body = body.clone();
                    let mut result = Vec::new();
                    for item in arr {
                        self.push_scope();
                        if let Some(p) = params.first() { self.set_local(p, item); }
                        let v = self.eval(&body)?;
                        self.pop_scope();
                        match v { Value::Array(inner) => result.extend(inner), other => result.push(other) }
                    }
                    Ok(Value::Array(result))
                }
                _ => Err("flat_map: (array, lambda)".into())
            }
            "count_where" => match (args.first(), args.get(1)) {
                (Some(Value::Array(arr)), Some(Value::Lambda(params, body))) => {
                    let arr = arr.clone(); let params = params.clone(); let body = body.clone();
                    let mut count = 0i64;
                    for item in arr {
                        self.push_scope();
                        if let Some(p) = params.first() { self.set_local(p, item); }
                        let v = self.eval(&body)?;
                        self.pop_scope();
                        if self.truthy(&v) { count += 1; }
                    }
                    Ok(Value::Int(count))
                }
                _ => Err("count_where: (array, lambda)".into())
            }
            "take" => match (args.first(), args.get(1)) {
                (Some(Value::Array(a)), Some(Value::Int(n))) => Ok(Value::Array(a.iter().take(*n as usize).cloned().collect())),
                _ => Err("take: (array, n)".into())
            }
            "drop" => match (args.first(), args.get(1)) {
                (Some(Value::Array(a)), Some(Value::Int(n))) => Ok(Value::Array(a.iter().skip(*n as usize).cloned().collect())),
                _ => Err("drop: (array, n)".into())
            }
            "chunks" => match (args.first(), args.get(1)) {
                (Some(Value::Array(a)), Some(Value::Int(n))) => {
                    let size = *n as usize;
                    Ok(Value::Array(a.chunks(size).map(|c| Value::Array(c.to_vec())).collect()))
                }
                _ => Err("chunks: (array, size)".into())
            }
            "concat" => {
                let mut result = Vec::new();
                for arg in &args { match arg { Value::Array(a) => result.extend(a.clone()), v => result.push(v.clone()) } }
                Ok(Value::Array(result))
            }
            "reverse" => match args.first() {
                Some(Value::Array(a)) => { let mut v = a.clone(); v.reverse(); Ok(Value::Array(v)) }
                Some(Value::Str(s)) => Ok(Value::Str(s.chars().rev().collect())),
                _ => Err("reverse: array or str".into())
            }
            "index_of" => match (args.first(), args.get(1)) {
                (Some(Value::Array(a)), Some(v)) => Ok(a.iter().position(|x| x.to_string() == v.to_string()).map(|i| Value::Int(i as i64)).unwrap_or(Value::Int(-1))),
                (Some(Value::Str(s)), Some(Value::Str(pat))) => Ok(s.find(pat.as_str()).map(|i| Value::Int(i as i64)).unwrap_or(Value::Int(-1))),
                _ => Err("index_of: (array, val) or (str, str)".into())
            }
            "slice" => match args.first() {
                Some(Value::Array(a)) => {
                    let from = match args.get(1) { Some(Value::Int(n)) => *n as usize, _ => 0 };
                    let to = match args.get(2) { Some(Value::Int(n)) => *n as usize, _ => a.len() };
                    Ok(Value::Array(a[from.min(a.len())..to.min(a.len())].to_vec()))
                }
                Some(Value::Str(s)) => {
                    let chars: Vec<char> = s.chars().collect();
                    let from = match args.get(1) { Some(Value::Int(n)) => *n as usize, _ => 0 };
                    let to = match args.get(2) { Some(Value::Int(n)) => *n as usize, _ => chars.len() };
                    Ok(Value::Str(chars[from.min(chars.len())..to.min(chars.len())].iter().collect()))
                }
                _ => Err("slice: (array|str, from, to)".into())
            }
            "sprintf" => {
                let template = match args.first() { Some(Value::Str(s)) => s.clone(), _ => return Err("sprintf: (template, ...)".into()) };
                let mut result = template.clone();
                for arg in args.iter().skip(1) { result = result.replacen("{}", &arg.to_string(), 1); }
                Ok(Value::Str(result))
            }
            _ => {
                if let Some(Value::Lambda(params, body)) = self.get(name) {
                    let params = params.clone(); let body = body.clone();
                    self.push_scope();
                    for (p, v) in params.iter().zip(args.iter()) { self.set_local(p, v.clone()); }
                    let v = self.eval(&body)?; self.pop_scope(); return Ok(v);
                }
                if let Some((params, body, _)) = self.functions.get(name).cloned() {
                    if self.deprecated_funcs.contains(name) {
                        eprintln!("\x1b[33mwarning\x1b[0m: '{}' is deprecated", name);
                    }
                    let is_cached = self.cache_store.contains_key(&format!("__cache_registered_{}__", name));
                    if is_cached {
                        let cache_key = format!("__cache_{}_{}__", name, args.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","));
                        if let Some(cached) = self.cache_store.get(&cache_key).cloned() {
                            return Ok(cached);
                        }
                        self.push_scope();
                        for ((pname, _), val) in params.iter().zip(args.iter()) { self.set_local(pname, val.clone()); }
                        let result = self.exec_block(&body)?; self.pop_scope();
                        let val = match result { Some(Signal::Return(v)) => v, _ => Value::Null };
                        self.cache_store.insert(cache_key, val.clone());
                        return Ok(val);
                    }
                    let is_timed = self.timer_funcs.contains(name);
                    let start = if is_timed {
                        SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
                    } else { 0 };
                    self.push_scope();
                    for ((pname, _), val) in params.iter().zip(args.iter()) { self.set_local(pname, val.clone()); }
                    let result = self.exec_block(&body)?; self.pop_scope();
                    if is_timed {
                        let elapsed = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0) - start;
                        eprintln!("\x1b[36m@timer\x1b[0m: {}() took {}ms", name, elapsed);
                    }
                    Ok(match result { Some(Signal::Return(v)) => v, _ => Value::Null })
                } else { Err(format!("Unknown function: {}", name)) }
            }
        }
    }
}
