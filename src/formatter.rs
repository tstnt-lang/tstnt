pub fn format(src: &str) -> String {
    let mut out = String::new();
    let mut indent = 0usize;
    let mut in_string = false;
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c == '"' && (i == 0 || chars[i - 1] != '\\') {
            in_string = !in_string;
            out.push(c);
            i += 1;
            continue;
        }

        if in_string { out.push(c); i += 1; continue; }

        if c == '#' {
            while i < chars.len() && chars[i] != '\n' { out.push(chars[i]); i += 1; }
            continue;
        }

        if c == '{' {
            out.push(' ');
            out.push('{');
            out.push('\n');
            indent += 1;
            push_indent(&mut out, indent);
            i += 1;
            continue;
        }

        if c == '}' {
            if out.ends_with("    ") || out.ends_with('\t') {
                let trim_len = out.trim_end_matches(' ').len();
                out.truncate(trim_len);
            }
            indent = indent.saturating_sub(1);
            out.push('\n');
            push_indent(&mut out, indent);
            out.push('}');
            out.push('\n');
            push_indent(&mut out, indent);
            i += 1;
            continue;
        }

        if c == '\n' {
            let trimmed = out.trim_end_matches(' ').to_string();
            out = trimmed;
            out.push('\n');
            push_indent(&mut out, indent);
            i += 1;
            while i < chars.len() && (chars[i] == ' ' || chars[i] == '\t') { i += 1; }
            continue;
        }

        out.push(c);
        i += 1;
    }

    out.trim().to_string() + "\n"
}

fn push_indent(s: &mut String, n: usize) {
    for _ in 0..n { s.push_str("    "); }
}
