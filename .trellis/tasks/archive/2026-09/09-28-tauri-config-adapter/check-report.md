# T03 独立检查报告

日期：2026-09-28。检查者：`/root/check_t03`。任务：`09-28-tauri-config-adapter`。基线：`34d8a85e0e48b793733835e0304c8ed33940fcee`。

## 结论与放行范围

T03 的配置适配、显式平台、共享 enable、严格 patch、版本绑定草稿和页面动作已通过本报告列出的独立验证。可以向 T09 提供已验证的 config typed-client 与页面消费前置契约。

正式前端 lint 仍失败：两个受保护用户脚本有 5 个既有 no-console 错误。完整 inventory/bindings drift gate、父任务集成、原生 WebView 和视觉验证仍开放。本报告不代表 T03 或父任务全部验收完成，不修改 task.json、PRD 勾选、提交或归档状态。

早期问题与失败过程保留于 [check-checkpoint.md](./check-checkpoint.md)。本报告取代其中的待验证结论，不删除原始失败证据。最终 28 个相关源码/规范指纹、两个受保护文件和检查结果见 [check-final-evidence.json](./research/check-final-evidence.json)。

## Findings (fixed)

### F1：patch 接受 enabled null 和空数组

- 文件：`ccr-ui/src-tauri/src/commands/config/adapter.rs:69`；`config/contract_tests.rs:145`。
- 问题：全 optional 的派生 struct 接受空序列；`Option<bool>` 把显式 null 当作省略。未知输入可能变成空 patch。
- 修复：实现者改为 object-map Deserialize，逐项按类型取值；enabled 仅接收 boolean，optional null 和 absent 分别表示清空和保留，剩余未知字段报固定错误。
- 复验：独立 config 17 项通过，包含错误类型、未知 key、非对象、missing/collision、无写入和有效 optional clear。

### F2：活动配置的 registry 回退未受保护

- 文件：`ccr-ui/src-tauri/src/commands/config.rs:160`；`config/adapter.rs:197,252`；`config/contract_tests.rs:277,367`。
- 问题：file current 为 ghost、registry current 为有效 profile 时，列表和 disable/delete 使用 raw marker，可能错误处理实际活动配置。
- 修复：实现者使用既有 Claude resolver；update/delete 在共享 application operation lock 内重新解析 current，然后执行 T01 mutation。查询不修复 marker。
- 复验：实际 handler 回退保护、等待 operation lock 后重读、desktop/service 多进程独立编辑均通过。并发 fixture 共 3 轮、6 个实际子进程；ignored 项为父测试显式启动的 helper。

### F3：旧草稿可能使用新 token 覆盖外部修改

- 文件：`ccr-ui/src/features/configs/components/EditConfigModal.tsx:43,56,80,95,113`；`lib/configForm.ts:35`。
- 问题：恢复旧全量草稿时若更新 baseline/version，会把旧值授权为当前 snapshot 的修改。
- 修复：实现者将 draft values、原 baseline、原 version 绑定存储。重新打开保留原 token；确认 Reload 才丢弃草稿并获取新 snapshot。
- 复验：实际表单编辑、关闭、外部变更、重开、冲突、确认 reload、再保存通过；后端 stale token fixture 验证拒绝且无写入。前后端证据分别执行。

### F4：加载失败后保留旧保存能力

- 文件：`ccr-ui/src/features/configs/components/EditConfigModal.tsx:43,64,113,182`；`tests/configs/edit-config-draft.smoke.test.tsx:92`。
- 问题：同一组件切换到另一个配置且读取失败时，旧 baseline 仍可能用于保存。
- 修复：实现者在每次 load 开始清除 baseline/loadedName；缺失或失败读取禁用保存，submit 再校验 snapshot 与目标名称。
- 复验：失败目标读取和直接 form submit 均不调用 update。

### F5：全局草稿保存明文凭据

- 文件：`ccr-ui/src/features/configs/components/AddConfigModal.tsx:44,53`；`EditConfigModal.tsx:80`。
- 问题：Add/Edit 草稿可把新 auth_token 保存到共享 draft 状态。
- 修复：实现者将凭据保留在组件表单状态，保存及恢复共享草稿时排除明文 auth_token；masked response 不回填凭据。
- 复验：edit draft 凭据排除和 config form/API 相关测试通过；list 脱敏、未编辑凭据保留由实际后端 fixture 验证。

### F6：启用用例没有连接到可见按钮

- 文件：`ccr-ui/src/features/configs/ConfigsView.tsx:27,140`；`components/ConfigList.tsx`；`components/ConfigCard.tsx:47`。
- 问题：存在 enable API/hook，disabled 行仍缺少调用真实 enable 的页面入口。
- 修复：实现者连接 View → List → Card 的 onEnable；disabled 行和历史 current+disabled 行显示 Enable，经过 warning confirmation 后调用共享 enable。
- 复验：检查者新增 `tests/configs/configs-actions.smoke.test.tsx`。7 项真实页面操作经 domain/generated/invokeRuntime 路径，验证 explicit Claude、enable flag、两种确认取消、committed warning、recovery/unchanged。只 mock 底层 Tauri invoke、无关查询与确认决定。持久化由独立 Rust fixture 证明。

### F7：迁移后测试调用和测试初始化不匹配

- 文件：`ccr-ui/tests/api/api-domain-idle.smoke.test.ts`；`tests/configs/edit-config-draft.smoke.test.tsx`；`tests/configs/configs-actions.smoke.test.tsx`。
- 修复者：检查者。getConfig 测试传入显式 Claude；将不可用的 Chai toBeDisabled 改为 DOM disabled 属性断言；新 action fixture 在 beforeEach 等待 setLocale('en-US')，使可访问名称与应用实际 locale 一致。
- 证据：初次 81 passed/2 failed 保留于 `check-frontend-initial.log`。新增 action fixture 未初始化 locale 时的 83 passed/6 failed 保留于 `check-frontend-final.log`。修复后 `check-frontend-verified.log` 为 26 files/90 passed；没有删除失败行为断言。

### F8：hook 依赖和 Rust 格式未通过正式检查

- 文件：`ccr-ui/src/features/configs/ConfigsView.tsx:26`；`ccr-ui/src-tauri/src/commands/config.rs` 及其 adapter/contract_tests；共享 profile lifecycle/contract。
- 修复者：检查者。将 handleEnable 解构为 callback 的直接依赖；使用 rustfmt 处理 T03 拥有的 Rust 文件。没有运行宽泛自动修复。
- 复验：最终 configs/API 90 项、最终 type-check 和官方 just fmt-check 通过。最终正式 lint 不再报告 T03 错误，仍报告 F9 的受保护文件错误。

## Findings (not fixed)

### F9：正式前端 lint 被受保护文件阻断

- `ccr-ui/.tmp-desktop-probe.mjs:20,24,92` 有 3 个 no-console；`ccr-ui/.tmp-insights-visual.mjs:236,381` 有 2 个。
- 未修原因：文件是主代理明确指定的既有用户产物，超出 T03 修改范围。没有删除、排除或修改。最终 SHA256 与父任务 implementation-baseline.json 一致。
- `bun run lint:ci` exit 1。后续 stylelint/style-lines 因 `&&` 未执行。不能将其他通过项记为正式 lint 通过。

### F10：完整生成门禁与跨域集成尚未通过

- 主代理完成 inventory exact 1 项和 config scoped export 7 项、原有行为 7 项；normalizer exit 0。原 snapshot 257 文件均保留，新增 3 个 DTO，见 `root-generation-evidence.json`。
- 检查者未重新生成。HEAD 与当前 manifest 均有 348 个 ID，无增删；risk/authorization/confirmation/audit/platform 五项元数据共 0 处变化，见 `research/check-contract-evidence.json`。
- 未完成项：完整 `tauri-command-inventory-check` / `tauri-bindings-check` drift gate、registry/辅助窗口 ACL 集成及 `just ci`。这些项目由主代理/T10 统一执行，静态元数据比较不替代运行验收。
- 首次生成 OS1224 后主代理未复现；原因未查明。没有把故障归因于未经验证的进程。

### F11：原生界面及其他平台未验证

- 本检查在 Windows 隔离目录中执行 Rust handler/application 和 DOM smoke。没有启动原生 WebView、真实账户或用户配置；没有进行 browser visual、macOS 或 Linux 验收。
- 未修原因：该项为验证范围限制，尚无证据要求修改产品代码。前端 mock IPC 和后端真实 fixture 的组合不构成 native end-to-end 验证。

### F12：T09 的既有失败反例由对应任务处理

- `tests/platforms/query-contract-baseline.smoke.test.tsx` 是主代理创建的 T09 baseline；本次 configs/API 过滤器未纳入该文件。
- 未修原因：跨任务 owner 边界。没有把该文件删除、跳过标记或改写为通过；本报告未声称完整 UI suite 通过。

## 规范及验收追溯

已复读实现者更新的 profile-application-lifecycle、typed-ipc-bindings、api-facade-boundary 及三个 index。enable 的 journal/重放/补偿、guard 后读取 current、strict object patch、显式 Claude、版本绑定草稿、凭据与 outcome 边界和当前实现一致。

| 验收 | 已验证机制和证据 | 结论边界 |
| --- | --- | --- |
| AC1 / R1 | 真实 ConfigsView switch/enable + 真实 Rust handler；runtime/current/enabled 一致；三平台 enable 成功、重放、逐写故障补偿 | 隔离行为证据通过；原生 UI 未验收 |
| AC2 / R2 | strict patch、missing/collision/stale 拒绝、未知字段与凭据保留、锁内 current 保护、6 子进程并发；真实表单草稿版本与失效能力回归 | 本次 scoped backend/frontend 证据通过 |
| AC3 / R3 | 旧请求缺平台/不支持平台无写入、生成 typed client、348 ID 与权限元数据比较、原有 action-scoped confirmation 测试 | scoped contract 通过；完整 drift/ACL 集成门禁开放 |

## Verification

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| Config Rust handlers | PASS：17 passed，1 ignored child helper | `check-backend-final.log` |
| Shared enable/compensation | PASS：1 passed，内部覆盖 Claude/Codex/Grok | `check-enable-final.log` |
| Configs/API smoke | PASS：26 files，90 passed | `check-frontend-verified.log` |
| ConfigsView actions 子集 | PASS：7 passed，包含在 90 项中 | `check-actions-final.log` |
| TypeCheck | PASS：bun run type-check，exit 0 | `check-typecheck-final.log` |
| Tauri strict Clippy | PASS：--bin ccr-desktop -- -D warnings，exit 0 | `check-clippy-initial.log` |
| Format | PASS：just fmt-check，exit 0 | `check-fmt-final.log` |
| Diff whitespace | PASS：git diff --check，exit 0 | `check-diff-final.log` |
| Formal frontend lint | FAIL：5 个受保护文件错误；0 个 T03 错误 | `check-lint-final.log` |
| Complete inventory/bindings guard、just ci | 未执行完成；交主代理/T10 | 不授予通过 |
| Native/visual/macOS/Linux | 未执行 | 不授予通过 |

独立 Rust 命令与退出码保存于 `check-backend-commands.json`。所有独立 Cargo test 均使用 `--skip export_bindings`，保持默认测试并行度。Clippy 输出中的既有 MSVC linker_messages 和 mbx 信息保留在日志中。

检查者没有修改受保护用户文件、任务状态、真实配置或外部服务，没有提交、推送、归档或启动额外 agent。
