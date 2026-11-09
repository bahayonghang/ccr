# GPUI 01 续作：性能基线当前边界

日期：2026-11-05（America/New_York）。源码基线：`0bf9ba48938fb286a581f4e9eb51e07f3273c629`，`dev`，开始时工作树干净。本轮是只读源码复查与规划状态同步，没有启动 Tauri/GPUI、没有运行 WSL/doctor、没有采集性能样本，也没有修改产品代码、依赖或历史原始证据。

## 宿主与收据

| 本轮仓库根命令 | 退出码 | 实际输出 |
| --- | --- | --- |
| `rustc -vV` | 0 | rustc 1.98.0；host `x86_64-pc-windows-msvc`；LLVM 22.1.8 |
| `rustup target list --installed` | 0 | 只有 `x86_64-pc-windows-msvc` |
| `git rev-parse HEAD` | 0 | `0bf9ba48938fb286a581f4e9eb51e07f3273c629` |
| `git status --short`（复查前） | 0 | 空 |

Windows AC1.1–1.4/native 输入沿用 `evidence.md` 与既有独立 check 的历史记录，本轮不重新认领。AC1.5 的 macOS 编译为 `not_run`；已安装 Windows target 不是 macOS 编译证据，历史 WSL Linux 编译也不是独立 Linux 桌面通过。本轮不安装跨平台 target，不重跑历史 Linux/WSL 探针。

## Tauri：10/17 隔离缺口仍未接通

下表只证明当前源码形状，不证明运行成功或真实路径被访问。

| 当前 owner / 定位 | 本轮源码事实 | 下一步必须满足的边界 |
| --- | --- | --- |
| `ccr-ui/src-tauri/src/main.rs:124–172` | logger、`migrate_catalogs`、`initialize_app_pool`、`create_usage_archive_pool`、`LlmusageRuntime::discover` 后才构造 AppState；没有先解析整应用 fixture 描述 | 显式路径必须在任何初始化前传给 owner；不能靠改全局 HOME/USERPROFILE 隔离 |
| `crates/ccr-db/src/database/mod.rs:68–144` | `ui_database_path` 仍取真实 home；两类 pool 对真实 home 跑迁移；archive legacy migration 还取 `get_db_path()` | app DB、archive DB 与 legacy migration source home 同时显式注入，复用迁移/schema |
| `crates/ccr-config/src/models/platform.rs:455–495`；`crates/ccr-config/src/managers/config/catalog/migrate.rs:29` | CCR_ROOT 控制 catalog root，启动遍历 implemented platforms；这个 override 不覆盖其他 owner | catalog path seam 不能被描述为 whole-app sandbox |
| `ccr-ui/src-tauri/src/state.rs:220–291,345,400` | 已有 load/save/quarantine 的显式 path 方法，但生产 AppState 仍调用 default-path load，后续 `save()` 仍取默认路径 | AppState 捕获同一 fixture preferences path，读、quarantine、写始终一致 |
| `ccr-ui/src-tauri/src/state.rs:362`；`crates/ccr-db/src/services/log_persistence.rs:34,69–150` | monitoring 仍构造默认 LogPersistenceService；持久化仍走全局 `database::with_connection` | fixture app pool 与 monitoring 必须对齐；现有 `DbAccess::Pool` 是可复用 seam，尚未接到该服务 |
| `crates/ccr-core/src/core/logging.rs:250–254` | logger 已支持非空 CCR_ROOT 下的 logs，缺省仍回 home | 这是可复用的单域隔离，不等于所有 DB/偏好/认证都隔离 |
| `ccr-ui/src-tauri/src/llmusage_adapter/mod.rs:37–45`；`crates/ccr-usage/src/paths.rs:15–27` | 已有 `from_paths`/`AppPaths::from_root`，默认 discover 支持 LLMUSAGE_HOME | 显式路径可复用；不复制 SQL、不链接 upstream llmusage crate；该 override 只隔离这一域 |
| `ccr-ui/src-tauri/src/main.rs:181,191,198,208–209,248`；`ccr-ui/src-tauri/src/platform/wsl.rs:231–263,288–292` | 默认 SSH trust、tray 初刷、pending OAuth 恢复、本地环境注册仍在生产启动；Windows 调用 `detect_wsl_distros_with_cache(false)`，有效磁盘缓存可直接返回，否则执行 WSL | 必须先绑定合成 fixture 与受控本地 owner；当前直接 launch 可能触发本轮禁止的 WSL 探测，因此不运行 |
| `ccr-ui/src-tauri/src/platform/local.rs:142–148,192–193` | Codex/Gemini/OpenCode 等路径仍来自默认 home，GROK_HOME 是局部覆盖 | 同一 fixture home/path provider 必须贯穿领域；不能只给 profiles 路径就启动整应用 |
| `ccr-ui/src-tauri/tauri.conf.json` 与生产 startup/builder | identity 仍为 `com.ccr.desktop`；所查启动链未配置独立 `data_directory`/`dataDirectory` | 采样需要独立 WebView storage/cache；仅重定向业务 DB 不隔离前端 localStorage/preferences |

这些缺口横跨 Tauri composition root、DB/migration、认证/环境和 WebView 生命周期。一次局部 logger 或 DB 改动不能使剩余启动安全；不在本轮 01 文档复查中扩成跨模块基准工程。

## GPUI 与测量：已有进展和仍缺部分

- `ccr-desktop/src/runtime/mod.rs:32–139` 已支持 `--probe`、成对 profiles/registry 路径和三条 Insights 绝对路径；三条 Insights 路径必须齐备，probe 不能混入它们。
- `ccr-desktop/src/lib.rs:59–93` 分开 probe、App 和 AppWithInsights；fixture 模式不会自动调用 local Insights discovery。`--probe` 仍是输入窗口，不能代表 S0 完整业务启动或最终应用内存。
- `ccr-desktop/src/runtime/ui_state.rs:71–93` 已在 fixture registry 同根建立 `ui-state.db`，使用显式空 migration home 和 `DbAccess::Pool`。这修复了该域的隔离需求，不能推导 Tauri whole-app 或两端同负载基线已经可用。
- `crates/ccr-usage/src/fixtures.rs:23,153,245` 的 schema 19 projection/seed API，以及 `crates/ccr-db/src/database/repositories/usage_repo.rs:909` 的 archive upsert，可用于后续统一生成器，继续由原 owner 持有 SQL。现有 per-domain fixture 不是已采样的 S0/S1/S2。
- 本轮所查 `ccr-desktop/src`、`ccr-desktop/scripts`、Tauri main 与 perf telemetry 未定位到与 OS process-create 对齐的 first-present/action-ready 试次收据。GPUI `open_window` 返回、`activate` 或 quit-hook 完成都不能替代这些指标。
- `ccr-ui/src/utils/perfTelemetry.ts:61–64,176–198,310` 仍是 JS performance clock、PerformanceObserver paint 和 route timing；`ccr-ui/src/main.tsx` 的 React mount 也不是 OS presentation。`ccr-ui/scripts/perf/perf-route-switch.mjs` 是 Playwright/DOM MutationObserver，不能作为 native 启动计时。
- `ccr-ui/scripts/verify-release-window.ps1:54–55,111` 仍是启动、等待、枚举窗口后 force stop，既不隔离全启动链，也不采真实 first-present；本轮没有运行它。

## 可执行恢复顺序与当前判定

1. 由单一产品 owner 接通 Tauri 全启动显式 fixture 路径，涵盖 catalog/app DB/archive DB/legacy home、日志、偏好、认证、SSH/环境及 WebView storage。隔离验证必须证明不回退真实 HOME，不触发 WSL/网络/OAuth 恢复；保留正常产品默认路径。
2. 用既有领域 API 生成两端相同 seed/schema/version 的 S0/S1/S2：S1 为 100 profiles、10,000 sessions、100,000 usage records；S2 为 10,000 候选行及 1,000 log events/s 持续 60 秒。保存 manifest/hash 与逐域核对，不把缺库等同零用量。
3. 在同机同 release 构建接通 external process-create、实际 presentation 和 action/UI commit 的试次/时钟映射；记录 WebView 进程树归属、仪器开销、锁文件/资产/fixture hash、有效 DPI 与实际刷新率。
4. 先收 Tauri，再收 GPUI。按原 `performance.md` 保留热启动 30 次、真正冷启动至少 5 次、S1 稳定 120 秒等全部样本；不放宽预算、不用平均 FPS 替代 frame percentiles。没有合格基线时不计算改善比例。
5. macOS 编译由真实对应机器提供命令、SHA/锁、host/toolchain、退出码与日志；在此之前 AC1.5 不关闭。尺寸/DPI 原生矩阵的既有 `user-deferred` 不因本报告解除。

当前原生性能全部 `not_run`，GPUI/Tauri 相对指标 `not_comparable`。无新性能 JSON、CPU/内存样本、百分位数或截图；没有把历史 2026-10-17 的环境/产物 hash 当成本轮 current build。01 保持 `in_progress`，不归档。因只改任务文档，不重复 09 已执行的 desktop/full-CI 门禁；本轮文档检查与独立复查结果另行记录。

## 本轮 implement 文档检查

- `git diff --check`：退出 0。
- 四份当前任务 Markdown 的显式尾随空白、task.json `in_progress` 与 PRD AC1.5 未勾选断言：退出 0；包括尚未入 index 的新收据。
- `python ./.trellis/scripts/task.py validate .trellis/research/archive/2026-11/10-17-gpui-01-foundation`：退出 0，两个 manifest 的各 3 个引用均有效。既有 `dependency-governance.md` 超过 32,768 bytes 注入上限的警告保留；不是新文件缺失，也不是 native/性能通过。
- `just desktop-fmt-check`、`just desktop-lint`、`just desktop-test`、`just ci`：本轮文档切片未重跑，不借用 09 的 0 退出码冒称本轮执行。

独立复查终态见 [check-continuation-20261105.md](check-continuation-20261105.md)。已修正带缓存 WSL 探测的精确边界并补全一处源码路径；文档/引用/状态检查通过，不改变原生或性能判定。
