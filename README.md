# uroman-rs

[![Crates.io](https://img.shields.io/crates/v/uroman.svg)](https://crates.io/crates/uroman)
[![CI](https://github.com/fulm-o/uroman-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/fulm-o/uroman-rs/actions)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](LICENSE)

A blazingly fast, self-contained Rust reimplementation of the `uroman` universal romanizer.

## Overview

`uroman-rs` is a complete rewrite of the original `uroman` (Universal Romanizer) in Rust. It provides high-speed, accurate romanization for a vast number of languages and writing systems, faithfully reproducing the behavior of the original implementation.


## ✨ Features

*   **🚀 Blazing Fast Performance**: Approximately **22 times faster** than the standard Python version, making it ideal for large-scale data processing. (See [Benchmark](#-benchmark))
*   **📦 Self-Contained**: A Pure Rust implementation with no dependency on external runtimes like Python or Perl. It compiles to a single, portable binary.
*   **🎯 High Fidelity**: A true drop-in replacement for the original `uroman`, passing its comprehensive test suite.
*   **🧰 Rich Output Formats**: Supports multiple output formats, including simple strings (`str`), and structured JSON data with character offsets (`edges`), alternatives (`alts`), and all lattice paths (`lattice`).
*   **🔧 Versatile**: Use it as a standalone Command-Line Interface (CLI) tool or as a library in your own Rust applications.

## 📦 Installation

You can use `uroman` both as a command-line tool and as a library in your own projects.

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