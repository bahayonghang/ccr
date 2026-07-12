# Implementation Plan

## Phase A: Lock Product Decision and Baselines

- [x] 确认默认主题策略：Mocha 为默认值，persisted explicit theme，auto 仅 opt-in。
- [x] 用 release 构建记录 CLI/logger、theme、App construction、first draw 分段基线与实施后 10 次样本。
- [x] 将基线保存到任务 research，不提交临时 probe binary/source。

## Phase B: Typed Profile Detail Model

- [x] 在 `ccr-tui/src/tui/ui.rs` 引入 `DetailKey`、`DetailTone`、`DetailField` 或等价小型 typed model。
- [x] 将通用、Codex、Claude detail builders 从 `Line` 直接构造迁移到 typed fields + shared renderer。
- [x] 添加 Codex `model_reasoning_effort` 提取与 known/unknown/missing 映射。
- [x] 为 model、effort、auth/provider、status、URL、cost 指定显式 tone。
- [x] 保持 token masking 和 raw lower-layer error 规则。

## Phase C: Layout

- [x] wide workspace 改为 46/54，并更新布局单测。
- [x] Focus/Context 去重 name/current/enabled。
- [x] 仅在存在反馈时分配 Status strip。
- [x] 按本地化 label display width 计算 label 列宽，覆盖 compact/standard/wide。
- [x] 更新英文/中文 TestBackend 断言，检查 URL、reasoning effort、Usage note、scrollbar 和 footer。

## Phase D: Startup Path

- [x] 按已确认策略扩展 `TuiConfig` theme 字段与兼容测试。
- [x] 让 `Ctrl+T` 保存 theme，保持 tab/selection/pagination/auth/toast/background state。
- [x] `run_tui*` 单次加载 config，并传给 i18n/theme/App。
- [x] 默认路径绕过同步 termbg；保留显式 override/opt-in 行为。
- [x] 在进入 alternate screen 前完成轻量 App 构造，进入后立即 first draw。
- [x] 为 terminal capability error、无效/缺失 theme 和持久化状态保留补测试。

## Phase E: Logger Hygiene

- [x] 单独测量 `init_file_only_logger` 热路径并确认其低于启动预算。
- [x] 修复 `ccr.log.YYYY-MM-DD` retention 匹配并添加临时目录测试。
- [x] 测量未超过启动预算，因此保持同步简单实现，不扩大为异步 cleanup。

## Validation

按窄到宽执行：

```powershell
cargo test -p ccr-config -- --test-threads=1
cargo test -p ccr-tui -- --test-threads=1
cargo test -p ccr -- --test-threads=1
just fmt-check
just lint-strict
git diff --check
```

若 logger 改动进入实现，再追加：

```powershell
cargo test -p ccr-core -- --test-threads=1
```

最终使用 release binary 在交互终端完成：

- [x] 10 次启动分段对比，默认 theme 阶段无约 100ms 固定等待。
- [x] 140x30 TestBackend 宽屏验证：详情宽于列表、无空 Status、关键字段高亮。
- [x] 80x20 和 100x30 中英文 TestBackend 验证：无重叠、无关键字段被硬截断。
- [x] `Ctrl+T` / `Ctrl+L` 持久化完整配置，且不重置 TUI 工作状态。

## Risk and Rollback Points

- Typed renderer 完成后先跑 `ccr-tui` tests，再删除旧 heuristic helper。
- theme config 和 construction order 分成独立提交边界，便于回退启动行为而保留 UI 改进。
- 不在同一改动中异步化全部 profile/runtime loading；当前证据不支持扩大状态机。

## Verification Evidence (2026-07-12)

- `just fmt-check`：通过。
- `just lint-strict`：通过。
- `cargo test -p ccr-config -- --test-threads=1`：63 passed。
- `cargo test -p ccr-tui -- --test-threads=1`：202 passed。
- `cargo test -p ccr-core -- --test-threads=1`：66 passed，2 ignored。
- `cargo test -p ccr -- --test-threads=1`：通过。
- `cargo test -p ccr-usage -- --test-threads=1`：33 passed。
- `git diff --check -- . ':(exclude)TODO.md'` 与任务目录 trailing-whitespace 检查通过。
- 全局 `git diff --check` 仅因既有用户改动 `TODO.md:2-3` 失败；本任务保持该文件不变，因此 PRD 中包含全局命令的最后一项暂不勾选。
