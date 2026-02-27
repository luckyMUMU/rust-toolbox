//! Tool registry for managing available tools (enum-based implementation)
//!
//! This module provides a high-performance, thread-safe tool registry using
//! the new enum-based Tool system.

use crate::core::ExecutionContext;
use crate::error::{Result, WorkflowError};
use crate::tools::types::{Tool, ToolId, ToolInput, ToolKind, ToolMetadata, ToolOutput};
use crate::tools::version::Version;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

/// High-performance tool registry using enum-based Tool system
///
/// Features:
/// - O(1) tool lookup by name
/// - O(1) tool lookup by ID
/// - Thread-safe concurrent access
/// - Version management
/// - Tool discovery and search
pub struct ToolRegistry {
    /// Primary storage: ToolId -> Tool
    tools: DashMap<ToolId, Tool>,
    /// Name index: tool name -> ToolId
    name_index: DashMap<String, ToolId>,
    /// Metadata cache: ToolId -> metadata
    metadata_cache: DashMap<ToolId, Arc<ToolMetadata>>,
    /// Version management: ToolId -> versions
    versions: DashMap<ToolId, Vec<Version>>,
    /// Category index: category -> [ToolId]
    category_index: DashMap<String, Vec<ToolId>>,
    /// Tag index: tag -> [ToolId]
    tag_index: DashMap<String, Vec<ToolId>>,
}

impl ToolRegistry {
    /// Create a new empty tool registry
    pub fn new() -> Self {
        Self {
            tools: DashMap::new(),
            name_index: DashMap::new(),
            metadata_cache: DashMap::new(),
            versions: DashMap::new(),
            category_index: DashMap::new(),
            tag_index: DashMap::new(),
        }
    }

    /// Create a new registry with initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            tools: DashMap::with_capacity(capacity),
            name_index: DashMap::with_capacity(capacity),
            metadata_cache: DashMap::with_capacity(capacity),
            versions: DashMap::with_capacity(capacity),
            category_index: DashMap::new(),
            tag_index: DashMap::new(),
        }
    }

    /// Register a new tool
    ///
    /// Returns the ToolId assigned to the tool
    pub fn register(&self, name: &str, tool: Tool) -> ToolId {
        let id = ToolId::new();
        let metadata = tool.metadata();

        // Store tool
        self.tools.insert(id, tool);

        // Update name index
        self.name_index.insert(name.to_string(), id);

        // Update metadata cache
        self.metadata_cache.insert(id, Arc::new(metadata.clone()));

        // Update version tracking
        self.versions
            .entry(id)
            .or_insert_with(Vec::new)
            .push(metadata.version.parse().unwrap_or(Version::new(0, 1, 0)));

        // Update category index
        if let Some(category) = &metadata.info.category {
            self.category_index
                .entry(category.clone())
                .or_insert_with(Vec::new)
                .push(id);
        }

        // Update tag index
        for tag in &metadata.info.tags {
            self.tag_index
                .entry(tag.clone())
                .or_insert_with(Vec::new)
                .push(id);
        }

        info!(tool_id = %id, tool_name = %name, "Tool registered");
        id
    }

    /// Get a tool by name
    ///
    /// O(1) lookup via name index
    pub fn get(&self, name: &str) -> Option<Tool> {
        let id = self.name_index.get(name)?;
        self.tools.get(&*id).map(|r| r.clone())
    }

    /// Get a tool by ID
    ///
    /// O(1) lookup
    pub fn get_by_id(&self, id: ToolId) -> Option<Tool> {
        self.tools.get(&id).map(|r| r.clone())
    }

    /// Get tool metadata by name
    pub fn get_metadata(&self, name: &str) -> Option<Arc<ToolMetadata>> {
        let id = self.name_index.get(name)?;
        self.metadata_cache.get(&*id).map(|r| r.clone())
    }

    /// Get tool metadata by ID
    pub fn get_metadata_by_id(&self, id: ToolId) -> Option<Arc<ToolMetadata>> {
        self.metadata_cache.get(&id).map(|r| r.clone())
    }

    /// Check if a tool exists
    pub fn contains(&self, name: &str) -> bool {
        self.name_index.contains_key(name)
    }

    /// Remove a tool by name
    pub fn remove(&self, name: &str) -> Option<Tool> {
        let id = self.name_index.remove(name)?.1;

        // Remove from all indexes
        self.metadata_cache.remove(&id);
        self.versions.remove(&id);

        // Remove from category index
        if let Some(metadata) = self.metadata_cache.get(&id) {
            if let Some(category) = &metadata.info.category {
                if let Some(mut ids) = self.category_index.get_mut(category) {
                    ids.retain(|&x| x != id);
                }
            }
        }

        // Remove from tag index
        if let Some(metadata) = self.metadata_cache.get(&id) {
            for tag in &metadata.info.tags {
                if let Some(mut ids) = self.tag_index.get_mut(tag) {
                    ids.retain(|&x| x != id);
                }
            }
        }

        info!(tool_id = %id, tool_name = %name, "Tool removed");
        self.tools.remove(&id).map(|r| r.1)
    }

    /// List all tool names
    pub fn list_names(&self) -> Vec<String> {
        self.name_index.iter().map(|e| e.key().clone()).collect()
    }

    /// List all tool IDs
    pub fn list_ids(&self) -> Vec<ToolId> {
        self.tools.iter().map(|e| *e.key()).collect()
    }

    /// List all tools with their metadata
    ///
    /// Returns a vector of ToolInfo for all registered tools
    pub fn list_tools(&self) -> Vec<crate::core::ToolInfo> {
        self.metadata_cache
            .iter()
            .map(|e| e.value().info.clone())
            .collect()
    }

    /// Get tool count
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Clear all tools
    pub fn clear(&self) {
        self.tools.clear();
        self.name_index.clear();
        self.metadata_cache.clear();
        self.versions.clear();
        self.category_index.clear();
        self.tag_index.clear();
        info!("Tool registry cleared");
    }

    /// Find tools by name pattern
    pub fn find_by_pattern(&self, pattern: &str) -> Vec<(String, ToolId)> {
        self.name_index
            .iter()
            .filter(|e| e.key().contains(pattern))
            .map(|e| (e.key().clone(), *e.value()))
            .collect()
    }

    /// Get tools by category
    pub fn get_by_category(&self, category: &str) -> Vec<Tool> {
        self.category_index
            .get(category)
            .map(|ids| ids.iter().filter_map(|&id| self.get_by_id(id)).collect())
            .unwrap_or_default()
    }

    /// Get tools by tag
    pub fn get_by_tag(&self, tag: &str) -> Vec<Tool> {
        self.tag_index
            .get(tag)
            .map(|ids| ids.iter().filter_map(|&id| self.get_by_id(id)).collect())
            .unwrap_or_default()
    }

    /// Get tools by kind
    pub fn get_by_kind(&self, kind: ToolKind) -> Vec<Tool> {
        self.tools
            .iter()
            .filter(|e| e.value().kind() == kind)
            .map(|e| e.value().clone())
            .collect()
    }

    /// Execute a tool by name
    ///
    /// This is a convenience method that looks up the tool and executes it
    pub async fn execute(
        &self,
        name: &str,
        input: ToolInput,
        ctx: ExecutionContext,
    ) -> Result<ToolOutput> {
        let tool = self
            .get(name)
            .ok_or_else(|| WorkflowError::tool_not_found(name))?;

        tool.execute(input, ctx).await
    }

    /// Execute a tool by ID
    pub async fn execute_by_id(
        &self,
        id: ToolId,
        input: ToolInput,
        ctx: ExecutionContext,
    ) -> Result<ToolOutput> {
        let tool = self
            .get_by_id(id)
            .ok_or_else(|| WorkflowError::tool_not_found(&format!("{}", id)))?;

        tool.execute(input, ctx).await
    }

    /// Get a tool by name (alias for get)
    ///
    /// For backward compatibility with old API
    pub fn get_tool(&self, name: &str) -> Option<Tool> {
        self.get(name)
    }

    /// Execute a tool by name (alias for execute)
    ///
    /// For backward compatibility with old API
    pub async fn execute_tool(
        &self,
        name: &str,
        input: ToolInput,
        ctx: ExecutionContext,
    ) -> Result<ToolOutput> {
        self.execute(name, input, ctx).await
    }

    /// Validate tool parameters
    ///
    /// For backward compatibility with old API
    pub fn validate_tool_params(&self, name: &str, params: &serde_json::Value) -> Result<()> {
        // Get tool metadata and validate parameters against schema
        let metadata = self
            .get_metadata(name)
            .ok_or_else(|| WorkflowError::tool_not_found(name))?;

        // Basic validation - check if required parameters are present
        if let Some(schema) = &metadata.input_schema {
            if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
                for req in required {
                    if let Some(req_str) = req.as_str() {
                        if params.get(req_str).is_none() {
                            return Err(WorkflowError::validation(format!(
                                "Missing required parameter: {}",
                                req_str
                            )));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get tool count
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    /// Check if tool exists
    pub fn has_tool(&self, name: &str) -> bool {
        self.name_index.contains_key(name)
    }

    /// Register a tool (alias for register)
    pub fn register_tool(&mut self, name: &str, tool: Tool) -> ToolId {
        self.register(name, tool)
    }

    /// Unregister a tool (alias for remove)
    pub fn unregister_tool(&mut self, name: &str) -> Option<Tool> {
        self.remove(name)
    }

    /// Get tool versions
    pub fn get_versions(&self, name: &str) -> Option<Vec<Version>> {
        let id = self.name_index.get(name)?;
        self.versions.get(&*id).map(|v| v.clone())
    }

    /// Check for version conflicts
    pub fn check_version_conflicts(&self) -> Vec<String> {
        let mut conflicts = Vec::new();

        // Group tools by name base (without version)
        let mut name_groups: HashMap<String, Vec<(ToolId, Version)>> = HashMap::new();

        for entry in self.versions.iter() {
            let id = *entry.key();
            if let Some(metadata) = self.metadata_cache.get(&id) {
                let name = &metadata.info.name;
                for version in entry.value().iter() {
                    name_groups
                        .entry(name.clone())
                        .or_insert_with(Vec::new)
                        .push((id, version.clone()));
                }
            }
        }

        // Check for conflicts within groups
        for (name, versions) in name_groups {
            if versions.len() > 1 {
                // Check if versions are compatible
                for i in 0..versions.len() {
                    for j in (i + 1)..versions.len() {
                        let v1 = &versions[i].1;
                        let v2 = &versions[j].1;

                        // Major version must match for compatibility
                        if v1.major != v2.major {
                            conflicts
                                .push(format!("{}: incompatible versions {} and {}", name, v1, v2));
                        }
                    }
                }
            }
        }

        conflicts
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for ToolRegistry
pub struct ToolRegistryBuilder {
    registry: ToolRegistry,
}

impl ToolRegistryBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            registry: ToolRegistry::new(),
        }
    }

    /// Add a tool to the registry
    pub fn register(self, name: &str, tool: Tool) -> Self {
        self.registry.register(name, tool);
        self
    }

    /// Build the registry
    pub fn build(self) -> ToolRegistry {
        self.registry
    }
}

impl Default for ToolRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ToolInfo;
    use crate::tools::types::{NativeTool, ResourceRequirements};

    fn create_test_tool(name: &str) -> Tool {
        use std::sync::Arc;
        let metadata = ToolMetadata {
            info: ToolInfo {
                name: name.to_string(),
                version: "1.0.0".to_string(),
                description: "Test tool".to_string(),
                parameters_schema: serde_json::Value::Null,
                return_schema: serde_json::Value::Null,
                category: Some("test".to_string()),
                tags: vec!["test".to_string()],
                dependencies: vec![],
                plugin_name: None,
                version_requirements: Default::default(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            kind: ToolKind::Native,
            input_schema: None,
            output_schema: None,
            examples: vec![],
            resource_requirements: ResourceRequirements::default(),
            version: "1.0.0".to_string(),
        };

        let native_tool = NativeTool::new(
            ToolId::new(),
            Arc::new(metadata),
            |_input, _ctx| async move { Ok(ToolOutput::success(serde_json::json!({"status": "ok"}))) },
        );

        Tool::Native(Arc::new(native_tool))
    }

    #[test]
    fn test_register_and_get() {
        let registry = ToolRegistry::new();
        let tool = create_test_tool("test_tool");

        let id = registry.register("test_tool", tool.clone());

        assert!(registry.contains("test_tool"));
        assert_eq!(registry.get("test_tool").unwrap().kind(), ToolKind::Native);
        assert_eq!(registry.get_by_id(id).unwrap().kind(), ToolKind::Native);
    }

    #[test]
    fn test_remove() {
        let registry = ToolRegistry::new();
        let tool = create_test_tool("test_tool");

        registry.register("test_tool", tool);
        assert!(registry.contains("test_tool"));

        registry.remove("test_tool");
        assert!(!registry.contains("test_tool"));
    }

    #[test]
    fn test_find_by_pattern() {
        let registry = ToolRegistry::new();

        registry.register("file_copy", create_test_tool("file_copy"));
        registry.register("file_move", create_test_tool("file_move"));
        registry.register("http_get", create_test_tool("http_get"));

        let file_tools = registry.find_by_pattern("file");
        assert_eq!(file_tools.len(), 2);

        let http_tools = registry.find_by_pattern("http");
        assert_eq!(http_tools.len(), 1);
    }

    #[test]
    fn test_get_by_category() {
        let registry = ToolRegistry::new();

        registry.register("tool1", create_test_tool("tool1"));
        registry.register("tool2", create_test_tool("tool2"));

        let tools = registry.get_by_category("test");
        assert_eq!(tools.len(), 2);
    }

    #[test]
    fn test_registry_builder() {
        let registry = ToolRegistryBuilder::new()
            .register("tool1", create_test_tool("tool1"))
            .register("tool2", create_test_tool("tool2"))
            .build();

        assert_eq!(registry.len(), 2);
    }
}
