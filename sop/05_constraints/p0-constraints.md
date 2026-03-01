---
version: v3.1.0
level: P0
---

# P0级约束

constraint_strength: 不可违背，违反即熔断

## 安全约束

```yaml
P0-SEC-001:
  name: 禁止硬编码密钥
  desc: 禁止在代码中硬编码密钥、密码、Token等敏感信息
  verify: 静态分析工具(git-secrets, truffleHog)
  handle: 构建失败，必须立即修复
  exception: 无例外

P0-SEC-002:
  name: 禁止使用已知漏洞库
  desc: 禁止使用存在已知安全漏洞的第三方库
  verify: 依赖扫描工具(npm audit, Snyk)
  handle: 构建失败，必须升级或替换
  exception: 无替代方案时需技术委员会审批

P0-SEC-003:
  name: 禁止关闭安全校验
  desc: 禁止在生产环境关闭身份验证、授权检查、输入验证
  verify: 代码审查 + 安全扫描
  handle: 构建失败，必须修复
  exception: 无例外

P0-SEC-004:
  name: 网络访问白名单
  desc: Agent 网络访问必须受 allowlist 限制，禁止访问未授权的外部资源
  verify: 网络代理审计
  handle: 阻止访问，记录违规详情（URL、时间、上下文），触发安全告警
  exception: 无例外
  allowlist:
    - pattern: "*.github.com"
      purpose: "代码仓库访问"
    - pattern: "*.npmjs.com"
      purpose: "NPM 包管理"
    - pattern: "pypi.org"
      purpose: "Python 包管理"
    - pattern: "*.crates.io"
      purpose: "Rust 包管理"
  runtime_verification:
    tool: network-proxy-auditor
    config:
      log_path: logs/network-audit.log
      alert_threshold: 3  # 连续 3 次违规触发告警
      block_mode: true    # 阻止非白名单访问
    detection_flow:
      - 拦截请求
      - 检查 allowlist
      - 记录违规日志
      - 触发告警（超过阈值）
    audit_log_format:
      timestamp: ISO8601
      source_skill: string
      target_url: string
      action: blocked | allowed
      reason: string

P0-SEC-005:
  name: 密钥安全注入
  desc: 密钥必须通过安全注入机制传递，模型不得接触明文密钥
  verify: 密钥审计日志
  handle: 阻止操作，记录违规详情（操作、时间、上下文），触发安全告警
  exception: 无例外
  injection:
    method: environment_variable
    pattern: "SECRET_${DOMAIN}_${KEY_NAME}"
  restrictions:
    - 禁止在日志中输出密钥
    - 禁止在代码中硬编码密钥
    - 禁止将密钥传递给未授权的 Skill
  implementation_guide:
    environment_variable_setup:
      example: |
        # .env 文件（不提交到版本控制）
        SECRET_ORDER_DB_PASSWORD=xxx
        SECRET_GITHUB_TOKEN=xxx
        
        # Docker Compose 配置
        services:
          app:
            environment:
              - SECRET_${DOMAIN}_${KEY_NAME}
    
    key_management_best_practices:
      - 使用密钥管理服务（如 AWS Secrets Manager、HashiCorp Vault）
      - 定期轮换密钥（建议 90 天）
      - 最小权限原则：仅授予必要的权限
      - 密钥访问审计日志
      - 密钥分级管理（开发/测试/生产环境隔离）
    
    skill_authorization:
      example: |
        # 密钥授权配置（skill-secrets.yaml）
        skill_secrets:
          sop-code-implementation:
            - SECRET_GITHUB_TOKEN
            - SECRET_NPM_TOKEN
          sop-test-implementation:
            - SECRET_GITHUB_TOKEN
          sop-code-explorer: []  # 不需要密钥
          sop-requirement-analyst: []
          sop-architecture-design: []
```

## 质量约束

```yaml
P0-QUAL-001:
  name: 核心模块测试覆盖率100%
  desc: 核心业务逻辑模块单元测试覆盖率必须达到100%
  verify: 覆盖率工具(istanbul, coverage.py, jacoco)
  handle: 构建失败，必须补充测试
  exception: 无例外
  core_modules:
    - 业务逻辑层(Service层)
    - 数据访问层(Repository层)
    - 领域模型层(Domain层)

P0-QUAL-002:
  name: 禁止强制解包
  desc: 禁止使用unwrap(), expect()等强制解包操作
  verify: 静态分析工具(ESLint, clippy)
  handle: 构建失败，必须使用安全解包
  exception: 性能关键路径且已证明安全时可例外(需审批)

P0-QUAL-003:
  name: 禁止忽略错误
  desc: 禁止忽略函数返回的错误(如 _ = func())
  verify: 静态分析工具
  handle: 构建失败，必须处理错误
  exception: 明确不需要处理时需注释说明
```

## 架构约束

```yaml
P0-ARCH-001:
  name: 禁止循环依赖
  desc: 模块间禁止存在循环依赖
  verify: 依赖分析工具(dependency-cruiser, madge)
  handle: 构建失败，必须重构
  exception: 无例外

P0-ARCH-002:
  name: 禁止跨层调用
  desc: 禁止跳过中间层直接调用(如Controller直接调用Repository)
  verify: 架构约束工具(ArchUnit)
  handle: 构建失败，必须修复
  exception: 无例外
```

## 合规约束

```yaml
P0-COMP-001:
  name: 数据隐私保护
  desc: 用户数据必须符合GDPR/个人信息保护法要求
  verify: 合规审计、代码审查
  handle: 构建失败，必须修复
  exception: 无例外

P0-COMP-002:
  name: 许可证合规
  desc: 第三方库许可证必须符合项目合规要求
  verify: 许可证扫描工具
  handle: 构建失败，必须替换
  exception: 法务审批
```

## 验证工具

```yaml
tools:
  - type: 密钥检测
    names: [git-secrets, truffleHog]
    integration: pre-commit hook
  - type: 漏洞扫描
    names: [npm audit, Snyk]
    integration: CI/CD
  - type: 覆盖率检查
    names: [istanbul, coverage.py]
    integration: npm test
  - type: 架构约束
    names: [ArchUnit, dependency-cruiser]
    integration: CI/CD
  - type: 网络审计
    names: [网络代理审计]
    integration: 实时监控
    purpose: P0-SEC-004 网络访问白名单验证
  - type: 密钥审计
    names: [密钥审计日志]
    integration: 实时监控
    purpose: P0-SEC-005 密钥安全注入验证
```
