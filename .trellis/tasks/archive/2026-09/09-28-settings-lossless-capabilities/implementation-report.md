# T08 实施报告

日期：2026-09-28。基线 commit：`34d8a85e0e48b793733835e0304c8ed33940fcee`。

T08 代码与相关自动化验证已完成，等待主会话安排独立检查。正式前端 lint 仍有既有失败；Web 视觉与原生桌面验收未执行。本报告不将任务标记完成，不提交或归档。

## 实现范围

- `ccr-ui/src/configs/settings-types.ts` 与 `settings.ts`：引入 `SettingsSnapshot { values, source, managedLocks? }`、`SettingsSaveInput`、`SettingsSaveResult`、raw-source callbacks 和字段 unset 描述。
- `settings-patch.ts`、`settings-codex-map.ts`、`settings-{claude,codex,grok,opencode}.ts`：按 dirtyKeys 构造请求；保存提交的嵌套对象时保留 snapshot 的未编辑字段。Codex notifications 数组保留原值并可见；OpenCode autoupdate 保留 `notify`，仅修改 TUI 时不写 runtime 文件。无脏字段时不发 mutation。
- `ccr-ui/src/features/platform/settings-model.ts`、`settings/BaseSettings.tsx`、`SettingsFieldControl.tsx`、`SettingsFormBody.tsx`、`SettingsSource.tsx`：消费快照和领域状态；展示锁原因、未知枚举和 Grok unset；恢复源码入口、Local-only probe、冲突恢复与 Profiles 链接。平台薄壳继续复用 BaseSettings。
- `ccr-ui/src/features/platform/editor/`：承接原有 editor composite、locale、CSP helper、extensions、theme 和 CSS。`features/editor/` 原 TS/TSX 入口保留 re-export。未扩大 feature import 豁免。原始内容留在编辑器局部状态；成功保存退出源码模式，再刷新 typed form。读取/确认回调保持稳定，避免重渲染重新确认或重新读取并覆盖 raw 草稿。
- `ccr-ui/src-tauri/src/commands/grok.rs`：只新增测试 helper 与 no-backup inventory fixture，未修改 Grok 生产逻辑。fixture 使用临时目录和合成内容，配置 `CCR_BACKUP_DIR`、`CCR_LOCK_DIR`，比较所有非目标配置、非锁文件的路径与字节。
- 行为测试：新增 `tests/platforms/settings-lossless.smoke.test.ts`、`settings-capabilities.smoke.test.tsx`；调整既有 shared Settings、platform surface、hardcode guard 和 CodeMirror 测试。
- 规范同步：`platform-surface-contracts.md`、`raw-config-editor-contracts.md`、`grok-settings-contracts.md`。

## 需求与证据

| 条目 | 实现和已验证行为 | 边界 |
| --- | --- | --- |
| R1 / AC1 | 真实 mapper 与 domain wrapper 经 mock transport 执行 read → 改一个字段 → save → reread；Codex 只修改 model 时请求只有 model，notifications 数组、未知枚举和扩展值保持；OpenCode 只修改其他字段时请求不携带 autoupdate。嵌套修改保留 sibling 值，snapshot 不被修改。 | mock transport 模拟各接口的对象合并语义；没有调用真实用户设置。 |
| R2 / AC2 | 路由挂载验证 Grok managed inputs disabled、原因和 Profiles 恢复链接；未知枚举仍显示当前值；只提交用户修改的字段。Grok unset 与 false 分开；非整数/超范围禁止保存。typed conflict/managed_locked 保留草稿并显示反馈。 | 后端仍承担锁、环境和取值校验。 |
| R3 / AC3 | Claude/Codex/Grok Settings 可打开实际 CodeMirror 编辑器；先明文确认，再读取 raw/layers。CAS 保存传原 token；conflict 保留内容并仅允许重载；invalid 显示错误标记；成功关闭并重载 typed form。WSL/SSH 的 Grok 路由不调用本地设置接口；支持 remote typed settings 的 Claude 仍禁止 remote raw 请求；拒绝明文确认不读文件。 | Web 与生产 WebView 交互/视觉待集中验收。 |
| R3 / AC3 无备份 | 页面展示 Grok 无备份与 policy layer 提示。Grok fixture 先建立已有 config 和两处备份哨兵，再验证 typed 成功、typed invalid、raw 成功、raw invalid、raw stale。成功只改目标 config；失败保留 config 与备份清单、哨兵字节。 | 测试在 Windows 临时目录执行。未操作真实账户。 |

## 已执行检查

命令以 repository root 为目录，带 `bun` 的前端命令以 `ccr-ui/` 为目录。

| 命令 | 结果 |
| --- | --- |
| `bun run test:smoke -- tests/platforms tests/configs tests/ui/code-source-editor.smoke.test.tsx tests/quality/hardcode-px-rgba.smoke.test.ts tests/api/api-facade-boundary.smoke.test.ts` | **32 files / 97 tests passed**，exit 0。包含真实 mapper、路由、实际 CodeMirror 与既有回归。 |
| `bun run type-check` | 通过，exit 0。 |
| `bun run build` | 通过，exit 0。保留 Vite configLoader、`@tailwind` 和大 chunk 提示；源码编辑器为按需加载 chunk。 |
| `bun run lint:ci` | **失败**，两个既有 `.tmp-*.mjs` 共 5 条 no-console。未删除、忽略或修改这些文件。 |
| `bun run lint:style` | 通过。正式 lint 在 ESLint 处停止，此项独立执行。 |
| `bun run check:style-lines` | 通过。 |
| `bun run check:arch-boundaries` | 4 组违规夹具均按预期拒绝。 |
| `bun run check:cycles` | 731 个文件，无循环。 |
| `bun run test:i18n` | 24 项检查通过，key-leak 自检通过。 |
| `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::grok::tests -- --test-threads=1` | **16 passed / 514 filtered out**。 |
| `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::grok::tests::settings_and_raw_saves_preserve_the_configured_backup_inventory -- --exact --test-threads=1` | 在 fixture 改为已有 config 后，**1 passed / 527 filtered out**；附加 integration suite 0 matched。 |
| `cargo fmt --manifest-path ccr-ui/src-tauri/Cargo.toml --all --check` | 首次通过。后续共享树检查发现另一个 owner 在 `crates/ccr-core/src/core/guarded_write.rs:675` 新增断言的多行格式差异；已回报主会话，未修改该文件。 |
| `rustfmt --check --edition 2024 ccr-ui/src-tauri/src/commands/grok.rs` | 当前 T08 Rust 文件通过，exit 0。 |
| `git diff --check` | 通过；Git 提示现有 LF/CRLF 转换，不属于 whitespace error。 |

最初扩大 smoke 范围时，新 CodeMirror CSP 测试使用 `.cm-editor` 文本选择器，未找到运行时样式。已根据实际 CodeMirror 样式修正为 `.cm-scroller`，仍断言 nonce 和可读 `style.sheet`。随后该文件 2 项与完整相关集合 97 项均通过。没有降低 CSP 断言或替换真实编辑器。

正式 lint 原始定位：

```text
ccr-ui/.tmp-desktop-probe.mjs:20:46  no-console
ccr-ui/.tmp-desktop-probe.mjs:24:1   no-console
ccr-ui/.tmp-desktop-probe.mjs:92:1   no-console
ccr-ui/.tmp-insights-visual.mjs:236:3 no-console
ccr-ui/.tmp-insights-visual.mjs:381:1 no-console
5 errors, 0 warnings; exit 1
```

## 交接与未验证边界

- T09 的 Settings 草稿/refetch/环境身份工作继续基于 `SettingsSnapshot` 和 `SettingsSaveInput`。当前 T08 没有声称后台 refetch 的 dirty draft 保留问题已经解决。
- 无 IPC DTO 或 registry 修改，未重新生成 bindings。Tauri 测试已实际编译当前共享 crates；未运行全仓 `just ci`、完整 `just ui-check` 或独立全量 Tauri clippy。
- 本轮未执行浏览器操作、截图、生产 WebView CSP 原生检查、WSL/SSH 实机或真实用户文件保存。DOM/nonce fixture 通过不等于原生视觉验收。
- `bun run build` 的 `icons:ensure` prehook 会生成全平台图标。主会话已接管对生成副作用的精确恢复；本 implementer 未回退图标。后续构建按主会话要求仅在该命令设置 `CCR_SKIP_ICON_GENERATION=1`。
- 共享工作区中的 T01、T05 和用户改动均保留；任务状态、父任务 ledger、独立检查和最终验收由主会话处理。

本次续接使用 `trellis-continue`、`trellis-before-dev`、`trellis-check` 和 `ccr-gate-recovery` 的本地规范与检查流程；未再次派发代理。
