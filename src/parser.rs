use crate::lexer::{Token, TokenType};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Node {
    Number(f64), Str(String), Interp(Vec<InterpPart>), Bool(bool), Null,
    Ident(String),
    BinOp(Box<Node>, String, Box<Node>),
    Unary(String, Box<Node>),
    Ternary(Box<Node>, Box<Node>, Box<Node>),
    Assign { name: String, type_ann: Option<String>, value: Box<Node>, mutable: bool },
    AssignOp { name: String, op: String, value: Box<Node> },
    MultiAssign { names: Vec<String>, value: Box<Node> },
    Return(Box<Node>), If { cond: Box<Node>, body: Vec<Node>, else_body: Option<Vec<Node>> },
    While { cond: Box<Node>, body: Vec<Node> },
    Loop { var: String, start: Box<Node>, end: Box<Node>, body: Vec<Node> },
    LoopIn { var: String, iter: Box<Node>, body: Vec<Node> },
    LoopEnum { idx: String, var: String, iter: Box<Node>, body: Vec<Node> },
    Repeat { count: Box<Node>, body: Vec<Node> },
    Match { expr: Box<Node>, arms: Vec<MatchArm> },
    Break, Continue,
    FuncCall { name: String, args: Vec<Node> },
    MethodCall { obj: Box<Node>, method: String, args: Vec<Node> },
    FieldAccess { obj: Box<Node>, field: String },
    OptionalAccess { obj: Box<Node>, field: String },
    OptionalCall { obj: Box<Node>, method: String, args: Vec<Node> },
    Index { obj: Box<Node>, idx: Box<Node> },
    FuncDef { name: String, params: Vec<(String, String)>, ret: Option<String>, body: Vec<Node>, is_async: bool },
    TestDef { name: String, body: Vec<Node> },
    Lambda { params: Vec<String>, body: Box<Node> },
    StructDef { name: String, fields: Vec<(String, String)> },
    UnitDef { name: String, fields: Vec<(String, String)>, methods: Vec<Node> },
    EnumDef { name: String, variants: Vec<String> },
    InterfaceDef { name: String, methods: Vec<String> },
    ImplBlock { target: String, iface: Option<String>, methods: Vec<Node> },
    Array(Vec<Node>), Tuple(Vec<Node>), Spread(Box<Node>),
    StructInit { name: String, fields: Vec<(String, Node)> },
    Ref(Box<Node>), Use(String), Mod(String),
    Try { body: Vec<Node>, catch_var: String, catch_body: Vec<Node> },
    Throw(Box<Node>), Await(Box<Node>), Spawn(Box<Node>),
    Pipe(Vec<Node>),
    In { value: Box<Node>, container: Box<Node> },
    Breakpoint,
    Decorator { name: String, target: Box<Node> },
}

#[derive(Debug, Clone)]
pub enum InterpPart { Literal(String), Expr(Box<Node>) }

#[derive(Debug, Clone)]
pub struct MatchArm { pub pattern: MatchPattern, pub guard: Option<Box<Node>>, pub body: Vec<Node> }

#[derive(Debug, Clone)]
pub enum MatchPattern { Literal(Node), Wildcard, Ident(String), Range(Box<Node>, Box<Node>) }

pub struct Parser { tokens: Vec<Token>, pos: usize }

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self { Parser { tokens, pos: 0 } }
    fn cur(&self) -> &Token { &self.tokens[self.pos] }
    fn peek(&self) -> &Token { let p = (self.pos+1).min(self.tokens.len()-1); &self.tokens[p] }
    fn advance(&mut self) -> Token { let t = self.tokens[self.pos].clone(); if self.pos < self.tokens.len()-1 { self.pos += 1; } t }
    fn expect(&mut self, kind: TokenType) -> Result<Token, String> {
        if self.cur().kind == kind { Ok(self.advance()) }
        else { Err(format!("Line {}: expected {:?}, got {:?}", self.cur().line, kind, self.cur().value)) }
    }
    pub fn parse(&mut self) -> Result<Vec<Node>, String> {
        let mut nodes = Vec::new();
        while self.cur().kind != TokenType::Eof { nodes.push(self.parse_top()?); }
        Ok(nodes)
    }
    fn parse_top(&mut self) -> Result<Node, String> {
        // Декоратор @cache, @timer, @deprecated
        if self.cur().kind == TokenType::At {
            self.advance();
            let name = self.read_any_ident()?;
            let target = self.parse_top()?;
            return Ok(Node::Decorator { name, target: Box::new(target) });
        }
        match self.cur().kind {
            TokenType::Use => self.parse_use(),
            TokenType::Mod => { self.advance(); Ok(Node::Mod(self.expect(TokenType::Ident)?.value)) }
            TokenType::Do | TokenType::Async => self.parse_func(),
            TokenType::Test => self.parse_test(),
            TokenType::Struct => self.parse_struct(),
            TokenType::Unit => self.parse_unit(),
            TokenType::Enum => self.parse_enum(),
            TokenType::Interface => self.parse_interface(),
            TokenType::Impl => self.parse_impl(),
            _ => self.parse_stmt(),
        }
    }
    fn parse_use(&mut self) -> Result<Node, String> {
        self.advance();
        // use "colors" или use "./file.tstnt" — строковый путь
        if self.cur().kind == TokenType::Str {
            return Ok(Node::Use(self.advance().value));
        }
        let mut parts = vec![self.read_any_ident()?];
        while self.cur().kind == TokenType::Dot { self.advance(); parts.push(self.read_any_ident()?); }
        Ok(Node::Use(parts.join(".")))
    }
    pub fn parse_func(&mut self) -> Result<Node, String> {
        let is_async = if self.cur().kind == TokenType::Async { self.advance(); true } else { false };
        self.expect(TokenType::Do)?;
        let name = self.expect(TokenType::Ident)?.value;
        // дженерики <T, U, ...> — просто запоминаем имена, используем как any
        let mut _generics: Vec<String> = Vec::new();
        if self.cur().kind == TokenType::Lt {
            self.advance();
            while self.cur().kind != TokenType::Gt {
                if self.cur().kind == TokenType::Ident { _generics.push(self.advance().value); }
                if self.cur().kind == TokenType::Comma { self.advance(); }
            }
            self.expect(TokenType::Gt)?;
        }
        let mut params = Vec::new();
        if self.cur().kind == TokenType::LParen {
            self.advance();
            while self.cur().kind != TokenType::RParen {
                let pname = self.expect(TokenType::Ident)?.value;
                self.expect(TokenType::Colon)?;
                params.push((pname, self.parse_type()?));
                if self.cur().kind == TokenType::Comma { self.advance(); }
            }
            self.expect(TokenType::RParen)?;
        }
        let ret = if self.cur().kind == TokenType::Arrow { self.advance(); Some(self.parse_type()?) } else { None };
        self.expect(TokenType::LBrace)?;
        let body = self.parse_block()?;
        self.expect(TokenType::RBrace)?;
        Ok(Node::FuncDef { name, params, ret, body, is_async })
    }
    fn parse_test(&mut self) -> Result<Node, String> {
        self.advance();
        let name = self.expect(TokenType::Ident)?.value;
        self.expect(TokenType::LBrace)?;
        let body = self.parse_block()?;
        self.expect(TokenType::RBrace)?;
        Ok(Node::TestDef { name, body })
    }
    fn parse_struct(&mut self) -> Result<Node, String> {
        self.advance();
        let name = self.expect(TokenType::Ident)?.value;
        self.expect(TokenType::LBrace)?;
        let mut fields = Vec::new();
        while self.cur().kind != TokenType::RBrace {
            let fname = self.expect(TokenType::Ident)?.value;
            self.expect(TokenType::Colon)?;
            fields.push((fname, self.parse_type()?));
        }
        self.expect(TokenType::RBrace)?;
        Ok(Node::StructDef { name, fields })
    }
    fn parse_unit(&mut self) -> Result<Node, String> {
        self.advance();
        let name = self.expect(TokenType::Ident)?.value;
        self.expect(TokenType::LBrace)?;
        let mut fields = Vec::new(); let mut methods = Vec::new();
        while self.cur().kind != TokenType::RBrace {
            if self.cur().kind == TokenType::Do || self.cur().kind == TokenType::Async { methods.push(self.parse_func()?); }
            else { let f = self.expect(TokenType::Ident)?.value; self.expect(TokenType::Colon)?; fields.push((f, self.parse_type()?)); }
        }
        self.expect(TokenType::RBrace)?;
        Ok(Node::UnitDef { name, fields, methods })
    }
    fn parse_enum(&mut self) -> Result<Node, String> {
        self.advance();
        let name = self.expect(TokenType::Ident)?.value;
        self.expect(TokenType::LBrace)?;
        let mut variants = Vec::new();
        while self.cur().kind != TokenType::RBrace { variants.push(self.expect(TokenType::Ident)?.value); if self.cur().kind == TokenType::Comma { self.advance(); } }
        self.expect(TokenType::RBrace)?;
        Ok(Node::EnumDef { name, variants })
    }
    fn parse_interface(&mut self) -> Result<Node, String> {
        self.advance();
        let name = self.expect(TokenType::Ident)?.value;
        self.expect(TokenType::LBrace)?;
        let mut methods = Vec::new();
        while self.cur().kind != TokenType::RBrace {
            self.expect(TokenType::Do)?;
            methods.push(self.expect(TokenType::Ident)?.value);
            if self.cur().kind == TokenType::LParen { self.advance(); while self.cur().kind != TokenType::RParen { self.advance(); } self.advance(); }
            if self.cur().kind == TokenType::Arrow { self.advance(); self.parse_type()?; }
        }
        self.expect(TokenType::RBrace)?;
        Ok(Node::InterfaceDef { name, methods })
    }
    fn parse_impl(&mut self) -> Result<Node, String> {
        self.advance();
        let target = self.expect(TokenType::Ident)?.value;
        let iface = if self.cur().kind == TokenType::Colon { self.advance(); Some(self.expect(TokenType::Ident)?.value) } else { None };
        self.expect(TokenType::LBrace)?;
        let mut methods = Vec::new();
        while self.cur().kind != TokenType::RBrace { methods.push(self.parse_func()?); }
        self.expect(TokenType::RBrace)?;
        Ok(Node::ImplBlock { target, iface, methods })
    }
    fn parse_type(&mut self) -> Result<String, String> {
        match self.cur().kind {
            TokenType::LBracket => { self.advance(); let i = self.parse_type()?; self.expect(TokenType::RBracket)?; Ok(format!("[{}]", i)) }
            TokenType::Amp => { self.advance(); Ok(format!("&{}", self.parse_type()?)) }
            TokenType::Ident => Ok(self.advance().value),
            _ => Err(format!("Line {}: expected type", self.cur().line))
        }
    }
    fn parse_block(&mut self) -> Result<Vec<Node>, String> {
        let mut stmts = Vec::new();
        while self.cur().kind != TokenType::RBrace && self.cur().kind != TokenType::Eof { stmts.push(self.parse_stmt()?); }
        Ok(stmts)
    }
    fn parse_stmt(&mut self) -> Result<Node, String> {
        match self.cur().kind {
            TokenType::Let => self.parse_let(),
            TokenType::Return => { self.advance(); Ok(Node::Return(Box::new(self.parse_expr()?))) }
            TokenType::If => self.parse_if(),
            TokenType::While => self.parse_while(),
            TokenType::Loop => self.parse_loop(),
            TokenType::Repeat => self.parse_repeat(),
            TokenType::Match => self.parse_match(),
            TokenType::Break => { self.advance(); Ok(Node::Break) }
            TokenType::Continue => { self.advance(); Ok(Node::Continue) }
            TokenType::Throw => { self.advance(); Ok(Node::Throw(Box::new(self.parse_expr()?))) }
            TokenType::Try => self.parse_try(),
            TokenType::Await => { self.advance(); Ok(Node::Await(Box::new(self.parse_expr()?))) }
            TokenType::Thread if self.peek().kind == TokenType::Dot => self.parse_expr(),
            TokenType::Thread => { self.advance(); Ok(Node::Spawn(Box::new(self.parse_expr()?))) }
            TokenType::Ident if self.peek().kind == TokenType::Eq => {
                let name = self.advance().value; self.advance();
                Ok(Node::Assign { name, type_ann: None, value: Box::new(self.parse_expr()?), mutable: true })
            }
            TokenType::Ident if matches!(self.peek().kind, TokenType::PlusEq | TokenType::MinusEq | TokenType::StarEq | TokenType::SlashEq) => {
                let name = self.advance().value;
                let op = self.advance().value.trim_end_matches('=').to_string();
                Ok(Node::AssignOp { name, op, value: Box::new(self.parse_expr()?) })
            }
            _ => self.parse_expr(),
        }
    }
    fn parse_let(&mut self) -> Result<Node, String> {
        self.advance();
        let mutable = if self.cur().kind == TokenType::Mut { self.advance(); true } else { false };
        if self.cur().kind == TokenType::LParen {
            self.advance();
            let mut names = Vec::new();
            while self.cur().kind != TokenType::RParen { names.push(self.expect(TokenType::Ident)?.value); if self.cur().kind == TokenType::Comma { self.advance(); } }
            self.expect(TokenType::RParen)?;
            self.expect(TokenType::Eq)?;
            return Ok(Node::MultiAssign { names, value: Box::new(self.parse_expr()?) });
        }
        let name = self.expect(TokenType::Ident)?.value;
        let type_ann = if self.cur().kind == TokenType::Colon { self.advance(); Some(self.parse_type()?) } else { None };
        self.expect(TokenType::Eq)?;
        Ok(Node::Assign { name, type_ann, value: Box::new(self.parse_expr()?), mutable })
    }
    fn parse_if(&mut self) -> Result<Node, String> {
        self.advance();
        let cond = self.parse_expr()?;
        self.expect(TokenType::LBrace)?; let body = self.parse_block()?; self.expect(TokenType::RBrace)?;
        let else_body = if self.cur().kind == TokenType::Else {
            self.advance();
            self.expect(TokenType::LBrace)?; let eb = self.parse_block()?; self.expect(TokenType::RBrace)?;
            Some(eb)
        } else { None };
        Ok(Node::If { cond: Box::new(cond), body, else_body })
    }
    fn parse_while(&mut self) -> Result<Node, String> {
        self.advance();
        let cond = self.parse_expr()?;
        self.expect(TokenType::LBrace)?; let body = self.parse_block()?; self.expect(TokenType::RBrace)?;
        Ok(Node::While { cond: Box::new(cond), body })
    }
    fn parse_loop(&mut self) -> Result<Node, String> {
        self.advance();
        let var = self.expect(TokenType::Ident)?.value;
        // loop i, item in arr  (enumerate)
        #[allow(unused_variables)]
        let enum_var = if self.cur().kind == TokenType::Comma {
            self.advance();
            Some(self.expect(TokenType::Ident)?.value)
        } else { None };
        self.expect(TokenType::In)?;
        // loop x in arr { } or loop x in 0..10 { }
        let iter_or_start = self.parse_primary()?;
        if self.cur().kind == TokenType::DotDot {
            self.advance();
            let end = self.parse_primary()?;
            self.expect(TokenType::LBrace)?; let body = self.parse_block()?; self.expect(TokenType::RBrace)?;
            Ok(Node::Loop { var, start: Box::new(iter_or_start), end: Box::new(end), body })
        } else {
            self.expect(TokenType::LBrace)?; let body = self.parse_block()?; self.expect(TokenType::RBrace)?;
            if let Some(idx_var) = enum_var {
                Ok(Node::LoopEnum { idx: var, var: idx_var, iter: Box::new(iter_or_start), body })
            } else {
                Ok(Node::LoopIn { var, iter: Box::new(iter_or_start), body })
            }
        }
    }
    fn parse_repeat(&mut self) -> Result<Node, String> {
        self.advance();
        let count = self.parse_expr()?;
        self.expect(TokenType::LBrace)?; let body = self.parse_block()?; self.expect(TokenType::RBrace)?;
        Ok(Node::Repeat { count: Box::new(count), body })
    }
    fn parse_match(&mut self) -> Result<Node, String> {
        self.advance();
        let expr = self.parse_match_expr()?;
        self.expect(TokenType::LBrace)?;
        let mut arms = Vec::new();
        while self.cur().kind != TokenType::RBrace {
            let pattern = self.parse_pattern()?;
            let guard = if self.cur().kind == TokenType::If { self.advance(); Some(Box::new(self.parse_expr()?)) } else { None };
            self.expect(TokenType::Arrow)?;
            let body = if self.cur().kind == TokenType::LBrace { self.advance(); let b = self.parse_block()?; self.expect(TokenType::RBrace)?; b } else { vec![self.parse_stmt()?] };
            arms.push(MatchArm { pattern, guard, body });
        }
        self.expect(TokenType::RBrace)?;
        Ok(Node::Match { expr: Box::new(expr), arms })
    }
    fn parse_match_expr(&mut self) -> Result<Node, String> {
        if self.cur().kind == TokenType::Ident { let n = self.advance().value; Ok(Node::Ident(n)) }
        else { self.parse_comparison() }
    }
    fn parse_pattern(&mut self) -> Result<MatchPattern, String> {
        match self.cur().kind {
            TokenType::Underscore => { self.advance(); Ok(MatchPattern::Wildcard) }
            TokenType::Number | TokenType::Minus => {
                let neg = if self.cur().kind == TokenType::Minus { self.advance(); true } else { false };
                let v: f64 = self.advance().value.parse().unwrap_or(0.0);
                let v = if neg { -v } else { v };
                if self.cur().kind == TokenType::DotDot { self.advance(); let e: f64 = self.advance().value.parse().unwrap_or(0.0); return Ok(MatchPattern::Range(Box::new(Node::Number(v)), Box::new(Node::Number(e)))); }
                Ok(MatchPattern::Literal(Node::Number(v)))
            }
            TokenType::Str => Ok(MatchPattern::Literal(Node::Str(self.advance().value))),
            TokenType::Bool => Ok(MatchPattern::Literal(Node::Bool(self.advance().value == "true"))),
            TokenType::Null => { self.advance(); Ok(MatchPattern::Literal(Node::Null)) }
            TokenType::Ident => Ok(MatchPattern::Ident(self.advance().value)),
            _ => Err(format!("Line {}: expected pattern", self.cur().line))
        }
    }
    fn parse_try(&mut self) -> Result<Node, String> {
        self.advance();
        self.expect(TokenType::LBrace)?; let body = self.parse_block()?; self.expect(TokenType::RBrace)?;
        self.expect(TokenType::Catch)?;
        let catch_var = self.expect(TokenType::Ident)?.value;
        self.expect(TokenType::LBrace)?; let catch_body = self.parse_block()?; self.expect(TokenType::RBrace)?;
        Ok(Node::Try { body, catch_var, catch_body })
    }
    fn parse_expr(&mut self) -> Result<Node, String> { self.parse_ternary() }
    fn parse_ternary(&mut self) -> Result<Node, String> {
        let left = self.parse_pipe()?;
        if self.cur().kind == TokenType::Question {
            self.advance();
            let then = self.parse_expr()?;
            self.expect(TokenType::Colon)?;
            let else_ = self.parse_expr()?;
            return Ok(Node::Ternary(Box::new(left), Box::new(then), Box::new(else_)));
        }
        Ok(left)
    }
    fn parse_pipe(&mut self) -> Result<Node, String> {
        let left = self.parse_in()?;
        if self.cur().kind == TokenType::Pipe && self.peek().kind != TokenType::Pipe {
            let mut stages = vec![left];
            while self.cur().kind == TokenType::Pipe && self.peek().kind != TokenType::Pipe { self.advance(); stages.push(self.parse_in()?); }
            return Ok(Node::Pipe(stages));
        }
        Ok(left)
    }
    fn parse_in(&mut self) -> Result<Node, String> {
        let left = self.parse_comparison()?;
        if self.cur().kind == TokenType::In {
            self.advance();
            let container = self.parse_comparison()?;
            return Ok(Node::In { value: Box::new(left), container: Box::new(container) });
        }
        Ok(left)
    }
    fn parse_comparison(&mut self) -> Result<Node, String> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.cur().kind {
                TokenType::EqEq => "==", TokenType::Neq => "!=",
                TokenType::Lt => "<", TokenType::Gt => ">",
                TokenType::LtEq => "<=", TokenType::GtEq => ">=",
                TokenType::AndAnd => "&&", TokenType::OrOr => "||",
                _ => break,
            }.to_string();
            self.advance();
            left = Node::BinOp(Box::new(left), op, Box::new(self.parse_additive()?));
        }
        Ok(left)
    }
    fn parse_additive(&mut self) -> Result<Node, String> {
        let mut left = self.parse_multiplicative()?;
        while matches!(self.cur().kind, TokenType::Plus | TokenType::Minus) { let op = self.advance().value; left = Node::BinOp(Box::new(left), op, Box::new(self.parse_multiplicative()?)); }
        Ok(left)
    }
    fn parse_multiplicative(&mut self) -> Result<Node, String> {
        let mut left = self.parse_unary()?;
        while matches!(self.cur().kind, TokenType::Star | TokenType::Slash | TokenType::Percent) { let op = self.advance().value; left = Node::BinOp(Box::new(left), op, Box::new(self.parse_unary()?)); }
        Ok(left)
    }
    fn parse_unary(&mut self) -> Result<Node, String> {
        if self.cur().kind == TokenType::Amp { self.advance(); return Ok(Node::Ref(Box::new(self.parse_primary()?))); }
        if self.cur().kind == TokenType::Minus { self.advance(); return Ok(Node::Unary("-".into(), Box::new(self.parse_primary()?))); }
        if self.cur().kind == TokenType::Bang { self.advance(); return Ok(Node::Unary("!".into(), Box::new(self.parse_primary()?))); }
        self.parse_postfix()
    }
    fn parse_postfix(&mut self) -> Result<Node, String> {
        let mut node = self.parse_primary()?;
        loop {
            if self.cur().kind == TokenType::QuestionDot {
                self.advance();
                let name = self.read_any_ident()?;
                if self.cur().kind == TokenType::LParen {
                    self.advance(); let args = self.parse_args()?; self.expect(TokenType::RParen)?;
                    node = Node::OptionalCall { obj: Box::new(node), method: name, args };
                } else {
                    node = Node::OptionalAccess { obj: Box::new(node), field: name };
                }
            } else if self.cur().kind == TokenType::Dot {
                self.advance();
                let name = self.read_any_ident()?;
                if self.cur().kind == TokenType::LParen {
                    self.advance(); let args = self.parse_args()?; self.expect(TokenType::RParen)?;
                    const STDLIB: &[&str] = &["io","math","str","strings","arr","json","time","env","fs","process","crypto","rand","fmt","sys","path","buf","regex","net","http","ws","db","cache","log","queue","zip","csv","xml","smtp","uuid","hash","signal","term","bench","thread","tg","game","input","db","color","os","math2","str2","net2","type","io2","arr2","json2","event","num","pickle","hashmap","set","zip2","xml","html","smtp","qr","matrix2","ini","toml","color2","base","bit","dns","signal2","pack","math3"];
                    if let Node::Ident(ref module) = node {
                        if STDLIB.contains(&module.as_str()) { node = Node::FuncCall { name: format!("{}.{}", module, name), args }; }
                        else { node = Node::MethodCall { obj: Box::new(Node::Ident(module.clone())), method: name, args }; }
                    } else { node = Node::MethodCall { obj: Box::new(node), method: name, args }; }
                } else { node = Node::FieldAccess { obj: Box::new(node), field: name }; }
            } else if self.cur().kind == TokenType::LBracket {
                self.advance(); let idx = self.parse_expr()?; self.expect(TokenType::RBracket)?;
                node = Node::Index { obj: Box::new(node), idx: Box::new(idx) };
            } else { break; }
        }
        Ok(node)
    }
    fn parse_interp(&self, raw: &str) -> Vec<InterpPart> {
        let mut parts = Vec::new(); let mut current = String::new();
        let chars: Vec<char> = raw.chars().collect(); let mut i = 0;
        while i < chars.len() {
            if chars[i] == '{' && i+1 < chars.len() && chars[i+1] != '{' {
                if !current.is_empty() { parts.push(InterpPart::Literal(current.clone())); current.clear(); }
                i += 1; let mut expr_str = String::new();
                while i < chars.len() && chars[i] != '}' { expr_str.push(chars[i]); i += 1; } i += 1;
                let toks = crate::lexer::Lexer::new(&expr_str).tokenize();
                if let Ok(mut ast) = Parser::new(toks).parse() { if !ast.is_empty() { parts.push(InterpPart::Expr(Box::new(ast.remove(0)))); } }
            } else { current.push(chars[i]); i += 1; }
        }
        if !current.is_empty() { parts.push(InterpPart::Literal(current)); }
        parts
    }
    fn parse_primary(&mut self) -> Result<Node, String> {
        match self.cur().kind.clone() {
            TokenType::Number => { let v: f64 = self.advance().value.parse().unwrap_or(0.0); Ok(Node::Number(v)) }
            TokenType::Float => { let v: f64 = self.advance().value.parse().unwrap_or(0.0); Ok(Node::Number(v)) }
            TokenType::Null => { self.advance(); Ok(Node::Null) }
            TokenType::Str => {
                let s = self.advance().value;
                if s.contains('{') { let parts = self.parse_interp(&s); if parts.iter().any(|p| matches!(p, InterpPart::Expr(_))) { return Ok(Node::Interp(parts)); } }
                Ok(Node::Str(s))
            }
            TokenType::Bool => Ok(Node::Bool(self.advance().value == "true")),
            TokenType::Pipe => {
                self.advance();
                let mut params = Vec::new();
                while self.cur().kind != TokenType::Pipe { params.push(self.expect(TokenType::Ident)?.value); if self.cur().kind == TokenType::Comma { self.advance(); } }
                self.expect(TokenType::Pipe)?;
                let body = if self.cur().kind == TokenType::LBrace { self.advance(); let s = self.parse_block()?; self.expect(TokenType::RBrace)?; Node::Array(s) } else { self.parse_expr()? };
                Ok(Node::Lambda { params, body: Box::new(body) })
            }
            TokenType::DotDotDot => { self.advance(); Ok(Node::Spread(Box::new(self.parse_primary()?))) }
            TokenType::Thread => {
                let name = self.advance().value;
                if self.cur().kind == TokenType::Dot {
                    self.advance();
                    let method = self.read_any_ident()?;
                    if self.cur().kind == TokenType::LParen {
                        self.advance();
                        let args = self.parse_args()?;
                        self.expect(TokenType::RParen)?;
                        return Ok(Node::FuncCall { name: format!("thread.{}", method), args });
                    }
                    return Ok(Node::FieldAccess { obj: Box::new(Node::Ident(name)), field: method });
                }
                Ok(Node::Ident(name))
            }
            TokenType::Ident => {
                let name = self.advance().value;
                if self.cur().kind == TokenType::LParen { self.advance(); let args = self.parse_args()?; self.expect(TokenType::RParen)?; return Ok(Node::FuncCall { name, args }); }
                if self.cur().kind == TokenType::LBrace && name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    self.advance(); let mut pairs = Vec::new();
                    while self.cur().kind != TokenType::RBrace {
                        let fname = self.expect(TokenType::Ident)?.value; self.expect(TokenType::Colon)?;
                        pairs.push((fname, self.parse_expr()?)); if self.cur().kind == TokenType::Comma { self.advance(); }
                    }
                    self.expect(TokenType::RBrace)?; return Ok(Node::StructInit { name, fields: pairs });
                }
                Ok(Node::Ident(name))
            }
            TokenType::LBracket => {
                self.advance(); let mut elements = Vec::new();
                while self.cur().kind != TokenType::RBracket { elements.push(self.parse_expr()?); if self.cur().kind == TokenType::Comma { self.advance(); } }
                self.expect(TokenType::RBracket)?; Ok(Node::Array(elements))
            }
            TokenType::LParen => {
                self.advance(); let mut elements = Vec::new();
                while self.cur().kind != TokenType::RParen { elements.push(self.parse_expr()?); if self.cur().kind == TokenType::Comma { self.advance(); } }
                self.expect(TokenType::RParen)?;
                if elements.len() == 1 { Ok(elements.remove(0)) } else { Ok(Node::Tuple(elements)) }
            }
            _ => Err(format!("Line {}: unexpected token {:?}", self.cur().line, self.cur().value))
        }
    }
    fn read_any_ident(&mut self) -> Result<String, String> {
        let val = self.cur().value.clone();
        match self.cur().kind {
            TokenType::Ident | TokenType::Send | TokenType::Recv |
            TokenType::Thread | TokenType::Async | TokenType::Await => { self.advance(); Ok(val) }
            _ => Err(format!("Line {}: expected Ident, got {:?}", self.cur().line, val))
        }
    }

    fn parse_args(&mut self) -> Result<Vec<Node>, String> {
        let mut args = Vec::new();
        while self.cur().kind != TokenType::RParen { args.push(self.parse_expr()?); if self.cur().kind == TokenType::Comma { self.advance(); } }
        Ok(args)
    }
}
