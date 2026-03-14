#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Number, Float, Str, Bool, Ident,
    Do, Let, Mut, Return, If, Else, While, Loop, Match, Use, Mod, Struct, Unit, Enum, In,
    Break, Continue, Try, Catch, Throw,
    Async, Await, Thread, Chan, Send, Recv,
    Interface, Impl, Test,
    Null, Repeat,
    Plus, Minus, Star, Slash, Percent, Bang,
    Eq, EqEq, Neq, Lt, Gt, LtEq, GtEq, Amp, AndAnd, OrOr, Pipe,
    Question, QuestionDot, Colon2,
    PlusEq, MinusEq, StarEq, SlashEq,
    DotDotDot,
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Colon, Comma, Dot, DotDot, Arrow, Underscore,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenType,
    pub value: String,
    pub line: usize,
}

pub struct Lexer { src: Vec<char>, pos: usize, pub line: usize }

impl Lexer {
    pub fn new(src: &str) -> Self { Lexer { src: src.chars().collect(), pos: 0, line: 1 } }
    fn cur(&self) -> Option<char> { self.src.get(self.pos).copied() }
    fn peek(&self) -> Option<char> { self.src.get(self.pos + 1).copied() }
    fn peek2(&self) -> Option<char> { self.src.get(self.pos + 2).copied() }
    fn advance(&mut self) -> char {
        let c = self.src[self.pos]; self.pos += 1;
        if c == '\n' { self.line += 1; } c
    }
    fn skip_ws(&mut self) { while let Some(c) = self.cur() { if c.is_whitespace() { self.advance(); } else { break; } } }
    fn skip_comment(&mut self) { while let Some(c) = self.cur() { if c == '\n' { break; } self.advance(); } }
    fn read_string(&mut self) -> String {
        self.advance(); let mut s = String::new();
        while let Some(c) = self.cur() {
            if c == '\\' { self.advance(); match self.cur() { Some('n') => { self.advance(); s.push('\n'); } Some('t') => { self.advance(); s.push('\t'); } Some('"') => { self.advance(); s.push('"'); } Some('\\') => { self.advance(); s.push('\\'); } _ => {} } continue; }
            if c == '"' { self.advance(); break; } s.push(self.advance());
        } s
    }
    fn read_number(&mut self) -> (String, bool) {
        let mut s = String::new(); let mut f = false;
        while let Some(c) = self.cur() {
            if c.is_ascii_digit() { s.push(self.advance()); }
            else if c == '.' && self.peek() != Some('.') && !f { f = true; s.push(self.advance()); }
            else { break; }
        } (s, f)
    }
    fn read_ident(&mut self) -> String {
        let mut s = String::new();
        while let Some(c) = self.cur() { if c.is_alphanumeric() || c == '_' { s.push(self.advance()); } else { break; } } s
    }
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            self.skip_ws();
            let Some(c) = self.cur() else { break };
            let line = self.line;
            if c == '#' { self.skip_comment(); continue; }
            if c == '"' { let s = self.read_string(); tokens.push(Token { kind: TokenType::Str, value: s, line }); continue; }
            if c.is_ascii_digit() { let (s, f) = self.read_number(); tokens.push(Token { kind: if f { TokenType::Float } else { TokenType::Number }, value: s, line }); continue; }
            if c.is_alphabetic() || c == '_' {
                let s = self.read_ident();
                let kind = match s.as_str() {
                    "do" => TokenType::Do, "let" => TokenType::Let, "mut" => TokenType::Mut,
                    "return" => TokenType::Return, "if" => TokenType::If, "else" => TokenType::Else,
                    "while" => TokenType::While, "loop" => TokenType::Loop, "match" => TokenType::Match,
                    "use" => TokenType::Use, "mod" => TokenType::Mod, "struct" => TokenType::Struct,
                    "unit" => TokenType::Unit, "enum" => TokenType::Enum, "in" => TokenType::In,
                    "break" => TokenType::Break, "continue" => TokenType::Continue,
                    "try" => TokenType::Try, "catch" => TokenType::Catch, "throw" => TokenType::Throw,
                    "async" => TokenType::Async, "await" => TokenType::Await,
                    "thread" => TokenType::Thread, "chan" => TokenType::Chan,
                    "send" => TokenType::Send, "recv" => TokenType::Recv,
                    "interface" => TokenType::Interface, "impl" => TokenType::Impl,
                    "test" => TokenType::Test, "null" => TokenType::Null,
                    "repeat" => TokenType::Repeat,
                    "true" | "false" => TokenType::Bool, "_" => TokenType::Underscore,
                    _ => TokenType::Ident,
                };
                tokens.push(Token { kind, value: s, line }); continue;
            }
            // multi-char
            if c == '.' && self.peek() == Some('.') && self.peek2() == Some('.') { self.advance(); self.advance(); self.advance(); tokens.push(Token { kind: TokenType::DotDotDot, value: "...".into(), line }); continue; }
            if c == '.' && self.peek() == Some('.') { self.advance(); self.advance(); tokens.push(Token { kind: TokenType::DotDot, value: "..".into(), line }); continue; }
            if c == '-' && self.peek() == Some('>') { self.advance(); self.advance(); tokens.push(Token { kind: TokenType::Arrow, value: "->".into(), line }); continue; }
            if c == ':' && self.peek() == Some(':') { self.advance(); self.advance(); tokens.push(Token { kind: TokenType::Colon2, value: "::".into(), line }); continue; }
            if c == '&' && self.peek() == Some('&') { self.advance(); self.advance(); tokens.push(Token { kind: TokenType::AndAnd, value: "&&".into(), line }); continue; }
            if c == '|' && self.peek() == Some('|') { self.advance(); self.advance(); tokens.push(Token { kind: TokenType::OrOr, value: "||".into(), line }); continue; }
            if c == '+' && self.peek() == Some('=') { self.advance(); self.advance(); tokens.push(Token { kind: TokenType::PlusEq, value: "+=".into(), line }); continue; }
            if c == '-' && self.peek() == Some('=') { self.advance(); self.advance(); tokens.push(Token { kind: TokenType::MinusEq, value: "-=".into(), line }); continue; }
            if c == '*' && self.peek() == Some('=') { self.advance(); self.advance(); tokens.push(Token { kind: TokenType::StarEq, value: "*=".into(), line }); continue; }
            if c == '/' && self.peek() == Some('=') { self.advance(); self.advance(); tokens.push(Token { kind: TokenType::SlashEq, value: "/=".into(), line }); continue; }
            if c == '=' { self.advance(); if self.cur() == Some('=') { self.advance(); tokens.push(Token { kind: TokenType::EqEq, value: "==".into(), line }); } else { tokens.push(Token { kind: TokenType::Eq, value: "=".into(), line }); } continue; }
            if c == '?' && self.peek() == Some('.') { self.advance(); self.advance(); tokens.push(Token { kind: TokenType::QuestionDot, value: "?.".into(), line }); continue; }
            if c == '!' { self.advance(); if self.cur() == Some('=') { self.advance(); tokens.push(Token { kind: TokenType::Neq, value: "!=".into(), line }); } else { tokens.push(Token { kind: TokenType::Bang, value: "!".into(), line }); } continue; }
            if c == '<' { self.advance(); if self.cur() == Some('=') { self.advance(); tokens.push(Token { kind: TokenType::LtEq, value: "<=".into(), line }); } else { tokens.push(Token { kind: TokenType::Lt, value: "<".into(), line }); } continue; }
            if c == '>' { self.advance(); if self.cur() == Some('=') { self.advance(); tokens.push(Token { kind: TokenType::GtEq, value: ">=".into(), line }); } else { tokens.push(Token { kind: TokenType::Gt, value: ">".into(), line }); } continue; }
            let kind = match c {
                '+' => TokenType::Plus, '-' => TokenType::Minus, '*' => TokenType::Star,
                '/' => TokenType::Slash, '%' => TokenType::Percent,
                '(' => TokenType::LParen, ')' => TokenType::RParen,
                '{' => TokenType::LBrace, '}' => TokenType::RBrace,
                '[' => TokenType::LBracket, ']' => TokenType::RBracket,
                ':' => TokenType::Colon, ',' => TokenType::Comma, '.' => TokenType::Dot,
                '&' => TokenType::Amp, '|' => TokenType::Pipe, '?' => TokenType::Question,
                _ => { self.advance(); continue; }
            };
            self.advance();
            tokens.push(Token { kind, value: c.to_string(), line });
        }
        tokens.push(Token { kind: TokenType::Eof, value: "".into(), line: self.line });
        tokens
    }
}
