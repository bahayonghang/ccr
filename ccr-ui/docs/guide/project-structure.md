# 项目结构

CCR UI 是 CCR 工作区的一部分（workspace 成员），前后端共享依赖与版本。

## 顶层结构
```
ccr/
|-- Cargo.toml           # workspace 依赖
|-- src/                 # CCR CLI + lib
|-- ccr-ui/              # 本项目
|   |-- backend/         # Axum 后端（workspace member）
|   `-- frontend/        # Vue 3 + Vite + Pinia + Tauri
|-- docs/                # 主项目文档
|-- tests/               # 集成测试
`-- justfile             # 通用开发任务
```

## CCR UI 目录
```
ccr-ui/
|-- backend/                         # Axum 后端（workspace member）
|   |-- src/
|   |   |-- main.rs                  # 入口和路由定义
|   |   |-- api/                     # API 层（HTTP 处理器）
|   |   |   |-- handlers/            # 各模块处理器
|   |   |   |   |-- config.rs
|   |   |   |   |-- command.rs
|   |   |   |   |-- sync.rs
|   |   |   |   |-- system.rs
|   |   |   |   |-- version.rs
|   |   |   |   |-- mcp.rs
|   |   |   |   |-- agents.rs
|   |   |   |   |-- slash_commands.rs
|   |   |   |   |-- plugins.rs
|   |   |   |   |-- converter.rs
|   |   |   |   `-- platform/
|   |   |   |       |-- codex.rs
|   |   |   |       |-- gemini.rs
|   |   |   |       |-- qwen.rs
|   |   |   |       `-- iflow.rs
|   |   |   `-- mod.rs
|   |   |-- services/                # Service 层（业务逻辑）
|   |   |   |-- mod.rs
|   |   |   |-- commands.rs          # 命令执行服务
|   |   |   `-- converter_service.rs # 配置转换服务
|   |   |-- managers/                # Manager 层（数据访问）
|   |   |   |-- mod.rs
|   |   |   |-- settings_manager.rs  # 设置管理
|   |   |   |-- plugins_manager.rs   # 插件管理
|   |   |   |-- markdown_manager.rs  # Markdown 处理
|   |   |   `-- config/              # 配置管理器
|   |   |       |-- claude_manager.rs
|   |   |       |-- codex_manager.rs
|   |   |       |-- gemini_manager.rs
|   |   |       |-- qwen_manager.rs
|   |   |       `-- platform_manager.rs
|   |   |-- models/                  # Model 层（数据结构）
|   |   |   |-- mod.rs
|   |   |   |-- api.rs               # API 模型（MCP、Agent、SlashCommand、Plugin）
|   |   |   |-- converter.rs         # 转换模型
|   |   |   `-- platforms/           # 平台特定模型
|   |   |       |-- codex.rs
|   |   |       |-- gemini.rs
|   |   |       `-- qwen.rs
|   |   |-- core/                    # Core 层（基础设施）
|   |   |   |-- mod.rs
|   |   |   |-- error.rs             # 错误类型定义
|   |   |   `-- executor.rs          # CCR 命令执行器
|   |   `-- utils/                   # Utils 层（工具函数）
|   |       |-- mod.rs
|   |       `-- config_reader.rs     # 配置读取工具
|   |-- Cargo.toml                   # Rust 依赖
|   `-- logs/                        # 日志文件（自动创建）
|-- frontend/                        # Vue 3 + Vite + Pinia + Tauri
|   |-- src/
|   |   |-- main.ts / App.vue        # 入口和根组件
|   |   |-- views/                   # 视图页面（40+ 页面）
|   |   |   |-- Dashboard.vue
|   |   |   |-- Configs.vue
|   |   |   |-- Commands.vue
|   |   |   |-- Sync.vue
|   |   |   |-- Stats.vue
|   |   |   |-- ClaudeView.vue
|   |   |   |-- CodexView.vue
|   |   |   |-- GeminiView.vue
|   |   |   `-- ...
|   |   |-- components/              # 可复用组件
|   |   |   |-- MainLayout.vue
|   |   |   |-- Sidebar.vue
|   |   |   `-- ...
|   |   |-- router/                  # 路由配置
|   |   |-- stores/                  # Pinia 状态管理
|   |   |-- api/                     # API 客户端
|   |   `-- styles/ utils/           # 样式和工具
|   |-- package.json / vite.config.ts / tailwind.config.js
|   `-- src-tauri/                   # Tauri 桌面模式配置
|-- docs/                            # 本地文档（VitePress）
|   |-- .vitepress/                  # VitePress 配置
|   |-- guide/                       # 用户指南
|   |-- reference/                   # 技术参考
|   |-- public/                      # 静态资源
|   `-- index.md / README.md
|-- scripts/                         # 自动化脚本
|-- justfile                         # UI 专属任务
`-- README.md
```

## 特性与构建
- 后端使用 workspace 依赖，推荐在仓库根运行 `cargo build --workspace` / `cargo test --workspace`。
- UI 入口命令：`ccr ui -p 3000 --backend-port 8081`，使用本地源码或 `~/.ccr/ccr-ui`，必要时自动下载。
- 可选特性沿用主工程：`--features web` 启用同步/API，`--features tui` 仅影响 CLI。

## 相关 just 任务
- `just s`：前后端开发模式
- `just quick-start`：依赖检查 + 安装 + 启动
- `just build` / `just run-prod`：生产构建与运行
- `just tauri-dev` / `just tauri-build`：桌面模式
- 全部任务：`just --list`
