//! Global runtime configuration for the LangChain framework.
//!
//! Provides thread-safe accessors for debug/verbose flags and the global
//! LLM cache. These settings are used throughout the framework to control
//! logging verbosity and response caching behavior.

use crate::caches::BaseCache;
use std::sync::Arc;

static DEBUG: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static VERBOSE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

use std::sync::OnceLock;

static LLM_CACHE: OnceLock<Option<Arc<dyn BaseCache>>> = OnceLock::new();

/// Returns the current global debug flag.
///
/// When `true`, components throughout the framework emit extra diagnostic
/// output.
pub fn get_debug() -> bool {
    DEBUG.load(std::sync::atomic::Ordering::Relaxed)
}

/// Sets the global debug flag.
pub fn set_debug(val: bool) {
    DEBUG.store(val, std::sync::atomic::Ordering::Relaxed);
}

/// Returns the current global verbose flag.
///
/// When `true`, components log additional informational messages.
pub fn get_verbose() -> bool {
    VERBOSE.load(std::sync::atomic::Ordering::Relaxed)
}

/// Sets the global verbose flag.
pub fn set_verbose(val: bool) {
    VERBOSE.store(val, std::sync::atomic::Ordering::Relaxed);
}

/// Returns a reference to the global LLM cache, if one has been set.
///
/// The cache is an [`Arc<dyn BaseCache>`] shared across all threads.
/// Returns `None` if no cache has been configured.
pub fn get_llm_cache() -> Option<&'static Option<Arc<dyn BaseCache>>> {
    Some(LLM_CACHE.get_or_init(|| None))
}

/// Sets the global LLM cache.
///
/// Pass `None` to disable caching. This can only be set once; subsequent
/// calls will be silently ignored (the first value wins), matching the
/// behaviour of [`OnceLock`].
pub fn set_llm_cache(cache: Option<Arc<dyn BaseCache>>) {
    let _ = LLM_CACHE.set(cache);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_flag() {
        let original = get_debug();
        set_debug(true);
        assert!(get_debug());
        set_debug(false);
        assert!(!get_debug());
        set_debug(original);
    }

    #[test]
    fn test_verbose_flag() {
        let original = get_verbose();
        set_verbose(true);
        assert!(get_verbose());
        set_verbose(false);
        assert!(!get_verbose());
        set_verbose(original);
    }

    #[test]
    fn test_debug_default_false() {
        let original = get_debug();
        set_debug(false);
        assert!(!get_debug());
        set_debug(original);
    }

    #[test]
    fn test_verbose_default_false() {
        let original = get_verbose();
        set_verbose(false);
        assert!(!get_verbose());
        set_verbose(original);
    }

    #[test]
    fn test_llm_cache_none_by_default() {
        let cache = get_llm_cache();
        assert!(cache.is_some());
        assert!(cache.unwrap().is_none());
    }

    #[test]
    fn test_debug_flag_toggle() {
        let original = get_debug();
        set_debug(true);
        assert!(get_debug());
        set_debug(false);
        assert!(!get_debug());
        set_debug(true);
        assert!(get_debug());
        set_debug(original);
    }

    #[test]
    fn test_verbose_independent_of_debug() {
        let orig_debug = get_debug();
        let orig_verbose = get_verbose();
        set_debug(true);
        set_verbose(false);
        assert!(get_debug());
        assert!(!get_verbose());
        set_debug(orig_debug);
        set_verbose(orig_verbose);
    }
}
