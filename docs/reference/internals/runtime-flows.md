# 运行时流程

本页记录当前代码里最重要的几条运行路径，便于把命令面和内部实现对齐。

## 1. CLI 入口与无子命令行为

```mermaid
sequenceDiagram
  participant User
  participant Main as main.rs
  participant Cli as cli/definitions.rs
  participant Dispatch as cli/dispatch.rs
  participant Tui as tui

  User->>Main: ccr ...
  Main->>Cli: 解析参数
  Main->>Dispatch: dispatch(&cli)
  alt 带子命令
    Dispatch-->>User: 路由到对应命令
  else 无子命令
    Dispatch->>Tui: run_tui()
  end
```

事实点：

- 默认构建启用 `tui` feature
- `ccr` 无子命令且无 `config_name` 时进入 TUI
- `ccr codex` 在无 action 时也被视为 TUI 模式
- `ccr opencode` 在无 action 时也会进入 OpenCode Auth 页签

## 2. Profile 切换

```mermaid
sequenceDiagram
  participant User
  participant Cmd as commands/profile
  participant Config as ConfigService
  participant Settings as SettingsService
  participant History as HistoryService

  User->>Cmd: ccr switch <name>
  Cmd->>Config: 读取当前平台注册表与 profiles
  Config-->>Cmd: 目标 profile
  Cmd->>Settings: 应用 settings / 备份 / 写入
  Settings-->>Cmd: 写入完成
  Cmd->>History: 记录掩码后的历史
```

相关组件：

- `ConfigService::get_config` / `set_current`
- `SettingsService::apply_config`
- `HistoryService`

## 3. `ccr ui`

```mermaid
flowchart TD
  A[ccr ui] --> B[dispatch_ui]
  B --> C[UiService]
  C --> D{本地 ccr-ui/ 存在?}
  D -- yes --> E[start_dev_mode]
  D -- no --> F{~/.ccr/ccr-ui/ 存在?}
  F -- yes --> G[start_local]
  F -- no --> H[prompt download / sync from GitHub]
```

当前代码显示的优先级：

1. 当前目录或父目录中的 `ccr-ui/`
2. `~/.ccr/ccr-ui/`
3. 下载 / 更新流程

## 4. Session 索引

```mermaid
flowchart LR
  A[ccr sessions *] --> B[SessionIndexer]
  B --> C[扫描平台 session 文件]
  C --> D[解析摘要 / 时间 / cwd / token 统计]
  D --> E[SessionStore]
  E --> F[(本地会话存储)]
```

代码边界：

- `sessions/indexer.rs`：批量扫描和 rebuild
- `storage/session_store.rs`：upsert、list、search、stats、prune

## 5. Codex 多账号 auth

```mermaid
flowchart TD
  A[ccr codex auth ...] --> B[CodexAuthService]
  B --> C[读取 ~/.codex/auth.json]
  B --> D[读取 CCR 管理的 registry / account auth 副本]
  B --> E[保存 / 切换 / 删除 / 导入导出]
  E --> F[必要时同步当前 runtime 配置]
  E --> G[创建 auth/backups]
```

该服务负责：

- 当前登录态识别
- 账号清单与过期状态
- 当前 auth 的备份与轮转
- 导入 / 导出与切换

## 6. OpenCode auth 迁移

```mermaid
flowchart TD
  A[ccr opencode auth import-codex] --> B[OpenCodeAuthService]
  B --> C[读取 CCR 已保存的 Codex registry 和 auth 快照]
  C --> D[筛选兼容的 ChatGPT OAuth 账号]
  D --> E[检查 OpenCode 中同名和 accountId 冲突]
  E --> F[只为新账号写入 OpenCode snapshot 和 registry]
  F --> G[保持当前 OpenCode runtime auth.json 不变]
```

该流程负责：

- 只读取已保存的 Codex 账号，不读取未保存的运行时登录
- 将兼容的 Codex Token 结构映射为 OpenCode `openai` OAuth 快照
- 跳过 API Key、缺少快照、无效快照和冲突账号
- 输出结构化迁移报告，供 CLI 和 TUI 复用

## 7. WebDAV 同步

```mermaid
flowchart TD
  A[sync config] --> B[写入连接配置]
  C[sync folder add/enable] --> D[注册同步目录]
  E[sync <platform> push/pull]
    --> F[SyncService]
  F --> G[过滤备份 / 历史 / 锁文件]
  F --> H[执行远端同步]
```

相关模块：

- `sync/config.rs`
- `sync/folder.rs`
- `sync/folder_manager.rs`
- `services/sync_service.rs`
