<div align="center">

<img src="tstnt_preview.png" alt="TSTNT Language" width="100%">

<br/>
<br/>

[![Version](https://img.shields.io/badge/version-1.0.5-3fb950?style=flat-square&logo=github)](https://github.com/tstnt-lang/tstnt/releases)
[![License](https://img.shields.io/badge/license-MIT-58a6ff?style=flat-square)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/built_with-Rust-f97316?style=flat-square&logo=rust)](https://rust-lang.org)
[![Packages](https://img.shields.io/badge/packages-140+-d29922?style=flat-square)](https://github.com/tstnt-lang/packages)
[![Docs](https://img.shields.io/badge/docs-tstnt--lang.github.io-8b949e?style=flat-square)](https://tstnt-lang.github.io)

**[Website](https://tstnt-lang.github.io) · [Docs](https://tstnt-lang.github.io/docs.html) · [Packages](https://github.com/tstnt-lang/packages)**

</div>

---

## Overview

TSTNT is a statically-typed scripting language with clean, readable syntax — inspired by Rust and Python.  
Ships as a **single binary** with zero external dependencies.

Built from scratch on Android (Termux) using Rust.

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
}
```

---

## What's new in v1.0.5

```
tstnt new <project>    Scaffold a new project with src/ tests/ pkg.json README.md
@cache @timer          Decorators on functions
@deprecated            Mark functions as deprecated
pickle                 Persist any value to file — save/load/list
hashmap                Built-in key-value map module
set                    Built-in set module with union/intersect/difference
default()              default(null, "fallback") → "fallback"
not_null()             not_null(42) → true
coalesce()             coalesce(null, null, "found") → "found"
clamp()                clamp(150, 0, 100) → 100
repeat_str()           repeat_str("ab", 3) → "ababab"
tap()                  prints value and returns it (debug helper)
+15 stdlib modules     xml html ini toml bit qr base color2 matrix2 dns smtp pack math3 zip2 signal2
+15 packages           tg-admin game-rpg formula cli-tools data-viz crypto-advanced...
Transpiler fixes       += loop-in enumerate spread struct-init now transpile correctly
```

---

## Installation

**Android (Termux)**

```bash
pkg install rust git
git clone https://github.com/tstnt-lang/tstnt
cd tstnt
cargo build --release
mkdir -p ~/bin && cp target/release/tstnt ~/bin/
echo 'export PATH=$PATH:~/bin' >> ~/.bashrc && source ~/.bashrc
```

**Linux / macOS**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/tstnt-lang/tstnt
cd tstnt && cargo build --release
sudo cp target/release/tstnt /usr/local/bin/
```

---

## Quick Start

```bash
tstnt new myproject         # scaffold project
tstnt src/main.tstnt        # run file
tstnt repl                   # interactive shell
tstnt test tests/test.tstnt # run tests
tstnt watch src/main.tstnt  # auto-restart on save
tstnt transpile f.tstnt py  # convert to Python
tstnt transpile f.tstnt js  # convert to JavaScript
tstnt pkg install logger    # install package
tstnt pkg search            # browse 140+ packages
tstnt --version             # show version
tstnt --secret              # 🐉
```

---

## Language

<details>
<summary>Variables & Types</summary>

```
let x: int    = 42
let f: float  = 3.14
let s: str    = "hello"
let b: bool   = true
let arr       = [1, 2, 3]
let mut n     = 0

n += 1
n -= 1
n *= 2

let (a, b) = (10, 20)
let merged = [0, ...arr]
```

Types: `int` `float` `str` `bool` `[T]` `(T,U)` `null` `any`

</details>

<details>
<summary>Functions, Generics & Decorators</summary>

```
do add(a: int, b: int) -> int {
    return a + b
}

do max_val<T>(a: T, b: T) -> T {
    return a > b ? a : b
}

@cache
do fib(n: int) -> int {
    if n <= 1 { return n }
    return fib(n-1) + fib(n-2)
}

@deprecated
do old_api() { }
```

</details>

<details>
<summary>Structs & Impl</summary>

```
struct User {
    name: str
    age:  int
}

impl User {
    do greet -> str { return "Hi, " + self.name }
    do is_adult -> bool { return self.age >= 18 }
}

let u = User { name: "Alice", age: 25 }
print(u.greet())
```

</details>

<details>
<summary>Match</summary>

```
match score {
    100      -> print("perfect!")
    80..99   -> print("great")
    _        -> print("keep going")
}

match n {
    x if x < 0 -> print("negative")
    x if x > 0 -> print("positive")
    _           -> print("zero")
}
```

</details>

<details>
<summary>Loops & Functional</summary>

```
loop i in 0..10 { print(i) }
loop item in fruits { print(item) }
loop i, item in fruits { print(str(i) + ": " + item) }
repeat 5 { print("hi") }

let doubled = map(nums, |x| x * 2)
let evens   = filter(nums, |x| x % 2 == 0)
let total   = reduce(nums, |acc x| acc + x, 0)
let result  = value | double | str
```

</details>

<details>
<summary>Optional Chaining & New Functions</summary>

```
let name = user?.profile?.name

default(null, "fallback")     # "fallback"
not_null(42)                   # true
coalesce(null, null, "found")  # "found"
clamp(150, 0, 100)             # 100
repeat_str("ab", 3)            # "ababab"
tap(value)                     # prints and returns
```

</details>

<details>
<summary>Pickle — Serialize anything</summary>

```
use pickle

let player = Player { name: "Alex", hp: 100, level: 5 }
pickle.save("session", player)

let loaded = pickle.load("session")
print(loaded.name)

print(pickle.list())
pickle.delete("session")
```

</details>

<details>
<summary>HashMap & Set</summary>

```
use hashmap
use set

let mut m = hashmap.new()
m = hashmap.set(m, "name", "Alice")
m = hashmap.set(m, "age", "25")
print(hashmap.get(m, "name"))
print(hashmap.keys(m))

let s = set.from([1, 2, 3, 2, 1])  # [1, 2, 3]
let s2 = set.add(s, 4)
print(set.union(s2, [5, 6]))
print(set.intersect(s, [2, 3, 4]))
```

</details>

<details>
<summary>Tests</summary>

```
test addition {
    assert_eq(1 + 1, 2)
    assert_ne("a", "b")
    assert(5 > 3)
}
```

```bash
tstnt test myfile.tstnt
# ✓ addition
# 1/1 passed
```

</details>

<details>
<summary>Telegram Bot</summary>

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
                tg.send_keyboard(upd.chat_id, "Welcome!",
                    [["Play", "Stats"], ["Help"]])
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

## CLI

```
tstnt new <project>              Scaffold project
tstnt <file>                     Run file
tstnt repl                        Interactive REPL
tstnt test <file>                Run tests
tstnt build <file>               Compile to .tst bytecode
tstnt run <file.tst>             Run bytecode
tstnt watch <file>               Auto-restart on change
tstnt fmt <file>                 Format code
tstnt transpile <f> [py|js]     Transpile
tstnt pkg install <n>         Install package
tstnt pkg uninstall <n>       Remove package
tstnt pkg list                   Installed packages
tstnt pkg search [query]         Browse 140+ packages
tstnt --version                  Version + ASCII art
tstnt --secret                   🐉
```

---

## Standard Library (50+ modules)

| Module | Description |
|--------|-------------|
| `io` | print, input, read/write files |
| `math` | sqrt, pow, abs, floor, ceil |
| `strings` | split, join, trim, upper, lower, replace |
| `arr` | push, pop, reverse, contains |
| `json` | parse, stringify |
| `fs` | read, write, append, exists, mkdir |
| `crypto` | sha256, md5, base64, hex |
| `rand` | int, float, bool, choice, shuffle |
| `time` | now, sleep |
| `db` | set, get, has, delete, keys, incr |
| `tg` | Telegram bot API |
| `thread` | spawn, sleep, mutex |
| `game` | state, inventory, dice, clamp |
| `pickle` | save, load, exists, list |
| `hashmap` | new, set, get, has, delete, keys, merge |
| `set` | new, add, remove, has, union, intersect |
| `xml` | tag, attr_tag, escape, wrap |
| `html` | tag, p, h1-h3, a, img, ul, table, page |
| `ini` | parse, get, stringify |
| `toml` | parse, get, get_bool, get_int |
| `bit` | and, or, xor, shl, shr, count_ones |
| `base` | to_bin, to_hex, to_oct, from_base |
| `math3` | factorial, fib, gcd, lcm, is_prime, lerp |
| `qr` | ascii, url |
| `color2` | rgb, bg, gradient_text, ocean, fire |
| `bench` | now_ms, elapsed |
| `term` | red, green, yellow, blue, bold |
| `sys` | os, arch, cwd, home, hostname |
| `uuid` | v4, nil |
| `hash` | fnv32, fnv64, crc32 |

---

## Packages (140+)

[github.com/tstnt-lang/packages](https://github.com/tstnt-lang/packages)

```bash
tstnt pkg install logger
tstnt pkg install tg-rpg
tstnt pkg install game-2d
tstnt pkg install stats
tstnt pkg install auth
tstnt pkg install tg-admin
tstnt pkg install data-viz
tstnt pkg install git-utils
```

---

## Editor Support

**Neovim**
```bash
mkdir -p ~/.config/nvim/syntax
cp editor/neovim/tstnt.vim ~/.config/nvim/syntax/
# init.vim: au BufRead,BufNewFile *.tstnt set filetype=tstnt
```

**VSCode** — copy `editor/vscode/` to `~/.vscode/extensions/tstnt-lang/`

**LSP** — `tstnt-lsp` binary included

---

## Changelog

### v1.0.5
- `tstnt new <project>` — project scaffolding
- `@cache @timer @deprecated` decorators
- `pickle` module — persist any value
- `hashmap` and `set` modules
- 6 new built-in functions: `default` `not_null` `coalesce` `clamp` `repeat_str` `tap`
- 15 new stdlib modules: `xml` `html` `ini` `toml` `bit` `qr` `base` `color2` `matrix2` `dns` `smtp` `pack` `math3` `zip2` `signal2`
- 15 new packages
- Transpiler: `+=` `loop in` `enumerate` `spread` `struct init` now work
- 15+ new easter eggs

### v0.8.0
- Optional chaining `obj?.field`
- `db` module — persistent key-value database
- 11 new modules
- Transpiler: `tstnt transpile file.tstnt py|js`
- Syntax highlighting for Neovim and VSCode

### v0.7.0
- Watch mode, pretty errors, ASCII art banner
- Package downloads from GitHub

### v0.6.0
- Enumerate loops, `game` and `input` modules

### v0.5.0
- Threads, generics, LSP, Telegram API

---

## License

MIT — see [LICENSE](LICENSE)

---

<div align="center">
  <sub>Built on Android with Rust · <a href="https://tstnt-lang.github.io">tstnt-lang.github.io</a></sub>
</div>
