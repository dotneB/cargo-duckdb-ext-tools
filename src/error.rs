//! Error handling for cargo-duckdb-ext-tools
//!
//! This module defines the error types used throughout the application,
//! providing unified error handling for both I/O operations and cargo metadata processing.

use thiserror::Error;

/// Error type for the cargo-duckdb-ext-tools
///
/// This enum encapsulates all possible error scenarios that can occur
/// during extension building and packaging operations.
#[derive(Error, Debug)]
pub(super) enum ToolsError {
    /// Wraps I/O errors that occur during file operations
    #[error("{0}")]
    IoError(#[from] std::io::Error),

    /// Wraps errors from cargo metadata operations
    #[error("{0}")]
    MetadataError(#[from] cargo_metadata::Error),
}
