//! Logging and console output utilities for cargo-duckdb-ext-tools
//!
//! This module provides a simple logging system with support for quiet mode,
//! allowing users to suppress console output when desired.

use std::sync::OnceLock;

/// Global flag controlling whether console output should be suppressed
///
/// When set to `true`, the `console!` macro will not produce any output.
/// This is used to implement the `--quiet` command line option.
pub(super) static QUITE: OnceLock<bool> = OnceLock::new();

/// Prints to standard output only when quiet mode is disabled
///
/// This macro behaves like `println!` but respects the global quiet flag.
/// It's used for all user-facing output in the tool.
#[macro_export]
macro_rules! console {
    ($($content:tt)*) => {
        if !*QUITE.get().unwrap_or(&false) {
            println!($($content)*);
        }
    };
}
