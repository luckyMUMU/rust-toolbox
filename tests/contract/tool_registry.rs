//! 工具注册表契约测试
//!
//! 定义工具注册表必须满足的契约

use serde_json::Value;
use std::sync::Arc;

/// 工具信息
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub version: String,
    pub description: String,
}

/// 工具执行结果
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub success: bool,
    pub output: Value,
    pub error: Option<String>,
}

/// 工具注册表契约 trait
///
/// 所有工具注册表实现必须满足此契约
pub trait ToolRegistryContract: Send + Sync {
    /// 错误类型
    type Error: std::error::Error + Send + Sync + 'static;

    /// 注册工具
    fn register(&self, name: &str, tool: impl Send + Sync + 'static) -> Result<(), Self::Error>;

    /// 注销工具
    fn unregister(&self, name: &str) -> Result<bool, Self::Error>;

    /// 获取工具
    fn get(&self, name: &str) -> Option<Arc<dyn Send + Sync>>;

    /// 检查工具是否存在
    fn contains(&self, name: &str) -> bool;

    /// 列出所有工具
    fn list_tools(&self) -> Vec<ToolInfo>;

    /// 执行工具
    fn execute(
        &self,
        name: &str,
        input: Value,
    ) -> impl std::future::Future<Output = Result<ToolResult, Self::Error>> + Send;
}

/// 工具注册表契约测试
pub struct ToolRegistryContractTests;

impl ToolRegistryContractTests {
    /// 测试：注册工具后应能找到
    pub fn test_register_and_contains<R: ToolRegistryContract>(
        registry: &R,
        tool: impl Send + Sync + 'static,
    ) -> Result<(), String> {
        registry
            .register("test_tool", tool)
            .map_err(|e| format!("注册失败: {}", e))?;

        if !registry.contains("test_tool") {
            return Err("注册后工具应存在".to_string());
        }

        Ok(())
    }

    /// 测试：注销工具后不应能找到
    pub fn test_unregister<R: ToolRegistryContract>(
        registry: &R,
        tool: impl Send + Sync + 'static + Clone,
    ) -> Result<(), String> {
        registry
            .register("test_tool", tool.clone())
            .map_err(|e| format!("注册失败: {}", e))?;

        let removed = registry
            .unregister("test_tool")
            .map_err(|e| format!("注销失败: {}", e))?;

        if !removed {
            return Err("注销应返回 true".to_string());
        }

        if registry.contains("test_tool") {
            return Err("注销后工具不应存在".to_string());
        }

        Ok(())
    }

    /// 测试：获取不存在的工具应返回 None
    pub fn test_get_nonexistent<R: ToolRegistryContract>(registry: &R) -> Result<(), String> {
        if registry.get("nonexistent_tool").is_some() {
            return Err("获取不存在的工具应返回 None".to_string());
        }

        Ok(())
    }

    /// 测试：列出工具应包含已注册的工具
    pub fn test_list_tools<R: ToolRegistryContract>(
        registry: &R,
        tool: impl Send + Sync + 'static + Clone,
    ) -> Result<(), String> {
        registry
            .register("test_tool", tool.clone())
            .map_err(|e| format!("注册失败: {}", e))?;

        let tools = registry.list_tools();

        if !tools.iter().any(|t| t.name == "test_tool") {
            return Err("工具列表应包含已注册的工具".to_string());
        }

        Ok(())
    }

    /// 运行所有契约测试
    pub fn run_all<R: ToolRegistryContract>(
        registry: &R,
        tool: impl Send + Sync + 'static + Clone,
    ) -> Vec<(String, Result<(), String>)> {
        vec![
            (
                "register_and_contains".to_string(),
                Self::test_register_and_contains(registry, tool.clone()),
            ),
            (
                "unregister".to_string(),
                Self::test_unregister(registry, tool.clone()),
            ),
            (
                "get_nonexistent".to_string(),
                Self::test_get_nonexistent(registry),
            ),
            (
                "list_tools".to_string(),
                Self::test_list_tools(registry, tool),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::RwLock;

    struct MockTool;

    struct MockToolRegistry {
        tools: RwLock<HashMap<String, Arc<dyn Send + Sync>>>,
    }

    impl MockToolRegistry {
        fn new() -> Self {
            Self {
                tools: RwLock::new(HashMap::new()),
            }
        }
    }

    impl ToolRegistryContract for MockToolRegistry {
        type Error = std::io::Error;

        fn register(
            &self,
            name: &str,
            tool: impl Send + Sync + 'static,
        ) -> Result<(), Self::Error> {
            self.tools
                .write()
                .unwrap()
                .insert(name.to_string(), Arc::new(tool));
            Ok(())
        }

        fn unregister(&self, name: &str) -> Result<bool, Self::Error> {
            Ok(self.tools.write().unwrap().remove(name).is_some())
        }

        fn get(&self, name: &str) -> Option<Arc<dyn Send + Sync>> {
            self.tools.read().unwrap().get(name).cloned()
        }

        fn contains(&self, name: &str) -> bool {
            self.tools.read().unwrap().contains_key(name)
        }

        fn list_tools(&self) -> Vec<ToolInfo> {
            self.tools
                .read()
                .unwrap()
                .keys()
                .map(|name| ToolInfo {
                    name: name.clone(),
                    version: "1.0.0".to_string(),
                    description: "Mock tool".to_string(),
                })
                .collect()
        }

        async fn execute(&self, _name: &str, _input: Value) -> Result<ToolResult, Self::Error> {
            Ok(ToolResult {
                success: true,
                output: Value::Null,
                error: None,
            })
        }
    }

    #[test]
    fn test_contract_register_and_contains() {
        let registry = MockToolRegistry::new();
        let result = ToolRegistryContractTests::test_register_and_contains(&registry, MockTool);
        assert!(result.is_ok());
    }
}
