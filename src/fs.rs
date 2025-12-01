//! File system operations for cargo-duckdb-ext-tools
//!
//! This module provides file system utilities specifically designed for
//! creating DuckDB extension files by duplicating dynamic libraries and
//! preparing them for metadata appending.

use crate::console;
use crate::logger::QUITE;
use std::fs::copy;
use std::fs::File;
use std::fs::OpenOptions;

/// Creates a duplicate of a file and opens it in append mode
///
/// This function is used to create the extension file by copying the
/// original dynamic library and then opening it for metadata appending.
///
/// # Arguments
/// * `source` - Path to the source dynamic library file
/// * `target` - Path where the extension file should be created
///
/// # Returns
/// A `File` handle opened in append mode for writing metadata
pub(super) fn open_duplicate(source: &str, target: &str) -> Result<File, std::io::Error> {
    console!("     Copying Library File ({source})");
    console!("     Copying Extension File ({target})");
    copy(source, target)?;
    OpenOptions::new()
        .append(true)
        .open(target)
}
