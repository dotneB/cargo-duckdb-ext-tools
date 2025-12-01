# cargo-duckdb-ext-tools

[![Crates.io](https://img.shields.io/crates/v/cargo-duckdb-ext-tools.svg)](https://crates.io/crates/cargo-duckdb-ext-tools)
[![Documentation](https://docs.rs/cargo-duckdb-ext-tools/badge.svg)](https://docs.rs/cargo-duckdb-ext-tools)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust-based toolkit for building and packaging DuckDB extensions without Python dependencies. Provides two cargo subcommands that streamline the development workflow for Rust-based DuckDB extensions.

## 🚀 Overview

DuckDB extensions are dynamic libraries (`.dylib`/`.so`/`.dll`) with a 534-byte metadata footer appended to the file. The official DuckDB Rust extension template relies on a Python script (`append_extension_metadata.py`) to add this metadata, requiring developers to maintain both Rust and Python environments.

This project eliminates the Python dependency by providing native Rust tooling that integrates seamlessly with cargo workflows.

### ✨ Features

- **Zero Python Dependencies**: Pure Rust implementation
- **Cargo-Native Integration**: Seamless integration with existing Rust workflows
- **Intelligent Defaults**: Automatic parameter inference from Cargo metadata
- **Cross-Platform Support**: Native and cross-compilation support
- **Two Tools**: Both low-level and high-level packaging options

### 💡 Use Cases

- Developing DuckDB extensions purely in Rust
- Automating extension packaging in CI/CD pipelines
- Cross-platform extension builds without platform-specific tooling
- Simplifying DuckDB extension development workflows

## 🛠️ Tools Provided

### 1. `cargo-duckdb-ext-pack`

A lower-level tool that appends DuckDB extension metadata to an existing dynamic library file. This is a direct replacement for the Python `append_extension_metadata.py` script.

#### Required Parameters
- `-i, --library-path`: Input dynamic library path
- `-o, --extension-path`: Output extension file path
- `-v, --extension-version`: Extension version (e.g., `v1.0.0`)
- `-p, --duckdb-platform`: Target platform (e.g., `osx_arm64`, `linux_amd64`)
- `-d, --duckdb-version`: DuckDB version (e.g., `v1.4.2`)

#### Optional Parameters
- `-a, --abi-type`: ABI type (default: `C_STRUCT_UNSTABLE`)
- `-q, --quiet`: Suppress output

#### Example
```bash
cargo duckdb-ext-pack \
  -i target/release/librusty_sheet.dylib \
  -o rusty_sheet.duckdb_extension \
  -v v0.4.0 \
  -p osx_arm64 \
  -d v1.4.2
```

### 2. `cargo-duckdb-ext-build`

A high-level tool that combines building and packaging in one step with intelligent defaults.

#### All Parameters Optional
- `-m, --manifest-path`: Path to Cargo.toml
- `-o, --extension-path`: Output extension file path
- `-v, --extension-version`: Extension version
- `-p, --duckdb-platform`: Target platform
- `-d, --duckdb-version`: DuckDB version
- `-a, --abi-type`: ABI type (default: `C_STRUCT_UNSTABLE`)
- `-q, --quiet`: Suppress output
- Arguments after `--`: Passed to `cargo build`

#### Intelligent Defaults

The tool automatically extracts build information using `cargo build --message-format=json` and derives:

1. **Library path**: From compiler artifacts with `cdylib` target kind
2. **Extension path**: `<project-name>.duckdb_extension` in the same directory as the library
3. **Extension version**: From the project's `Cargo.toml` version field
4. **Platform**:
   - From target triple (for cross-compilation)
   - From host architecture (for native builds)
5. **DuckDB version**: From `duckdb` or `libduckdb-sys` dependency version

#### Example
```bash
cargo duckdb-ext-build -- --release --target x86_64-unknown-linux-gnu
```

This executes:
1. `cargo build --release --target x86_64-unknown-linux-gnu`
2. `cargo duckdb-ext-pack` with auto-detected parameters

Output: `target/x86_64-unknown-linux-gnu/release/<project-name>.duckdb_extension`

## 📦 Installation

```bash
cargo install cargo-duckdb-ext-tools
```

## 🚀 Quick Start

### For Most Projects

Simply use:
```bash
cargo duckdb-ext-build -- --release
```

### Cross-compilation

```bash
cargo duckdb-ext-build -- --release --target aarch64-unknown-linux-gnu
```

### Custom Parameters

Override defaults when needed:
```bash
cargo duckdb-ext-build \
  -v v2.0.0 \
  -p linux_amd64_gcc4 \
  -- --release
```

## 🌍 Platform Support

Tested on:
- macOS (Apple Silicon and Intel)
- Linux (x86_64, aarch64)
- Windows (via cross-compilation)

### Platform Mapping

The tool automatically maps Rust target triples to DuckDB platform identifiers:

| Rust Target Triple | DuckDB Platform |
|-------------------|-----------------|
| `x86_64-apple-darwin` | `osx_amd64` |
| `aarch64-apple-darwin` | `osx_arm64` |
| `x86_64-unknown-linux-gnu` | `linux_amd64` |
| `aarch64-unknown-linux-gnu` | `linux_arm64` |
| `x86_64-pc-windows-msvc` | `windows_amd64` |

## 🆘 Support

For questions or issues:
- **GitHub Issues**: https://github.com/redraiment/cargo-duckdb-ext-tools/issues
- **Email**: Zhang, Zepeng <redraiment@gmail.com>

## 📄 License

MIT License - see [LICENSE](LICENSE) file for full license text.

## 🙏 Acknowledgments

- DuckDB team for the excellent extension system
- Rust community for the amazing tooling ecosystem
- Contributors and users of this project
