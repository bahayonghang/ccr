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
|-- backend/
|   |-- src/
|   |   |-- main.rs               # 入口
|   |   |-- handlers/             # API 路由（config/command/sync/system/...）
|   |   |-- executor/             # 调用 CCR CLI
|   |   |-- settings_manager.rs   # settings.json 读写
|   |   |-- plugins_manager.rs    # 插件与知识库
|   |   `-- claude_config_manager.rs
|   |-- Cargo.toml
|   `-- README.md
|-- frontend/
|   |-- src/
|   |   |-- main.ts / App.vue
|   |   |-- views/                # Dashboard、Configs、Commands、Sync、Stats 等
|   |   |-- components/           # 布局与业务组件
|   |   |-- router/ stores/ api/  # 路由、状态、API 客户端
|   |   `-- styles/ utils/
|   |-- package.json / vite.config.ts / tailwind.config.js
|   `-- src-tauri/                # 桌面模式
|-- docs/                         # 本地文档（VitePress）
|-- scripts/                      # 自动化脚本
|-- justfile                      # UI 专属任务
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
