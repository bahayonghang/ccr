# Profile registry 契约与平台色 token — 执行计划

## 步骤

### 1. 前置调研

- [ ] `rg -n "color-platform-" ccr-ui/src` 列出全部消费点，写入 `notes.md`（R11）。
- [ ] 读 `types/claude.ts` 的 `ClaudeProfile`、`types/codex.ts` 的 `CodexProfile`、`types/generated/grok/GrokProfileDto.ts`，确认 `project()` 需要的字段名与可空性。
- [ ] 确定 R5 的 Claude slot3 字段：统计真实 profiles 中 `provider` / `effort_level` / `account` 的填充率，按 design.md 的规则选定。结论与三个数字写入 `notes.md`。
- [ ] 确认 `theme-token-contracts.md` 冻结段的登记格式与是否要求独立 token-governance 任务；结论写入 `notes.md`。

**审阅点**：R5 结论、token 消费点清单、治理登记方式三项需先记录，再进入编码。若治理规格要求独立任务，停下与用户确认，不擅自跳过。

### 2. 展示投影与凭据剥离

- [ ] 新建 `configs/profileDisplayRecord.ts`：`ProfileDisplayRecord` 类型 + `toVendorKey()`，按父任务 design.md 的六条规则。
- [ ] 新建 `configs/profileCredentials.ts`：`stripCredentials(record, secretKeys)`，深拷贝后删除指定 key。
- [ ] 新建 `ccr-ui/tests/fixtures/profiles.ts`：三平台 typed DTO 夹具，含当前应用、disabled、长描述、多标签、空 baseUrl、同 host 不同 path 六种形态。
- [ ] `tests/profiles-vendor-key.smoke.test.ts`：八类等价类。
- [ ] `tests/profile-credentials.smoke.test.ts`：sentinel 深度扫描。

### 3. Presentation 与 adapter 类型

- [ ] 新建 `configs/profilePresentation.ts`：类型 + claude / codex / grok / antigravity 四份实例，`project()` 按 design.md 的两张表实现。
- [ ] 新建 `configs/profileEditorAdapter.ts`：五个类型导出，无值导出。
- [ ] i18n：新增 `configPathKey`、`fieldSlots[].labelKey`、`badges[].labelKey`、`authLabelKey` 引用的 key，`zh-CN` 与 `en-US` 同步。
- [ ] `tests/profile-presentation.smoke.test.ts`：结构断言 + 三平台投影 + Grok display URL 约束。

### 4. Token 扩展

- [ ] 在 `tokens.css` 暗色块中为六平台补 `-surface` / `-border` / `-text`，取设计稿值（gemini 按规则推导）。
- [ ] 新增 `--color-platform-antigravity` 与 `--color-platform-antigravity-rgb`。
- [ ] 更新 `--color-platform-codex` / `--color-platform-grok` 及其 `-rgb`。
- [ ] 在明色块中按 design.md 的推导规则补齐同样六组三角色，写最终 hex，不用 `color-mix()`。
- [ ] `tests/platform-color-tokens.smoke.test.ts`：齐备性 + 明色对比度 + 无 `color-mix()`。
- [ ] 逐一走查步骤 1 记录的消费点，标注确认结论。

**回滚点**：token 变更单独提交，与步骤 2、3 分开。

### 5. 名称治理登记

- [ ] 统计新增名称数量（预期 20），与 `rg -o '\-\-color-platform-[a-z-]+' ccr-ui/src/styles/tokens.css | sort -u | wc -l` 的前后差值核对。
- [ ] 更新 `.trellis/spec/ccr-ui/frontend/theme-token-contracts.md` 的冻结段名称增量叙述。
- [ ] 运行治理测试：

```bash
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/theme-switch.smoke.test.tsx tests/token-single-point.smoke.test.tsx
```

### 6. 验证

```bash
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/profile-presentation.smoke.test.ts tests/profile-credentials.smoke.test.ts tests/profiles-vendor-key.smoke.test.ts tests/platform-color-tokens.smoke.test.ts
```

```bash
just frontend-check-quick
```

- [ ] `rg -l "smoke.test" ccr-ui/src` 结果为空（测试未落错位置）。
- [ ] `rg -n "export (const|function)" ccr-ui/src/configs/profileEditorAdapter.ts` 结果为空（AC13）。
- [ ] 确认 `git diff --stat` 中不含 `ccr-ui/src/features/` 与 `ccr-ui/src/components/` 的改动。

## 验收对照

完成后逐条勾选 `prd.md` 的 AC1–AC14。

## 风险

- 明色平台色推导可能与首页现有明色观感冲突。优先保证对比度达标，观感差异记入 `notes.md`。
- 若 `theme-token-contracts.md` 的治理条款要求 20 个新名称必须走独立 token-governance 任务，本任务停在步骤 5 并上报，不绕过登记继续。
