# T03 实施报告

日期：2026-09-28。任务：`09-28-tauri-config-adapter`。工作区基线：`34d8a85e0e48b793733835e0304c8ed33940fcee`。

## 当前结论

T03 的后端配置适配、共享启用用例、typed config 协议和前端消费已落盘。实际 Rust handler、共享 enable 与 7 项 ConfigsView 页面动作的独立复验通过。configs/API 最终汇总 26 files/90 passed；type-check、Tauri strict clippy、just fmt-check 和 git diff --check 通过。正式 lint 仍被两个受保护用户脚本的 5 个 no-console 错误阻断。完整 bindings guard、父任务集成和原生界面验收仍开放。

本报告记录实施者完成的修改和当前证据。独立检查最终结论由检查者维护于 [check-report.md](./check-report.md)；早期发现和复审过程见 [check-checkpoint.md](./check-checkpoint.md)。本报告不修改任务状态、PRD 验收勾选或父任务 ledger。

## 实施结果

### 共享 application 与直接 repository 适配

- `profile_lifecycle.rs` 新增 `enable_profile(ApplyProfileRequest)`。启用策略和平台激活在同一个 execute/WriteJournal 内完成，沿用 T02 outcome、重放和补偿。
- 最小公开既有 `operation_lock(Platform)`。Tauri update-disable/delete 在同一个同步 worker 内持锁、重新解析实际 current、校验后进入 T01 mutation。持 guard 时禁止递归调用 apply/enable/update_profile。
- generic config 的 switch 已移除永久失败的 legacy switch 调用。enable 显式传 true 并使用共享 enable 用例。
- add/update/duplicate/delete 使用 T01 repository/service；rename 使用 T02 update_profile。未增加一套文件写入、备份、CAS、ACL 或 journal 算法。
- list 和活动配置保护使用 Claude 既有 current resolver，覆盖 file marker 无效时的 registry 回退。查询保持纯读，未自动修复 marker。

### 平台和协议决策

- generic `/configs` 保留 Claude 兼容范围。`ConfigPlatform` 仅有 Claude；Codex/Grok 专属 profiles 页面保持各自专属命令。
- 后端缺失 platform 返回包含 `platform_required` 的错误；其他值返回 `config_platform_unsupported`。不根据 registry 顺序推断平台。只读 `listConfigs()` 保留已知 Claude 默认映射，mutation 要求显式平台。
- `ConfigPatchInput` 是严格 object DTO：省略保留，可选字段 null 删除，enabled 只允许 boolean；拒绝未知 key、错误类型和数组等非对象。
- `ConfigInfo.version` 返回 T01 快照 token，update 把 `expectedVersion` 传入 CAS。保留未编辑字段、未知 TOML、datetime、凭据和非活动 marker。
- `ConfigMutationResult` 返回 platform/name 与可选 `ProfileOutcome`，不返回原始 profile 秘密。patch DTO 不派生 Debug；list 保持脱敏。
- command IDs 保留。`update_config` 从 legacy config_extended 迁入 typed config；typed/exact 277→278，base/Windows 340/348。risk、authorization、confirmation、audit 规则保留；module concurrency gate 随 domain 迁移，业务一致性仍由 application/resource/leaf 锁保证。

### 前端和草稿

- domain/generated 客户端传显式 Claude；tauri.ts 保留兼容 re-export。
- ConfigsView → ConfigList → ConfigCard 提供可见 Enable 动作；disabled 行经确认发送 enable=true。历史 current 且 disabled 的行也可启用。
- hook 消费 ProfileOutcome：提交后警告只显示 warning，不重复激活；unchanged/recovery 刷新后显示错误，不显示成功。只在 activation_committed 时更新 current store。
- Edit 仅发送与 baseline 不同的字段和原始 token。修改为空的可选字段发送 null；未改字段省略。
- 草稿保存 values/baseline/version。旧草稿重开后仍携带旧 token；外部更新触发 CAS 拒绝，用户确认 Reload 后才获取新的 baseline/token。
- 每次 load 开始清除保存 capability；失败或目标缺失时禁用 Save，直接 submit 也不调用 update。
- masked token 不填回表单；Add/Edit 全局草稿均排除明文 auth_token。新凭据仅存于组件表单状态。

## 文件范围与责任

| 范围 | 文件 |
| --- | --- |
| 共享应用与 fixture | `crates/ccr-cli/src/application/profile_lifecycle.rs`、`profile_contract.rs` |
| Tauri adapter 与回归 | `ccr-ui/src-tauri/src/commands/config.rs`、`config/adapter.rs`、`config/contract_tests.rs` |
| registry 同源入口 | `ccr-ui/src-tauri/src/commands/handler_registry.rs`；生成诊断和最后生成窗口已交主代理 |
| 前端 API | `ccr-ui/src/api/domains/config.ts`、`src/api/tauri.ts` |
| 配置页 | `features/configs/queries.ts`、`hooks/useConfigsPage.ts`、`lib/configForm.ts`、`ConfigsView.tsx`、`components/{AddConfigModal,EditConfigModal,ConfigList,ConfigCard}.tsx` |
| 前端测试 | configs/API 相关 smoke；前端代码及最后回归现由检查者独占，完整清单以独立报告和最终 diff 为准 |
| 同源产物 | config generated client、ConfigInfo 及新增 ConfigPlatform/ConfigPatchInput/ConfigMutationResult，manifest/capabilities/permissions/inventory；主代理生成，未手写 generated |
| 规范 | profile-application-lifecycle、typed-ipc-bindings、api-facade-boundary 及各自 index |
| 实施证据 | 本任务目录的 backend 日志、snapshot、生成证据和本报告 |

未修改用户真实配置，测试使用临时目录和合成账户。未改受保护 .tmp 脚本或 Insights 任务状态。未提交、推送或归档。

## 已执行检查

| 检查 | 实际结果与证据 |
| --- | --- |
| 首次后端编译 | 失败：私有 TestHome 与 toml 版本类型不匹配。已修；保留 `backend-first.log` |
| 第二次 config 测试 | 12 passed / 1 failed；strict patch 接受空数组。已改 object-map 反序列化，保留 `backend-second.log` |
| 实施者 config 复验 | 17 passed / 0 failed / 1 ignored 子进程 helper；`backend-third.log` |
| 独立 config 复验 | 17 passed / 0 failed / 1 ignored；`check-backend-final.log` |
| 独立共享 enable fixture | 1 passed / 0 failed；`check-enable-final.log` |
| 主代理 inventory 生成 | 1 passed；`root-inventory-path.log`、`root-generation-evidence.json` |
| 主代理 config scoped export | 7 export + 7 原有行为测试通过；`root-config-exports.log` |
| 生成文件核对 | 原 snapshot 257 文件全部保留，新增 3，当前 260；normalizer 返回 0，无内容修复。详见 `root-generation-evidence.json` |
| 前端页面动作与 API | 独立检查者确认 7 项 ConfigsView action 测试通过；configs/API 最终 26 files/90 passed，见 `check-frontend-verified.log` |
| 类型、静态与格式 | 最终 type-check、Tauri strict clippy、官方 just fmt-check、git diff --check 通过；正式 lint callback dependency 已修，仍有受保护脚本的 5 个 no-console，见 `check-lint-final.log` |
| 全量 bindings / 正式 gates | 未在本任务宣告通过，由主代理/T10 统一执行与记录 |

后端 config 命令：

```text
CCR_SKIP_ICON_GENERATION=1
cargo --offline --locked --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::config -- --skip export_bindings
```

测试保留默认并行度。`config_subprocess` 是供父测试启动的 ignored helper；多进程用例实际运行 3 轮、每轮 2 个子进程，验证 desktop handler 与 ConfigService 对同一 profile 的独立字段修改都保留。共享 enable fixture 在 Claude/Codex/Grok 上验证成功、重放和逐写故障补偿。

首次 inventory 生成曾因 Windows OS1224 失败。主代理后来重试未复现，未停止 Vite 或更改环境。原因未查明。已保留错误日志和 257 文件字节快照；不将故障归因于未经证明的进程。

## 验收追溯与剩余边界

| 标准 | 当前证据 | 剩余事项 |
| --- | --- | --- |
| AC1 switch/enable | 实际 handler 的 runtime/current/enabled、独立共享补偿和 7 项可见页面动作测试通过；configs/API 90 项通过 | 原生 WebView 验收 |
| AC2 strict patch/并发 | 后端错误输入、missing/collision/stale、未知字段保留、registry 回退保护、锁内复读和 6 子进程通过；版本绑定草稿实际表单回归通过 | 正式 lint 仍被受保护脚本阻断；type-check 已通过 |
| AC3 registry/bindings | 340/348 与 typed/exact 278 已生成；manifest/DTO scoped export 保留原文件 | 完整 inventory/bindings drift gate、registry/ACL/confirmation 集成结论 |

- 前端正式 lint 已知有受保护 .tmp 脚本的 5 个 no-console 基线错误；没有修改、删除或排除这些文件。以主代理最新正式 gate 为准。
- T09 的 `tests/platforms/query-contract-baseline.smoke.test.tsx` 属于其他任务，本任务未修改。
- `just ci`、native WebView、真实账户、真实用户配置和全平台视觉未验收。隔离 fixture 和 DOM 测试不替代这些验证。
- 本轮收尾仅固化规范和实施证据；独立检查已修复 ConfigsView callback dependency，并修复 T03 Rust 格式。主代理与检查者维护最后汇总和后续结果。
