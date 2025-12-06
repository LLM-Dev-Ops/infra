//! JSON utilities for LLM-Dev-Ops infrastructure.
//!
//! Provides:
//! - JSON value wrapper with path queries
//! - Streaming JSON parsing
//! - JSON diff and merge utilities
//! - WASM-compatible API

use infra_errors::{InfraError, InfraResult, SerializationFormat};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// JSON value wrapper with additional capabilities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Json(serde_json::Value);

impl Json {
    // Constructors

    /// Create a null JSON value
    #[must_use]
    pub fn null() -> Self {
        Self(serde_json::Value::Null)
    }

    /// Create a boolean JSON value
    #[must_use]
    pub fn bool(v: bool) -> Self {
        Self(serde_json::Value::Bool(v))
    }

    /// Create a number JSON value
    #[must_use]
    pub fn number(v: impl Into<serde_json::Number>) -> Self {
        Self(serde_json::Value::Number(v.into()))
    }

    /// Create a string JSON value
    #[must_use]
    pub fn string(v: impl Into<String>) -> Self {
        Self(serde_json::Value::String(v.into()))
    }

    /// Create an array JSON value
    #[must_use]
    pub fn array(v: Vec<Json>) -> Self {
        Self(serde_json::Value::Array(
            v.into_iter().map(|j| j.0).collect(),
        ))
    }

    /// Create an object JSON value
    #[must_use]
    pub fn object(v: impl IntoIterator<Item = (String, Json)>) -> Self {
        Self(serde_json::Value::Object(
            v.into_iter().map(|(k, j)| (k, j.0)).collect(),
        ))
    }

    // Parsing

    /// Parse JSON from a string
    pub fn parse(s: &str) -> InfraResult<Self> {
        serde_json::from_str(s).map(Self).map_err(|e| {
            InfraError::Serialization {
                format: SerializationFormat::Json,
                message: e.to_string(),
                location: Some(format!("line {}, column {}", e.line(), e.column())),
                context: None,
            }
        })
    }

    /// Parse JSON from bytes
    pub fn parse_bytes(bytes: &[u8]) -> InfraResult<Self> {
        serde_json::from_slice(bytes).map(Self).map_err(Into::into)
    }

    // Serialization

    /// Convert to a compact JSON string
    #[must_use]
    pub fn to_string(&self) -> String {
        serde_json::to_string(&self.0).unwrap_or_default()
    }

    /// Convert to a pretty-printed JSON string
    #[must_use]
    pub fn to_string_pretty(&self) -> String {
        serde_json::to_string_pretty(&self.0).unwrap_or_default()
    }

    /// Convert to bytes
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&self.0).unwrap_or_default()
    }

    // Type conversions

    /// Create from a serializable value
    pub fn from_value<T: Serialize>(value: &T) -> InfraResult<Self> {
        serde_json::to_value(value).map(Self).map_err(Into::into)
    }

    /// Convert to a deserializable value
    pub fn to_value<T: DeserializeOwned>(&self) -> InfraResult<T> {
        serde_json::from_value(self.0.clone()).map_err(Into::into)
    }

    /// Get the inner serde_json::Value
    #[must_use]
    pub fn into_inner(self) -> serde_json::Value {
        self.0
    }

    /// Get a reference to the inner value
    #[must_use]
    pub fn as_inner(&self) -> &serde_json::Value {
        &self.0
    }

    // Path queries

    /// Get a value by dot-notation path (e.g., "foo.bar.baz")
    #[must_use]
    pub fn get_path(&self, path: &str) -> Option<Json> {
        let mut current = &self.0;

        for part in path.split('.') {
            // Handle array index notation [0]
            if let Some(idx_str) = part.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                let idx: usize = idx_str.parse().ok()?;
                current = current.get(idx)?;
            } else {
                current = current.get(part)?;
            }
        }

        Some(Json(current.clone()))
    }

    /// Set a value at a dot-notation path
    pub fn set_path(&mut self, path: &str, value: Json) -> InfraResult<()> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &mut self.0;

        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part - set the value
                if let Some(obj) = current.as_object_mut() {
                    obj.insert((*part).to_string(), value.0);
                    return Ok(());
                }
                return Err(InfraError::validation("Cannot set path on non-object"));
            }

            // Navigate deeper
            if let Some(obj) = current.as_object_mut() {
                current = obj.entry(*part).or_insert(serde_json::Value::Object(
                    serde_json::Map::new(),
                ));
            } else {
                return Err(InfraError::validation("Cannot navigate through non-object"));
            }
        }

        Ok(())
    }

    // Type checks

    #[must_use]
    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }

    #[must_use]
    pub fn is_bool(&self) -> bool {
        self.0.is_boolean()
    }

    #[must_use]
    pub fn is_number(&self) -> bool {
        self.0.is_number()
    }

    #[must_use]
    pub fn is_string(&self) -> bool {
        self.0.is_string()
    }

    #[must_use]
    pub fn is_array(&self) -> bool {
        self.0.is_array()
    }

    #[must_use]
    pub fn is_object(&self) -> bool {
        self.0.is_object()
    }

    // Accessors

    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        self.0.as_str()
    }

    #[must_use]
    pub fn as_i64(&self) -> Option<i64> {
        self.0.as_i64()
    }

    #[must_use]
    pub fn as_u64(&self) -> Option<u64> {
        self.0.as_u64()
    }

    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        self.0.as_f64()
    }

    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        self.0.as_bool()
    }

    #[must_use]
    pub fn as_array(&self) -> Option<Vec<Json>> {
        self.0.as_array().map(|arr| {
            arr.iter()
                .map(|v| Json(v.clone()))
                .collect()
        })
    }

    #[must_use]
    pub fn as_object(&self) -> Option<HashMap<String, Json>> {
        self.0.as_object().map(|obj| {
            obj.iter()
                .map(|(k, v)| (k.clone(), Json(v.clone())))
                .collect()
        })
    }
}

impl Default for Json {
    fn default() -> Self {
        Self::null()
    }
}

impl From<serde_json::Value> for Json {
    fn from(v: serde_json::Value) -> Self {
        Self(v)
    }
}

impl From<Json> for serde_json::Value {
    fn from(j: Json) -> Self {
        j.0
    }
}

impl From<&str> for Json {
    fn from(s: &str) -> Self {
        Self::string(s)
    }
}

impl From<String> for Json {
    fn from(s: String) -> Self {
        Self::string(s)
    }
}

impl From<i64> for Json {
    fn from(n: i64) -> Self {
        Self(serde_json::Value::Number(n.into()))
    }
}

impl From<bool> for Json {
    fn from(b: bool) -> Self {
        Self::bool(b)
    }
}

impl std::fmt::Display for Json {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// JSON diff result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JsonDiff {
    Added { path: String, value: Json },
    Removed { path: String, value: Json },
    Changed { path: String, old: Json, new: Json },
}

/// Compute the diff between two JSON values
#[must_use]
pub fn diff(a: &Json, b: &Json) -> Vec<JsonDiff> {
    diff_recursive(&a.0, &b.0, String::new())
}

fn diff_recursive(a: &serde_json::Value, b: &serde_json::Value, path: String) -> Vec<JsonDiff> {
    let mut diffs = Vec::new();

    match (a, b) {
        (serde_json::Value::Object(obj_a), serde_json::Value::Object(obj_b)) => {
            // Find added/changed keys
            for (key, val_b) in obj_b {
                let new_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };

                match obj_a.get(key) {
                    Some(val_a) => {
                        diffs.extend(diff_recursive(val_a, val_b, new_path));
                    }
                    None => {
                        diffs.push(JsonDiff::Added {
                            path: new_path,
                            value: Json(val_b.clone()),
                        });
                    }
                }
            }

            // Find removed keys
            for (key, val_a) in obj_a {
                if !obj_b.contains_key(key) {
                    let new_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{path}.{key}")
                    };
                    diffs.push(JsonDiff::Removed {
                        path: new_path,
                        value: Json(val_a.clone()),
                    });
                }
            }
        }
        (serde_json::Value::Array(arr_a), serde_json::Value::Array(arr_b)) => {
            for (i, (val_a, val_b)) in arr_a.iter().zip(arr_b.iter()).enumerate() {
                diffs.extend(diff_recursive(val_a, val_b, format!("{path}[{i}]")));
            }

            // Handle length differences
            if arr_b.len() > arr_a.len() {
                for (i, val) in arr_b[arr_a.len()..].iter().enumerate() {
                    diffs.push(JsonDiff::Added {
                        path: format!("{}[{}]", path, arr_a.len() + i),
                        value: Json(val.clone()),
                    });
                }
            } else if arr_a.len() > arr_b.len() {
                for (i, val) in arr_a[arr_b.len()..].iter().enumerate() {
                    diffs.push(JsonDiff::Removed {
                        path: format!("{}[{}]", path, arr_b.len() + i),
                        value: Json(val.clone()),
                    });
                }
            }
        }
        _ if a != b => {
            diffs.push(JsonDiff::Changed {
                path,
                old: Json(a.clone()),
                new: Json(b.clone()),
            });
        }
        _ => {}
    }

    diffs
}

/// Merge two JSON values (RFC 7396 JSON Merge Patch)
#[must_use]
pub fn merge(base: &Json, patch: &Json) -> Json {
    Json(merge_recursive(&base.0, &patch.0))
}

fn merge_recursive(base: &serde_json::Value, patch: &serde_json::Value) -> serde_json::Value {
    match (base, patch) {
        (serde_json::Value::Object(base_obj), serde_json::Value::Object(patch_obj)) => {
            let mut result = base_obj.clone();

            for (key, patch_val) in patch_obj {
                if patch_val.is_null() {
                    result.remove(key);
                } else if let Some(base_val) = result.get(key) {
                    result.insert(key.clone(), merge_recursive(base_val, patch_val));
                } else {
                    result.insert(key.clone(), patch_val.clone());
                }
            }

            serde_json::Value::Object(result)
        }
        _ => patch.clone(),
    }
}

/// Macro for creating JSON objects easily
#[macro_export]
macro_rules! json {
    ($($json:tt)+) => {
        $crate::Json::from(serde_json::json!($($json)+))
    };
}

// WASM bindings
#[cfg(feature = "wasm")]
mod wasm {
    use super::*;

    #[wasm_bindgen]
    pub struct JsJson {
        inner: Json,
    }

    #[wasm_bindgen]
    impl JsJson {
        #[wasm_bindgen(constructor)]
        pub fn parse(s: &str) -> Result<JsJson, JsValue> {
            Json::parse(s)
                .map(|j| JsJson { inner: j })
                .map_err(|e| JsValue::from_str(&e.to_string()))
        }

        pub fn stringify(&self) -> String {
            self.inner.to_string()
        }

        #[wasm_bindgen(js_name = stringifyPretty)]
        pub fn stringify_pretty(&self) -> String {
            self.inner.to_string_pretty()
        }

        pub fn get(&self, path: &str) -> JsValue {
            match self.inner.get_path(path) {
                Some(j) => JsValue::from_str(&j.to_string()),
                None => JsValue::UNDEFINED,
            }
        }

        #[wasm_bindgen(js_name = isNull)]
        pub fn is_null(&self) -> bool {
            self.inner.is_null()
        }

        #[wasm_bindgen(js_name = isObject)]
        pub fn is_object(&self) -> bool {
            self.inner.is_object()
        }

        #[wasm_bindgen(js_name = isArray)]
        pub fn is_array(&self) -> bool {
            self.inner.is_array()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_parse() {
        let json = Json::parse(r#"{"name": "test", "value": 42}"#).unwrap();
        assert!(json.is_object());
        assert_eq!(json.get_path("name").unwrap().as_str(), Some("test"));
        assert_eq!(json.get_path("value").unwrap().as_i64(), Some(42));
    }

    #[test]
    fn test_json_path() {
        let json = Json::parse(r#"{"a": {"b": {"c": 123}}}"#).unwrap();
        assert_eq!(json.get_path("a.b.c").unwrap().as_i64(), Some(123));
    }

    #[test]
    fn test_json_diff() {
        let a = Json::parse(r#"{"x": 1, "y": 2}"#).unwrap();
        let b = Json::parse(r#"{"x": 1, "y": 3, "z": 4}"#).unwrap();

        let diffs = diff(&a, &b);
        assert_eq!(diffs.len(), 2);
    }

    #[test]
    fn test_json_merge() {
        let base = Json::parse(r#"{"a": 1, "b": 2}"#).unwrap();
        let patch = Json::parse(r#"{"b": 3, "c": 4}"#).unwrap();

        let result = merge(&base, &patch);
        assert_eq!(result.get_path("a").unwrap().as_i64(), Some(1));
        assert_eq!(result.get_path("b").unwrap().as_i64(), Some(3));
        assert_eq!(result.get_path("c").unwrap().as_i64(), Some(4));
    }
}
