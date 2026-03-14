use std::collections::HashMap;
use std::fmt;
use crate::parser::Node;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64), Float(f64), Str(String), Bool(bool),
    Array(Vec<Value>), Tuple(Vec<Value>),
    Struct(String, HashMap<String, Value>),
    Lambda(Vec<String>, Node),
    Null,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => { if n.fract() == 0.0 { write!(f, "{}", *n as i64) } else { write!(f, "{}", n) } }
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::Lambda(p, _) => write!(f, "<lambda({})>", p.join(", ")),
            Value::Array(a) => { write!(f, "[")?; for (i, v) in a.iter().enumerate() { if i > 0 { write!(f, ", ")?; } write!(f, "{}", v)?; } write!(f, "]") }
            Value::Tuple(t) => { write!(f, "(")?; for (i, v) in t.iter().enumerate() { if i > 0 { write!(f, ", ")?; } write!(f, "{}", v)?; } write!(f, ")") }
            Value::Struct(name, fields) => { write!(f, "{} {{", name)?; for (i, (k, v)) in fields.iter().enumerate() { if i > 0 { write!(f, ", ")?; } write!(f, "{}: {}", k, v)?; } write!(f, "}}") }
        }
    }
}
