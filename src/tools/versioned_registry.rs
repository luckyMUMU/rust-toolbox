//! 版本化工具注册表
//!
//! 提供多版本工具的注册和管理能力

use crate::error::{Result, WorkflowError};
use crate::tools::{
    DependencyResolver, ResolutionResult, Tool, ToolId, ToolMetadata, ToolVersion, Version,
    VersionConflict, VersionRequirement,
};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

/// 版本化工具注册表
///
/// 支持同一工具的多个版本共存，并提供版本选择策略
pub struct VersionedToolRegistry {
    /// 工具存储：工具名 -> 版本 -> 工具实例
    tools: DashMap<String, HashMap<Version, Tool>>,
    /// 默认版本：工具名 -> 默认版本
    default_versions: DashMap<String, Version>,
    /// 工具元数据缓存
    metadata_cache: DashMap<String, HashMap<Version, Arc<ToolMetadata>>>,
    /// 依赖解析器
    resolver: DashMap<String, DependencyResolver>,
    /// 版本选择策略
    version_strategy: VersionSelectionStrategy,
}

/// 版本选择策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionSelectionStrategy {
    /// 总是选择最新版本
    Latest,
    /// 选择默认版本
    Default,
    /// 选择兼容的最高版本
    Compatible,
    /// 必须显式指定版本
    Explicit,
}

impl Default for VersionSelectionStrategy {
    fn default() -> Self {
        Self::Latest
    }
}

impl VersionedToolRegistry {
    /// 创建新的版本化工具注册表
    pub fn new() -> Self {
        Self {
            tools: DashMap::new(),
            default_versions: DashMap::new(),
            metadata_cache: DashMap::new(),
            resolver: DashMap::new(),
            version_strategy: VersionSelectionStrategy::default(),
        }
    }

    /// 使用指定策略创建注册表
    pub fn with_strategy(strategy: VersionSelectionStrategy) -> Self {
        Self {
            tools: DashMap::new(),
            default_versions: DashMap::new(),
            metadata_cache: DashMap::new(),
            resolver: DashMap::new(),
            version_strategy: strategy,
        }
    }

    /// 注册工具版本
    pub fn register(&self, name: &str, version: &Version, tool: Tool) -> Result<ToolId> {
        let id = ToolId::new();
        let metadata = tool.metadata();

        self.tools
            .entry(name.to_string())
            .or_default()
            .insert(version.clone(), tool);

        self.metadata_cache
            .entry(name.to_string())
            .or_default()
            .insert(version.clone(), Arc::new(metadata));

        if !self.default_versions.contains_key(name) {
            self.default_versions
                .insert(name.to_string(), version.clone());
        }

        info!(
            tool_name = %name,
            version = %version,
            tool_id = %id,
            "注册工具版本"
        );

        Ok(id)
    }

    /// 注销工具版本
    pub fn unregister(&self, name: &str, version: &Version) -> Option<Tool> {
        let removed = self
            .tools
            .get_mut(name)
            .and_then(|mut versions| versions.remove(version));

        if removed.is_some() {
            self.metadata_cache
                .get_mut(name)
                .and_then(|mut cache| cache.remove(version));

            if let Some(default) = self.default_versions.get(name) {
                if default.value() == version {
                    self.default_versions.remove(name);

                    if let Some(versions) = self.tools.get(name) {
                        if let Some(latest) = versions.keys().max() {
                            self.default_versions
                                .insert(name.to_string(), latest.clone());
                        }
                    }
                }
            }

            info!(
                tool_name = %name,
                version = %version,
                "注销工具版本"
            );
        }

        removed
    }

    /// 获取工具（使用默认版本策略）
    pub fn get(&self, name: &str) -> Option<Tool> {
        self.get_with_strategy(name, self.version_strategy, None)
    }

    /// 获取指定版本的工具
    pub fn get_version(&self, name: &str, version: &Version) -> Option<Tool> {
        self.tools
            .get(name)
            .and_then(|versions| versions.get(version).cloned())
    }

    /// 使用指定策略获取工具
    pub fn get_with_strategy(
        &self,
        name: &str,
        strategy: VersionSelectionStrategy,
        requirement: Option<&VersionRequirement>,
    ) -> Option<Tool> {
        let versions = self.tools.get(name)?;

        let version: Version = match strategy {
            VersionSelectionStrategy::Latest => versions.keys().max()?.clone(),
            VersionSelectionStrategy::Default => self.default_versions.get(name)?.value().clone(),
            VersionSelectionStrategy::Compatible => {
                if let Some(req) = requirement {
                    versions.keys().filter(|v| v.satisfies(req)).max()?.clone()
                } else {
                    versions.keys().max()?.clone()
                }
            }
            VersionSelectionStrategy::Explicit => {
                return None;
            }
        };

        versions.get(&version).cloned()
    }

    /// 设置默认版本
    pub fn set_default_version(&self, name: &str, version: &Version) -> Result<()> {
        if !self
            .tools
            .get(name)
            .map(|v| v.contains_key(version))
            .unwrap_or(false)
        {
            return Err(WorkflowError::not_found(format!(
                "工具 {} 版本 {} 不存在",
                name, version
            )));
        }

        self.default_versions
            .insert(name.to_string(), version.clone());
        info!(tool_name = %name, version = %version, "设置默认版本");
        Ok(())
    }

    /// 获取默认版本
    pub fn get_default_version(&self, name: &str) -> Option<Version> {
        self.default_versions.get(name).map(|v| v.clone())
    }

    /// 列出工具的所有版本
    pub fn list_versions(&self, name: &str) -> Vec<Version> {
        self.tools
            .get(name)
            .map(|versions| versions.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// 列出所有工具
    pub fn list_tools(&self) -> Vec<String> {
        self.tools.iter().map(|e| e.key().clone()).collect()
    }

    /// 列出所有工具及其版本
    pub fn list_all_versions(&self) -> Vec<(String, Vec<Version>)> {
        self.tools
            .iter()
            .map(|e| (e.key().clone(), e.value().keys().cloned().collect()))
            .collect()
    }

    /// 获取工具元数据
    pub fn get_metadata(&self, name: &str, version: &Version) -> Option<Arc<ToolMetadata>> {
        self.metadata_cache
            .get(name)
            .and_then(|cache| cache.get(version).cloned())
    }

    /// 检查工具是否存在
    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// 检查特定版本是否存在
    pub fn contains_version(&self, name: &str, version: &Version) -> bool {
        self.tools
            .get(name)
            .map(|versions| versions.contains_key(version))
            .unwrap_or(false)
    }

    /// 获取工具数量
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    /// 获取总版本数
    pub fn version_count(&self) -> usize {
        self.tools.iter().map(|e| e.value().len()).sum()
    }

    /// 检查版本冲突
    pub fn check_conflicts(&self) -> Vec<VersionConflict> {
        let mut conflicts = Vec::new();

        for entry in self.tools.iter() {
            let name = entry.key();
            let versions = entry.value();

            let version_list: Vec<_> = versions.keys().collect();

            for i in 0..version_list.len() {
                for j in (i + 1)..version_list.len() {
                    let v1 = version_list[i];
                    let v2 = version_list[j];

                    if v1.major != v2.major {
                        conflicts.push(VersionConflict {
                            tool_name: name.clone(),
                            required_versions: vec![
                                VersionRequirement::Exact(v1.clone()),
                                VersionRequirement::Exact(v2.clone()),
                            ],
                            available_versions: version_list.iter().map(|v| (*v).clone()).collect(),
                            conflict_type: crate::core::version::ConflictType::IncompatibleVersions,
                        });
                    }
                }
            }
        }

        conflicts
    }

    /// 解析依赖
    pub fn resolve_dependencies(
        &self,
        requirements: Vec<crate::core::version::ToolDependency>,
    ) -> Result<ResolutionResult> {
        let mut resolver = DependencyResolver::new();

        for entry in self.tools.iter() {
            let name = entry.key();
            let versions = entry.value();

            for version in versions.keys() {
                let tool_version = ToolVersion::new(name.clone(), version.clone());
                resolver.add_tool_version(tool_version);
            }
        }

        resolver.resolve_dependencies(requirements)
    }

    /// 清空注册表
    pub fn clear(&self) {
        self.tools.clear();
        self.default_versions.clear();
        self.metadata_cache.clear();
        info!("版本化工具注册表已清空");
    }

    /// 获取版本选择策略
    pub fn strategy(&self) -> VersionSelectionStrategy {
        self.version_strategy
    }

    /// 设置版本选择策略
    pub fn set_strategy(&mut self, strategy: VersionSelectionStrategy) {
        self.version_strategy = strategy;
    }
}

impl Default for VersionedToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 版本化工具注册表构建器
pub struct VersionedToolRegistryBuilder {
    registry: VersionedToolRegistry,
}

impl VersionedToolRegistryBuilder {
    /// 创建构建器
    pub fn new() -> Self {
        Self {
            registry: VersionedToolRegistry::new(),
        }
    }

    /// 设置版本选择策略
    pub fn with_strategy(mut self, strategy: VersionSelectionStrategy) -> Self {
        self.registry.version_strategy = strategy;
        self
    }

    /// 注册工具
    pub fn register(self, name: &str, version: &Version, tool: Tool) -> Self {
        let _ = self.registry.register(name, version, tool);
        self
    }

    /// 设置默认版本
    pub fn set_default(self, name: &str, version: &Version) -> Self {
        let _ = self.registry.set_default_version(name, version);
        self
    }

    /// 构建注册表
    pub fn build(self) -> VersionedToolRegistry {
        self.registry
    }
}

impl Default for VersionedToolRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ToolInfo;
    use crate::tools::{NativeTool, NativeToolBuilder, ResourceRequirements, ToolKind};

    fn create_test_tool(name: &str, version: &str) -> Tool {
        let metadata = ToolMetadata {
            info: ToolInfo {
                name: name.to_string(),
                version: version.to_string(),
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
            version: version.to_string(),
        };

        let native_tool = NativeTool::new(
            ToolId::new(),
            Arc::new(metadata),
            |_input, _ctx| async move {
                Ok(crate::tools::ToolOutput::success(
                    serde_json::json!({"status": "ok"}),
                ))
            },
        );

        Tool::Native(Arc::new(native_tool))
    }

    #[test]
    fn test_register_multiple_versions() {
        let registry = VersionedToolRegistry::new();

        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(2, 0, 0);

        registry
            .register("test_tool", &v1, create_test_tool("test_tool", "1.0.0"))
            .unwrap();
        registry
            .register("test_tool", &v2, create_test_tool("test_tool", "2.0.0"))
            .unwrap();

        let versions = registry.list_versions("test_tool");
        assert_eq!(versions.len(), 2);
    }

    #[test]
    fn test_get_latest_version() {
        let registry = VersionedToolRegistry::new();

        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(2, 0, 0);

        registry
            .register("test_tool", &v1, create_test_tool("test_tool", "1.0.0"))
            .unwrap();
        registry
            .register("test_tool", &v2, create_test_tool("test_tool", "2.0.0"))
            .unwrap();

        let tool = registry.get("test_tool");
        assert!(tool.is_some());
    }

    #[test]
    fn test_set_default_version() {
        let registry = VersionedToolRegistry::new();

        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(2, 0, 0);

        registry
            .register("test_tool", &v1, create_test_tool("test_tool", "1.0.0"))
            .unwrap();
        registry
            .register("test_tool", &v2, create_test_tool("test_tool", "2.0.0"))
            .unwrap();

        registry.set_default_version("test_tool", &v1).unwrap();

        let default = registry.get_default_version("test_tool");
        assert_eq!(default, Some(v1));
    }

    #[test]
    fn test_version_selection_strategy() {
        let registry = VersionedToolRegistry::with_strategy(VersionSelectionStrategy::Default);
        assert_eq!(registry.strategy(), VersionSelectionStrategy::Default);
    }

    #[test]
    fn test_unregister_version() {
        let registry = VersionedToolRegistry::new();

        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(2, 0, 0);

        registry
            .register("test_tool", &v1, create_test_tool("test_tool", "1.0.0"))
            .unwrap();
        registry
            .register("test_tool", &v2, create_test_tool("test_tool", "2.0.0"))
            .unwrap();

        let removed = registry.unregister("test_tool", &v1);
        assert!(removed.is_some());

        let versions = registry.list_versions("test_tool");
        assert_eq!(versions.len(), 1);
    }

    #[test]
    fn test_builder() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(2, 0, 0);

        let registry = VersionedToolRegistryBuilder::new()
            .with_strategy(VersionSelectionStrategy::Latest)
            .register("tool1", &v1, create_test_tool("tool1", "1.0.0"))
            .register("tool1", &v2, create_test_tool("tool1", "2.0.0"))
            .build();

        assert_eq!(registry.tool_count(), 1);
        assert_eq!(registry.list_versions("tool1").len(), 2);
    }
}
