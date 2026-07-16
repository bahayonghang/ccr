# CCR 运行时架构图设计

## 交付形态

- 使用 Archify `architecture` renderer，而不是手写 SVG。
- 可复现输入：任务目录内 `ccr-runtime-architecture.architecture.json`。
- 发布输出：`docs/public/architecture/ccr-runtime-architecture.html`。
- 单图采用从左到右的主叙事：用户入口 -> 本机 CCR 运行时 -> 本地数据/主机集成 -> 外部执行文件与网络服务。

## 组件分组

1. 用户入口
   - CLI/TUI 用户
   - CCR Desktop（Vue WebView）
   - VS Code 扩展
2. 本机 CCR 进程
   - `ccr` binary + `ccr-cli` / `ccr-tui`
   - Tauri invoke backend + AppState
   - 共享领域层：config/core/types、Codex/skills/sync/checkin/usage
3. 本地数据与主机边界
   - `~/.ccr` 配置、profiles、history、backup、locks
   - `ccr-ui.db` / usage archive SQLite
   - `~/.llmusage/llmusage.db` 只读投影
   - AI CLI runtime 文件与本机环境（Claude、Codex、OpenCode、WSL/SSH）
4. 外部依赖
   - `llmusage` CLI 子进程
   - WebDAV 服务
   - 签到/供应商 HTTPS API
   - AI CLI / provider runtime

## 数据流

- Desktop Vue -> Tauri backend：`invoke` IPC；backend 返回稳定 DTO/事件。
- VS Code -> `ccr` binary：`execFile` 子进程，JSON/stdout 返回；只读展示可退回本地文件读取。
- CLI/TUI/Tauri -> 共享领域层：进程内 Rust 调用。
- 共享领域层 -> `~/.ccr`：带锁、备份、敏感权限和原子替换的配置读写。
- Desktop -> `ccr-ui.db`：桌面业务读写；Desktop/usage -> llmusage DB：只读 SQLite。
- Desktop -> `llmusage`：`sync --json-events` 子进程；子进程拥有 llmusage DB 的采集、迁移和写入责任。
- Sync domain -> WebDAV：经 TLS 的 WebDAV/Basic Auth 上传下载。
- Check-in domain -> provider API：带 cookie/token 的 HTTPS 请求；响应回流桌面状态与本地记录。

## 信任边界

- `region`：本机用户会话，包围 CCR 进程和本机数据；表示这些组件共享 OS 用户权限，但不代表同一进程。
- `security-group`：CCR 管理的敏感本地数据，包围 `~/.ccr` 与桌面 SQLite；标注 token/profile/history/凭据风险。
- `security-group`：第三方本地执行文件，包围 `llmusage` 和 AI CLI/runtime；强调子进程与 PATH/版本信任。
- `region`：外部网络，包围 WebDAV 与供应商 API；跨界连接使用 security/emphasis 变体并标注协议与敏感数据类型。

## 取舍

- 13 个 workspace crate 聚合成少量运行时职责节点。完整 crate 依赖已经由 `docs/reference/architecture.md` 与 crate map 维护；本图优先展示运行时与信任关系。
- 不绘制普通 Rust/JS 编译依赖，因为它们不是独立部署或运行时信任主体。
- 使用 cards 承载安全说明、数据所有权和排除项，减少箭头数量。
- 不启用 trace 动画；这是审阅型架构图，不是演示动画。

## 兼容与回滚

- HTML 为独立静态资源，不参与应用运行时。
- 删除目标 HTML 与任务内 JSON 即可完整回滚，不影响现有 Markdown 架构文档。
