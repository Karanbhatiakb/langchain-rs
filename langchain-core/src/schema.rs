//! Schema validation utilities for JSON schemas.

use crate::errors::*;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct SchemaProperty {
    pub name: String,
    pub prop_type: SchemaType,
    pub required: bool,
    pub description: Option<String>,
    pub default: Option<Value>,
    pub enum_values: Option<Vec<Value>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SchemaType {
    String,
    Number,
    Integer,
    Boolean,
    Array,
    Object,
    Null,
    Any,
}

impl SchemaType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "string" => SchemaType::String,
            "number" => SchemaType::Number,
            "integer" => SchemaType::Integer,
            "boolean" => SchemaType::Boolean,
            "array" => SchemaType::Array,
            "object" => SchemaType::Object,
            "null" => SchemaType::Null,
            _ => SchemaType::Any,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            SchemaType::String => "string",
            SchemaType::Number => "number",
            SchemaType::Integer => "integer",
            SchemaType::Boolean => "boolean",
            SchemaType::Array => "array",
            SchemaType::Object => "object",
            SchemaType::Null => "null",
            SchemaType::Any => "any",
        }
    }
}

pub fn validate_json_against_schema(value: &Value, schema: &Value) -> Result<()> {
    if let Some(schema_type) = schema.get("type").and_then(|t| t.as_str()) {
        validate_type(value, schema_type)?;
    }

    if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
        if let Value::Object(obj) = value {
            let present: HashSet<&str> = obj.keys().map(|k| k.as_str()).collect();
            for req in required {
                if let Some(key) = req.as_str() {
                    if !present.contains(key) {
                        return Err(ChainError::ValidationError(format!(
                            "Missing required field: {}",
                            key
                        )));
                    }
                }
            }
        }
    }

    if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
        if let Value::Object(obj) = value {
            for (key, prop_schema) in properties {
                if let Some(val) = obj.get(key) {
                    validate_json_against_schema(val, prop_schema)?;
                }
            }
        }
    }

    if let Some(items) = schema.get("items") {
        if let Value::Array(arr) = value {
            for item in arr {
                validate_json_against_schema(item, items)?;
            }
        }
    }

    if let Some(enum_vals) = schema.get("enum").and_then(|e| e.as_array()) {
        if !enum_vals.contains(value) {
            return Err(ChainError::ValidationError(format!(
                "Value {:?} not in enum {:?}",
                value, enum_vals
            )));
        }
    }

    if let Some(min) = schema.get("minimum").and_then(|v| v.as_f64()) {
        if let Some(num) = value.as_f64() {
            if num < min {
                return Err(ChainError::ValidationError(format!(
                    "Value {} is less than minimum {}",
                    num, min
                )));
            }
        }
    }

    if let Some(max) = schema.get("maximum").and_then(|v| v.as_f64()) {
        if let Some(num) = value.as_f64() {
            if num > max {
                return Err(ChainError::ValidationError(format!(
                    "Value {} is greater than maximum {}",
                    num, max
                )));
            }
        }
    }

    if let Some(min_len) = schema.get("minLength").and_then(|v| v.as_u64()) {
        if let Some(s) = value.as_str() {
            if (s.len() as u64) < min_len {
                return Err(ChainError::ValidationError(format!(
                    "String length {} is less than minLength {}",
                    s.len(),
                    min_len
                )));
            }
        }
    }

    if let Some(max_len) = schema.get("maxLength").and_then(|v| v.as_u64()) {
        if let Some(s) = value.as_str() {
            if (s.len() as u64) > max_len {
                return Err(ChainError::ValidationError(format!(
                    "String length {} is greater than maxLength {}",
                    s.len(),
                    max_len
                )));
            }
        }
    }

    Ok(())
}

fn validate_type(value: &Value, expected_type: &str) -> Result<()> {
    let valid = match expected_type {
        "string" => value.is_string(),
        "number" => value.is_number(),
        "integer" => value.is_i64() || value.is_u64(),
        "boolean" => value.is_boolean(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        "null" => value.is_null(),
        _ => true,
    };

    if !valid {
        return Err(ChainError::ValidationError(format!(
            "Expected type '{}' but got value: {:?}",
            expected_type, value
        )));
    }

    Ok(())
}

pub fn coerce_type(value: &Value, target_type: &SchemaType) -> Result<Value> {
    match target_type {
        SchemaType::String => Ok(Value::String(match value {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        })),
        SchemaType::Number => match value {
            Value::Number(n) => Ok(Value::Number(n.clone())),
            Value::String(s) => s
                .parse::<f64>()
                .map(|f| serde_json::to_value(f).unwrap_or_default())
                .map_err(|_| ChainError::ValidationError(format!("Cannot coerce '{}' to number", s))),
            _ => Err(ChainError::ValidationError(format!(
                "Cannot coerce {:?} to number",
                value
            ))),
        },
        SchemaType::Integer => match value {
            Value::Number(n) => Ok(Value::Number(n.clone())),
            Value::String(s) => s
                .parse::<i64>()
                .map(|i| serde_json::to_value(i).unwrap_or_default())
                .map_err(|_| ChainError::ValidationError(format!("Cannot coerce '{}' to integer", s))),
            _ => Err(ChainError::ValidationError(format!(
                "Cannot coerce {:?} to integer",
                value
            ))),
        },
        SchemaType::Boolean => match value {
            Value::Bool(b) => Ok(Value::Bool(*b)),
            Value::String(s) => match s.to_lowercase().as_str() {
                "true" | "1" | "yes" => Ok(Value::Bool(true)),
                "false" | "0" | "no" => Ok(Value::Bool(false)),
                _ => Err(ChainError::ValidationError(format!(
                    "Cannot coerce '{}' to boolean",
                    s
                ))),
            },
            _ => Err(ChainError::ValidationError(format!(
                "Cannot coerce {:?} to boolean",
                value
            ))),
        },
        SchemaType::Array => match value {
            Value::Array(_) => Ok(value.clone()),
            _ => Ok(Value::Array(vec![value.clone()])),
        },
        SchemaType::Object => match value {
            Value::Object(_) => Ok(value.clone()),
            _ => Err(ChainError::ValidationError(format!(
                "Cannot coerce {:?} to object",
                value
            ))),
        },
        SchemaType::Null => Ok(Value::Null),
        SchemaType::Any => Ok(value.clone()),
    }
}

pub fn extract_required_fields(schema: &Value) -> Vec<String> {
    schema
        .get("required")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

pub fn extract_properties(schema: &Value) -> HashMap<String, SchemaProperty> {
    let mut props = HashMap::new();
    let required: HashSet<String> = schema
        .get("required")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
        for (name, prop_schema) in properties {
            let prop_type = prop_schema
                .get("type")
                .and_then(|t| t.as_str())
                .map(SchemaType::from_str)
                .unwrap_or(SchemaType::Any);

            let description = prop_schema
                .get("description")
                .and_then(|d| d.as_str())
                .map(|s| s.to_string());

            let default = prop_schema.get("default").cloned();

            let enum_values = prop_schema
                .get("enum")
                .and_then(|e| e.as_array())
                .map(|a| a.clone());

            props.insert(
                name.clone(),
                SchemaProperty {
                    name: name.clone(),
                    prop_type,
                    required: required.contains(name),
                    description,
                    default,
                    enum_values,
                },
            );
        }
    }

    props
}

pub fn fill_defaults(value: &mut Value, schema: &Value) -> Result<()> {
    if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
        if let Value::Object(obj) = value {
            for (key, prop_schema) in properties {
                if !obj.contains_key(key) {
                    if let Some(default) = prop_schema.get("default") {
                        obj.insert(key.clone(), default.clone());
                    }
                }
            }
        }
    }
    Ok(())
}
