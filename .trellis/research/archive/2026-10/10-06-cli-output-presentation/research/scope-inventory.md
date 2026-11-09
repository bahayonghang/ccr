# 文件范围与迁移责任

基线：`9fafc1116bc7d54004ab3b224b33c2f08cae6fcb`。数量为源码引用，包含测试或注释。以下文件是调查范围，不是强制编辑清单。

C1：`crates/ccr-core/src/core/logging.rs`、新增 CLI feedback helper 和注册、共享输出测试。C2/C3 复用 C1，不重复定义渲染策略。

| 文件 | 引用数 | 责任 | 实施处置 |
| --- | ---: | --- | --- |
| `crates/ccr-cli/src/cli/dispatch.rs` | 11 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/claude/auth/current.rs` | 20 | C2 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/claude/auth/delete.rs` | 10 | C2 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/claude/auth/list.rs` | 9 | C2 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/claude/auth/off.rs` | 4 | C2 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/claude/auth/save.rs` | 7 | C2 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/claude/auth/switch.rs` | 6 | C2 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/claude/profile.rs` | 7 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/codex/auth/current.rs` | 27 | C2 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/codex/auth/delete.rs` | 11 | C2 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/codex/auth/export.rs` | 11 | C2 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/codex/auth/import.rs` | 27 | C2 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/codex/auth/list.rs` | 18 | C2 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/codex/auth/rename.rs` | 2 | C2 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/codex/auth/repair.rs` | 5 | C2 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/codex/auth/save.rs` | 12 | C2 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/codex/auth/switch.rs` | 13 | C2 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/codex/auth/sync.rs` | 10 | C2 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/codex/auth/update.rs` | 1 | C2 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/codex/env.rs` | 1 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/codex/fix.rs` | 61 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/codex/profile.rs` | 6 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/codex/quota.rs` | 4 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/codex/sessions.rs` | 4 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/codex/sync_history.rs` | 7 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/common/prompt.rs` | 3 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/common/table.rs` | 1 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/data/budget.rs` | 22 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/data/export.rs` | 15 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/data/history.rs` | 10 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/data/import.rs` | 17 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/data/pricing.rs` | 18 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/data/stats.rs` | 29 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/grok/auth.rs` | 10 | C2 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/grok/profile.rs` | 7 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/lifecycle/clean.rs` | 47 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/lifecycle/clear.rs` | 20 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/lifecycle/init.rs` | 25 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/lifecycle/optimize.rs` | 12 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/platform/current.rs` | 8 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/platform/info.rs` | 7 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/platform/init.rs` | 13 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/platform/list.rs` | 5 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/platform/profile.rs` | 1 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/platform/profile_init.rs` | 6 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/platform/profile_open.rs` | 3 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/platform/switch.rs` | 4 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/profile/add.rs` | 18 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/profile/current.rs` | 18 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/profile/delete.rs` | 22 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/profile/disable.rs` | 5 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/profile/enable.rs` | 3 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/profile/list.rs` | 8 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/profile/switch.rs` | 23 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/project/init.rs` | 14 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/provider_cmd.rs` | 12 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/sessions_cmd.rs` | 20 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/temp_cmd.rs` | 22 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/temp_token.rs` | 15 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/update.rs` | 17 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/commands/update_failure.rs` | 6 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/services/ui_service.rs` | 63 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |
| `crates/ccr-cli/src/sync/commands.rs` | 69 | C3 | 实施时记录语义迁移、共享覆盖或保留理由 |

## 附加与保留范围

- C2 还需复核 `codex/auth/off.rs`（委托 shared auth-off printer，未匹配 ColorOutput），不修改无展示的 Auth mod 文件。
- C3：`commands/doctor_cmd.rs`（独立渲染），`services/doctor_service.rs`（label 公共兼容调查，保留 service 业务）。
- C3：`crates/ccr-codex/src/platforms/codex.rs` 与 `crates/ccr-sync/src/sync/content_selector.rs` 的 shared 输出影响。
- C2/C3：新增 root `tests/commands/output_presentation.rs` 与 commands.rs 注册；必要的既有命令测试。
- C3：docs/reference/commands/clean.md、doctor.md 及 docs/en/reference/commands 对应镜像。
- 主会话：最终 live CLI 展示 spec 与 backend index。
- `commands/common/table.rs` 的引用仅 mask_sensitive，默认保留；不改表格布局。
- `commands/common/prompt.rs` 只复核消息，不修改交互条件和输入行为。
- 无相关文字变化的文件可以不编辑；处置理由必须覆盖源码清单。

## 规模

CLI 引用文件 63；C2 18；C3 45。Auth 目录与额外 Doctor 文件按上述边界补充。
