# rt-plugin-pinyin

`rt-plugin-pinyin` 是 Rust 工具箱的一个插件，用于将中文文本转换为拼音。

## 功能 (Features)

*   将中文文本转换为带声调或不带声调的拼音。
*   支持中英文混合文本转换。

## 安装 (Installation)

1.  **克隆仓库**:
    ```bash
    git clone https://github.com/your-repo/rust-tool.git
    cd rust-tool
    ```
2.  **构建插件**:
    ```bash
    cargo build --release --package rt-plugin-pinyin
    ```
    这将在 `target/release/` 目录下生成 `rt-plugin-pinyin.exe` (Windows) 或 `rt-plugin-pinyin` (Linux/macOS) 可执行文件。

3.  **部署插件**:
    将生成的可执行文件复制到 `rt-cli` 或 `rt-gui` 可执行文件所在目录下的 `plugins` 文件夹中。
    例如：
    ```
    rust-tool/
    ├── target/release/rt-cli.exe
    ├── target/release/rt-gui.exe
    └── target/release/plugins/
        └── rt-plugin-pinyin.exe
    ```

## 使用 (Usage)

### 命令行界面 (CLI)

通过 `rt-cli` 调用 `rt-plugin-pinyin` 插件：

1.  **查看插件信息**:
    ```bash
    cargo run --bin rt-cli -- spec text.pinyin
    ```
    这将输出插件的元数据，包括名称、描述、输入/输出模式等。

2.  **运行插件**:
    插件接受一个 JSON 字符串作为输入，包含 `text` (要转换的中文文本) 和 `tone` (布尔值，是否包含声调，默认为 `true`)。

    **示例 1: 带声调转换**
    ```bash
    echo '{"text": "你好世界", "tone": true}' | cargo run --bin rt-cli -- run text.pinyin
    # 预期输出: {"pinyin":"nǐ hǎo shì jiè"}
    ```

    **示例 2: 不带声调转换**
    ```bash
    echo '{"text": "你好世界", "tone": false}' | cargo run --bin rt-cli -- run text.pinyin
    # 预期输出: {"pinyin":"ni hao shi jie"}
    ```

    **示例 3: 混合文本转换**
    ```bash
    echo '{"text": "Hello世界", "tone": true}' | cargo run --bin rt-cli -- run text.pinyin
    # 预期输出: {"pinyin":"Hello shì jiè"}
    ```

### 图形用户界面 (GUI)

如果使用 `rt-gui`，插件将自动加载，并在界面中提供相应的输入表单和输出显示。

## 开发 (Development)

### 国际化 (Internationalization)

插件支持多语言，通过 `locales` 目录下的 `tool.en.json` (英文) 和 `tool.zh.json` (中文) 文件进行配置。

### 结构 (Structure)

*   `src/main.rs`: 插件主逻辑，实现 `Tool` trait。
*   `src/i18n.rs`: 国际化模块，负责加载和管理多语言文本。
*   `locales/`: 包含多语言 JSON 文件。

## 许可证 (License)

[根据您的项目选择合适的许可证，例如 MIT 或 Apache 2.0]
