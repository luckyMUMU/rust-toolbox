//! 统一工具执行器
//!
//! 提供工具执行的统一入口，集成中间件栈和外部执行器

use crate::core::ExecutionContext;
use crate::error::{Result, WorkflowError};
use crate::tools::{
    ExecutionResult, ExecutorFactory, ExecutorType, ExternalExecutor,
    Middleware, MiddlewareStack, MiddlewareStackBuilder,
    Tool, ToolInput, ToolOutput, ToolRegistry,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// 工具执行器配置
#[derive(Debug, Clone)]
pub struct ToolExecutorConfig {
    /// 默认超时时间
    pub default_timeout: Duration,
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试延迟
    pub retry_delay: Duration,
    /// 是否启用缓存
    pub enable_cache: bool,
    /// 缓存 TTL
    pub cache_ttl: Duration,
    /// 是否启用熔断器
    pub enable_circuit_breaker: bool,
    /// 熔断器阈值
    pub circuit_breaker_threshold: u32,
    /// 是否启用指标收集
    pub enable_metrics: bool,
}

impl Default for ToolExecutorConfig {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(300),
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            enable_cache: true,
            cache_ttl: Duration::from_secs(3600),
            enable_circuit_breaker: true,
            circuit_breaker_threshold: 5,
            enable_metrics: true,
        }
    }
}

/// 统一工具执行器
pub struct ToolExecutor {
    /// 工具注册表
    registry: Arc<ToolRegistry>,
    /// 中间件栈
    middleware_stack: MiddlewareStack,
    /// 外部执行器工厂
    executor_factory: ExecutorFactory,
    /// 外部执行器缓存
    external_executors: HashMap<ExecutorType, Box<dyn ExternalExecutor>>,
    /// 配置
    config: ToolExecutorConfig,
}

impl ToolExecutor {
    /// 创建新的工具执行器
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self {
            registry,
            middleware_stack: MiddlewareStack::new(),
            executor_factory: ExecutorFactory::default(),
            external_executors: HashMap::new(),
            config: ToolExecutorConfig::default(),
        }
    }

    /// 使用配置创建执行器
    pub fn with_config(registry: Arc<ToolRegistry>, config: ToolExecutorConfig) -> Self {
        Self {
            registry,
            middleware_stack: MiddlewareStack::new(),
            executor_factory: ExecutorFactory::default(),
            external_executors: HashMap::new(),
            config,
        }
    }

    /// 添加中间件
    pub fn with_middleware<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.middleware_stack = MiddlewareStackBuilder::new()
            .with(middleware)
            .build();
        self
    }

    /// 构建中间件栈
    pub fn build_middleware_stack(mut self) -> Self {
        let mut builder = MiddlewareStackBuilder::new();
        
        if self.config.enable_metrics {
            builder = builder.with(crate::tools::MetricsMiddleware::new());
        }
        
        if self.config.enable_circuit_breaker {
            builder = builder.with(crate::tools::CircuitBreakerMiddleware::new(
                self.config.circuit_breaker_threshold,
                Duration::from_secs(60),
            ));
        }
        
        builder = builder
            .with(crate::tools::TimeoutMiddleware::new(self.config.default_timeout))
            .with(crate::tools::RetryMiddleware::new(
                self.config.max_retries,
                self.config.retry_delay,
            ));
        
        if self.config.enable_cache {
            builder = builder.with(crate::tools::CacheMiddleware::new(
                self.config.cache_ttl,
            ));
        }
        
        builder = builder.with(crate::tools::LoggingMiddleware::new());
        
        self.middleware_stack = builder.build();
        self
    }

    /// 执行工具
    pub async fn execute(
        &self,
        tool_name: &str,
        input: ToolInput,
        ctx: ExecutionContext,
    ) -> Result<ToolOutput> {
        let tool = self.registry
            .get(tool_name)
            .ok_or_else(|| WorkflowError::tool_not_found(tool_name))?;
        
        self.execute_tool(tool, input, ctx).await
    }

    /// 执行工具实例
    async fn execute_tool(
        &self,
        tool: Tool,
        input: ToolInput,
        ctx: ExecutionContext,
    ) -> Result<ToolOutput> {
        let context = crate::tools::MiddlewareContext {
            tool_name: tool.name().to_string(),
            tool_kind: tool.kind(),
            input: input.clone(),
            execution_context: ctx.clone(),
        };
        
        let tool_clone = tool.clone();
        let input_clone = input.clone();
        let ctx_clone = ctx.clone();
        
        self.middleware_stack.execute(context, move || {
            let tool = tool_clone;
            let input = input_clone;
            let ctx = ctx_clone;
            
            async move {
                tool.execute(input, ctx).await
            }
        }).await
    }

    /// 执行外部工具
    pub async fn execute_external(
        &self,
        executor_type: ExecutorType,
        command: &str,
        args: &[String],
        input: Option<serde_json::Value>,
        ctx: &ExecutionContext,
    ) -> Result<ExecutionResult> {
        let executor = self.get_executor(executor_type)?;
        executor.execute(command, args, input, ctx).await
    }

    /// 获取外部执行器
    fn get_executor(&self, executor_type: ExecutorType) -> Result<&dyn ExternalExecutor> {
        Ok(match executor_type {
            ExecutorType::Python => {
                self.external_executors
                    .entry(ExecutorType::Python)
                    .or_insert_with(|| {
                        Box::new(crate::tools::PythonExecutor::new(
                            crate::tools::ExternalExecutorConfig::default(),
                        ))
                    })
                    .as_ref()
            }
            ExecutorType::NodeJs => {
                self.external_executors
                    .entry(ExecutorType::NodeJs)
                    .or_insert_with(|| {
                        Box::new(crate::tools::NodeJsExecutor::new(
                            crate::tools::ExternalExecutorConfig::default(),
                        ))
                    })
                    .as_ref()
            }
            ExecutorType::Docker => {
                self.external_executors
                    .entry(ExecutorType::Docker)
                    .or_insert_with(|| {
                        Box::new(crate::tools::DockerExecutor::new(
                            crate::tools::ExternalExecutorConfig::default(),
                        ))
                    })
                    .as_ref()
            }
        })
    }

    /// 批量执行工具
    pub async fn execute_batch(
        &self,
        tasks: Vec<(&str, ToolInput, ExecutionContext)>,
    ) -> Vec<Result<ToolOutput>> {
        let futures: Vec<_> = tasks
            .into_iter()
            .map(|(name, input, ctx)| self.execute(name, input, ctx))
            .collect();
        
        futures::future::join_all(futures).await
    }

    /// 并行执行工具
    pub async fn execute_parallel(
        &self,
        tasks: Vec<(&str, ToolInput, ExecutionContext)>,
    ) -> Vec<Result<ToolOutput>> {
        let futures: Vec<_> = tasks
            .into_iter()
            .map(|(name, input, ctx)| {
                let registry = self.registry.clone();
                let middleware = self.middleware_stack.clone();
                
                async move {
                    let tool = registry
                        .get(name)
                        .ok_or_else(|| WorkflowError::tool_not_found(name))?;
                    
                    let context = crate::tools::MiddlewareContext {
                        tool_name: tool.name().to_string(),
                        tool_kind: tool.kind(),
                        input: input.clone(),
                        execution_context: ctx.clone(),
                    };
                    
                    let tool_clone = tool.clone();
                    let input_clone = input.clone();
                    let ctx_clone = ctx.clone();
                    
                    middleware.execute(context, move || {
                        let tool = tool_clone;
                        let input = input_clone;
                        let ctx = ctx_clone;
                        
                        async move {
                            tool.execute(input, ctx).await
                        }
                    }).await
                }
            })
            .collect();
        
        futures::future::join_all(futures).await
    }

    /// 检查工具是否可用
    pub fn is_tool_available(&self, name: &str) -> bool {
        self.registry.contains(name)
    }

    /// 获取工具列表
    pub fn list_tools(&self) -> Vec<String> {
        self.registry.list_names()
    }

    /// 获取配置
    pub fn config(&self) -> &ToolExecutorConfig {
        &self.config
    }
}

/// 工具执行器构建器
pub struct ToolExecutorBuilder {
    registry: Arc<ToolRegistry>,
    config: ToolExecutorConfig,
    middlewares: Vec<Box<dyn Middleware>>,
}

impl ToolExecutorBuilder {
    /// 创建构建器
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self {
            registry,
            config: ToolExecutorConfig::default(),
            middlewares: Vec::new(),
        }
    }

    /// 设置配置
    pub fn with_config(mut self, config: ToolExecutorConfig) -> Self {
        self.config = config;
        self
    }

    /// 添加中间件
    pub fn with_middleware<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.middlewares.push(Box::new(middleware));
        self
    }

    /// 构建执行器
    pub fn build(self) -> ToolExecutor {
        let mut builder = MiddlewareStackBuilder::new();
        
        for middleware in self.middlewares {
            builder = builder.with_boxed(middleware);
        }
        
        ToolExecutor {
            registry: self.registry,
            middleware_stack: builder.build(),
            executor_factory: ExecutorFactory::default(),
            external_executors: HashMap::new(),
            config: self.config,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_config_default() {
        let config = ToolExecutorConfig::default();
        assert_eq!(config.max_retries, 3);
        assert!(config.enable_cache);
        assert!(config.enable_circuit_breaker);
    }

    #[test]
    fn test_executor_creation() {
        let registry = Arc::new(ToolRegistry::new());
        let executor = ToolExecutor::new(registry);
        
        assert!(executor.list_tools().is_empty());
    }

    #[test]
    fn test_executor_builder() {
        let registry = Arc::new(ToolRegistry::new());
        let executor = ToolExecutorBuilder::new(registry)
            .with_config(ToolExecutorConfig::default())
            .build();
        
        assert!(executor.list_tools().is_empty());
    }
}
