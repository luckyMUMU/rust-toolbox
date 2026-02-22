//! Data context and slot system for workflow execution.
//!
//! This module provides the data passing mechanism between workflow components,
//! inspired by LiteFlow's Slot concept.
//!
//! # Key Features
//!
//! - **Type-safe slot values**: Values are wrapped with type information
//! - **Scoped contexts**: Support for nested execution scopes (loops, conditions)
//! - **Thread-safe global slots**: Shared across the entire workflow execution
//! - **Node-local slots**: Private to each component execution

pub mod slot;

pub use slot::SlotValue;

use crate::error::{Result, WorkflowError};
use dashmap::DashMap;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Data context for workflow execution.
///
/// Provides a mechanism for components to share data through named slots.
/// Supports both global (workflow-wide) and node-local (component-private) slots.
///
/// # Example
///
/// ```ignore
/// use workflow_toolkit::workflow::context::DataContext;
/// use serde_json::json;
///
/// let mut context = DataContext::new();
///
/// // Set a global slot
/// context.set_global("user_id", json!(12345)).unwrap();
///
/// // Get a typed value from global slot
/// let user_id: i64 = context.get_global("user_id").unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct DataContext {
    /// Global slots shared across the entire workflow
    global_slots: Arc<DashMap<String, SlotValue>>,

    /// Node-local slots (private to current component)
    node_slots: HashMap<String, SlotValue>,

    /// Parent node chain (for nested scopes like loops/conditions)
    parent_chain: Vec<String>,

    /// Current node ID (if in a node execution context)
    current_node: Option<String>,
}

impl DataContext {
    /// Create a new empty data context
    pub fn new() -> Self {
        Self {
            global_slots: Arc::new(DashMap::new()),
            node_slots: HashMap::new(),
            parent_chain: Vec::new(),
            current_node: None,
        }
    }

    /// Create a new context with initial global values
    pub fn with_initial_values(values: HashMap<String, Value>) -> Result<Self> {
        let context = Self::new();
        for (key, value) in values {
            context.set_global_raw(&key, value)?;
        }
        Ok(context)
    }

    // ==================== Global Slot Operations ====================

    /// Set a global slot value (any serializable type)
    pub fn set_global<T: Serialize>(&self, key: impl Into<String>, value: T) -> Result<()> {
        let slot_value = SlotValue::new(value)?;
        self.global_slots.insert(key.into(), slot_value);
        Ok(())
    }

    /// Set a global slot value from raw JSON
    pub fn set_global_raw(&self, key: impl Into<String>, value: Value) -> Result<()> {
        let slot_value = SlotValue::from_value(value);
        self.global_slots.insert(key.into(), slot_value);
        Ok(())
    }

    /// Get a typed value from global slot
    pub fn get_global<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
        let slot = self
            .global_slots
            .get(key)
            .ok_or_else(|| WorkflowError::slot_not_found(key))?;
        slot.as_type()
    }

    /// Get a raw JSON value from global slot
    pub fn get_global_raw(&self, key: &str) -> Result<Value> {
        let slot = self
            .global_slots
            .get(key)
            .ok_or_else(|| WorkflowError::slot_not_found(key))?;
        Ok(slot.value().value().clone())
    }

    /// Check if a global slot exists
    pub fn has_global(&self, key: &str) -> bool {
        self.global_slots.contains_key(key)
    }

    /// Remove a global slot
    pub fn remove_global(&self, key: &str) -> Option<SlotValue> {
        self.global_slots.remove(key).map(|(_, v)| v)
    }

    /// Get all global slot keys
    pub fn global_keys(&self) -> Vec<String> {
        self.global_slots
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    // ==================== Node-Local Slot Operations ====================

    /// Set a node-local slot value
    pub fn set_node<T: Serialize>(&mut self, key: impl Into<String>, value: T) -> Result<()> {
        let slot_value = SlotValue::new(value)?;
        self.node_slots.insert(key.into(), slot_value);
        Ok(())
    }

    /// Set a node-local slot value from raw JSON
    pub fn set_node_raw(&mut self, key: impl Into<String>, value: Value) {
        let slot_value = SlotValue::from_value(value);
        self.node_slots.insert(key.into(), slot_value);
    }

    /// Get a typed value from node-local slot
    pub fn get_node<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
        let slot = self
            .node_slots
            .get(key)
            .ok_or_else(|| WorkflowError::slot_not_found(format!("node:{}", key)))?;
        slot.as_type()
    }

    /// Get a typed value from node-local slot, returning default if not found
    pub fn get_node_or_default<T: DeserializeOwned + Default>(&self, key: &str) -> T {
        self.get_node(key).unwrap_or_default()
    }

    /// Check if a node-local slot exists
    pub fn has_node(&self, key: &str) -> bool {
        self.node_slots.contains_key(key)
    }

    /// Remove a node-local slot
    pub fn remove_node(&mut self, key: &str) -> Option<SlotValue> {
        self.node_slots.remove(key)
    }

    /// Clear all node-local slots
    pub fn clear_node_slots(&mut self) {
        self.node_slots.clear();
    }

    // ==================== Scope Management ====================

    /// Enter a new execution scope (e.g., for loops or nested conditions)
    ///
    /// Creates a child context that shares global slots but has its own
    /// node-local slots.
    pub fn enter_scope(&self, node_id: impl Into<String>) -> Self {
        let mut new_chain = self.parent_chain.clone();
        new_chain.push(node_id.into());
        Self {
            global_slots: Arc::clone(&self.global_slots),
            node_slots: HashMap::new(),
            parent_chain: new_chain,
            current_node: None,
        }
    }

    /// Set the current node ID
    pub fn set_current_node(&mut self, node_id: impl Into<String>) {
        self.current_node = Some(node_id.into());
    }

    /// Get the current node ID
    pub fn current_node(&self) -> Option<&str> {
        self.current_node.as_deref()
    }

    /// Get the parent chain (for debugging nested scopes)
    pub fn parent_chain(&self) -> &[String] {
        &self.parent_chain
    }

    /// Get the current scope depth
    pub fn scope_depth(&self) -> usize {
        self.parent_chain.len()
    }

    // ==================== Convenience Methods ====================

    /// Store the output of a node execution in a standard location
    pub fn store_node_output(&self, node_id: &str, output: Value) -> Result<()> {
        // Store with _output suffix (legacy/explicit)
        self.set_global(format!("{}_output", node_id), output.clone())?;
        // Store with node_id directly (convenience for templates)
        self.set_global(node_id, output)
    }

    /// Retrieve the output of a previously executed node
    pub fn get_node_output<T: DeserializeOwned>(&self, node_id: &str) -> Result<T> {
        self.get_global(&format!("{}_output", node_id))
    }

    /// Check if a node's output exists
    pub fn has_node_output(&self, node_id: &str) -> bool {
        self.has_global(&format!("{}_output", node_id))
    }

    /// Get the initial input parameters for the workflow
    pub fn get_input_params<T: DeserializeOwned>(&self) -> Result<T> {
        self.get_global("input_params")
    }

    /// Set the initial input parameters for the workflow
    pub fn set_input_params<T: Serialize>(&self, params: T) -> Result<()> {
        self.set_global("input_params", params)
    }

    // ==================== Serialization ====================

    /// Export all global slots as a HashMap for serialization
    pub fn export_global_slots(&self) -> HashMap<String, Value> {
        self.global_slots
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().value().clone()))
            .collect()
    }

    /// Import global slots from a HashMap (for restoring from checkpoint)
    pub fn import_global_slots(&self, slots: HashMap<String, Value>) -> Result<()> {
        for (key, value) in slots {
            self.set_global_raw(key, value)?;
        }
        Ok(())
    }

    /// 获取所有变量（全局和节点本地）
    pub fn all_variables(&self) -> Vec<(String, Value)> {
        let mut result: Vec<(String, Value)> = self.global_slots
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().value().clone()))
            .collect();
        
        for (key, value) in &self.node_slots {
            result.push((key.clone(), value.value().clone()));
        }
        
        result
    }
}

impl Default for DataContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_global_slots() {
        let context = DataContext::new();

        // Set and get string
        context.set_global("name", "test").unwrap();
        let name: String = context.get_global("name").unwrap();
        assert_eq!(name, "test");

        // Set and get number
        context.set_global("count", 42i64).unwrap();
        let count: i64 = context.get_global("count").unwrap();
        assert_eq!(count, 42);

        // Set and get JSON object
        context.set_global("data", json!({"key": "value"})).unwrap();
        let data: Value = context.get_global("data").unwrap();
        assert_eq!(data["key"], "value");
    }

    #[test]
    fn test_node_slots() {
        let mut context = DataContext::new();

        context.set_node("loop_index", 0i64).unwrap();
        let index: i64 = context.get_node("loop_index").unwrap();
        assert_eq!(index, 0);

        // Default value
        let missing: i64 = context.get_node_or_default("missing");
        assert_eq!(missing, 0);
    }

    #[test]
    fn test_scope_management() {
        let context = DataContext::new();
        context.set_global("shared", "value").unwrap();

        // Enter a child scope
        let mut child = context.enter_scope("loop_1");
        child.set_node("local", "child_value").unwrap();

        // Child can access global slots
        let shared: String = child.get_global("shared").unwrap();
        assert_eq!(shared, "value");

        // Parent doesn't have child's node slots
        assert!(!context.has_node("local"));

        // Child scope depth
        assert_eq!(child.scope_depth(), 1);
    }

    #[test]
    fn test_node_output() {
        let context = DataContext::new();

        context
            .store_node_output("step1", json!({"result": "success"}))
            .unwrap();

        assert!(context.has_node_output("step1"));

        let output: Value = context.get_node_output("step1").unwrap();
        assert_eq!(output["result"], "success");
    }
}
