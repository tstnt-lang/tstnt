<div align="center">

```
  _____  ___  _____  _  _ _____
 |_   _|/ __||_   _|| \| |_   _|
   | |  \__ \  | |  | .` | | |
   |_|  |___/  |_|  |_|\_| |_|
```

**A modern, fast, and expressive programming language**  
*Built on Android. Runs everywhere Rust does.*

[![Version](https://img.shields.io/badge/version-0.8.0-7fffb2?style=flat-square)](https://github.com/tstnt-lang/tstnt/releases)
[![License](https://img.shields.io/badge/license-MIT-7fb2ff?style=flat-square)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/built_with-Rust-ff7f7f?style=flat-square)](https://rust-lang.org)
[![Packages](https://img.shields.io/badge/packages-100+-ffcc7f?style=flat-square)](https://github.com/tstnt-lang/packages)

[Website](https://tstnt-lang.github.io) · [Docs](https://tstnt-lang.github.io/docs.html) · [Packages](https://github.com/tstnt-lang/packages) · [Changelog](#changelog)

</div>

---

## What is TSTNT?

TSTNT is a statically-typed scripting language with clean, readable syntax — inspired by Rust and Python. It ships as a single binary with zero external dependencies, a built-in package manager, database, Telegram API, transpiler to Python/JS, LSP server, and 100+ community packages.

```
use "colors"
use "logger"

struct Player {
    name: str
    hp:   int
    level: int
}

impl Player {
    do greet -> str {
        return "Hero: " + self.name + " · Lv." + str(self.level)
    }
}

do main {
    let p = Player { name: "Alex", hp: 100, level: 5 }
    print(colors.green(p.greet()))
    logger.info("Game started!")

    let scores = [42, 87, 13, 95, 61]
    let top    = filter(scores, |x| x > 60)
    let total  = reduce(top, |acc x| acc + x, 0)
    print("Top scores total: " + str(total))
}
```

---

## Features

| | Feature | Description |
|---|---|---|
| ⚡ | **Zero dependencies** | Single binary. No runtime, no VM, no install hell |
| 📦 | **Package manager** | `tstnt pkg install logger` — 100+ packages on GitHub |
| 🤖 | **Telegram built-in** | First-class bot API with keyboards, callbacks, RPG engine |
| 🔄 | **Transpiler** | Convert TSTNT → Python or JavaScript automatically |
| 🧪 | **Built-in testing** | `test` blocks with `assert_eq`, run with `tstnt test` |
| 🗄 | **Built-in database** | Persistent key-value store, no SQLite needed |
| 🔍 | **LSP server** | Autocomplete + hover for Neovim and VSCode |
| 👁 | **Watch mode** | Auto-restart on file change |
| 🧵 | **Threads & mutex** | Real OS threads with shared state |
| 📱 | **Android native** | Designed and built entirely on Termux |
| 🎮 | **Game library** | 2D vectors, collision, ASCII canvas, inventory, dice |
| 🔐 | **Crypto built-in** | SHA256, MD5, Base64, hex — no dependencies |

---

## Installation

### Android (Termux)

```bash
pkg install rust git
git clone https://github.com/tstnt-lang/tstnt
cd tstnt
cargo build --release
mkdir -p ~/bin
cp target/release/tstnt ~/bin/tstnt
echo 'export PATH=$PATH:~/bin' >> ~/.bashrc && source ~/.bashrc
tstnt --version
```

### Linux / macOS

```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

git clone https://github.com/tstnt-lang/tstnt
cd tstnt
cargo build --release
sudo cp target/release/tstnt /usr/local/bin/
tstnt --version
```

---

## Quick Start

```bash
# Create a file
echo 'do main { print("hello world") }' > hello.tstnt

# Run it
tstnt hello.tstnt

# Interactive REPL
tstnt repl

# Install packages
tstnt pkg install logger
tstnt pkg install colors
tstnt pkg search

# Watch mode (auto-restart on save)
tstnt watch hello.tstnt

# Run tests
tstnt test tests.tstnt

# Transpile to Python
tstnt transpile hello.tstnt py

# Transpile to JavaScript
tstnt transpile hello.tstnt js
```

---

## Language Overview

<details>
<summary><strong>Variables & Types</strong></summary>

```
let x: int   = 42
let f: float = 3.14
let s: str   = "hello"
let b: bool  = true
let arr      = [1, 2, 3]
let mut n    = 0          # mutable

n += 1
n -= 1
n *= 2

let (a, b) = (10, 20)    # multi-assign
let c = [0, ...arr, 4]   # spread
```

Types: `int` `float` `str` `bool` `[T]` `(T, U)` `null` `any`

</details>

<details>
<summary><strong>Functions & Generics</strong></summary>

```
do add(a: int, b: int) -> int {
    return a + b
}

do greet(name: str) {
    print("Hello, " + name)
}

async do fetch(url: str) -> str {
    return await net.get(url)
}

# Generics
do max_val<T>(a: T, b: T) -> T {
    return a > b ? a : b
}

print(max_val(3, 7))        # 7
print(max_val("a", "z"))    # z
```

</details>

<details>
<summary><strong>Structs & Impl</strong></summary>

```
struct User {
    name: str
    age:  int
}

impl User {
    do greet -> str {
        return "Hi, I am " + self.name
    }
    do is_adult -> bool {
        return self.age >= 18
    }
}

let u = User { name: "Alice", age: 25 }
print(u.greet())
print(u.is_adult())
```

</details>

<details>
<summary><strong>Match & Pattern Matching</strong></summary>

```
match score {
    100      -> print("perfect!")
    80..99   -> print("great")
    60..79   -> print("ok")
    _        -> print("try again")
}

# Match with guard
match n {
    x if x < 0 -> print("negative")
    x if x > 0 -> print("positive")
    _           -> print("zero")
}

# Match on null
match user?.name {
    null -> print("no user")
    name -> print("Hello, " + name)
}
```

</details>

<details>
<summary><strong>Loops & Functional</strong></summary>

```
# Range
loop i in 0..10 { print(i) }

# Array
loop item in fruits { print(item) }

# Enumerate
loop i, item in fruits {
    print(str(i) + ": " + item)
}

# Repeat N times
repeat 5 { print("hi") }

# Functional
let nums    = [1, 2, 3, 4, 5]
let doubled = map(nums, |x| x * 2)
let evens   = filter(nums, |x| x % 2 == 0)
let total   = reduce(nums, |acc x| acc + x, 0)

# Pipe
let result = value | double | str
```

</details>

<details>
<summary><strong>Error Handling</strong></summary>

```
try {
    throw "something went wrong"
} catch e {
    print("Caught: " + e)
}
```

</details>

<details>
<summary><strong>Tests</strong></summary>

```
do add(a: int, b: int) -> int { return a + b }

test addition {
    assert_eq(add(1, 2), 3)
    assert_ne(add(1, 1), 3)
    assert(add(5, 5) > 9)
}

test strings {
    assert_eq(len("hello"), 5)
    assert_eq("a" + "b", "ab")
}
```

```bash
tstnt test myfile.tstnt
# ✓ addition
# ✓ strings
# 2/2 passed
```

</details>

<details>
<summary><strong>Telegram Bot</strong></summary>

```
use tg
use thread

do main {
    tg.token("YOUR_BOT_TOKEN")
    tg.delete_webhook()

    let mut offset = 0
    while true {
        let updates = tg.get_updates(offset)
        loop upd in updates {
            offset = upd.update_id + 1
            if upd.text == "/start" {
                tg.send_keyboard(upd.chat_id,
                    "Welcome! Choose an action:",
                    [["⚔️ Fight", "🏪 Shop"], ["📊 Stats", "🗺 Quest"]])
            } else {
                tg.send(upd.chat_id, "You said: " + upd.text)
            }
        }
        thread.sleep(500)
    }
}
```

</details>

---

## CLI Reference

```
tstnt <file.tstnt>               Run a file
tstnt repl                        Interactive shell
tstnt test <file.tstnt>          Run test blocks
tstnt build <file.tstnt>         Compile to .tst bytecode
tstnt run <file.tst>             Run compiled bytecode
tstnt watch <file.tstnt>         Auto-restart on change
tstnt fmt <file.tstnt>           Format code
tstnt transpile <file> [py|js]   Transpile to Python or JS
tstnt pkg install <name>         Install a package
tstnt pkg uninstall <name>       Remove a package
tstnt pkg list                   List installed packages
tstnt pkg search [query]         Search available packages
tstnt --version                  Version + ASCII art
tstnt --secret                   🐉
```

---

## Standard Library

| Module | Key Functions |
|--------|--------------|
| `io` | `print`, `input`, `read_file`, `write_file` |
| `math` | `sqrt`, `pow`, `abs`, `floor`, `ceil`, `min`, `max` |
| `strings` | `split`, `join`, `trim`, `upper`, `lower`, `contains`, `replace` |
| `arr` | `push`, `pop`, `first`, `last`, `reverse`, `contains` |
| `json` | `parse`, `stringify` |
| `fs` | `read`, `write`, `append`, `exists`, `delete`, `mkdir`, `ls` |
| `crypto` | `sha256`, `md5`, `base64_encode`, `base64_decode`, `hex_encode` |
| `rand` | `int`, `float`, `bool`, `choice`, `shuffle` |
| `time` | `now`, `sleep` |
| `env` | `get`, `set`, `args` |
| `sys` | `os`, `arch`, `cwd`, `home`, `hostname`, `cpu_count` |
| `db` | `set`, `get`, `has`, `delete`, `keys`, `count`, `incr` |
| `term` | `red`, `green`, `yellow`, `blue`, `bold`, `dim`, `reset` |
| `tg` | `token`, `send`, `send_keyboard`, `send_inline`, `get_updates` |
| `thread` | `spawn`, `sleep`, `mutex_new`, `mutex_get`, `mutex_set` |
| `game` | `set`, `get`, `inv_add`, `inv_list`, `roll`, `clamp`, `lerp` |
| `bench` | `now_ms`, `now_us`, `elapsed` |
| `hash` | `fnv32`, `fnv64`, `crc32`, `djb2` |
| `uuid` | `v4`, `nil` |
| `log` | `info`, `warn`, `error`, `debug` |

---

## Packages

100+ packages available at [github.com/tstnt-lang/packages](https://github.com/tstnt-lang/packages)

```bash
tstnt pkg install logger        # Colored logging
tstnt pkg install colors        # Terminal colors
tstnt pkg install stats         # mean, median, std_dev
tstnt pkg install fake-data     # Generate test data
tstnt pkg install auth          # Register, login, sessions
tstnt pkg install tg-rpg        # Telegram RPG engine
tstnt pkg install game-2d       # 2D vectors and ASCII canvas
tstnt pkg install ansi          # Full ANSI terminal control
tstnt pkg install benchmark-suite  # Benchmark runner
tstnt pkg install migrate       # Database migrations
```

Use in code:

```
use "logger"
use "ansi"
use "ansi/effects"    # multi-file packages supported

do main {
    logger.info("Starting...")
    print(ansi.rgb(255, 100, 0, "Orange!"))
    print(effects.rainbow("TSTNT"))
}
```

---

## Editor Support

### Neovim

```bash
mkdir -p ~/.config/nvim/syntax
cp editor/neovim/tstnt.vim ~/.config/nvim/syntax/
```

Add to `~/.config/nvim/init.vim`:
```vim
au BufRead,BufNewFile *.tstnt set filetype=tstnt
```

### VSCode

Copy `editor/vscode/` to `~/.vscode/extensions/tstnt-lang/`

### LSP Server

```bash
cp target/release/tstnt-lsp ~/bin/
```

Configure in your editor to use `tstnt-lsp` for `.tstnt` files.

---

## Project Structure

```
tstnt/
├── src/
│   ├── main.rs           CLI entry point
│   ├── lexer.rs          Tokenizer
│   ├── parser.rs         AST parser
│   ├── interpreter.rs    Tree-walking interpreter
│   ├── value.rs          Value types
│   ├── compiler.rs       Bytecode compiler
│   ├── transpiler.rs     Python/JS transpiler
│   ├── repl.rs           Interactive REPL
│   ├── formatter.rs      Code formatter
│   ├── pkg.rs            Package manager
│   ├── stdlib/           30+ built-in modules
│   │   ├── mod.rs
│   │   ├── io.rs
│   │   ├── math.rs
│   │   ├── tg.rs         Telegram API
│   │   ├── db.rs         Key-value database
│   │   ├── game.rs       Game utilities
│   │   └── ...
│   ├── vm/               Bytecode VM
│   │   ├── mod.rs
│   │   ├── opcode.rs
│   │   ├── chunk.rs
│   │   └── codegen.rs
│   ├── lsp/              LSP server
│   │   └── main.rs
│   └── builtin_pkgs/     Offline package cache
│       ├── colors.tstnt
│       ├── logger.tstnt
│       └── ...
├── editor/
│   ├── neovim/tstnt.vim
│   └── vscode/tstnt.tmLanguage.json
├── test.tstnt
├── test_suite.tstnt
└── test_tg.tstnt
```

---

## Changelog

### v0.8.0
- `obj?.field` and `obj?.method()` — optional chaining
- `db` module — persistent key-value database
- 11 new built-in modules: `color`, `os`, `math2`, `str2`, `net2`, `type`, `io2`, `arr2`, `json2`, `event`, `num`
- Transpiler: `tstnt transpile file.tstnt py|js`
- Syntax highlighting for Neovim and VSCode
- Easter eggs in `print()` and `--secret`

### v0.7.0
- Watch mode: `tstnt watch`
- Beautiful error messages with line pointer
- ASCII art version banner
- `tstnt pkg install` downloads from `github.com/tstnt-lang/packages`

### v0.6.0
- `loop i, item in arr` — enumerate
- `if` as expression
- Functions as values
- `game`, `input` modules
- 11 built-in packages

### v0.5.0
- `thread.spawn`, mutex, real OS threads
- Generics: `do max<T>(a: T, b: T) -> T`
- LSP server (`tstnt-lsp`)
- `tg` module — Telegram bot API

---

## License

MIT — see [LICENSE](LICENSE)

---

<div align="center">
  <sub>Built with ❤️ on Android · <a href="https://tstnt-lang.github.io">tstnt-lang.github.io</a></sub>
</div>
