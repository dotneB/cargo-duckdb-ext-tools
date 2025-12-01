//! High-level builder for DuckDB extensions
//!
//! This module provides the `duckdb-ext-build` subcommand that combines
//! compilation and packaging in a single operation with intelligent defaults.

use crate::error::ToolsError;
use crate::fs::open_duplicate;
use crate::logger::QUITE;
use crate::packer::Packer;
use cargo_metadata::camino::Utf8PathBuf;
use cargo_metadata::semver::Version;
use cargo_metadata::Message;
use cargo_metadata::Metadata;
use cargo_metadata::MetadataCommand;
use cargo_metadata::Package;
use cargo_metadata::PackageId;
use cargo_metadata::TargetKind;
use cargo_metadata::{Artifact, PackageName};
use clap::Parser;
use std::collections::HashMap;
use std::env::consts::ARCH;
use std::env::consts::OS;
use std::io::BufRead;
use std::process::Command;
use std::process::Stdio;
use std::str::FromStr;
use target_lexicon::Architecture;
use target_lexicon::OperatingSystem;
use target_lexicon::Triple;

/// Command line options for the `duckdb-ext-build` subcommand
///
/// This struct defines parameters for the high-level build command that
/// combines compilation and packaging with intelligent defaults.
#[derive(Parser, Debug)]
#[command(
    name = "duckdb-ext-build",
    version,
    author,
    about = "",
    long_about = ""
)]
pub(super) struct BuilderOptions {
    /// Path to Cargo.toml (defaults to current directory)
    #[arg(short = 'm', long, value_name = "MANIFEST-PATH")]
    manifest_path: Option<String>,

    /// Output extension file path (auto-detected if not specified)
    #[arg(short = 'o', long, value_name = "EXTENSION-PATH")]
    extension_path: Option<String>,

    /// Extension version (auto-detected from Cargo.toml if not specified)
    #[arg(short = 'v', long, value_name = "EXTENSION-VERSION")]
    extension_version: Option<String>,

    /// Target platform (auto-detected from build target if not specified)
    #[arg(short = 'p', long, value_name = "DUCKDB-PLATFORM")]
    duckdb_platform: Option<String>,

    /// DuckDB version (auto-detected from dependencies if not specified)
    #[arg(short = 'd', long, value_name = "DUCKDB-VERSION")]
    duckdb_version: Option<String>,

    /// ABI type (defaults to "C_STRUCT_UNSTABLE")
    #[arg(
        short = 'a',
        long,
        value_name = "ABI-TYPE",
        default_value = "C_STRUCT_UNSTABLE"
    )]
    abi_type: String,

    /// Suppress console output
    #[arg(short = 'q', long, default_value_t = false)]
    quiet: bool,

    /// Additional arguments passed to `cargo build`
    #[arg(raw = true)]
    args: Vec<String>,
}

impl BuilderOptions {
    /// Creates a cargo build command with JSON message format
    ///
    /// This sets up the cargo command to produce JSON output that can be
    /// parsed to extract build artifact information.
    fn cargo(&self) -> Command {
        let mut command = Command::new("cargo");
        let mut args = vec!["build".to_string(), "--message-format=json".to_string()];
        args.extend_from_slice(&self.args);
        command.args(&args);
        command.stdout(Stdio::piped());
        command
    }

    /// Opens and parses the Cargo.toml manifest
    ///
    /// This extracts metadata about the project including dependencies,
    /// package information, and workspace configuration.
    fn open_manifest(&self) -> Result<Metadata, ToolsError> {
        let mut command = MetadataCommand::new();
        if let Some(manifest_path) = self.manifest_path.as_ref() {
            command.manifest_path(manifest_path);
        }
        command.verbose(!self.quiet);
        Ok(command.exec()?)
    }
}

/// High-level builder that orchestrates compilation and packaging
///
/// This struct holds all the context needed to build DuckDB extensions,
/// including build command configuration, metadata, and packaging parameters.
pub(super) struct Builder {
    /// Cargo build command with JSON output enabled
    command: Command,
    /// Target directory where build artifacts are located
    target_directory: Utf8PathBuf,
    /// Optional override for extension output path
    extension_path: Option<String>,
    /// Optional override for extension version
    extension_version: Option<String>,
    /// Optional override for target platform
    duckdb_platform: Option<String>,
    /// DuckDB version compatibility
    duckdb_version: String,
    /// ABI type specification
    abi_type: String,
    /// List of packages in the workspace that produce CDyLib targets
    packages: Vec<Package>,
}

impl TryFrom<BuilderOptions> for Builder {
    type Error = ToolsError;

    /// Constructs a Builder from command line options
    ///
    /// This conversion extracts project metadata, detects DuckDB version
    /// from dependencies, and filters packages that produce dynamic libraries.
    fn try_from(parameters: BuilderOptions) -> Result<Self, Self::Error> {
        QUITE.set(parameters.quiet).expect("Failed to set quiet");
        let metadata = parameters.open_manifest()?;
        let target_directory = metadata.target_directory;

        // Auto-detect DuckDB version from dependencies
        let duckdb_version = metadata
            .packages
            .iter()
            .find_map(|package| {
                if package.name == "duckdb" || package.name == "libduckdb-sys" {
                    Some(format!("v{}", package.version))
                } else {
                    None
                }
            });

        // Filter packages that are workspace members and produce CDyLib targets
        let packages = metadata
            .packages
            .into_iter()
            .filter(|package| metadata.workspace_members.contains(&package.id))
            .filter(|package| package.targets.iter().any(|target| target.kind.contains(&TargetKind::CDyLib)))
            .collect::<Vec<_>>();

        Ok(Self {
            command: parameters.cargo(),
            target_directory,
            extension_path: parameters.extension_path,
            extension_version: parameters.extension_version,
            duckdb_platform: parameters.duckdb_platform,
            duckdb_version: parameters.duckdb_version.or(duckdb_version).expect("Missing duckdb version"),
            abi_type: parameters.abi_type,
            packages,
        })
    }
}

impl Builder {
    /// Executes the build process and creates Packer instances for each artifact
    ///
    /// This method:
    /// 1. Runs cargo build with JSON output
    /// 2. Parses the JSON messages to find CDyLib artifacts
    /// 3. Matches artifacts with their corresponding packages
    /// 4. Creates Packer instances for packaging each extension
    pub(super) fn build(&mut self) -> Result<Vec<Packer>, ToolsError> {
        let packages = self
            .packages
            .iter()
            .map(|package| (package.id.to_owned(), package))
            .collect::<HashMap<PackageId, &Package>>();

        // Execute cargo build and process JSON output
        self
            .command
            .spawn()?
            .wait_with_output()?
            .stdout
            .lines()
            .filter_map(Result::ok)
            .map(|json| serde_json::from_str::<Message>(&json))
            .filter_map(Result::ok)
            .filter_map(|message| match message {
                Message::CompilerArtifact(artifact) if artifact.target.kind.contains(&TargetKind::CDyLib) => Some(artifact),
                _ => None,
            })
            .filter_map(|artifact| packages
                .get(&artifact.package_id)
                .map(|package| (package, artifact))
            )
            .flat_map(|(package, artifact)| self.packs(package, &artifact))
            .collect::<Result<Vec<_>, _>>()
    }

    /// Creates Packer instances for all filenames in an artifact
    ///
    /// This processes each filename in the artifact, filtering for valid
    /// dynamic library files within the target directory.
    fn packs(&self, package: &Package, artifact: &Artifact) -> Vec<Result<Packer, ToolsError>> {
        artifact.filenames
            .iter()
            .filter_map(|filename| filename.canonicalize_utf8().ok())
            .filter(|filename| filename.starts_with(&self.target_directory))
            .map(|filename| self.pack(&package.name, &package.version, &filename))
            .collect::<Vec<_>>()
    }

    /// Creates a Packer instance for a specific library file
    ///
    /// This method applies intelligent defaults for all parameters:
    /// - Extension path: auto-generated from package name
    /// - Extension version: extracted from Cargo.toml
    /// - Platform: detected from build target or host system
    /// - DuckDB version: from dependencies or user override
    fn pack(&self, package_name: &PackageName, package_version: &Version, filename: &Utf8PathBuf) -> Result<Packer, ToolsError> {
        let library_path = filename.to_string();
        let extension_path = self.extension_path
            .to_owned()
            .unwrap_or_else(|| self.artifact_extension_path(filename, package_name));
        let extension_version = self.extension_version
            .to_owned()
            .unwrap_or_else(|| format!("v{package_version}"));
        let duckdb_platform = self.duckdb_platform
            .to_owned()
            .or_else(|| self.artifact_duckdb_platform(filename))
            .unwrap_or_else(Self::default_duckdb_platform);
        let duckdb_version = self.duckdb_version.to_owned();
        let abi_type = self.abi_type.to_owned();

        let file = open_duplicate(&library_path, &extension_path)?;
        Ok(Packer {
            file,
            extension_version,
            duckdb_platform,
            duckdb_version,
            abi_type,
        })
    }

    /// Generates the extension file path from the library path and package name
    ///
    /// This replaces the library filename with the package name and changes
    /// the extension to `.duckdb_extension`.
    fn artifact_extension_path(&self, filename: &Utf8PathBuf, package_name: &str) -> String {
        let mut path = filename.clone();
        path.set_file_name(package_name.replace('-', "_"));
        path.set_extension("duckdb_extension");
        path.to_string()
    }

    /// Extracts DuckDB platform identifier from the build artifact path
    ///
    /// This method analyzes the target triple from the build directory
    /// structure and maps it to DuckDB platform identifiers.
    fn artifact_duckdb_platform(&self, filename: &Utf8PathBuf) -> Option<String> {
        filename.strip_prefix(&self.target_directory)
            .ok()
            .and_then(|path| path.components().next())
            .map(|target| Triple::from_str(target.as_str()))
            .and_then(Result::ok)
            .map(|triple| {
                let os = match triple.operating_system {
                    OperatingSystem::Linux => "linux",
                    OperatingSystem::MacOSX(_) => "osx",
                    OperatingSystem::Windows => "windows",
                    _ => panic!("Unsupported operating system: {}", triple),
                };
                let arch = match triple.architecture {
                    Architecture::X86_64 => "amd64",
                    Architecture::Aarch64(_) => "arm64",
                    Architecture::X86_32(_) => "amd",
                    Architecture::Arm(_) => "arm",
                    _ => panic!("Unsupported architecture: {}", triple),
                };
                format!("{os}_{arch}")
            })
    }

    /// Provides a default platform identifier based on the host system
    ///
    /// This is used when no target triple is detected in the build path,
    /// typically for native builds without explicit target specification.
    fn default_duckdb_platform() -> String {
        let os = match OS {
            "macos" => "osx",
            os => os,
        };
        let arch = match ARCH {
            "x86" => "amd",
            "x86_64" => "amd64",
            "aarch64" => "arm64",
            arch => arch,
        };
        format!("{os}_{arch}")
    }
}
