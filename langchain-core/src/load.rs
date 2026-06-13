//! Serialization and deserialization utilities.
//!
//! Provides the [`Serializable`] trait plus helper functions for JSON-based
//! serialization ([`serde_dumps`], [`serde_loads`]) and JSON-value-based
//! conversion ([`dumpd`], [`loadd`]).

use crate::errors::*;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// A trait for objects that can be serialized to and deserialized from JSON.
///
/// Implementors provide `dumps` (to JSON string) and `loads` (from JSON
/// string) methods.
pub trait Serializable: Send + Sync + 'static {
    /// Serializes this object to a JSON string.
    ///
    /// # Errors
    /// Returns [`ChainError::SerializationError`] if serialization fails.
    fn dumps(&self) -> Result<String>;

    /// Deserializes an object of this type from a JSON string.
    ///
    /// # Errors
    /// Returns [`ChainError::SerializationError`] if deserialization fails.
    fn loads(s: &str) -> Result<Self>
    where
        Self: Sized;
}

/// Serializes a value to a JSON string.
///
/// This is the free-function equivalent of [`Serializable::dumps`] for any
/// [`Serialize`] type.
///
/// # Errors
/// Returns [`ChainError::SerializationError`] if serialization fails.
pub fn serde_dumps<T: Serialize>(val: &T) -> Result<String> {
    serde_json::to_string(val)
        .map_err(|e| ChainError::SerializationError(format!("{}", e)))
}

/// Deserializes a value from a JSON string.
///
/// This is the free-function equivalent of [`Serializable::loads`] for any
/// [`DeserializeOwned`] type.
///
/// # Errors
/// Returns [`ChainError::SerializationError`] if deserialization fails.
pub fn serde_loads<T: DeserializeOwned>(s: &str) -> Result<T> {
    serde_json::from_str(s)
        .map_err(|e| ChainError::SerializationError(format!("{}", e)))
}

/// Serializes a value to a [`serde_json::Value`].
///
/// Useful when you need an intermediate JSON representation (e.g., to
/// embed inside a larger structure) rather than a string.
///
/// # Errors
/// Returns [`ChainError::SerializationError`] if serialization fails.
pub fn dumpd<T: Serialize>(val: &T) -> Result<serde_json::Value> {
    serde_json::to_value(val)
        .map_err(|e| ChainError::SerializationError(format!("{}", e)))
}

/// Deserializes a value from a [`serde_json::Value`].
///
/// The inverse of [`dumpd`].
///
/// # Errors
/// Returns [`ChainError::SerializationError`] if deserialization fails.
pub fn loadd<T: DeserializeOwned>(val: &serde_json::Value) -> Result<T> {
    serde_json::from_value(val.clone())
        .map_err(|e| ChainError::SerializationError(format!("{}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Point {
        x: i32,
        y: i32,
    }

    impl Serializable for Point {
        fn dumps(&self) -> Result<String> {
            serde_dumps(self)
        }

        fn loads(s: &str) -> Result<Self> {
            serde_loads(s)
        }
    }

    #[test]
    fn test_serde_dumps_loads_roundtrip() {
        let p = Point { x: 1, y: 2 };
        let s = serde_dumps(&p).unwrap();
        let p2: Point = serde_loads(&s).unwrap();
        assert_eq!(p, p2);
    }

    #[test]
    fn test_dumpd_loadd_roundtrip() {
        let p = Point { x: 3, y: 4 };
        let v = dumpd(&p).unwrap();
        let p2: Point = loadd(&v).unwrap();
        assert_eq!(p, p2);
    }

    #[test]
    fn test_serde_dumps_hashmap() {
        let m = HashMap::from([("key".to_string(), "value".to_string())]);
        let s = serde_dumps(&m).unwrap();
        assert!(s.contains("key"));
        assert!(s.contains("value"));
    }

    #[test]
    fn test_serde_loads_invalid_json() {
        let result: Result<Point> = serde_loads("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_loadd_invalid_value() {
        let v = serde_json::json!("not an object");
        let result: Result<Point> = loadd(&v);
        assert!(result.is_err());
    }

    #[test]
    fn test_serializable_trait() {
        let p = Point { x: 5, y: 6 };
        let s = p.dumps().unwrap();
        let p2 = Point::loads(&s).unwrap();
        assert_eq!(p, p2);
    }

    #[test]
    fn test_serde_dumps_empty_struct() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Empty;
        let v = Empty;
        let s = serde_dumps(&v).unwrap();
        let v2: Empty = serde_loads(&s).unwrap();
        assert_eq!(v, v2);
    }

    #[test]
    fn test_serde_dumps_nested() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Outer {
            inner: Point,
            label: String,
        }
        let v = Outer { inner: Point { x: 10, y: 20 }, label: "test".into() };
        let s = serde_dumps(&v).unwrap();
        let v2: Outer = serde_loads(&s).unwrap();
        assert_eq!(v, v2);
    }

    #[test]
    fn test_dumpd_nested_json() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Wrapper { items: Vec<Point> }
        let v = Wrapper { items: vec![Point { x: 1, y: 2 }, Point { x: 3, y: 4 }] };
        let json_val = dumpd(&v).unwrap();
        let v2: Wrapper = loadd(&json_val).unwrap();
        assert_eq!(v, v2);
    }

    #[test]
    fn test_serde_loads_empty_string() {
        let result: Result<Point> = serde_loads("");
        assert!(result.is_err());
    }

    #[test]
    fn test_serde_dumps_roundtrip_complex_key() {
        let mut map = std::collections::BTreeMap::new();
        map.insert("a".to_string(), vec![1, 2, 3]);
        map.insert("b".to_string(), vec![4, 5, 6]);
        let s = serde_dumps(&map).unwrap();
        let map2: std::collections::BTreeMap<String, Vec<i32>> = serde_loads(&s).unwrap();
        assert_eq!(map, map2);
    }

    #[test]
    fn test_dumpd_large_value() {
        let v: Vec<Point> = (0..100).map(|i| Point { x: i, y: i * 2 }).collect();
        let json_val = dumpd(&v).unwrap();
        let v2: Vec<Point> = loadd(&json_val).unwrap();
        assert_eq!(v.len(), v2.len());
        assert_eq!(v[0], v2[0]);
        assert_eq!(v[99], v2[99]);
    }

    #[test]
    fn test_serializable_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        fn assert_send_sync<T: Send + Sync + 'static>() {}
        assert_send_sync::<Point>();
        assert_send::<Point>();
        assert_sync::<Point>();
    }

    #[test]
    fn test_serde_dumps_unicode() {
        let s = serde_dumps(&"héllo wörld 🎉").unwrap();
        let v: String = serde_loads(&s).unwrap();
        assert_eq!(v, "héllo wörld 🎉");
    }
}
