# Implement: 首页 Insights 界面设计定稿

## Checklist

### 1. 读取约束

- [x] 读 `ccr-ui/DESIGN.md`、`ccr-ui/PRODUCT.md`、`ccr-ui/AGENTS.md` Design Context、`.trellis/spec/ccr-ui/frontend/theme-token-contracts.md`、`dashboard-presentation-contracts.md`、`usage-chart-stability-contracts.md`。
- [x] 读 backend 子任务 `design.md` §5，列出可用字段。

### 2. 首屏图表越界排查（只读）

- [x] 按 design §5 复现并定位；结论写 `research/chart-overflow.md`。

### 3. 颜色与梯度

- [x] 加载 `dataviz` 技能。为 4 个新来源选身份色（亮/暗各一），与现有 6 个平台色做色差校验。
- [x] 按 design Q1 选定热力图梯度，Q2 定合并阈值 N。
- [x] 校验器输出写 `research/palette-validation.md`。

### 4. 构图与组件形式

- [x] 加载 `frontend-design` 技能，决定 design Q3-Q5。
- [x] 加载 `artifact-design` 与 `design` 技能，产出设计画布：暗色全页、亮色全页、四种空态、1280px 布局。发布 Artifact，把链接写入任务 notes。
- [x] 用户在画布上确认后继续（停止点：等待用户确认，不自行定稿）。

### 5. 写入设计源

- [x] `ccr-ui/DESIGN.md`：Platform Identity 段补 4 色；Components 段新增 Insights 签名组件条目（prd AC2）；Do/Don't 段补 Insights 相关条目。
- [x] `ccr-ui/src/styles/tokens.css`：layer 1 新增来源色（主色、`-rgb`、`-surface`、`-border`、`-text`，与现有 6 平台同结构）与热力图梯度 token；亮/暗两套。
- [x] `ccr-ui/.impeccable/design.json` 同步。
- [x] token 治理登记：生成 `research/token-names-before.txt` / `token-names-after.txt`，在 `.trellis/spec/ccr-ui/frontend/theme-token-contracts.md` 追加登记段。

### 6. 验证

- [x] `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/theme/`（13 文件 102 例通过）
- [ ] `cd ccr-ui && bun run test`（724/725 通过；失败 1 例为 `tests/usage/usage-date-window.smoke.test.ts` all_time 跨度，本机时区 America/Chicago 在 2026-01-01 与 2026-04-10 之间有夏令时切换，得 99 而非 100；该测试与 `src/views/usage/dateWindow.ts` 本任务未改）
- [x] `rg -o '[0-9]+px' -g '*.css' ccr-ui/src/styles | wc -l` 不超过已登记豁免数（theme-token-contracts §4）。（136 ≤ 152；本任务新增 0 处 px）

## Review Gates

- 步骤 4 末尾：用户确认设计画布。
- 步骤 5 完成后：dispatch `frontend-quality-reviewer` 检查 token 分层与 DESIGN.md 一致性。

## Rollback Points

- `DESIGN.md`、`tokens.css`、`design.json`、规格登记段在同一提交中，撤回时一起撤回。
