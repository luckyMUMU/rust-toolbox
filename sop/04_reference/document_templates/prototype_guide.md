---
version: v2.4.0
updated: 2026-02-22
---

# 原型设计规范

**层级**: L3 - 功能级 (UI项目)  
**位置**: `docs/01_requirements/prototypes/[module]/`  
**创建者**: sop-requirement-analyst  
**规范**: 定义界面原型文件格式和设计标准

---

## 目录结构

```
docs/01_requirements/prototypes/
├── [module]/
│   ├── [feature]_wireframe.drawio    # 线框图
│   ├── [feature]_mockup.fig          # 高保真原型
│   ├── [feature]_interaction.md      # 交互说明
│   └── assets/                       # 图片资源
│       ├── icons/
│       └── images/
└── design_system.md                  # 设计规范 (可选)
```

---

## 原型文件格式

| 类型 | 格式 | 工具 | 用途 |
|------|------|------|------|
| 线框图 | `.drawio`, `.png` | Draw.io, Figma | 低保真，快速迭代 |
| 高保真 | `.fig`, `.sketch`, `.xd` | Figma, Sketch, Adobe XD | 最终设计稿 |
| 交互说明 | `.md` | Markdown | 补充原型无法表达的细节 |
| 设计规范 | `.md` | Markdown | 全局设计标准 |

---

## 线框图规范

### 内容要求
- 页面布局结构
- 核心元素位置
- 导航关系
- 简单标注

### 命名规范
```
[module]_[feature]_wireframe_[version].[ext]

示例:
order_create_wireframe_v1.drawio
user_profile_wireframe_v2.png
```

---

## 高保真原型规范

### 内容要求
- 视觉设计
- 组件状态 (默认/悬停/点击/禁用)
- 交互动效说明
- 响应式布局 (多设备)

### 命名规范
```
[module]_[feature]_mockup_[version].[ext]

示例:
order_create_mockup_v1.fig
user_profile_mockup_v2.sketch
```

### 页面清单
| 页面 | 状态 | 说明 |
|------|------|------|
| [页面1] | ✅ 完成 | [说明] |
| [页面2] | 🚧 进行中 | [说明] |

---

## 交互说明模板

```markdown
# [功能] 交互说明

## 页面: [页面名称]

### 元素: [元素名称]

#### 默认状态
- 样式: [描述]
- 位置: [描述]

#### 交互: [触发方式]

**触发条件**: [条件]

**系统响应**:
1. [响应步骤1]
2. [响应步骤2]

**状态变化**:
- 从: [状态A]
- 到: [状态B]

**异常处理**:
- [异常情况] → [处理方式]

#### 动效说明 (可选)
- 类型: [淡入/滑动/缩放]
- 时长: [毫秒]
- 缓动: [ease/linear]

---

## 页面流转

```
[页面A] --[操作]--> [页面B]
```
```

---

## 设计规范 (design_system.md)

### 颜色
| 名称 | 色值 | 用途 |
|------|------|------|
| Primary | `#[色值]` | 主色调 |
| Secondary | `#[色值]` | 辅助色 |
| Error | `#[色值]` | 错误提示 |

### 字体
| 层级 | 大小 | 字重 | 用途 |
|------|------|------|------|
| H1 | [大小] | [字重] | 页面标题 |
| H2 | [大小] | [字重] | 区块标题 |
| Body | [大小] | [字重] | 正文 |

### 间距
| 名称 | 值 | 用途 |
|------|-----|------|
| xs | [值] | 紧凑间距 |
| sm | [值] | 小间距 |
| md | [值] | 中间距 |
| lg | [值] | 大间距 |

### 组件
- [按钮样式]
- [输入框样式]
- [卡片样式]

---

## 与FRD的关系

FRD 中的原型章节链接到此处：

```markdown
## 3. 界面原型

### 3.1 线框图
👉 [线框图](docs/01_requirements/prototypes/[module]/[feature]_wireframe.drawio)

### 3.2 高保真原型
👉 [高保真原型](docs/01_requirements/prototypes/[module]/[feature]_mockup.fig)

### 3.3 交互说明
👉 [交互说明](docs/01_requirements/prototypes/[module]/[feature]_interaction.md)
```

---

## 约束

✅ **必须**:
- 线框图和高保真原型至少有一种
- 交互说明补充关键交互细节
- 命名规范统一

❌ **禁止**:
- 在原型中包含实现细节
- 原型文件过大（应压缩图片）
- 版本混乱（使用版本号管理）
