//! DuckDB extension metadata packer
//!
//! This module implements the core functionality for appending DuckDB extension
//! metadata to dynamic library files, transforming them into DuckDB extensions.
//! It handles the 534-byte footer structure that contains version, platform,
//! and compatibility information.

use crate::console;
use crate::error::ToolsError;
use crate::fs::open_duplicate;
use crate::logger::QUITE;
use clap::Parser;
use std::fs::File;
use std::io::Write;

/// Command line options for the `duckdb-ext-pack` subcommand
///
/// This struct defines all the parameters required to append DuckDB
/// extension metadata to an existing dynamic library file.
#[derive(Parser, Debug)]
#[command(name = "duckdb-ext-pack", version, author, about = "", long_about = "")]
pub(super) struct PackerOptions {
    /// Path to the input dynamic library file
    #[arg(short = 'i', long, value_name = "LIBRARY-PATH")]
    library_path: String,

    /// Path where the output extension file should be created
    #[arg(short = 'o', long, value_name = "EXTENSION-PATH")]
    extension_path: String,

    /// Version of the extension (e.g., "v1.0.0")
    #[arg(short = 'v', long, value_name = "EXTENSION-VERSION")]
    extension_version: String,

    /// Target platform identifier (e.g., "osx_arm64", "linux_amd64")
    #[arg(short = 'p', long, value_name = "DUCKDB-PLATFORM")]
    duckdb_platform: String,

    /// DuckDB version the extension is built for (e.g., "v1.4.2")
    #[arg(short = 'd', long, value_name = "DUCKDB-VERSION")]
    duckdb_version: String,

    /// ABI type for the extension (defaults to "C_STRUCT_UNSTABLE")
    #[arg(short = 'a', long, value_name = "ABI-TYPE", default_value = "C_STRUCT_UNSTABLE")]
    abi_type: String,

    /// Suppress console output
    #[arg(short = 'q', long, default_value_t = false)]
    quiet: bool,
}

/// Core component responsible for writing DuckDB extension metadata
///
/// This struct holds the file handle and metadata needed to append
/// the 534-byte footer that transforms a dynamic library into a
/// DuckDB extension.
pub(super) struct Packer {
    /// File handle opened in append mode for writing metadata
    pub(super) file: File,
    /// Version string of the extension (must start with 'v')
    pub(super) extension_version: String,
    /// Target platform identifier
    pub(super) duckdb_platform: String,
    /// DuckDB version compatibility
    pub(super) duckdb_version: String,
    /// ABI type specification
    pub(super) abi_type: String,
}

impl TryFrom<PackerOptions> for Packer {
    type Error = ToolsError;

    /// Constructs a Packer from command line options
    ///
    /// This conversion sets up the global quiet flag and creates the
    /// extension file by duplicating the source library.
    fn try_from(parameters: PackerOptions) -> Result<Self, Self::Error> {
        QUITE.set(parameters.quiet).expect("Failed to set quiet");
        let file = open_duplicate(&parameters.library_path, &parameters.extension_path)?;
        Ok(Self {
            file,
            extension_version: parameters.extension_version,
            duckdb_platform: parameters.duckdb_platform,
            duckdb_version: parameters.duckdb_version,
            abi_type: parameters.abi_type,
        })
    }
}

impl Packer {
    /// Appends the 534-byte DuckDB extension metadata footer to the file
    ///
    /// This method writes the complete metadata structure including:
    /// - Start signature
    /// - Reserved fields
    /// - ABI type
    /// - Extension version
    /// - DuckDB version
    /// - Platform identifier
    /// - Unknown field (always "4")
    /// - Signature padding
    pub(super) fn write_metadata(&mut self) -> Result<(), ToolsError> {
        Self::write_start_signature(&mut self.file)?;
        // Write 3 empty 32-byte fields (reserved for future use)
        for _ in 0..3 {
            Self::write_field(&mut self.file, "")?;
        }
        console!("     Packing ABI Type ({})", self.abi_type);
        Self::write_field(&mut self.file, self.abi_type.as_str())?;
        console!("     Packing Extension Version ({})", self.extension_version);
        Self::write_field(&mut self.file, self.extension_version.as_str())?;
        console!("     Packing DuckDB Version ({})", self.duckdb_version);
        Self::write_field(&mut self.file, self.duckdb_version.as_str())?;
        console!("     Packing DuckDB Platform ({})", self.duckdb_platform);
        Self::write_field(&mut self.file, self.duckdb_platform.as_str())?;
        // Unknown field, always "4" in current DuckDB format
        Self::write_field(&mut self.file, "4")?;
        // Write 8 empty 32-byte fields for signature padding
        for _ in 0..8 {
            Self::write_field(&mut self.file, "")?;
        }
        console!("    Finished DuckDB Extension");
        Ok(())
    }

    /// Writes the fixed start signature for DuckDB extension files
    ///
    /// The signature is a specific byte sequence that identifies the file
    /// as a DuckDB extension. This is the first part of the 534-byte footer.
    fn write_start_signature(file: &mut File) -> Result<(), std::io::Error> {
        file.write_all(&vec![
            0, 147, 4, 16, 100, 117, 99, 107, 100, 98, 95, 115, 105, 103, 110, 97, 116, 117, 114,
            101, 128, 4,
        ])
    }

    /// Writes a 32-byte field with the given content
    ///
    /// This pads the content to exactly 32 bytes with null bytes.
    /// If the content is longer than 32 bytes, it will be truncated.
    fn write_field(file: &mut File, content: &str) -> Result<(), std::io::Error> {
        let mut bytes = [0u8; 32];
        bytes[..content.len()].copy_from_slice(content.as_bytes());
        file.write_all(bytes.as_ref())
    }
}
