# ADR-Plugin-003: WASM 插件沙箱隔离策略

## 状态
- [x] 已接受
- [ ] 已废弃
- [ ] 已替代

## 背景 (Context)

### 问题描述
WASM 插件需要在安全的环境中执行，防止恶意代码对宿主系统造成损害。需要设计一个多层次的沙箱隔离策略。

### 约束条件
- 必须限制文件系统访问
- 必须限制网络访问
- 必须限制内存使用
- 必须限制 CPU 使用
- 必须支持不同安全级别的插件

### 影响范围
- `src/plugins/wasm.rs` - WASM 插件实现
- `src/plugins/wasm_sandbox.rs` - 沙箱隔离
- `src/plugins/wasm_limits.rs` - 资源限制

## 决策 (Decision)

### 选择的方案
采用 **多层次沙箱隔离策略**，包括：
1. 安全级别分级
2. 文件系统权限控制
3. 网络权限控制
4. 资源限制（内存、CPU、时间）
5. 审计日志

### 决策理由
多层次隔离提供纵深防御，不同安全级别满足不同场景需求。

## 选项对比 (Options Considered)

| 选项 | 优点 | 缺点 | 结论 |
|------|------|------|------|
| **多层次沙箱** (已选择) | 灵活、可配置、纵深防御 | 实现复杂 | ✅ 选择 |
| 完全隔离 | 最安全 | 功能受限、性能差 | ❌ 不选 |
| 无隔离 | 性能最好 | 不安全 | ❌ 不选 |
| 容器隔离 | 强隔离 | 资源开销大 | ⚠️ 备选 |

## 详细设计

### 1. 安全级别

```rust
pub enum SandboxLevel {
    Unrestricted,  // 无限制 - 仅用于信任模块
    Basic,         // 基础隔离 - 限制文件系统和网络
    Strict,        // 严格隔离 - 禁止所有外部访问
    Maximum,       // 最大隔离 - 额外限制 CPU 和内存
}
```

### 2. 文件系统权限

```rust
pub struct FileSystemPermissions {
    pub read_paths: Vec<PathBuf>,      // 允许读取的目录
    pub write_paths: Vec<PathBuf>,     // 允许写入的目录
    pub execute_paths: Vec<PathBuf>,   // 允许执行的目录
    pub allow_temp: bool,              // 是否允许临时文件
    pub allow_cwd: bool,               // 是否允许当前目录
}
```

### 3. 网络权限

```rust
pub struct NetworkPermissions {
    pub enabled: bool,                   // 是否允许网络
    pub allowed_hosts: HashSet<String>,  // 允许的主机
    pub allowed_ports: HashSet<u16>,     // 允许的端口
    pub allow_dns: bool,                 // 是否允许 DNS
    pub allow_http: bool,                // 是否允许 HTTP
    pub allow_https: bool,               // 是否允许 HTTPS
}
```

### 4. 资源限制

```rust
pub struct WasmResourceLimits {
    pub max_memory: u64,           // 最大内存
    pub max_heap_memory: u64,      // 最大堆内存
    pub max_stack_size: u64,       // 最大栈大小
    pub max_execution_time: Duration,  // 最大执行时间
    pub max_fuel: Option<u64>,     // 最大燃料（指令计数）
}
```

## 影响 (Consequences)

### 正面影响
- ✅ 多层次安全防护
- ✅ 灵活的安全级别配置
- ✅ 完整的审计日志
- ✅ 资源使用可控

### 负面影响/风险
- ⚠️ 增加实现复杂度 → 通过模块化设计缓解
- ⚠️ 可能影响性能 → 通过优化减少开销
- ⚠️ 需要依赖 wasmtime → 需要添加依赖

### 技术债务
- wasmtime 依赖尚未添加 → 需要在 Cargo.toml 中添加

## 实施计划

### 已完成
- [x] 设计沙箱隔离架构
- [x] 实现 `WasmSandbox` 和 `WasmSandboxConfig`
- [x] 实现 `ResourceLimiter` 和 `WasmResourceLimits`
- [x] 实现审计日志

### 待完成
- [ ] 添加 wasmtime 依赖
- [ ] 集成 wasmtime 的燃料限制
- [ ] 实现 seccomp 过滤

## 决策记录

| 日期 | 决策人 | 动作 | 说明 |
|------|--------|------|------|
| 2026-02-20 | Architecture Team | 创建 | 设计 WASM 沙箱隔离策略 |
| 2026-03-01 | Architecture Team | 迁移 | 从 .temp/ADR/ 迁移到正式文档目录 |
