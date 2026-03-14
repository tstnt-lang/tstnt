use crate::value::Value;

pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    match func {
        "format" => {
            match args.first() {
                Some(Value::Str(template)) => {
                    let mut result = template.clone();
                    for (i, arg) in args.iter().skip(1).enumerate() {
                        result = result.replacen("{}", &arg.to_string(), 1);
                        result = result.replace(&format!("{{{}}}", i), &arg.to_string());
                    }
                    Ok(Value::Str(result))
                }
                _ => Err("fmt.format: expected str template".into())
            }
        }
        "pad_left" => match (args.first(), args.get(1), args.get(2)) {
            (Some(Value::Str(s)), Some(Value::Int(n)), Some(Value::Str(ch))) => {
                let pad = ch.chars().next().unwrap_or(' ');
                let mut out = s.clone();
                while (out.len() as i64) < *n { out.insert(0, pad); }
                Ok(Value::Str(out))
            }
            _ => Err("fmt.pad_left: expected (str, int, str)".into())
        },
        "pad_right" => match (args.first(), args.get(1), args.get(2)) {
            (Some(Value::Str(s)), Some(Value::Int(n)), Some(Value::Str(ch))) => {
                let pad = ch.chars().next().unwrap_or(' ');
                let mut out = s.clone();
                while (out.len() as i64) < *n { out.push(pad); }
                Ok(Value::Str(out))
            }
            _ => Err("fmt.pad_right: expected (str, int, str)".into())
        },
        "repeat" => match (args.first(), args.get(1)) {
            (Some(Value::Str(s)), Some(Value::Int(n))) => {
                Ok(Value::Str(s.repeat(*n as usize)))
            }
            _ => Err("fmt.repeat: expected (str, int)".into())
        },
        "int_hex" => match args.first() {
            Some(Value::Int(n)) => Ok(Value::Str(format!("{:x}", n))),
            _ => Err("fmt.int_hex: expected int".into())
        },
        "int_bin" => match args.first() {
            Some(Value::Int(n)) => Ok(Value::Str(format!("{:b}", n))),
            _ => Err("fmt.int_bin: expected int".into())
        },
        "int_oct" => match args.first() {
            Some(Value::Int(n)) => Ok(Value::Str(format!("{:o}", n))),
            _ => Err("fmt.int_oct: expected int".into())
        },
        "float_fixed" => match (args.first(), args.get(1)) {
            (Some(Value::Float(f)), Some(Value::Int(n))) => {
                Ok(Value::Str(format!("{:.prec$}", f, prec = *n as usize)))
            }
            (Some(Value::Int(i)), Some(Value::Int(n))) => {
                Ok(Value::Str(format!("{:.prec$}", *i as f64, prec = *n as usize)))
            }
            _ => Err("fmt.float_fixed: expected (float, int)".into())
        },
        _ => Err(format!("fmt.{}: unknown function", func))
    }
}
