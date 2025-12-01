//! Task dispatching and command line parsing for cargo-duckdb-ext-tools
//!
//! This module handles the routing of command line invocations to the appropriate
//! subcommands (`duckdb-ext-build` and `duckdb-ext-pack`), parsing arguments
//! and executing the corresponding operations.

use crate::builder::Builder;
use crate::builder::BuilderOptions;
use crate::error::ToolsError;
use crate::packer::Packer;
use crate::packer::PackerOptions;
use clap::Parser;
use std::env::args;

/// Represents the different tasks that can be executed
///
/// This enum distinguishes between the two main operations:
/// - Build: Combines compilation and packaging
/// - Pack: Only appends metadata to existing libraries
#[derive(Debug)]
pub(crate) enum Task {
    Build(Vec<String>),
    Pack(Vec<String>),
}

impl Task {
    /// Creates a new Task by parsing command line arguments
    ///
    /// This method analyzes the command line to determine which subcommand
    /// was invoked and extracts the relevant arguments for that command.
    pub(crate) fn new() -> Task {
        let mut iterator = args().peekable();
        let mut arguments = Vec::new();
        while let Some(program) = iterator.next() {
            if let Some(argument) = iterator.peek() && argument.contains("duckdb-ext-") {
                continue
            } else {
                arguments.push(program);
                break;
            }
        }
        arguments.extend(iterator);
        if let Some(program) = arguments.get(0) {
            if program.ends_with("duckdb-ext-build") {
                return Task::Build(arguments);
            }
            if program.ends_with("duckdb-ext-pack") {
                return Task::Pack(arguments);
            }
        }
        panic!("Unsupported task: {arguments:?}");
    }

    /// Executes the selected task
    ///
    /// For Build tasks: parses options, builds the project, and packages extensions
    /// For Pack tasks: parses options and appends metadata to existing libraries
    pub(crate) fn execute(&self) -> Result<(), ToolsError> {
        if let Task::Build(args) = self {
            let options = BuilderOptions::parse_from(args);
            let mut builder = Builder::try_from(options)?;
            for mut packer in builder.build()? {
                packer.write_metadata()?;
            }
        } else if let Task::Pack(args) = self {
            let options = PackerOptions::parse_from(args);
            let mut packer = Packer::try_from(options)?;
            packer.write_metadata()?;
        }
        Ok(())
    }
}
