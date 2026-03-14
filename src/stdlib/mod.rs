use crate::value::Value;

pub mod io; pub mod math; pub mod strings; pub mod arr; pub mod json;
pub mod time; pub mod env; pub mod fs; pub mod process; pub mod crypto;
pub mod rand; pub mod fmt; pub mod sys; pub mod path; pub mod buf;
pub mod regex; pub mod net; pub mod log; pub mod uuid; pub mod term;
pub mod bench; pub mod csv; pub mod hash;
pub mod thread;
pub mod tg;
pub mod game;
pub mod input;
pub mod db;
pub mod color;
pub mod os;
pub mod math2;
pub mod str2;
pub mod net2;
pub mod type_;
pub mod io2;
pub mod arr2;
pub mod json2;
pub mod event;
pub mod num;

pub fn call(module: &str, func: &str, args: Vec<Value>) -> Result<Value, String> {
    match module {
        "io" => io::call(func, args),
        "math" => math::call(func, args),
        "str" | "strings" => strings::call(func, args),
        "arr" => arr::call(func, args),
        "json" => json::call(func, args),
        "time" => time::call(func, args),
        "env" => env::call(func, args),
        "fs" => fs::call(func, args),
        "process" => process::call(func, args),
        "crypto" => crypto::call(func, args),
        "rand" => rand::call(func, args),
        "fmt" => fmt::call(func, args),
        "sys" => sys::call(func, args),
        "path" => path::call(func, args),
        "buf" => buf::call(func, args),
        "regex" => regex::call(func, args),
        "net" | "http" | "tcp" => net::call(module, func, args),
        "log" => log::call(func, args),
        "uuid" => uuid::call(func, args),
        "term" => term::call(func, args),
        "bench" => bench::call(func, args),
        "csv" => csv::call(func, args),
        "hash" => hash::call(func, args),
        "thread" => thread::call(func, args),
        "tg" => tg::call(func, args),
        "game" => game::call(func, args),
        "input" => input::call(func, args),
        "db" => db::call(func, args),
        "color" => color::call(func, args),
        "os" => os::call(func, args),
        "math2" => math2::call(func, args),
        "str2" => str2::call(func, args),
        "net2" => net2::call(func, args),
        "type" => type_::call(func, args),
        "io2" => io2::call(func, args),
        "arr2" => arr2::call(func, args),
        "json2" => json2::call(func, args),
        "event" => event::call(func, args),
        "num" => num::call(func, args),
        _ => Err(format!("{}.{}: unknown module", module, func))
    }
}
