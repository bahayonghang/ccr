# CCR 架构设计（v3.9.0）

> 面向 Rust 2024 版本的分层架构；默认开启 `web` + `tui` 特性。CCR 本身提供 CLI/TUI/轻量 Web API，同时作为 ccr-ui 后端的核心依赖。

## 总览

- **工作区 (Workspace)**：根 crate `ccr` + `ccr-ui/backend`（Axum 服务）+ `ccr-ui/frontend`（Vue3+Vite+Pinia+Tauri）+ `docs`（VitePress）。
- **配置模式**：`Unified`（默认，`~/.ccr/config.toml` + `platforms/<name>/profiles.toml`）与 `Legacy`（兼容 `~/.ccs_config.toml`）并存。
- **接口形态**：CLI（Clap 解析）、TUI（Ratatui）、轻量 Web API (`ccr web`，Axum) 与完整 CCR UI (`ccr ui`，自动检测本地/用户目录/远程下载)。
- **核心能力**：多平台注册表与切换、配置 CRUD、审计与备份、临时覆盖、WebDAV 多目录同步、技能/提示词管理、成本统计。

## 工作区结构

```
ccr/                     # Workspace root
├─ Cargo.toml            # workspace + shared deps (clap/serde/tokio/axum/...)
├─ src/                  # 核心 CLI/库
├─ ccr-ui/
│  ├─ backend/           # Axum 后端，直接依赖 ccr crate（禁用默认特性）
│  └─ frontend/          # Vue3 + Vite + Pinia + Tailwind + Tauri
├─ docs/                 # VitePress 文档（中/英）
├─ examples/             # 配置示例
└─ tests/                # 集成测试
```

### 配置与数据路径

```
~/.ccr/                      # Unified 模式（默认）
  ├─ config.toml             # 平台注册表（current_platform 等）
  ├─ platforms/
  │   ├─ claude/profiles.toml
  │   ├─ codex/profiles.toml
  │   └─ gemini/profiles.toml
  ├─ backups/<platform>/     # 自动备份
  ├─ history/<platform>.json # 审计历史
  └─ ccr-ui/                 # UI 依赖/缓存

~/.ccs_config.toml           # Legacy 模式（兼容 CCS）
~/.claude/settings.json      # 直接写入 Claude Code 设置
```

## 分层架构

```
CLI / Web API / TUI
      │
      ▼
  Services（业务编排）
      │
      ▼
 Managers（数据访问/持久化）
      │
      ▼
 Core & Utils（基础设施）
```

### 模块职责

- **CLI 层 (`src/commands/`)**
  - 子模块化：`platform/`、`profile/`、`lifecycle/`、`data/`、`common/`，以及独立命令 `sync_cmd`、`ui`、`skills_cmd`、`prompts_cmd`、`check_cmd`、`update`。
  - Clap 派生的顶层路由在 `main.rs`，支持快捷 `ccr <profile>` 直接切换。
- **服务层 (`src/services/`)**
  - `ConfigService`（配置切换/导入导出/验证）、`SettingsService`（settings.json）、`HistoryService`、`BackupService` & `MultiBackupService`、`SyncService`（WebDAV）、`UiService`（CCR UI 启动编排）。
- **管理层 (`src/managers/`)**
  - 配置：`ConfigManager`（Legacy）、`PlatformConfigManager`（Unified 注册表）、`SyncConfigManager`/`SyncFolderManager`、`TempOverrideManager`。
  - 数据：`SettingsManager`、`HistoryManager`、`CostTracker`（统计）、`PromptsManager`、`SkillsManager`、`ConflictChecker`。
- **核心层 (`src/core/`)**
  - `error`（统一错误/退出码）、`lock`（文件锁 + 进程内互斥）、`atomic_writer`、`fileio`、`file_manager`、`logging`（tracing + 彩色输出）。
- **模型与平台 (`src/models/`, `src/platforms/`)**
  - `Platform`/`PlatformPaths`/`ProfileConfig`，具体实现：Claude/Codex/Gemini（Qwen/iFlow stub）。
  - `PlatformRegistry`/`PlatformDetector` 提供平台枚举、检测、信息展示。
- **同步 (`src/sync/`)**
  - `SyncService` 基于 `reqwest_dav`，支持目录递归、智能过滤、允许列表、远程目录保障。
  - `content_selector` 用于交互式选择同步内容；`commands` 覆盖 folder/all/dynamic 子命令。
- **界面**
  - `web/`：Axum 轻量 API（缓存系统信息、JSON 响应、错误包装）。
  - `tui/`：Ratatui 视图与主题；可选 `tui` 特性编译。

### 依赖方向

- CLI/Web/TUI 仅调用 Service；Service 依赖 Managers；Managers 依赖 Core/Utils；Models/Platforms/Utils 可被上层共享。
- `ccr-ui/backend` 直接复用 `ccr` crate（关闭默认特性），在自身层实现路由/节流/中间件。

## 核心流程

### Profile 切换（Unified 默认）

1) CLI 解析（`commands::switch_command` 或快捷 `ccr <name>`）  
2) `ConfigService`：读取 `config.toml` → 定位当前平台 → 加载目标 `profiles.toml`  
3) `SettingsService`：获取文件锁 → 备份现有 `settings.json` → 原子写入新配置  
4) `HistoryService`：记录操作、环境变量差异（自动掩码）  
5) 可选：`TempOverrideManager` 注入临时 token/base_url/model  

### 平台管理

- `platform list/current/info/init/switch` 通过 `PlatformConfigManager` 维护 `config.toml` 中的注册表与当前平台指针。
- 平台实现 `PlatformConfig` trait，暴露路径与 profile 读写；未实现的平台返回 `PlatformNotSupported`。

### WebDAV 多目录同步

1) `sync config` 写入 WebDAV 连接信息 (`SyncConfigManager`)  
2) `sync folder ...` 注册/启用目录（默认挂载 `~/.ccr`、`platforms/*` 等）  
3) `sync push/pull`：`SyncService` 递归遍历，过滤备份/历史/locks/UI，支持 `--force`、交互式内容选择、单目录或 `sync all`  

### CCR UI 启动

- `UiService` 依序检查 `./ccr-ui` → `~/.ccr/ccr-ui` → GitHub 下载（交互确认），然后启动前后端；端口可通过 `-p/--backend-port` 覆盖。

## 可靠性与性能

- **并发安全**：文件锁 + 进程内互斥，原子写入避免损坏。
- **备份**：切换/导入前自动备份，`MultiBackupService` 支持多平台备份清理。
- **日志**：`CCR_LOG_LEVEL` 控制等级；终端彩色 + `~/.ccr/logs/` 按日轮转。
- **性能**：`rayon` 并行验证，`fileio` 统一 I/O，dev profile `opt-level=1` + 依赖 `opt-level=2`，Axum 层缓存系统信息。

## 测试与质量

- 单元测试覆盖平台/管理器/锁等核心模块；`tests/` 进行端到端集成（临时目录隔离）。
- 默认零 `panic!`，错误类型集中在 `CcrError`；命令返回退出码。

## 参考与扩展

- 新命令：置于 `src/commands/<domain>/`，在 `mod.rs` 导出并在 `main.rs` 路由。
- 新平台：实现 `PlatformConfig` + 在 `platforms::create_platform` 注册。
- 新同步源：扩展 `SyncService` 或在 `sync::commands` 中增加内容选择器策略。
