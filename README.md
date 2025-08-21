<div align="center">
  <h1>uroman-rs</h1>
  <p>
    A blazingly fast, self-contained Rust reimplementation of the <code>uroman</code> universal romanizer.
  </p>
  <p>
    <a href="https://crates.io/crates/uroman"><img src="https://img.shields.io/crates/v/uroman.svg" alt="Crates.io"></a>
    <a href="https://github.com/fulm-o/uroman-rs/actions/workflows/rust.yml"><img src="https://github.com/fulm-o/uroman-rs/actions/workflows/rust.yml/badge.svg" alt="CI"></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache--2.0-blue.svg" alt="License: Apache-2.0"></a>
  </p>
</div>

## Overview

`uroman-rs` is a complete rewrite of the original `uroman` (Universal Romanizer) in Rust. It provides high-speed, accurate romanization for a vast number of languages and writing systems, faithfully reproducing the behavior of the original implementation.

As a reimplementation, it is designed to be a drop-in replacement that passes the original's comprehensive test suite. This means its romanization logic, including its strengths and limitations, is identical to the original. For effective use, we recommend reviewing the original authors' documentation on [Reversibility](https://github.com/isi-nlp/uroman?tab=readme-ov-file#reversibility) and [Known Limitations](https://github.com/isi-nlp/uroman?tab=readme-ov-file#limitations).

In the same spirit of fidelity, this project respects the licensing of the original `uroman` software. `uroman-rs` is licensed under the Apache License 2.0, and includes the original's license as required. For full details, please refer to the [License section](#license-and-acknowledgements).

## Features

*   **Performance**: Achieves approximately **22x the speed** of the standard Python version, making it ideal for large-scale data processing. (See [Benchmark](#benchmark))
*   **Robustness**: Fixes several edge-case bugs present in the original implementation, ensuring safer processing of diverse inputs. (See [Bug Fixes](#bug-fixes))
*   **Self-Contained**: A pure Rust implementation with no dependency on external runtimes. It compiles to a single, portable binary.
*   **High Fidelity**: Faithfully reproduces the behavior of the original `uroman` and passes its test suite.
*   **Rich Output Formats**: Supports multiple output formats, including simple strings (`str`) and structured JSON data (`edges`, `alts`, `lattice`).
*   **Versatile**: Can be used as a standalone Command-Line Interface (CLI) tool or as a library in your Rust applications.

## Installation

The `uroman-rs` project is available as a crate named uroman. You can use it both as a command-line tool and as a library in your Rust projects.

### As a Command-Line Tool

To install the `uroman-rs` command-line tool, run the following:

```bash
cargo install uroman
```

This will install the executable as `uroman-rs` on your system.

### As a Library

Add `uroman-rs` to your project's Cargo.toml.
For library usage, it's recommended to disable default features to avoid pulling in CLI-specific dependencies.

```bash
cargo add uroman --no-default-features
```

## Usage

### Command-Line Interface (CLI)

`uroman-rs` can be used directly from your terminal.

**Show sample conversions:**
See examples of how various scripts are romanized.

```bash
uroman-rs --sample
```

**View all options:**

Display the help message for a full list of commands and flags.
```bash
uroman-rs --help
```

**Use in REPL mode:**

Run `uroman-rs` without any arguments to process input line by line. Press `Ctrl+D` to exit.

```bash
$ uroman-rs
>> こんにちは、世界！
konnichiha, shijie!
>> ᚺᚨᛚᛚᛟ ᚹᛟᚱᛚᛞ
hallo world
>> (Ctrl+D)
```


### Library

```rust
// Uroman::new() is infallible and does not return a `Result`.
let uroman = Uroman::new();

let romanized_string/*: String*/ = uroman.romanize_string::<rom_format::Str>(
    "✨ユーロマン✨",
    Some("jpn"),
).to_output_string();

assert_eq!(romanized_string, "✨yuuroman✨");
println!("{romanized_string}");
```
For more advanced examples, please see the examples/ directory.

## Benchmark

Performance was measured against the original Python implementation using [`hyperfine`](https://github.com/sharkdp/hyperfine).

*   **Test File**: `multi-script.txt` from the original `uroman` repository.
*   **Environment**: Intel Core i7-14700, WSL2 (Ubuntu 24.04)

| Implementation                | Mean Time (± σ)       | Relative Performance |
|-------------------------------|-----------------------|----------------------|
| **`uroman-rs` (This project)**| **99.3 ms ± 3.6 ms**  | **~22.0x faster**     |
| `uroman.py` (via `uv run`)    | 2180 ms ± 26 ms       | Baseline             |

## Bug Fixes

`uroman-rs` aims to be not only a faithful reimplementation but also a more robust one. It handles several edge cases that can cause the original `uroman.py` script to crash.

For example, the original script panics on inputs with incomplete fractional patterns like `"百分之"` ("percent of..."). This occurs because the script expects a subsequent number but does not safely handle cases where one is not found, leading to a `NoneType` attribute error. This issue has been reported to the original author (see [isi-nlp/uroman#16](https://github.com/isi-nlp/uroman/issues/16)).

```sh
$ uv run uroman.py "百分之多少"
Traceback (most recent call last):
  ...
  File ".../uroman.py", line 1100, in romanize_string_core
    lat.add_numbers(self, **args)
  File ".../uroman.py", line 2112, in add_numbers
    if right_edge.value is None:
AttributeError: 'NoneType' object has no attribute 'value'
```

In contrast, `uroman-rs` handles this input safely and provides a reasonable fallback romanization, demonstrating its enhanced reliability:

```sh
$ cargo run -r -- "百分之多少"
  Running `target/release/uroman-rs '百分之多少'`
baifenzhiduoshao
```

This ensures higher stability when processing large and diverse text corpora where such edge cases may appear.

## License

This project is licensed under the Apache License, Version 2.0.

### Acknowledgements

`uroman-rs` is a Rust implementation of the original `uroman` software by Ulf Hermjakob. As such, it is a derivative work and includes the original license notice in the `NOTICE` file.

Please be aware that any academic publication of projects using `uroman-rs` should acknowledge the use of the original `uroman` software as specified in its license. For details, please see the `NOTICE` file.
