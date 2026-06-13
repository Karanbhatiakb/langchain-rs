//! Runtime environment detection for the LangChain framework.
//!
//! Provides [`get_runtime_env`] which inspects the current process and
//! operating system to build a [`RuntimeEnv`] snapshot. This is useful
//! for telemetry, debugging, and conditional feature selection.

use serde::{Deserialize, Serialize};

/// A snapshot of the current runtime environment.
///
/// Captures the LangChain crate version, Rust toolchain version, operating
/// system, CPU architecture, and the number of available CPU cores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeEnv {
    /// The version of the `langchain-core` crate.
    pub langchain_version: String,
    /// The Rust toolchain version used to compile the crate (equivalent to
    /// the `python_version` field in the Python LangChain runtime env).
    pub rust_version: String,
    /// The operating system name (e.g. `"macos"`, `"linux"`, `"windows"`).
    pub os: String,
    /// The CPU architecture (e.g. `"x86_64"`, `"aarch64"`).
    pub arch: String,
    /// The number of logical CPU cores available.
    pub cpu_count: usize,
}

/// Returns a [`RuntimeEnv`] describing the current runtime.
///
/// The `langchain_version` is set to the crate version from `Cargo.toml`
/// (via `env!`). The `rust_version` is read from the `RUSTC` compiler
/// at compile time. The `os` and `arch` fields are derived from
/// `std::env::consts`, and `cpu_count` is obtained from
/// `std::thread::available_parallelism`.
///
/// # Example
///
/// ```rust,no_run
/// use langchain_core::env::get_runtime_env;
///
/// let env = get_runtime_env();
/// println!("Running on {} ({})", env.os, env.arch);
/// ```
pub fn get_runtime_env() -> RuntimeEnv {
    RuntimeEnv {
        langchain_version: env!("CARGO_PKG_VERSION").to_string(),
        rust_version: rustc_version_runtime(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        cpu_count: std::thread::available_parallelism()
            .map(|v| v.get())
            .unwrap_or(1),
    }
}

/// Best-effort detection of the Rust compiler version at runtime.
///
/// Tries the `RUSTC` environment variable first (which Cargo sets during
/// builds), then falls back to running `rustc --version`. Returns
/// `"unknown"` if neither approach succeeds.
fn rustc_version_runtime() -> String {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let output = std::process::Command::new(&rustc)
        .arg("--version")
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let version = String::from_utf8_lossy(&out.stdout);
            version
                .trim()
                .strip_prefix("rustc ")
                .unwrap_or(version.trim())
                .to_string()
        }
        _ => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_runtime_env() {
        let env = get_runtime_env();
        assert!(!env.langchain_version.is_empty());
        assert!(!env.rust_version.is_empty());
        assert!(!env.os.is_empty());
        assert!(!env.arch.is_empty());
        assert!(env.cpu_count > 0);
    }
}
