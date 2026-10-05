# 统一 Profile 页面设计语言 — 执行计划（父任务）

父任务只负责排序、跨子任务门禁与最终集成。具体步骤在各子任务的 `implement.md`。

## 顺序

```
1. 08-26-profile-registry-tokens        （无依赖，必须先完成）
2. 08-26-profile-list-surface  ┐
   08-26-profile-editor        ┘        （并行，均依赖 1）
3. 08-26-profile-rollout                （依赖 1 2）
```

`list-surface` 与 `editor` 并行时的冲突面：两者都会改 `components/profiles/profiles-shared.css`。约定 `editor` 只写 `profile-editor-shell.css`，共享 token 与 chip、按钮等原子类由 `list-surface` 在 `profiles-shared.css` 中先落地，`editor` 引用不重复定义。

## 跨子任务门禁

每个子任务在自身 `check` 通过后，还需满足：

- 不删除任何现有文件（删除统一在 `rollout` 执行）。
- 不改 `src/configs/profiles.ts` 中已有的数据操作字段与导出。
- 不改路由表与 Tauri 命令签名，不改后端凭据序列化。
- 新增文案同时进 `zh-CN` 与 `en-US`，`bun run check:i18n` 通过。
- **测试落位**：新增契约测试全部在 `ccr-ui/tests/*.smoke.test.ts(x)`。检查方式：`rg -l "smoke.test" ccr-ui/src` 结果为空。
- **规格同步**：若某步骤改变了 `profiles-page-contracts.md` / `raw-config-editor-contracts.md` / `theme-token-contracts.md` 中的既有条款，在同一子任务内更新规格并补测试；不得留到 rollout。
- **验收口径不可运行时改写**：AC 无法达成时回到规划修订，不在实施或 rollout 阶段降级。

## 验证命令

各子任务的 focused 命令写在自身 `implement.md`。子任务收尾统一跑：

```bash
just frontend-check-quick
```

最终集成（在 `rollout` 收尾）：

```bash
just ui-check
```

## 集成检查清单（rollout 阶段执行）

前置条件按父任务 `design.md`「视觉与响应式验收条件」一节固定，不逐项重述。

- [ ] 三个平台页面在四种主题组合 × 两个 viewport 下逐一走查，对照 `research/design-source.md` 的结构清单。
- [ ] 骨架顺序核验：Header → Off 横幅（`can_off` 为真时）→ StatStrip → QuickRail → Toolbar → 列表 → Inspector。
- [ ] 900×800 下测量表格容器与 body 的 `scrollWidth` / `clientWidth`，记录数值。
- [ ] 新建与编辑在三平台各走一遍，含条件必填校验、保存、保存并应用。
- [ ] Claude 与 Codex 的 source mode 走一遍：明文警告、冲突、激活冲突、保存后全量刷新。
- [ ] 凭据 sentinel 测试通过（AC5 的六处断言）。
- [ ] Profile 相关文件 grep 硬编码 hex，结果为空。
- [ ] `rg` 确认 `components/profiles/` 无零消费组件残留。
- [ ] 父任务 `prd.md` 的 AC1–AC28 逐条勾选。

## 回滚点

- 每个子任务一个提交，提交信息使用 `feat(ui)` / `refactor(ui)` 前缀并带子任务 slug。
- token 变更（`registry-tokens`）单独成一个提交，与契约结构变更分开，便于单独回滚色值。
- `rollout` 的删除步骤单独成提交，与接线提交分开。
