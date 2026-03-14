use crate::value::Value;
pub fn call(func: &str, args: Vec<Value>) -> Result<Value, String> {
    let n1 = || match args.first() { Some(Value::Int(n)) => *n, _ => 0 };
    let n2 = || match args.get(1) { Some(Value::Int(n)) => *n, _ => 0 };
    match func {
        "and" => Ok(Value::Int(n1() & n2())),
        "or"  => Ok(Value::Int(n1() | n2())),
        "xor" => Ok(Value::Int(n1() ^ n2())),
        "not" => Ok(Value::Int(!n1())),
        "shl" => Ok(Value::Int(n1() << n2())),
        "shr" => Ok(Value::Int(n1() >> n2())),
        "get_bit" => Ok(Value::Int((n1() >> n2()) & 1)),
        "set_bit" => Ok(Value::Int(n1() | (1 << n2()))),
        "clear_bit" => Ok(Value::Int(n1() & !(1 << n2()))),
        "count_ones" => Ok(Value::Int(n1().count_ones() as i64)),
        "count_zeros" => Ok(Value::Int(n1().count_zeros() as i64)),
        "leading_zeros" => Ok(Value::Int(n1().leading_zeros() as i64)),
        "trailing_zeros" => Ok(Value::Int(n1().trailing_zeros() as i64)),
        "is_power_of_two" => Ok(Value::Bool(n1() > 0 && (n1() & (n1()-1)) == 0)),
        _ => Err(format!("bit.{}: unknown", func))
    }
}
