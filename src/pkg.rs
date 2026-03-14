use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn pkg_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".tstnt").join("packages")
}

fn installed_path(name: &str) -> PathBuf { pkg_dir().join(name) }

fn curl_download(url: &str, dest: &str) -> bool {
    Command::new("curl")
        .args(["-s", "--max-time", "15", "-o", dest, url])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

// Встроенные пакеты (работают без интернета)
fn builtin(name: &str) -> Option<&'static str> {
    match name {
        "colors" => Some(include_str!("builtin_pkgs/colors.tstnt")),
        "math-extra" => Some(include_str!("builtin_pkgs/math-extra.tstnt")),
        "strings-extra" => Some(include_str!("builtin_pkgs/strings-extra.tstnt")),
        "dotenv" => Some(include_str!("builtin_pkgs/dotenv.tstnt")),
        "rpg-engine" => Some(include_str!("builtin_pkgs/rpg-engine.tstnt")),
        "validation" => Some(include_str!("builtin_pkgs/validation.tstnt")),
        "array-extra" => Some(include_str!("builtin_pkgs/array-extra.tstnt")),
        "datetime" => Some(include_str!("builtin_pkgs/datetime.tstnt")),
        "cli" => Some(include_str!("builtin_pkgs/cli.tstnt")),
        "http-client" => Some(include_str!("builtin_pkgs/http-client.tstnt")),
        "tg-bot" => Some(include_str!("builtin_pkgs/tg-bot.tstnt")),
        _ => None,
    }
}

pub fn install(name: &str) {
    if name.is_empty() { eprintln!("Usage: tstnt pkg install <name>"); return; }
    let dir = installed_path(name);
    if dir.exists() { println!("Already installed: {}", name); return; }
    fs::create_dir_all(&dir).ok();
    let file = dir.join("main.tstnt");

    // Сначала встроенный
    if let Some(code) = builtin(name) {
        fs::write(&file, code).ok();
        println!("✓ Installed: {}", name);
        return;
    }

    // Потом GitHub
    let url = format!("https://raw.githubusercontent.com/tstnt-lang/packages/main/{}/main.tstnt", name);
    println!("Downloading {} from github.com/tstnt-lang/packages...", name);
    if curl_download(&url, file.to_str().unwrap()) {
        println!("✓ Installed: {}", name);
    } else {
        fs::remove_dir_all(&dir).ok();
        eprintln!("✗ Package not found: {}", name);
        eprintln!("  Available: tstnt pkg search");
    }
}

pub fn uninstall(name: &str) {
    let dir = installed_path(name);
    if dir.exists() { fs::remove_dir_all(&dir).ok(); println!("✓ Uninstalled: {}", name); }
    else { eprintln!("Not installed: {}", name); }
}

pub fn list() {
    let dir = pkg_dir();
    if !dir.exists() { println!("No packages installed."); return; }
    match fs::read_dir(&dir) {
        Ok(entries) => {
            let pkgs: Vec<_> = entries.flatten().map(|e| e.file_name().to_string_lossy().to_string()).collect();
            if pkgs.is_empty() { println!("No packages installed."); }
            else { println!("Installed packages:"); for p in pkgs { println!("  {}", p); } }
        }
        Err(_) => println!("No packages installed.")
    }
}

pub fn search(query: &str) {
    let all = [
        ("colors", "Terminal colors: red, green, bold..."),
        ("math-extra", "factorial, fib, gcd, is_prime..."),
        ("strings-extra", "slugify, pad, repeat_str..."),
        ("dotenv", "Load .env files"),
        ("rpg-engine", "RPG structs: Player, Enemy, calc_damage..."),
        ("validation", "is_email, is_url, is_number..."),
        ("array-extra", "any_match, all_match, count_val..."),
        ("datetime", "now_sec, format_time, sleep_sec..."),
        ("cli", "arg, flag, usage..."),
        ("http-client", "get, post, get_json..."),
        ("tg-bot", "bot_start, bot_run, reply..."),
    ];
    println!("Packages (github.com/tstnt-lang/packages):\n");
    for (name, desc) in all {
        if query.is_empty() || name.contains(query) || desc.contains(query) {
            println!("  {:20} {}", name, desc);
        }
    }
}
