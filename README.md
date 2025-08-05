# uroman-rs

[![Crates.io](https://img.shields.io/crates/v/uroman.svg)](https://crates.io/crates/uroman)
[![CI](https://github.com/fulm-o/uroman-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/fulm-o/uroman-rs/actions)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](LICENSE)

A blazingly fast, self-contained Rust reimplementation of the `uroman` universal romanizer.

## Overview

`uroman-rs` is a complete rewrite of the original `uroman` (Universal Romanizer) in Rust. It provides high-speed, accurate romanization for a vast number of languages and writing systems, faithfully reproducing the behavior of the original implementation.

This project is licensed under the Apache License 2.0. As a reimplementation, it respects and includes the license of the original `uroman` software. For full details, please refer to the [License section](#license) below.

## ✨ Features

*   **🚀 Blazing Fast Performance**: Approximately **22 times faster** than the standard Python version, making it ideal for large-scale data processing. (See [Benchmark](#-benchmark))
*   **📦 Self-Contained**: A Pure Rust implementation with no dependency on external runtimes like Python or Perl. It compiles to a single, portable binary.
*   **🎯 High Fidelity**: A true drop-in replacement for the original `uroman`, passing its comprehensive test suite.
*   **🧰 Rich Output Formats**: Supports multiple output formats, including simple strings (`str`), and structured JSON data with character offsets (`edges`), alternatives (`alts`), and all lattice paths (`lattice`).
*   **🔧 Versatile**: Use it as a standalone Command-Line Interface (CLI) tool or as a library in your own Rust applications.

## 📝 A Note on Romanization Logic and Limitations

`uroman-rs` is a high-fidelity reimplementation of the original `uroman` and passes its comprehensive test suite. This means its romanization logic, including its strengths and limitations, is identical to the original implementation created by NLP researchers.

The original authors provide excellent documentation on the specific behaviors of the romanizer. To use `uroman-rs` effectively, we recommend reviewing these details, especially concerning:

*   **[Reversibility](https://github.com/isi-nlp/uroman?tab=readme-ov-file#reversibility)**: Details on whether the romanization process can be reliably reversed.
*   **[Known Limitations](https://github.com/isi-nlp/uroman?tab=readme-ov-file#limitations)**: Important information about cases where `uroman` may not perform as expected.


## 📦 Installation

The `uroman-rs` project is available as a crate named uroman. You can use it both as a command-line tool and as a library in your Rust projects.

### As a Command-Line Tool

To install the `uroman-rs` command-line tool, run the following:

```bash
cargo install uroman
```

This will install the executable as `uroman-rs` on your system.

### As a Library

To use `uroman` as a library, add it to your project's dependencies.

```bash
cargo add uroman
```

## ⚙️ Usage

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


**Use in REPL:**

Run `uroman-rs` without any arguments to process input line by line. Press `Ctrl+D` to exit.

```bash
$ uroman-rs
>> こんにちは、世界！
konnichiha, shijie!
>> ᚺᚨᛚᛚᛟ ᚹᛟᚱᛚᛞ
hallo world
>> (Ctrl+D)
```


### As a Library

Here is a basic example to get you started.

```bash
cargo add uroman
```

```rust
// `Uroman::new()` is an infallible operation.
// It doesn't return a `Result`, so no error handling is needed.
let uroman = Uroman::new();

let romanized_string/*: String*/ = uroman.romanize_string::<rom_format::Str>(
    "✨ユーロマン✨",
    Some("jpn"),
).to_output_string();

assert_eq!(romanized_string, "✨yuuroman✨");
println!("{romanized_string}");
```

For more advanced use cases, including file processing and generating detailed JSON output, please see the code in the [`examples/`](./examples) directory.


## 🚀 Benchmark

`uroman-rs` offers a dramatic performance improvement over the standard Python implementation. To provide a fair and robust comparison, we used the [`hyperfine`](https://github.com/sharkdp/hyperfine) benchmarking tool to measure the total execution time for a common task.

### Test Environment
*   **CPU**: [Intel(R) Core(TM) i7-14700]
*   **OS**: [WSL2 Ubuntu 24.04]
*   **Tool**: `hyperfine` v1.18.0
*   **Test File**: `multi-script.txt` from the original `uroman` repository.

### Results

| Implementation                | Mean Time (± σ)       | Performance                   |
|-------------------------------|-----------------------|-------------------------------|
| **`uroman-rs` (This project)**| **99.3 ms ± 3.6 ms**  | **~22x Faster**               |
| `uroman.py` (via `uv run`)    | **2180 ms ± 26 ms** | Baseline                      |


## License

This project is licensed under the Apache License, Version 2.0.

### Acknowledgements

`uroman-rs` is a Rust implementation of the original `uroman` software by Ulf Hermjakob. As such, it is a derivative work and includes the original license notice in the `NOTICE` file.

Please be aware that any academic publication of projects using `uroman-rs` should acknowledge the use of the original `uroman` software as specified in its license. For details, please see the `NOTICE` file.