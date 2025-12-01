# cargo-duckdb-ext-tools

[![Crates.io](https://img.shields.io/crates/v/cargo-duckdb-ext-tools.svg)](https://crates.io/crates/cargo-duckdb-ext-tools)
[![Documentation](https://docs.rs/cargo-duckdb-ext-tools/badge.svg)](https://docs.rs/cargo-duckdb-ext-tools)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

一个基于 Rust 的工具集，用于构建和打包 DuckDB 扩展，无需 Python 依赖。提供两个 cargo 子命令，简化基于 Rust 的 DuckDB 扩展开发工作流。

## 🚀 概述

DuckDB 扩展是动态库文件（`.dylib`/`.so`/`.dll`），在文件末尾附加了一个 534 字节的元数据页脚。官方的 DuckDB Rust 扩展模板依赖 Python 脚本（`append_extension_metadata.py`）来添加此元数据，要求开发者同时维护 Rust 和 Python 环境。

本项目通过提供原生 Rust 工具消除了 Python 依赖，与 cargo 工作流无缝集成。

### ✨ 特性

- **零 Python 依赖**: 纯 Rust 实现
- **Cargo 原生集成**: 与现有 Rust 工作流无缝集成
- **智能默认值**: 从 Cargo 元数据自动推断参数
- **跨平台支持**: 原生和交叉编译支持
- **两个工具**: 提供低级和高级打包选项

### 💡 使用场景

- 纯粹使用 Rust 开发 DuckDB 扩展
- 在 CI/CD 流水线中自动化扩展打包
- 跨平台扩展构建，无需平台特定工具
- 简化 DuckDB 扩展开发工作流

## 🛠️ 提供的工具

### 1. `cargo-duckdb-ext-pack`

一个低级工具，将 DuckDB 扩展元数据附加到现有的动态库文件。这是 Python `append_extension_metadata.py` 脚本的直接替代品。

#### 必需参数
- `-i, --library-path`: 输入动态库路径
- `-o, --extension-path`: 输出扩展文件路径
- `-v, --extension-version`: 扩展版本（例如 `v1.0.0`）
- `-p, --duckdb-platform`: 目标平台（例如 `osx_arm64`, `linux_amd64`）
- `-d, --duckdb-version`: DuckDB 版本（例如 `v1.4.2`）

#### 可选参数
- `-a, --abi-type`: ABI 类型（默认：`C_STRUCT_UNSTABLE`）
- `-q, --quiet`: 抑制输出

#### 示例
```bash
cargo duckdb-ext-pack \
  -i target/release/librusty_sheet.dylib \
  -o rusty_sheet.duckdb_extension \
  -v v0.4.0 \
  -p osx_arm64 \
  -d v1.4.2
```

### 2. `cargo-duckdb-ext-build`

一个高级工具，结合构建和打包于一步，具有智能默认值。

#### 所有参数可选
- `-m, --manifest-path`: Cargo.toml 路径
- `-o, --extension-path`: 输出扩展文件路径
- `-v, --extension-version`: 扩展版本
- `-p, --duckdb-platform`: 目标平台
- `-d, --duckdb-version`: DuckDB 版本
- `-a, --abi-type`: ABI 类型（默认：`C_STRUCT_UNSTABLE`）
- `-q, --quiet`: 抑制输出
- `--` 后的参数：传递给 `cargo build`

#### 智能默认值

该工具使用 `cargo build --message-format=json` 自动提取构建信息并推导：

1. **库路径**: 来自具有 `cdylib` 目标类型的编译器工件
2. **扩展路径**: 与库相同目录中的 `<项目名称>.duckdb_extension`
3. **扩展版本**: 来自项目的 `Cargo.toml` 版本字段
4. **平台**:
   - 来自目标三元组（用于交叉编译）
   - 来自主机架构（用于原生构建）
5. **DuckDB 版本**: 来自 `duckdb` 或 `libduckdb-sys` 依赖版本

#### 示例
```bash
cargo duckdb-ext-build -- --release --target x86_64-unknown-linux-gnu
```

这将执行：
1. `cargo build --release --target x86_64-unknown-linux-gnu`
2. 使用自动检测的参数执行 `cargo duckdb-ext-pack`

输出：`target/x86_64-unknown-linux-gnu/release/<项目名称>.duckdb_extension`

## 📦 安装

```bash
cargo install cargo-duckdb-ext-tools
```

## 🚀 快速开始

### 对于大多数项目

只需使用：
```bash
cargo duckdb-ext-build -- --release
```

### 交叉编译

```bash
cargo duckdb-ext-build -- --release --target aarch64-unknown-linux-gnu
```

### 自定义参数

需要时覆盖默认值：
```bash
cargo duckdb-ext-build \
  -v v2.0.0 \
  -p linux_amd64_gcc4 \
  -- --release
```

## 🌍 平台支持

已在以下平台测试：
- macOS（Apple Silicon 和 Intel）
- Linux（x86_64, aarch64）
- Windows（通过交叉编译）

### 平台映射

该工具自动将 Rust 目标三元组映射到 DuckDB 平台标识符：

| Rust 目标三元组 | DuckDB 平台 |
|----------------|-------------|
| `x86_64-apple-darwin` | `osx_amd64` |
| `aarch64-apple-darwin` | `osx_arm64` |
| `x86_64-unknown-linux-gnu` | `linux_amd64` |
| `aarch64-unknown-linux-gnu` | `linux_arm64` |
| `x86_64-pc-windows-msvc` | `windows_amd64` |

## 🆘 支持

如有问题或疑问：
- **GitHub Issues**: https://github.com/redraiment/cargo-duckdb-ext-tools/issues
- **邮箱**: Zhang, Zepeng <redraiment@gmail.com>

## 📄 许可证

MIT 许可证 - 查看 [LICENSE](LICENSE) 文件获取完整许可证文本。

## 🙏 致谢

- DuckDB 团队提供的优秀扩展系统
- Rust 社区提供的惊人工具生态系统
- 本项目的贡献者和用户
