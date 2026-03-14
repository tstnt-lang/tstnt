use crate::parser::Node;

pub fn compile(ast: &[Node]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"TST\x02");
    let src = format!("{:?}", ast);
    let len = src.len() as u32;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(src.as_bytes());
    out
}
