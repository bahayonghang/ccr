# 集成检查收据（trellis-check，2026-11-07）

范围：`ccr-desktop/` 全部未提交改动（材质、侧栏/胶囊、LensBar、Overview 重排、jobs/mcp/insights/monitoring/platform_navigation 触点、`tests/native_shell.rs`）。工具链：`rustup` 1.98.0（`desktop.py` 解析）。所有命令在 `ccr-desktop/` 下运行。

## 门禁收据（检查修复后的最终工作树）

| 命令 | 退出码 | 结果 | 时间 (UTC-5) |
| --- | --- | --- | --- |
| `just desktop-fmt-check` | 0 | 无格式差异 | 2026-11-07T11:07:51 起 |
| `just desktop-lint`（`clippy --all-targets -D warnings`） | 0 | 无告警 | 同上，紧随 fmt |
| `just desktop-test` | 0 | Python unittest 16 OK；accesskit 补丁 3 passed；lib 245 passed / 4 ignored（含 1 MiB 栈守卫 `environment_views_fit_native_windows_stack_with_isolated_locks`）；main 0；native_shell 10 passed / 1 ignored；native_ui 2 passed；runtime_shutdown 2 passed / 1 ignored | 11:10:09 – 11:12:06 |
| `just desktop-test-ui` | 0 | native_shell 10 passed / 1 ignored；native_ui 2 passed | 11:12:06 之后 |

门禁之后只改了 `DESIGN.md` 中一条归档研究路径（文档，不影响编译与测试）。

## 不稳定测试诊断

### 目标：`features::resources::claude_files::tests::shell_claude_resources_write_confirm_conflict_and_keep_private_fields`

原失败日志已丢失，无法确认失败点。复现尝试（全部为当前工作树构建的 lib 测试二进制）：

| 方式 | 次数 | 失败 |
| --- | --- | --- |
| 完整 lib 套件（默认并行，含 1 MiB 栈守卫子进程） | 20（run1 + full_1…19） + 最终门禁 1 | 0 |
| 单测隔离，默认栈 | 20 | 0 |
| 单测隔离，`RUST_MIN_STACK=1048576` + 毒化 `CCR_LOCK_DIR`（与守卫子进程相同条件） | 20 | 0 |
| 8 路并发（4×claude_files + 4×mcp）× 6 轮 | 24 | 0 |

单测耗时约 4.9 s（并发下约 6.3 s），每个 `wait` 的 10 s 墙钟截止仍有余量。

是否可能由本次改动引起（逐项排查）：

- `Shell::new` 新增 `LensBar` 实体与两次同步 `embed_lens`：无任务、无定时器、无焦点调用、无 I/O。
- `LensBar` 只在 Dashboard（Overview / Monitoring）内渲染；该测试第一帧后立即 `nav-claude-code → tool-files`，之后不再渲染 LensBar。
- 本次检查新增的 `on_next_frame` 焦点恢复仅在胶囊内键盘激活时安排，并以“外壳仍处于该目的地与 lens”为前提；该测试不走此路径。
- 新增的场景光斑、阴影与材质均为静态常量，没有动画或计时器；侧栏 `group_hover` 只依赖指针位置。
- 布局变化（场景边距 12→20、上下文胶囊行高度）是确定性的，若影响可见性会在每次运行都失败，而 41 次运行全部通过。
- 进程级共享状态：`ccr-desktop/src` 与 `ccr-app/src` 中没有 `set_var`；仅有 `benchmark::s2::USED` 与 `OWNED_PROCESSES` 两个全局，均与本测试无关。

结论：在 ≥5 次完整运行（实为 21 次）和 64 次单测/并发运行中均未复现。本次改动没有引入影响该测试路径的计时、焦点或共享状态。该测试依赖 10 s 墙钟截止与真实文件 I/O，满载机器上的偶发超时是最可能的历史原因，但没有失败日志，无法证实。保持测试与截止不变，不重试、不放宽。

### 运行中新发现：`features::mcp::tests::mcp_native_actions_write_confirm_and_keep_conflicting_drafts`

完整套件第 2 次运行（`full_2`，未开 `RUST_BACKTRACE`）失败一次：`src/features/mcp/tests.rs:62:10` `called Option::unwrap() on a None value`，即 `patch_id` 没有找到 “MCP JSON patch” 节点（高级 JSON 补丁区未展开）。随后：隔离 30 次、8 路并发 24 次、开启回溯的完整套件 14 次（full_6…19）加最终门禁全部通过，未再复现，因此无法确定是哪一处 `patch_id` 调用失败。

分析：`wait_for_editor` 在保存按钮可用后，只在找到 `mcp-common-form` 时点击“高级”，未展开时不报错；若点击未切换（例如 `form.patch` 返回错误写入 message），后续 `patch_id` 才会 panic。这一测试结构早于本次改动；本次对 MCP 只把目标选择器容器换成共享 `page_header::capsule`（样式），以及外壳上下文胶囊行与场景边距的确定性布局变化，均无计时成分。是否早于本次改动未经基线验证：用共享 target 的 HEAD worktree 构建因工具链解析不一致（1.99 vs 1.98）失败，已移除该 worktree，未影响主 target（之后门禁全部增量通过）。记录为未解决的偶发失败，不修改、不放宽断言。

## 本次检查修复

1. **键盘焦点回归（已修复）**：嵌入式 LensBar 随 lens 切换从 Overview 移到 Monitoring，Kit `Button` 以元素路径保存焦点，切换后焦点丢失（新增探针断言 `tool-monitoring.focused() == Some(false)`）。`LensBar` 现持有稳定焦点锚点，键盘激活后下一帧把焦点还给激活项；回调前校验外壳仍处于该目的地与 lens，离页后不抢焦点。`tests/native_shell.rs` 增加 Tab → Enter → Shift+Tab → Space 键盘路径断言（`simulate_next_frame` 驱动下一帧）。
2. `theme::tabular_figures()` 改为静态一次分配，渲染路径只克隆 Arc。
3. `README.md` 删除“实色降级”旧说法；`DESIGN.md` 记录 LensBar 焦点恢复，并修正归档研究路径。

## 未运行

原生像素、light/dark 原生截图、UIA/读屏、IME、macOS/Linux、DPI/尺寸矩阵（用户延期）未在本次检查中运行，不记通过。

## 主会话复验（2026-11-07，修复 release 编译错误与日均小数位之后）
- `just --justfile ccr-desktop/justfile desktop-ci` → exit 0（fmt-check、check、lint、test：lib 245 passed / 4 ignored、native_shell 10 passed / 1 ignored；release build Finished）。
- `just --justfile ccr-desktop/justfile desktop-test-ui` → exit 0（10 passed / 1 ignored；2 passed）。
