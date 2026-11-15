# sync - WebDAV 配置同步

`ccr sync` 提供基于 WebDAV 协议的多目录与配置文件云端同步能力，默认适配坚果云，并兼容 Nextcloud / ownCloud 等标准 WebDAV 服务。

## 用法

```bash
# 交互式配置与基础同步
ccr sync config
ccr sync status
ccr sync push [--force] [-i]
ccr sync pull [--force]

# 同步目录注册与管理
ccr sync folder list
ccr sync folder add <name> <local-path> -r <remote-path>
ccr sync folder remove <name>
ccr sync folder enable <name>
ccr sync folder disable <name>
ccr sync folder info <name>

# 对全部已启用目录或单个已注册目录操作
ccr sync all push|pull|status
ccr sync <folder> push|pull|status
```

## 同步模式与智能过滤

CCR 支持以下两类 WebDAV 同步范围：

- **Unified 目录同步（默认推荐）**：同步 `~/.ccr/`（或 `CCR_ROOT`）下的平台注册表（`config.toml`）与各平台配置资产，自动过滤缓存与临时目录。
- **已注册目录同步（`sync folder` / `sync all`）**：并行管理 `~/.claude`、`~/.codex`、`~/.gemini` 等宿主或自定义配置目录。

### 自动排除项

同步过程中自动跳过无需跨设备传输的大体积、SQLite 共享内存日志或非权威投影文件：

- 目录：`backups/`、`history/`、`ccr-ui/`、`.locks/`、`.git/`
- 文件与模式：`profiles.toml`、`profiles.db-wal`、`profiles.db-shm`、`*.tmp`、`*.lock`、`*.bak`、`.DS_Store`、`Thumbs.db`、`desktop.ini`

> [!WARNING]
> **敏感凭据说明**：对于包含明文 Bearer 的敏感同步资产（如 `codex-config` 对应的 `~/.codex/config.toml`），同步到 WebDAV 时凭据会封装在 v2 加密信封内并要求独立的操作口令。同时建议 WebDAV 服务端使用应用专用密码与 `https://` 连接。

## 核心子命令说明

| 子命令 | 说明 |
|--------|------|
| `sync config` | 交互式配置 WebDAV 服务器地址、账号、应用密码与远程路径，并测试连接 |
| `sync status` | 查看当前同步配置与远端目录状态 |
| `sync push [--force] [-i]` | 上传本地配置到远端；`-f/--force` 跳过覆盖确认，`-i` 交互式选择同步范围 |
| `sync pull [--force]` | 从远端拉取配置到本地（覆盖前自动创建 `before_pull` 本地备份） |
| `sync folder ...` | 注册、查看、启用/禁用或移除独立同步目录 |
| `sync all ...` | 对所有已启用的注册目录批量执行 `push`、`pull` 或 `status` |

## 常见使用场景

### 场景 1：多设备同步多平台配置

```bash
# 设备 A：初始化并配置各平台后推送到云端
ccr claude profile init
ccr codex profile init
ccr grok profile init

ccr sync config
ccr sync push

# 设备 B：配置相同 WebDAV 后拉取并校验
ccr sync config
ccr sync pull
ccr platform list
ccr current
ccr validate
```

### 场景 2：注册并批量同步宿主目录

```bash
ccr sync folder add claude ~/.claude -r /ccr-sync/claude
ccr sync folder enable claude
ccr sync folder list

# 批量查看状态并上传
ccr sync all status
ccr sync all push
```

### 场景 3：离线归档补充

除 WebDAV 云端同步外，仍建议定期使用平台 `profile export` 生成本地 `ExchangeV1` 快照归档：

```bash
ccr claude profile export --no-secrets -o ~/backups/claude-profiles.toml
ccr codex profile export --no-secrets -o ~/backups/codex-profiles.toml
```

## 当前契约边界

- `ccr sync` 只执行当前 CLI 帮助中公开暴露的 `config`、`status`、`push`、`pull`、`folder`、`all` 与已注册目录快捷子命令。
- 后台自动轮询同步与服务端多版本合并并非当前 CLI 契约；执行覆盖性 `pull` / `push` 前建议先运行 `status` 确认远端状态。

## 相关命令

- [`claude`](./claude) / [`codex`](./codex) / [`grok`](./grok) - 平台 Profile 配置库与 `ExchangeV1` 导入导出
- [`current`](./current) - 查看当前运行时状态
- [`clean`](./clean) - 清理本地旧备份与缓存
