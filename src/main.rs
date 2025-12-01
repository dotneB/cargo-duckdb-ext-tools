//! Main entry point for cargo-duckdb-ext-tools
//!
//! This binary provides two cargo subcommands:
//! - `duckdb-ext-pack`: Appends DuckDB extension metadata to dynamic libraries
//! - `duckdb-ext-build`: Builds and packages DuckDB extensions in one step

mod builder;
mod error;
mod fs;
mod logger;
mod packer;
mod task;

use crate::task::Task;
use anyhow::Result;

/// Main entry point that delegates to the appropriate task based on command line arguments
fn main() -> Result<()> {
    let task = Task::new();
    task.execute()?;
    Ok(())
}
