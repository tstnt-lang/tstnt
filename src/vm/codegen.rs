use std::collections::HashMap;
use crate::parser::Node;
use crate::vm::chunk::{Chunk, Const};
use crate::vm::opcode::Op;

pub struct FuncProto {
    pub name: String,
    pub params: Vec<String>,
    pub chunk: Chunk,
}

#[allow(dead_code)]
pub struct Codegen {
    pub chunk: Chunk,
    pub funcs: Vec<FuncProto>,
    func_offsets: HashMap<String, u16>,
    locals: Vec<HashMap<String, u16>>,
    globals: HashMap<String, u16>,
    global_count: u16,
    line: usize,
}

#[allow(dead_code)]
impl Codegen {
    pub fn new() -> Self {
        Codegen {
            chunk: Chunk::new(),
            funcs: Vec::new(),
            func_offsets: HashMap::new(),
            locals: Vec::new(),
            globals: HashMap::new(),
            global_count: 0,
            line: 1,
        }
    }

    pub fn compile(&mut self, ast: &[Node]) -> Result<(), String> {
        for node in ast {
            if let Node::FuncDef { name, params, body, .. } = node {
                self.compile_func(name, params, body)?;
            }
        }

        let main_const = self.chunk.add_const(Const::Str("main".into()));
        self.chunk.write_op(Op::CallNative, 0);
        self.chunk.write_u16(main_const, 0);
        self.chunk.write(0, 0);
        self.chunk.write_op(Op::Halt, 0);
        Ok(())
    }

    fn compile_func(&mut self, name: &str, params: &[(String, String)], body: &[Node]) -> Result<(), String> {
        let mut sub = Codegen::new();
        sub.locals.push(HashMap::new());

        for (i, (pname, _)) in params.iter().enumerate() {
            sub.locals.last_mut().unwrap().insert(pname.clone(), i as u16);
        }

        for node in body {
            sub.compile_node(node)?;
        }

        sub.chunk.write_op(Op::PushNull, sub.line);
        sub.chunk.write_op(Op::Return, sub.line);

        let offset = self.chunk.add_const(Const::Str(name.into()));
        self.func_offsets.insert(name.into(), offset);

        self.funcs.push(FuncProto {
            name: name.into(),
            params: params.iter().map(|(p, _)| p.clone()).collect(),
            chunk: sub.chunk,
        });

        Ok(())
    }

    fn compile_node(&mut self, node: &Node) -> Result<(), String> {
        match node {
            Node::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    let idx = self.chunk.add_const(Const::Int(*n as i64));
                    self.chunk.write_op(Op::PushInt, self.line);
                    self.chunk.write_u16(idx, self.line);
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
            Node::Bool(b) => {
                self.chunk.write_op(Op::PushBool, self.line);
                self.chunk.write(if *b { 1 } else { 0 }, self.line);
            }
            Node::Ident(name) => {
                if let Some(&idx) = self.locals.iter().rev().flat_map(|s| s.get(name.as_str())).next() {
                    self.chunk.write_op(Op::LoadLocal, self.line);
                    self.chunk.write_u16(idx, self.line);
                } else if let Some(&idx) = self.globals.get(name.as_str()) {
                    self.chunk.write_op(Op::LoadGlobal, self.line);
                    self.chunk.write_u16(idx, self.line);
                } else {
                    let idx = self.chunk.add_const(Const::Str(name.clone()));
                    self.chunk.write_op(Op::Load, self.line);
                    self.chunk.write_u16(idx, self.line);
                }
            }
            Node::Assign { name, value, .. } => {
                self.compile_node(value)?;
                let local_idx = self.locals.iter().rev().flat_map(|s| s.get(name.as_str())).next().copied();
                if let Some(idx) = local_idx {
                    self.chunk.write_op(Op::StoreLocal, self.line);
                    self.chunk.write_u16(idx, self.line);
                } else {
                    let idx = if let Some(&i) = self.globals.get(name.as_str()) {
                        i
                    } else {
                        let i = self.global_count;
                        self.globals.insert(name.clone(), i);
                        self.global_count += 1;
                        i
                    };
                    self.chunk.write_op(Op::StoreGlobal, self.line);
                    self.chunk.write_u16(idx, self.line);
                }
            }
            Node::BinOp(left, op, right) => {
                self.compile_node(left)?;
                self.compile_node(right)?;
                let opcode = match op.as_str() {
                    "+" => Op::Add, "-" => Op::Sub, "*" => Op::Mul,
                    "/" => Op::Div, "%" => Op::Mod,
                    "==" => Op::Eq, "!=" => Op::Neq,
                    "<" => Op::Lt, ">" => Op::Gt,
                    "<=" => Op::LtEq, ">=" => Op::GtEq,
                    _ => return Err(format!("Unknown op: {}", op))
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
            Node::Return(val) => {
                self.compile_node(val)?;
                self.chunk.write_op(Op::Return, self.line);
            }
            Node::If { cond, body, else_body } => {
                self.compile_node(cond)?;
                self.chunk.write_op(Op::JumpIfNot, self.line);
                let jump_pos = self.chunk.len();
                self.chunk.write_u16(0, self.line);

                for s in body { self.compile_node(s)?; }

                if let Some(eb) = else_body {
                    self.chunk.write_op(Op::Jump, self.line);
                    let end_jump = self.chunk.len();
                    self.chunk.write_u16(0, self.line);
                    let after_if = self.chunk.len() as u16;
                    self.chunk.patch_u16(jump_pos, after_if);
                    for s in eb { self.compile_node(s)?; }
                    let after_else = self.chunk.len() as u16;
                    self.chunk.patch_u16(end_jump, after_else);
                } else {
                    let after_if = self.chunk.len() as u16;
                    self.chunk.patch_u16(jump_pos, after_if);
                }
            }
            Node::Loop { var, start, end, body } => {
                self.compile_node(start)?;
                let var_idx = self.global_count;
                self.globals.insert(var.clone(), var_idx);
                self.global_count += 1;
                self.chunk.write_op(Op::StoreGlobal, self.line);
                self.chunk.write_u16(var_idx, self.line);

                let loop_start = self.chunk.len() as u16;

                self.chunk.write_op(Op::LoadGlobal, self.line);
                self.chunk.write_u16(var_idx, self.line);
                self.compile_node(end)?;
                self.chunk.write_op(Op::Lt, self.line);

                self.chunk.write_op(Op::JumpIfNot, self.line);
                let exit_jump = self.chunk.len();
                self.chunk.write_u16(0, self.line);

                for s in body { self.compile_node(s)?; }

                self.chunk.write_op(Op::LoadGlobal, self.line);
                self.chunk.write_u16(var_idx, self.line);
                let one_idx = self.chunk.add_const(Const::Int(1));
                self.chunk.write_op(Op::PushInt, self.line);
                self.chunk.write_u16(one_idx, self.line);
                self.chunk.write_op(Op::Add, self.line);
                self.chunk.write_op(Op::StoreGlobal, self.line);
                self.chunk.write_u16(var_idx, self.line);

                self.chunk.write_op(Op::Jump, self.line);
                self.chunk.write_u16(loop_start, self.line);

                let after_loop = self.chunk.len() as u16;
                self.chunk.patch_u16(exit_jump, after_loop);
            }
            Node::FuncCall { name, args } => {
                for arg in args { self.compile_node(arg)?; }
                let idx = self.chunk.add_const(Const::Str(name.clone()));
                self.chunk.write_op(Op::CallNative, self.line);
                self.chunk.write_u16(idx, self.line);
                self.chunk.write(args.len() as u8, self.line);
            }
            Node::MethodCall { obj, method, args } => {
                self.compile_node(obj)?;
                for arg in args { self.compile_node(arg)?; }
                let idx = self.chunk.add_const(Const::Str(method.clone()));
                self.chunk.write_op(Op::CallNative, self.line);
                self.chunk.write_u16(idx, self.line);
                self.chunk.write((args.len() + 1) as u8, self.line);
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
            Node::StructInit { name, fields } => {
                for (_, val) in fields { self.compile_node(val)?; }
                let name_idx = self.chunk.add_const(Const::Str(name.clone()));
                let fields_idx = self.chunk.add_const(Const::Str(
                    fields.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>().join(",")
                ));
                self.chunk.write_op(Op::MakeStruct, self.line);
                self.chunk.write_u16(name_idx, self.line);
                self.chunk.write_u16(fields_idx, self.line);
                self.chunk.write_u16(fields.len() as u16, self.line);
            }
            Node::FieldAccess { obj, field } => {
                self.compile_node(obj)?;
                let idx = self.chunk.add_const(Const::Str(field.clone()));
                self.chunk.write_op(Op::FieldGet, self.line);
                self.chunk.write_u16(idx, self.line);
            }
            Node::Ref(inner) => self.compile_node(inner)?,
            _ => {}
        }
        Ok(())
    }
}
