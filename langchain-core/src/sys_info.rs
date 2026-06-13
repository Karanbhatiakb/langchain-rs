//! System information utilities for the LangChain framework.
//!
//! Provides [`get_sys_info`] which collects key system properties and
//! returns them as a [`serde_json::Value`]. This is useful for diagnostics,
//! health checks, and support tooling.

use crate::env::get_runtime_env;

/// Returns system information as a JSON value.
///
/// The returned object includes:
///
/// - `"langchain_version"` — the `langchain-core` crate version.
/// - `"rust_version"` — the Rust compiler version.
/// - `"os"` — the operating system name.
/// - `"arch"` — the CPU architecture.
/// - `"cpu_count"` — the number of logical CPU cores.
/// - `"thread_name"` — the name of the current thread (or `"unknown"`).
///
/// # Example
///
/// ```rust,no_run
/// use langchain_core::sys_info::get_sys_info;
///
/// let info = get_sys_info();
/// println!("{}", serde_json::to_string_pretty(&info).unwrap());
/// ```
pub fn get_sys_info() -> serde_json::Value {
    let env = get_runtime_env();

    let thread_name = std::thread::current()
        .name()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    serde_json::json!({
        "langchain_version": env.langchain_version,
        "rust_version": env.rust_version,
        "os": env.os,
        "arch": env.arch,
        "cpu_count": env.cpu_count,
        "thread_name": thread_name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_sys_info() {
        let info = get_sys_info();
        assert!(info.get("langchain_version").is_some());
        assert!(info.get("rust_version").is_some());
        assert!(info.get("os").is_some());
        assert!(info.get("arch").is_some());
        assert!(info.get("cpu_count").is_some());
        assert!(info.get("thread_name").is_some());
    }

    #[test]
    fn test_sys_info_values_are_strings() {
        let info = get_sys_info();
        assert!(info["langchain_version"].as_str().is_some());
        assert!(info["rust_version"].as_str().is_some());
        assert!(info["thread_name"].as_str().is_some());
    }

    #[test]
    fn test_sys_info_cpu_count_positive() {
        let info = get_sys_info();
        let cpu = info["cpu_count"].as_u64().unwrap_or(0);
        assert!(cpu >= 1);
    }

    #[test]
    fn test_sys_info_os_is_not_empty() {
        let info = get_sys_info();
        let os = info["os"].as_str().unwrap_or("");
        assert!(!os.is_empty());
    }

    #[test]
    fn test_sys_info_arch_is_not_empty() {
        let info = get_sys_info();
        let arch = info["arch"].as_str().unwrap_or("");
        assert!(!arch.is_empty());
    }

    #[test]
    fn test_sys_info_json_structure() {
        let info = get_sys_info();
        assert!(info.is_object());
        let obj = info.as_object().unwrap();
        assert!(obj.contains_key("langchain_version"));
        assert!(obj.contains_key("rust_version"));
        assert!(obj.contains_key("os"));
        assert!(obj.contains_key("arch"));
        assert!(obj.contains_key("cpu_count"));
        assert!(obj.contains_key("thread_name"));
        assert_eq!(obj.len(), 6);
    }

    #[test]
    fn test_sys_info_thread_name() {
        let info = get_sys_info();
        let name = info["thread_name"].as_str().unwrap_or("");
        assert!(!name.is_empty(), "thread name should not be empty");
    }

    #[test]
    fn test_sys_info_is_serializable() {
        let info = get_sys_info();
        let json = serde_json::to_string(&info).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(info, parsed);
    }
}
