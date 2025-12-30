//! Version management and configuration handling for the `cave` CLI.
//!
//! It provides utilities for:
//! - Validating version formats (numeric, `stable`, `testing`).
//! - Checking if a version exists locally or on a remote registry.
//! - Pulling missing versions from Docker Hub or a private registry.
//! - Storing the selected version either globally or locally.
//! - Printing available local and remote versions in a clear CLI display.
//! - Tracking version usage statistics in a local JSON file.
//!
//! Errors are centralized in the [`CaveError`] enum, which provides
//! descriptive messages for all failure cases.

use std::{
    cmp::Ordering, fmt, fs, io::{self, Write}, path::{Path, PathBuf}
};
use crate::docker::*;
use crate::config::{read_config};
use colored::*;
use regex::Regex;
// TODO : uncomment to have registry option
//use crate::config::Config;


/// Different error types that can occur when using the `cave` CLI.
#[derive(Debug)]
pub enum CaveError {
    /// Invalid version format.
    InvalidFormat(String),
    /// Requested version is not available locally or remotely.
    VersionNotAvailable(String),
    /// The user aborted the operation.
    UserAborted,
    /// Input/output error.
    IoError(io::Error),
    /// Docker-related error (commands, connection, etc.).
    DockerError(String),
    /// HOME directory not found.
    HomeNotFound,
    /// File not found.
    FileNotFound(String),
    /// Installed version is missing on the system.
    VersionNotInstalled(String),
    /// HTTP request error.
    HttpError(String),
    /// Docker is not installed.
    NoDocker,
    /// No internet connection for the client
    NoInternetConnection,
    /// JSON serialization/deserialization error.
    SerdeError(serde_json::Error),
    /// code_aster related error (commands, wrong file, etc.).
    CodeAsterError(String),
    ///error encountered during the execution data saving
    TelemetryError(String)
}

impl fmt::Display for CaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CaveError::InvalidFormat(ver) =>
                write!(f, "Invalid version input: '{}'. Expected stable, testing or under this format: xx.x.xx", ver),
            CaveError::VersionNotAvailable(ver) =>
                write!(f, "Version '{}' is not available. Run `cave available` or see on https://hub.docker.com/r/simvia/code_aster.", ver),
            CaveError::UserAborted =>
                write!(f, "No version pinned. Operation cancelled by user."),
            CaveError::IoError(e) =>
                write!(f, "I/O error: {}", e),
            CaveError::DockerError(msg) =>
                write!(f, "Docker error: {}", msg),
            CaveError::HomeNotFound =>
                write!(f, "Home not found."),
            CaveError::FileNotFound(msg) =>
                write!(f, "{}", msg),
            CaveError::VersionNotInstalled(ver) =>
                write!(f, "Invalid version : '{}', not installed. Run cave pin {}.", ver, ver),
            CaveError::HttpError(e) =>
                write!(f, "Error pulling image versions : {}", e),
            CaveError::NoDocker =>
                write!(f, "Docker not found. Please install Docker and try again."),
            CaveError::NoInternetConnection =>
                write!(f, "Error: No internet connection detected. Please check your network and try again."),
            CaveError::SerdeError(e) =>
                write!(f, "I/O error: {}", e),
            CaveError::CodeAsterError(msg) =>
            write!(f, "code_aster error: {}", msg),
            CaveError::TelemetryError(msg) =>
            write!(f, "telemetry error: {}", msg),
        }
    }
}

impl From<io::Error> for CaveError {
    fn from(e: io::Error) -> Self {
        CaveError::IoError(e)
    }
}

/// Sets the `code_aster` version to use, with an option to set it as the default.
///
/// - If `version` is `"stable"` or `"testing"`, resolves to the real version via [`version_under_tag`].
/// - Otherwise, validates the format `xx.x.xx` and pulls the version if it is missing.
///
/// # Errors
/// - [`CaveError::InvalidFormat`] if the version string is in an invalid format.
/// - [`CaveError::VersionNotAvailable`] if the version is not found locally or remotely.
/// - [`CaveError::UserAborted`] if the user cancels when asked to download.
/// - [`CaveError::IoError`] on file writing issues.
/// - [`CaveError::DockerError`] if a pull via Docker fails.
///
/// # Example
/// ```
/// set_version("22.0.1".to_string(), true).expect("Unable to set version");
/// ```
pub fn set_version(version: String, default_version: bool) -> Result<(), CaveError> {
    let true_version: String;

    if version == "stable" || version == "testing" {
        if !internet_available() {
            return Err(CaveError::NoInternetConnection);
        }
        true_version = version_under_tag(version.clone())?;
    } else {
        let version_regex = Regex::new(r"^\d{1,2}\.\d{1,2}\.\d{1,2}$").unwrap();
        if !version_regex.is_match(&version) {
            return Err(CaveError::InvalidFormat(version));
        }
        true_version = version.clone();
    }

    let exists_locally = exists_locally(&true_version)?;
    let version_ok = if exists_locally {
        true_version
    } else {
        let exists_remotely = exists_remotely(&true_version)?;
        if exists_remotely {
            println!("Version '{}' not installed. Download it? (y/n):", true_version);
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() == "y" {
                pull_version(&true_version)?;
                true_version
            } else {
                return Err(CaveError::UserAborted);
            }
        } else {
            return Err(CaveError::VersionNotAvailable(true_version));
        }
    };

    let path: PathBuf = if default_version {
        let home = dirs::home_dir().ok_or(CaveError::HomeNotFound)?;
        home.join(".cave")
    } else {
        PathBuf::from(".cave")
    };

    let version_to_write: String = if version == "stable" || version == "testing" {
        format!("{}:{}", version, version_ok)
    } else {
        version_ok
    };

    let mut file = fs::File::create(&path)?;
    writeln!(file, "{}", version_to_write)?;
    Ok(())
}

/// Runs `code_aster` with the currently set version from `.cave`.
///
/// - Optionally accepts a `.export` file as the last argument.
/// - Remaining arguments are passed directly to `run_aster`.
///
/// # Errors
/// - [`CaveError::VersionNotInstalled`] if the configured version is not installed locally.
/// - [`CaveError::FileNotFound`] if the `.export` file does not exist.
/// - Any error returned by [`docker_aster`].
///
/// # Example
/// ```
/// run_aster(&vec!["--help".to_string()]).expect("Failed to run code_aster");
/// ```
pub fn run_aster(args: &Vec<String>) -> Result<(), CaveError> {
    let version = read_cave_version()?;
    if !exists_locally(&version)? {
        return Err(CaveError::VersionNotInstalled(version));
    }

    let (export, rest_args): (Option<String>, Vec<String>) = match args.split_last() {
        Some((last, rest)) if last.ends_with(".export") => {
            find_export_file(last)?;
            (Some(last.clone()), rest.to_vec())
        }
        _ => (None, args.to_vec()),
    };

    docker_aster(&version, &export, &rest_args)?;
    Ok(())
}

/// Prints a list of locally available versions filtered by an optionnal prefix.
///
/// # Example
/// ```
/// print_local_versions("22".to_string()).unwrap();
/// ```
pub fn print_local_versions(prefix: String) -> Result<(), CaveError> {
    let versions = local_versions()?;
    let mut numeric_versions: Vec<_> = versions
        .into_iter()
        .filter(|v| v.chars().next().map_or(false, |c| c.is_ascii_digit()))
        .filter(|v| v.starts_with(&prefix))
        .collect();

    numeric_versions.sort_by(|a, b| version_cmp(a, b));

    if !numeric_versions.is_empty() {
        let per_line = 6;
        let column_width = 12;
        for chunk in numeric_versions.chunks(per_line) {
            let line = chunk
                .iter()
                .map(|v| format!("{:<width$}", v, width = column_width))
                .collect::<String>();
            println!("  {}", line.trim_end());
        }
    }
    Ok(())
}

/// Prints a list of remotely available versions filtered by a prefix.
///
/// - If a private registry is configured, also prints its versions.
/// - Labels which versions are `stable` or `testing`.
/// - Highlights installed versions in blue.
///
/// # Example
/// ```
/// let cfg = read_config().unwrap();
/// print_remote_versions("22".to_string(), cfg).unwrap();
/// ```
pub fn print_remote_versions(prefix: String) -> Result<(), CaveError> {
    // TODO : uncomment to have registry option, add , cfg: Config in the arguments
    //
    // if let Some(reg) = &cfg.registry {
    //     let registry_versions = registry_versions(&reg)?;
    //     println!("Versions on the registry : ");
    //     println!("{:#?}", registry_versions);
    // }

    if !internet_available() {
        return Err(CaveError::NoInternetConnection);
    }
    let versions = remote_versions()?;

    let mut numeric_versions: Vec<_> = versions
        .iter()
        .filter(|(tag, _)| tag.chars().next().unwrap_or('x').is_ascii_digit())
        .filter(|(tag, _)| tag.starts_with(&prefix))
        .cloned()
        .collect();

    numeric_versions.sort_by(|(a, _), (b, _)| version_cmp(a, b));

    if numeric_versions.is_empty() {
        println!("No code_aster versions found on simvia dockerhub");
    } else {
        println!("{:<15}{}", "Tag", "Date");
        let (stable_version, testing_version) = get_stable_and_testing()?;
        for (tag, date) in numeric_versions {
            let short_date = date
                .get(0..13)
                .map(|s| s.replace('T', " ") + "h")
                .unwrap_or_else(|| "unknown".to_string());
            let mut image = String::new();
            if tag == stable_version {
                image = "stable".to_string()
            }
            if tag == testing_version {
                image = "testing".to_string()
            }
            let installed = exists_locally(&tag)?;
            if installed {
                println!("{:<15}{:<15}{:<15}", tag.blue().bold(), short_date.blue().bold(), image);
            } else {
                println!("{:<15}{:<15}{:<15}", tag, short_date, image);
            }
        }
    }
    Ok(())
}

fn version_cmp(a: &str, b: &str) -> Ordering {
    let parse = |s: &str| {
        s.split('.')
            .filter_map(|part| part.parse::<u32>().ok())
            .collect::<Vec<_>>()
    };
    parse(a).cmp(&parse(b))
}

use std::net::TcpStream;
use std::time::Duration;

//check the internet connection 
fn internet_available() -> bool {
    TcpStream::connect_timeout(
        &"8.8.8.8:53".parse().unwrap(), // Google DNS
        Duration::from_secs(2)
    ).is_ok()
}


/// Reads the currently configured `code_aster` version from the `.cave` file.
///
/// This function checks in first the **local** `.cave` file in the current directory,
/// if not found search for the **global** version file in `~/.cave`
///
/// If the stored version is in the form `stable:<version>` or `testing:<version>`  
/// and `auto_update` is enabled in the configuration, it will:
/// - Check if the "stable" or "testing" tag now points to a newer version.
/// - Automatically update the `.cave` file if the newer version is already installed.
/// - Optionally prompt the user to install the updated version if missing.
///
/// # Returns
/// - The actual version string to be used (e.g., `"22.0.1"`).
///
/// # Errors
/// - [`CaveError::HomeNotFound`] if the HOME directory cannot be determined.
/// - [`CaveError::FileNotFound`] if no `.cave` file is found.
/// - [`CaveError::IoError`] if reading or writing `.cave` fails.
/// - [`CaveError::DockerError`] or [`CaveError::HttpError`] if checking for updates fails.
/// - [`CaveError::NoDocker`] if Docker is required and is not installed.
///
/// # Example
/// ```
/// let current_version = read_cave_version().unwrap();
/// println!("Currently configured version: {}", current_version);
/// ```
fn read_cave_version() -> Result<String, CaveError> {
    let home = dirs::home_dir().ok_or(CaveError::HomeNotFound)?;
    let config = read_config()?;
    let auto_update = config.auto_update;

    let mut cave_file: Option<PathBuf> = None;
    let global = home.join(".cave");
    if global.exists() {
        cave_file = Some(global);
    }
    let local = Path::new(".cave");
    if local.exists() {
        cave_file = Some(local.to_path_buf());
    }
    let cave_file = cave_file.ok_or_else(|| {
        CaveError::FileNotFound(
            "No version found. Use `cave use <version>` or `cave pin <version>`.".to_string(),
        )
    })?;

    let content = fs::read_to_string(&cave_file).map_err(CaveError::IoError)?;
    let content = content.trim();

    if content.starts_with("stable:") || content.starts_with("testing:") {
        let parts: Vec<&str> = content.splitn(2, ':').collect();
        let tag = parts[0];
        let old_version = parts[1];
        if auto_update {
            if internet_available() {
                let new_version = version_under_tag(tag.to_string())?;
                if new_version != old_version {
                    if !exists_locally(&new_version)? {
                        println!("{} version updated. Install new version? (y/n):", tag);
                        let mut input = String::new();
                        io::stdin().read_line(&mut input)?;
                        if input.trim().to_lowercase() == "y" {
                            pull_version(&new_version)?;
                            let version_to_write = format!("{}:{}", tag, new_version);
                            fs::write(&cave_file, version_to_write).map_err(CaveError::IoError)?;
                            return Ok(new_version);
                        }
                        return Ok(old_version.to_string());
                    }
                    let version_to_write = format!("{}:{}", tag, new_version);
                    fs::write(&cave_file, version_to_write).map_err(CaveError::IoError)?;
                    return Ok(new_version);
                }
            }
        }
        Ok(old_version.to_string())
    } else {
        Ok(content.to_string())
    }
}

pub fn find_export_file(requested: &str) -> Result<(), CaveError> {
    let path = Path::new(requested);
    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("export") {
        Ok(())
    } else {
        Err(CaveError::FileNotFound(format!(
            "Export file '{}' not found or invalid.",
            requested
        )))
    }
}

