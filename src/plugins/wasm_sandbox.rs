//! WASM 沙箱隔离实现
//!
//! 提供安全的 WASM 执行环境，限制文件系统、网络和系统调用访问

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// 沙箱安全级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxLevel {
    /// 无限制 - 仅用于信任的本地模块
    Unrestricted,
    /// 基础隔离 - 限制文件系统和网络
    Basic,
    /// 严格隔离 - 禁止所有外部访问
    Strict,
    /// 最大隔离 - 额外限制 CPU 和内存
    Maximum,
}

impl Default for SandboxLevel {
    fn default() -> Self {
        Self::Strict
    }
}

/// 文件系统权限配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemPermissions {
    /// 允许读取的目录
    pub read_paths: Vec<PathBuf>,
    /// 允许写入的目录
    pub write_paths: Vec<PathBuf>,
    /// 允许执行的目录
    pub execute_paths: Vec<PathBuf>,
    /// 是否允许创建临时文件
    pub allow_temp: bool,
    /// 是否允许访问当前工作目录
    pub allow_cwd: bool,
}

impl Default for FileSystemPermissions {
    fn default() -> Self {
        Self {
            read_paths: Vec::new(),
            write_paths: Vec::new(),
            execute_paths: Vec::new(),
            allow_temp: true,
            allow_cwd: false,
        }
    }
}

impl FileSystemPermissions {
    /// 创建只读权限
    pub fn read_only(paths: Vec<PathBuf>) -> Self {
        Self {
            read_paths: paths,
            write_paths: Vec::new(),
            execute_paths: Vec::new(),
            allow_temp: false,
            allow_cwd: false,
        }
    }

    /// 创建完全禁止权限
    pub fn denied() -> Self {
        Self {
            read_paths: Vec::new(),
            write_paths: Vec::new(),
            execute_paths: Vec::new(),
            allow_temp: false,
            allow_cwd: false,
        }
    }
}

/// 网络权限配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPermissions {
    /// 是否允许网络访问
    pub enabled: bool,
    /// 允许连接的主机列表
    pub allowed_hosts: HashSet<String>,
    /// 允许连接的端口列表
    pub allowed_ports: HashSet<u16>,
    /// 是否允许 DNS 解析
    pub allow_dns: bool,
    /// 是否允许 HTTP 请求
    pub allow_http: bool,
    /// 是否允许 HTTPS 请求
    pub allow_https: bool,
}

impl Default for NetworkPermissions {
    fn default() -> Self {
        Self {
            enabled: false,
            allowed_hosts: HashSet::new(),
            allowed_ports: HashSet::new(),
            allow_dns: false,
            allow_http: false,
            allow_https: false,
        }
    }
}

impl NetworkPermissions {
    /// 创建允许特定主机的权限
    pub fn allowed_hosts(hosts: Vec<String>) -> Self {
        Self {
            enabled: true,
            allowed_hosts: hosts.into_iter().collect(),
            allowed_ports: [80, 443].into_iter().collect(),
            allow_dns: true,
            allow_http: true,
            allow_https: true,
        }
    }

    /// 创建完全禁止权限
    pub fn denied() -> Self {
        Self::default()
    }
}

/// 系统调用限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyscallPermissions {
    /// 是否允许进程创建
    pub allow_fork: bool,
    /// 是否允许执行外部程序
    pub allow_exec: bool,
    /// 是否允许信号处理
    pub allow_signals: bool,
    /// 是否允许共享内存
    pub allow_shared_memory: bool,
    /// 禁止的系统调用列表
    pub blocked_syscalls: HashSet<String>,
}

impl Default for SyscallPermissions {
    fn default() -> Self {
        Self {
            allow_fork: false,
            allow_exec: false,
            allow_signals: false,
            allow_shared_memory: false,
            blocked_syscalls: HashSet::new(),
        }
    }
}

/// WASM 沙箱配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmSandboxConfig {
    /// 沙箱安全级别
    pub level: SandboxLevel,
    /// 文件系统权限
    pub filesystem: FileSystemPermissions,
    /// 网络权限
    pub network: NetworkPermissions,
    /// 系统调用权限
    pub syscalls: SyscallPermissions,
    /// 环境变量白名单
    pub allowed_env_vars: HashSet<String>,
    /// 环境变量注入
    pub injected_env_vars: HashMap<String, String>,
    /// 是否启用审计日志
    pub enable_audit_log: bool,
    /// 是否启用系统调用过滤
    pub enable_seccomp: bool,
}

impl Default for WasmSandboxConfig {
    fn default() -> Self {
        Self {
            level: SandboxLevel::Strict,
            filesystem: FileSystemPermissions::denied(),
            network: NetworkPermissions::denied(),
            syscalls: SyscallPermissions::default(),
            allowed_env_vars: HashSet::new(),
            injected_env_vars: HashMap::new(),
            enable_audit_log: true,
            enable_seccomp: false,
        }
    }
}

impl WasmSandboxConfig {
    /// 创建无限制配置（仅用于信任模块）
    pub fn unrestricted() -> Self {
        Self {
            level: SandboxLevel::Unrestricted,
            filesystem: FileSystemPermissions {
                read_paths: vec![PathBuf::from("/")],
                write_paths: vec![PathBuf::from("/")],
                execute_paths: vec![PathBuf::from("/")],
                allow_temp: true,
                allow_cwd: true,
            },
            network: NetworkPermissions {
                enabled: true,
                allowed_hosts: HashSet::new(),
                allowed_ports: HashSet::new(),
                allow_dns: true,
                allow_http: true,
                allow_https: true,
            },
            syscalls: SyscallPermissions {
                allow_fork: true,
                allow_exec: true,
                allow_signals: true,
                allow_shared_memory: true,
                blocked_syscalls: HashSet::new(),
            },
            allowed_env_vars: HashSet::new(),
            injected_env_vars: HashMap::new(),
            enable_audit_log: false,
            enable_seccomp: false,
        }
    }

    /// 创建基础隔离配置
    pub fn basic() -> Self {
        Self {
            level: SandboxLevel::Basic,
            filesystem: FileSystemPermissions {
                read_paths: vec![PathBuf::from("/tmp")],
                write_paths: vec![PathBuf::from("/tmp")],
                execute_paths: Vec::new(),
                allow_temp: true,
                allow_cwd: false,
            },
            network: NetworkPermissions::denied(),
            syscalls: SyscallPermissions::default(),
            allowed_env_vars: ["PATH", "HOME"].into_iter().map(String::from).collect(),
            injected_env_vars: HashMap::new(),
            enable_audit_log: true,
            enable_seccomp: false,
        }
    }

    /// 创建严格隔离配置
    pub fn strict() -> Self {
        Self {
            level: SandboxLevel::Strict,
            filesystem: FileSystemPermissions::denied(),
            network: NetworkPermissions::denied(),
            syscalls: SyscallPermissions::default(),
            allowed_env_vars: HashSet::new(),
            injected_env_vars: HashMap::new(),
            enable_audit_log: true,
            enable_seccomp: true,
        }
    }

    /// 创建最大隔离配置
    pub fn maximum() -> Self {
        Self {
            level: SandboxLevel::Maximum,
            filesystem: FileSystemPermissions::denied(),
            network: NetworkPermissions::denied(),
            syscalls: SyscallPermissions::default(),
            allowed_env_vars: HashSet::new(),
            injected_env_vars: HashMap::new(),
            enable_audit_log: true,
            enable_seccomp: true,
        }
    }

    /// 根据安全级别创建配置
    pub fn from_level(level: SandboxLevel) -> Self {
        match level {
            SandboxLevel::Unrestricted => Self::unrestricted(),
            SandboxLevel::Basic => Self::basic(),
            SandboxLevel::Strict => Self::strict(),
            SandboxLevel::Maximum => Self::maximum(),
        }
    }
}

/// WASM 沙箱实例
pub struct WasmSandbox {
    config: WasmSandboxConfig,
    audit_log: Vec<AuditEntry>,
}

/// 审计日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: AuditEventType,
    pub resource: String,
    pub action: String,
    pub allowed: bool,
    pub reason: Option<String>,
}

/// 审计事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    FileRead,
    FileWrite,
    FileExecute,
    NetworkConnect,
    NetworkDns,
    Syscall,
    EnvAccess,
}

impl WasmSandbox {
    /// 创建新的沙箱实例
    pub fn new(config: WasmSandboxConfig) -> Self {
        info!("创建 WASM 沙箱，安全级别: {:?}", config.level);
        Self {
            config,
            audit_log: Vec::new(),
        }
    }

    /// 获取沙箱配置
    pub fn config(&self) -> &WasmSandboxConfig {
        &self.config
    }

    /// 检查文件读取权限
    pub fn check_file_read(&mut self, path: &PathBuf) -> bool {
        let allowed = self
            .config
            .filesystem
            .read_paths
            .iter()
            .any(|p| path.starts_with(p));

        if self.config.enable_audit_log {
            self.log_audit(
                AuditEventType::FileRead,
                path.to_string_lossy().to_string(),
                "read".to_string(),
                allowed,
                None,
            );
        }

        if !allowed {
            warn!("沙箱阻止文件读取: {:?}", path);
        }

        allowed
    }

    /// 检查文件写入权限
    pub fn check_file_write(&mut self, path: &PathBuf) -> bool {
        let allowed = self
            .config
            .filesystem
            .write_paths
            .iter()
            .any(|p| path.starts_with(p));

        if self.config.enable_audit_log {
            self.log_audit(
                AuditEventType::FileWrite,
                path.to_string_lossy().to_string(),
                "write".to_string(),
                allowed,
                None,
            );
        }

        if !allowed {
            warn!("沙箱阻止文件写入: {:?}", path);
        }

        allowed
    }

    /// 检查网络连接权限
    pub fn check_network_connect(&mut self, host: &str, port: u16) -> bool {
        if !self.config.network.enabled {
            self.log_audit(
                AuditEventType::NetworkConnect,
                format!("{}:{}", host, port),
                "connect".to_string(),
                false,
                Some("网络访问已禁用".to_string()),
            );
            return false;
        }

        let host_allowed = self.config.network.allowed_hosts.is_empty()
            || self.config.network.allowed_hosts.contains(host);
        let port_allowed = self.config.network.allowed_ports.is_empty()
            || self.config.network.allowed_ports.contains(&port);

        let allowed = host_allowed && port_allowed;

        if self.config.enable_audit_log {
            self.log_audit(
                AuditEventType::NetworkConnect,
                format!("{}:{}", host, port),
                "connect".to_string(),
                allowed,
                if !allowed {
                    Some("主机或端口不在白名单中".to_string())
                } else {
                    None
                },
            );
        }

        if !allowed {
            warn!("沙箱阻止网络连接: {}:{}", host, port);
        }

        allowed
    }

    /// 检查环境变量访问权限
    pub fn check_env_access(&mut self, var_name: &str) -> bool {
        let allowed = self.config.allowed_env_vars.contains(var_name);

        if self.config.enable_audit_log {
            self.log_audit(
                AuditEventType::EnvAccess,
                var_name.to_string(),
                "access".to_string(),
                allowed,
                None,
            );
        }

        allowed
    }

    /// 获取注入的环境变量
    pub fn get_injected_env(&self) -> &HashMap<String, String> {
        &self.config.injected_env_vars
    }

    /// 记录审计日志
    fn log_audit(
        &mut self,
        event_type: AuditEventType,
        resource: String,
        action: String,
        allowed: bool,
        reason: Option<String>,
    ) {
        let entry = AuditEntry {
            timestamp: chrono::Utc::now(),
            event_type,
            resource,
            action,
            allowed,
            reason,
        };
        debug!("审计日志: {:?}", entry);
        self.audit_log.push(entry);
    }

    /// 获取审计日志
    pub fn get_audit_log(&self) -> &[AuditEntry] {
        &self.audit_log
    }

    /// 清空审计日志
    pub fn clear_audit_log(&mut self) {
        self.audit_log.clear();
    }

    /// 导出审计日志为 JSON
    pub fn export_audit_log(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(&self.audit_log)
    }
}

/// 沙箱构建器
pub struct WasmSandboxBuilder {
    config: WasmSandboxConfig,
}

impl WasmSandboxBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            config: WasmSandboxConfig::default(),
        }
    }

    /// 设置安全级别
    pub fn level(mut self, level: SandboxLevel) -> Self {
        self.config = WasmSandboxConfig::from_level(level);
        self
    }

    /// 添加只读路径
    pub fn add_read_path(mut self, path: PathBuf) -> Self {
        self.config.filesystem.read_paths.push(path);
        self
    }

    /// 添加写入路径
    pub fn add_write_path(mut self, path: PathBuf) -> Self {
        self.config.filesystem.write_paths.push(path);
        self
    }

    /// 允许网络访问特定主机
    pub fn allow_host(mut self, host: String) -> Self {
        self.config.network.enabled = true;
        self.config.network.allowed_hosts.insert(host);
        self
    }

    /// 允许特定端口
    pub fn allow_port(mut self, port: u16) -> Self {
        self.config.network.allowed_ports.insert(port);
        self
    }

    /// 允许环境变量
    pub fn allow_env_var(mut self, var: String) -> Self {
        self.config.allowed_env_vars.insert(var);
        self
    }

    /// 注入环境变量
    pub fn inject_env_var(mut self, key: String, value: String) -> Self {
        self.config.injected_env_vars.insert(key, value);
        self
    }

    /// 启用审计日志
    pub fn enable_audit(mut self, enable: bool) -> Self {
        self.config.enable_audit_log = enable;
        self
    }

    /// 启用 seccomp 过滤
    pub fn enable_seccomp(mut self, enable: bool) -> Self {
        self.config.enable_seccomp = enable;
        self
    }

    /// 构建沙箱
    pub fn build(self) -> WasmSandbox {
        WasmSandbox::new(self.config)
    }
}

impl Default for WasmSandboxBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_config_default() {
        let config = WasmSandboxConfig::default();
        assert_eq!(config.level, SandboxLevel::Strict);
        assert!(!config.network.enabled);
    }

    #[test]
    fn test_sandbox_config_from_level() {
        let config = WasmSandboxConfig::from_level(SandboxLevel::Basic);
        assert_eq!(config.level, SandboxLevel::Basic);
        assert!(config.filesystem.allow_temp);
    }

    #[test]
    fn test_sandbox_file_check() {
        let mut sandbox = WasmSandbox::new(WasmSandboxConfig::basic());

        assert!(sandbox.check_file_read(&PathBuf::from("/tmp/test.txt")));
        assert!(!sandbox.check_file_read(&PathBuf::from("/etc/passwd")));
    }

    #[test]
    fn test_sandbox_network_check() {
        let mut sandbox = WasmSandboxBuilder::new()
            .allow_host("api.example.com".to_string())
            .allow_port(443)
            .build();

        assert!(sandbox.check_network_connect("api.example.com", 443));
        assert!(!sandbox.check_network_connect("malicious.com", 80));
    }

    #[test]
    fn test_sandbox_builder() {
        let sandbox = WasmSandboxBuilder::new()
            .level(SandboxLevel::Strict)
            .enable_audit(true)
            .build();

        assert_eq!(sandbox.config().level, SandboxLevel::Strict);
        assert!(sandbox.config().enable_audit_log);
    }

    #[test]
    fn test_audit_log() {
        let mut sandbox = WasmSandbox::new(WasmSandboxConfig::strict());
        sandbox.check_file_read(&PathBuf::from("/etc/passwd"));

        let log = sandbox.get_audit_log();
        assert_eq!(log.len(), 1);
        assert!(!log[0].allowed);
    }
}
