# GPUI 迁移研究

核查日期：2026-10-17。代码基线：dev / 1dfe776b。本文区分源码事实、设计建议、尚未运行的验证。

## 仓库事实

| 证据 | 事实 | 设计影响 |
| --- | --- | --- |
| `Cargo.toml:1` | 根 workspace 显式列出领域 crates，default-members 为 CLI | GPUI 单独 workspace，不改变默认 CLI 构建图 |
| `ccr-ui/src-tauri/Cargo.toml:1` | Tauri 已独立 workspace，package 为 ccr-desktop | 新目录 ccr-desktop，但 package/bin 用 ccr-gpui |
| `ccr-ui/src/api/generated/command-manifest.json:1` | 340 base / 348 Windows commands，278 typed contracts | 用逐命令迁移台账防止功能遗漏，不能把入口数当业务需求数 |
| `ccr-ui/src/config/platformDescriptors.ts:1` | 平台区分 settings/profiles/auth/mcp/agents/plugins/commands 能力 | 原生导航由能力驱动，不能给所有平台硬套相同页面 |
| `ccr-ui/src/shell/routeCatalog.ts:14` | 包含配置、认证、资源、同步、命令、用量、预算、签到与观测等页面及重定向 | 区分有效功能、别名入口和 Tauri 壳内部操作 |
| `crates/ccr-usage/src/lib.rs:1` | 使用量 SQL 与只读投影已有共享 owner | GPUI 直接消费 ccr-usage，不复制 SQL，不链接上游 llmusage Rust crate |
| `ccr-ui/src-tauri/src/services/usage.rs:897` | 用量服务同步且 State-free，由命令层负责 spawn_blocking | 可逐域迁出编排，UI 不执行同步 DB 查询 |
| `ccr-ui/src-tauri/src/services/agent_sessions.rs:253` | 会话列表、详情、归档刷新已存在服务函数 | 保留分页与保真度语义，迁移时复用 fixture |
| `ccr-ui/src-tauri/src/platform/mod.rs:1` | ExecutionEnvironment 已统一 Local / WSL / SSH | 通用性拆为 OS 支持、执行环境支持、平台能力支持 |
| `ccr-ui/src-tauri/src/commands/handler_registry.rs:8` | 命令有风险、授权、确认、并发、超时分类 | 去除 IPC 不代表去除业务入口策略 |
| `ccr-ui/src-tauri/src/state.rs:102` | AppState 聚合缓存、环境、任务、托盘等状态 | 不把整块 AppState 复制进 GPUI |
| `justfile:1472` | ui-* 路由给 Tauri，另有 tauri-* 和 CLI 无前缀命令 | 新入口独占 desktop-* |
| `rust-toolchain.toml:3` | 仓库工具链为 1.98.0，领域 crates 声明 MSRV 1.95 | 新依赖先验证当前工具链，不无意抬高共享库 MSRV |

## 性能判断

用户反馈明确了改进目标，但仓库读取不能证明 Tauri 性能问题由 WebView 单独造成。GPUI 省去 WebView/JS/IPC 的部分开销，数据库扫描、外部进程、网络与渲染失效仍可阻塞或占用资源。没有运行基准，不声明百分比提升。

必须先测同机 release 的启动、空闲 CPU/内存、列表帧耗时、交互延迟、后台任务下的响应，再决定是否达到迁移目标。测试 fixture 与真实用户配置隔离。

## 依赖证据

- [GPUI 官方 crate](https://docs.rs/crate/gpui/0.2.2)：当前 latest 页面指向 0.2.2，发布日期 2025-10-22。不能把该旧发布线和最新组件随意混用。
- [GPUI Kit 0.7.3](https://docs.rs/crate/gpui-kit/0.7.3)：2026-10-12 发布，绑定 gpui-pre 0.3.9 并统一导出平台、基础层、组件和资源；优先作为单一应用入口。
- [GPUI Component 0.7.3](https://docs.rs/crate/gpui-component/0.7.3)：依赖 gpui-pre =0.3.9；组件功能按需启用。
- [上游入门指南](https://gpui-kit.com/docs/getting-started/)：Kit 默认使用固定的上游快照；macOS 文本平台 feature 由 Kit 配好。原生平台实际运行仍需验证。
- [上游项目说明](https://github.com/longbridge/gpui-kit)：提供虚拟列表、可访问性与 UI 测试能力。这些是上游能力声明，不是 CCR 的验收结果。

建议候选为 gpui-kit =0.7.3、默认上游 GPUI 后端。不开启可选 GPUI Fast、JS 扩展、WebView 和不需要的语言解析器。实施时再次查注册表、核查 MSRV/features/license，生成独立 Cargo.lock，并以 --locked 验证。其余依赖优先复用领域 crates 的版本；若最新版本需要公共库升级，单列兼容性变更与回归，不假称已全部更新。

## 复用与重写

直接复用：ccr-config、ccr-core、ccr-cli 的结构化服务、ccr-codex、ccr-db、ccr-usage、ccr-sync、ccr-checkin、ccr-skills、ccr-store、ccr-types 中实际需要的 API。

逐域提取：Tauri 目录内真正框架无关、两个桌面都需要的业务编排、任务状态机和环境适配，落入新 ccr-app 或已有责任明确的领域 crate。只提取被当前迁移域消费的代码；Tauri wire DTO、AppHandle、tray、IPC 注册保留在其 adapter。

必须重写：React 视图、TanStack 状态订阅、WebView 路由、CSS、窗口/托盘交互。原生层采用 typed state/action，无字符串 invoke 桥、无本地 HTTP 桥。

不得复制：数据库 schema、用量 SQL、profile 事务/原子写、密钥脱敏、子进程清理与 SSH trust 规则。

## 未验证

尚未解析新 Cargo 图、编译 GPUI、打开原生窗口、运行屏幕阅读器/IME、采集截图或性能基线。网页和文档证据不替代这些验收。
