<div align="center">

[![Version](https://img.shields.io/badge/version-1.1.0-3fb950?style=flat-square)](https://github.com/tstnt-lang/tstnt/releases)
[![License](https://img.shields.io/badge/license-MIT-58a6ff?style=flat-square)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/built_with-Rust-f97316?style=flat-square&logo=rust)](https://rust-lang.org)
[![Packages](https://img.shields.io/badge/packages-140+-d29922?style=flat-square)](https://github.com/tstnt-lang/packages)

**[Website](https://tstnt-lang.github.io) · [Docs](https://tstnt-lang.github.io/docs.html) · [Packages](https://github.com/tstnt-lang/packages)**

</div>

---

## Overview

TSTNT is a statically-typed scripting language with clean syntax — inspired by Rust and Python.
Ships as a **single binary** with zero external dependencies. Built on Android (Termux) using Rust.

---

## Install

```bash
curl -fsSL https://tstnt-lang.github.io/install.sh | sh
```

**Windows:**
```powershell
irm https://tstnt-lang.github.io/install.ps1 | iex
```

**Build from source:**
```bash
git clone https://github.com/tstnt-lang/tstnt
cd tstnt && cargo build --release
cp target/release/tstnt ~/bin/
```

---

## Quick Start

```bash
tstnt new myproject          # scaffold project
tstnt src/main.tstnt         # run
tstnt repl                   # interactive shell
tstnt test tests/test.tstnt  # run tests
tstnt check src/main.tstnt   # syntax check
tstnt bench src/main.tstnt   # benchmark
tstnt watch src/main.tstnt   # auto-restart
tstnt pkg install logger     # install package
tstnt pkg search             # browse 140+ packages
```

---

## Language

```
use "colors"

struct Player {
    name: str
    hp:   int
}

impl Player {
    do greet -> str { return "Hero: " + self.name }
}

@cache
do fib(n: int) -> int {
    if n <= 1 { return n }
    return fib(n-1) + fib(n-2)
}

do main {
    let name = "Alice"
    print("Hello, {name}!")              # string interpolation

    let text = """
    multiline
    string
    """

    let s = Shape::Circle(5.0)           # enum with data
    print(greet(name: "Alex", age: 25))  # named args

    if let user = db.get("users", "1") {
        print(user.name)                 # if let / walrus
    }

    let nums = [1, 2, 3, 4, 5]
    let first = find(nums, |x| x > 3)
    let all   = every(nums, |x| x > 0)
    let pairs = flat_map(nums, |x| [x, x*2])
}
```

---

## CLI

| Command | Description |
|---------|-------------|
| `tstnt <file>` | Run file |
| `tstnt repl` | Interactive REPL |
| `tstnt new <name>` | Scaffold project |
| `tstnt test <file>` | Run tests |
| `tstnt check <file>` | Syntax check |
| `tstnt bench <file> [n]` | Benchmark |
| `tstnt watch <file>` | Auto-restart |
| `tstnt build <file>` | Compile to bytecode |
| `tstnt fmt <file>` | Format |
| `tstnt transpile <f> py\|js` | Transpile |
| `tstnt pkg install <n>` | Install package |
| `tstnt pkg search` | Browse packages |

---

## Standard Library (55+ modules)

`io` `math` `math2` `math3` `strings` `arr` `arr2` `str2` `json` `fs` `crypto` `rand` `time` `db` `sql` `server` `ws` `template` `zip3` `img` `tui2` `result` `fmt2` `term` `color` `color2` `tg` `thread` `game` `pickle` `hashmap` `set` `regex` `uuid` `hash` `bench` `csv` `xml` `html` `ini` `toml` `bit` `qr` `base` `dns` `smtp` `net` `net2` `env` `sys` `path` `process` `os` `input` `log` `buf`

---

## Changelog

### v1.1.0
- String interpolation `"Hello, {name}!"`
- Multiline strings `""" ... """`
- Enum with data `Shape::Circle(5.0)`
- Named arguments `greet(name: "Alice")`
- `if let` walrus operator
- New stdlib: `ws`, `template`, `zip3`, `img`
- `tstnt check` — syntax check without running
- `tstnt bench` — built-in benchmarking
- Bytecode cache for faster reruns

### v1.0.5
- `module.func()` works without `use`
- Real `@cache`, `@timer`, `@deprecated`
- New stdlib: `sql`, `server`, `tui2`, `result`, `fmt2`
- New builtins: `find`, `every`, `any`, `flat_map`, `take`, `drop`, `chunks`, `slice`, `sprintf`

### v1.0.0
- First stable release, 140+ packages
- Install script

### v0.9.0
- Decorators, `pickle`, `hashmap`, `set`
- 15 new stdlib modules, project scaffolding

---

## License

MIT — see [LICENSE](LICENSE)

<div align="center">
  <sub>Built on Android with Rust · <a href="https://tstnt-lang.github.io">tstnt-lang.github.io</a></sub>
</div>
