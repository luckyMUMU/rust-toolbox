//! Type-safe slot value wrapper.
//!
//! Provides a wrapper around JSON values with type information
//! for safer data passing between workflow components.

use crate::error::{Result, WorkflowError};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// A type-safe wrapper around a JSON value.
///
/// Stores the original type name for debugging purposes
/// and provides type-checked extraction methods.
#[derive(Clone, Serialize, Deserialize)]
pub struct SlotValue {
    /// The underlying JSON value
    value: Value,
    /// The original Rust type name (for debugging)
    type_name: String,
}

impl SlotValue {
    /// Create a new slot value from any serializable type
    pub fn new<T: Serialize>(value: T) -> Result<Self> {
        let json_value = serde_json::to_value(value)
            .map_err(|e| WorkflowError::serialization(&format!("Failed to serialize: {}", e)))?;
        Ok(Self {
            value: json_value,
            type_name: std::any::type_name::<T>().to_string(),
        })
    }

    /// Create a slot value directly from a JSON value
    pub fn from_value(value: Value) -> Self {
        Self {
            value,
            type_name: "serde_json::Value".to_string(),
        }
    }

    /// Get the underlying JSON value
    pub fn value(&self) -> &Value {
        &self.value
    }

    /// Get the stored type name
    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    /// Extract the value as a specific type
    pub fn as_type<T: DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_value(self.value.clone()).map_err(|e| {
            WorkflowError::type_conversion(&format!(
                "Failed to convert slot value to {}: {}",
                std::any::type_name::<T>(),
                e
            ))
        })
    }

    /// Try to extract as a string
    pub fn as_string(&self) -> Option<&str> {
        self.value.as_str()
    }

    /// Try to extract as an i64
    pub fn as_i64(&self) -> Option<i64> {
        self.value.as_i64()
    }

    /// Try to extract as an f64
    pub fn as_f64(&self) -> Option<f64> {
        self.value.as_f64()
    }

    /// Try to extract as a bool
    pub fn as_bool(&self) -> Option<bool> {
        self.value.as_bool()
    }

    /// Try to extract as an array
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        self.value.as_array()
    }

    /// Try to extract as an object
    pub fn as_object(&self) -> Option<&serde_json::Map<String, Value>> {
        self.value.as_object()
    }

    /// Check if the value is null
    pub fn is_null(&self) -> bool {
        self.value.is_null()
    }

    /// Check if the value is a string
    pub fn is_string(&self) -> bool {
        self.value.is_string()
    }

    /// Check if the value is a number
    pub fn is_number(&self) -> bool {
        self.value.is_number()
    }

    /// Check if the value is a boolean
    pub fn is_bool(&self) -> bool {
        self.value.is_boolean()
    }

    /// Check if the value is an array
    pub fn is_array(&self) -> bool {
        self.value.is_array()
    }

    /// Check if the value is an object
    pub fn is_object(&self) -> bool {
        self.value.is_object()
    }

    /// Get a nested value using a dot-separated path
    ///
    /// # Example
    ///
    /// ```ignore
    /// let slot = SlotValue::new(json!({"user": {"name": "Alice"}})).unwrap();
    /// let name = slot.get_path("user.name");
    /// ```
    pub fn get_path(&self, path: &str) -> Option<&Value> {
        let mut current = &self.value;
        for part in path.split('.') {
            match current {
                Value::Object(map) => {
                    current = map.get(part)?;
                }
                Value::Array(arr) => {
                    let index: usize = part.parse().ok()?;
                    current = arr.get(index)?;
                }
                _ => return None,
            }
        }
        Some(current)
    }
}

impl fmt::Debug for SlotValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SlotValue")
            .field("type", &self.type_name)
            .field("value", &self.value)
            .finish()
    }
}

impl fmt::Display for SlotValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl From<Value> for SlotValue {
    fn from(value: Value) -> Self {
        Self::from_value(value)
    }
}

impl From<&str> for SlotValue {
    fn from(s: &str) -> Self {
        Self {
            value: Value::String(s.to_string()),
            type_name: "&str".to_string(),
        }
    }
}

impl From<String> for SlotValue {
    fn from(s: String) -> Self {
        Self {
            value: Value::String(s),
            type_name: "String".to_string(),
        }
    }
}

impl From<i64> for SlotValue {
    fn from(n: i64) -> Self {
        Self {
            value: Value::Number(n.into()),
            type_name: "i64".to_string(),
        }
    }
}

impl From<f64> for SlotValue {
    fn from(n: f64) -> Self {
        Self {
            value: serde_json::Number::from_f64(n)
                .map(Value::Number)
                .unwrap_or(Value::Null),
            type_name: "f64".to_string(),
        }
    }
}

impl From<bool> for SlotValue {
    fn from(b: bool) -> Self {
        Self {
            value: Value::Bool(b),
            type_name: "bool".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_slot_value_creation() {
        let slot = SlotValue::new("hello").unwrap();
        assert!(slot.is_string());
        assert_eq!(slot.as_string(), Some("hello"));

        let slot = SlotValue::new(42i64).unwrap();
        assert!(slot.is_number());
        assert_eq!(slot.as_i64(), Some(42));

        let slot = SlotValue::new(true).unwrap();
        assert!(slot.is_bool());
        assert_eq!(slot.as_bool(), Some(true));
    }

    #[test]
    fn test_slot_value_type_conversion() {
        let slot = SlotValue::new(json!({"name": "Alice", "age": 30})).unwrap();

        #[derive(Debug, Deserialize, PartialEq)]
        struct User {
            name: String,
            age: i32,
        }

        let user: User = slot.as_type().unwrap();
        assert_eq!(user.name, "Alice");
        assert_eq!(user.age, 30);
    }

    #[test]
    fn test_slot_value_path_access() {
        let slot = SlotValue::new(json!({
            "user": {
                "profile": {
                    "name": "Alice"
                }
            },
            "items": ["a", "b", "c"]
        }))
        .unwrap();

        assert_eq!(slot.get_path("user.profile.name"), Some(&json!("Alice")));
        assert_eq!(slot.get_path("items.1"), Some(&json!("b")));
        assert_eq!(slot.get_path("nonexistent"), None);
    }

    #[test]
    fn test_slot_value_from_impls() {
        let slot: SlotValue = "hello".into();
        assert_eq!(slot.as_string(), Some("hello"));

        let slot: SlotValue = 42i64.into();
        assert_eq!(slot.as_i64(), Some(42));

        let slot: SlotValue = true.into();
        assert_eq!(slot.as_bool(), Some(true));
    }
}
