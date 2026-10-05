# Profile registry 契约与平台色 token

父任务：`08-26-profile-design-language`

## Goal

建立统一 Profile 页面所需的基础契约与 token：展示投影类型、供应商 canonical key、凭据剥离、`ProfilePresentation`、`ProfileEditorAdapter` 类型，以及六平台的平台色四角色。本子任务不产生可见 UI 变化，只为后续三个子任务提供数据来源与类型边界。

契约的权威定义在父任务 `design.md` 的「契约一」至「契约五」。本文件只补实例层面的取值与决策。

## Requirements

- R1：新增 `src/configs/profileDisplayRecord.ts`，导出 `ProfileDisplayRecord` 类型与 `toVendorKey(baseUrl)` 供应商 canonical key 函数。算法按父任务 `design.md`「供应商 canonical key」六条规则实现。
- R2：新增 `src/configs/profileCredentials.ts`，导出 `stripCredentials(record, secretKeys)`。三平台各自声明 `secretKeys`：claude `['auth_token']`、codex `['auth_token']`、grok `[]`。
- R3：新增 `src/configs/profilePresentation.ts`，导出 `ProfileFieldSlot`、`ProfilePresentation` 类型与 claude / codex / grok / antigravity 四份实例。每份实例的 `project()` 入参是该平台 typed DTO（`ClaudeProfile` / `CodexProfile` / `GrokProfileDto`），不是 `ProfileRecord`。
- R4：新增 `src/configs/profileEditorAdapter.ts`，只导出 `ProfileEditorFieldKind`、`ProfileEditorFieldSpec`、`ProfileEditorSection`、`ProfileWriteOutcome`、`ProfileEditorAdapter` 五个类型。实现体归 `08-26-profile-editor`。
- R5：确定 Claude 第四字段槽位的替代字段。成品稿的「最近使用」在 `ClaudeProfile` DTO 中无对应字段，需从已有字段中选取（候选：`provider`、`effort_level`、`account`）。
- R6：Grok 的 `project()` 把 `profile_kind` 输出为 `badges` 中的一项，`base_url_display` 用于 slot 展示，`reasoning_effort` 占 slot3。`base_url_display` 不进入任何写入路径。
- R7：扩展平台色 token，为 claude / codex / grok / antigravity / opencode / gemini 各补 `-surface`、`-border`、`-text` 三角色，明暗主题分别定义。antigravity 另补 `--color-platform-antigravity` 与 `--color-platform-antigravity-rgb`（当前完全不存在）。
- R8：按父任务决策 1 更新 `--color-platform-codex` 为 `#7cab82`、`--color-platform-grok` 为 `#a79bc4`，同步更新对应的 `-rgb`。
- R9：明色取值一律写为 hex 字面量，不使用 `color-mix()`，使对比度可由 CSS 文本直接解析计算。
- R10：按 `theme-token-contracts.md` 完成 token 名称治理：统计新增名称数量、更新该文档冻结段的名称增量叙述、确认新名称归层一 `tokens.css` 明暗块（不进 `@theme` / `@theme inline`、不需要 bridge 映射），并运行该文档要求的主题测试。
- R11：列出所有消费 `--color-platform-*` 的位置，逐一确认色值变更后的视觉可接受性。
- R12：全部测试位于 `ccr-ui/tests/*.smoke.test.ts(x)`，不放 `src/**/__tests__/`。

## Acceptance Criteria

- [x] AC1（R3）：`ProfilePresentation` 类型与四份实例（claude / codex / grok / antigravity）存在且通过 `bun run type-check`。
- [x] AC2（R3）：结构断言——四份实例的 `fieldSlots` 长度为 4，`glyph` 为单字符，`configFile` 与 `configPathKey` 非空。
- [x] AC3（R3、R1）：三平台各一条 `project()` 投影测试，用该平台真实 typed DTO 夹具输入，断言 `slots` 四项、`vendorKey`、`authKey`、`badges`、`searchText` 的取值，且断言取值来源是 DTO 上 `ProfileRecord` 不包含的字段（Claude 用 R5 选定字段，Codex 用 `wire_api`，Grok 用 `reasoning_effort`）。
- [x] AC4（R6）：Grok 的 `project()` 输出的 `badges` 含一项 `profile_kind`；`searchText` 与 `slots` 均不含 `base_url_display` 以外的 URL 形式，且 `base_url_display` 不出现在任何写入路径。
- [x] AC5（R2）：`stripCredentials` 测试——以随机 sentinel 作为 `auth_token` 输入 Claude 与 Codex 的 DTO 夹具，断言返回对象的任意深度均不含该 sentinel；grok 夹具原样返回。
- [x] AC6（R1）：`toVendorKey` 等价类测试，覆盖大小写、默认端口与显式端口、userinfo、IPv6、尾点、无协议输入、空值、非法输入八类，每类至少一条用例。
- [x] AC7（R5）：Claude slot3 字段的选定结论与依据（三个候选的实测填充率、敏感性判断）写入本任务 `notes.md`。
- [x] AC8（R7、R8）：六个平台的 dot / rgb / surface / border / text 五个 token 在明暗两套主题下都有取值，测试解析 `tokens.css` 断言无缺项。
- [x] AC9（R9）：明色主题下每个平台 `-text` 对 `-surface` 的对比度不低于 4.5:1，由 hex 直接计算断言，测试中不出现 `color-mix()` 求值。
- [x] AC10（R10）：`theme-token-contracts.md` 的冻结段已记录本次名称增量与数量；`bunx vitest run --config vitest.smoke.config.ts tests/theme-switch.smoke.test.tsx tests/token-single-point.smoke.test.tsx` 通过。
- [x] AC11（R11）：`rg -n "color-platform-" ccr-ui/src` 的全部消费点清单写入 `notes.md`，每项标注确认结论。
- [x] AC12（R12）：新增测试文件为 `ccr-ui/tests/profile-presentation.smoke.test.ts`、`tests/profile-credentials.smoke.test.ts`、`tests/profiles-vendor-key.smoke.test.ts`、`tests/platform-color-tokens.smoke.test.ts`，`bun run test:smoke` 能发现并执行；`rg -l "smoke.test" ccr-ui/src` 为空。
- [x] AC13（R4）：`profileEditorAdapter.ts` 只含类型导出，`rg -n "export (const|function)" ccr-ui/src/configs/profileEditorAdapter.ts` 结果为空。
- [x] AC14（R1-R12）：`just frontend-check-quick` 通过，且 `git diff --stat` 中不含 `ccr-ui/src/features/` 与 `ccr-ui/src/components/` 的改动。

## Constraints

- 不修改 `src/configs/profiles.ts` 的已有字段与导出。
- 不新增 Tauri 命令，不改后端，不改后端凭据序列化。
- 明色主题的平台色需自行推导，设计稿只提供暗色值。推导后必须验证对比度，不得直接取暗色值。
- 不实现任何 `ProfileEditorAdapter` 实例；本任务只交付类型。
- `project()` 必须是纯函数，不做 IO，不读 store。
- R5 的字段选择需说明理由，写入 `notes.md`。

## Notes

- 设计稿的平台色四件套见 `../08-26-profile-design-language/research/design-source.md` 的「平台元数据表」一节。
- 现有 token 定义在 `ccr-ui/src/styles/tokens.css` 的「平台专属色」注释块。
- 本任务是审阅报告 TPR-01、TPR-04、TPR-09、TPR-11、TPR-13 的落地入口。
