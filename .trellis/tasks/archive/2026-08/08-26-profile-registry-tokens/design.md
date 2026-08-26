# Profile registry 契约与平台色 token — 技术设计

契约类型定义在父任务 `design.md`。本文件只写实例取值、推导规则与测试。

## 文件边界

新增：

- `ccr-ui/src/configs/profileDisplayRecord.ts` — `ProfileDisplayRecord` 类型 + `toVendorKey()`
- `ccr-ui/src/configs/profileCredentials.ts` — `stripCredentials()`
- `ccr-ui/src/configs/profilePresentation.ts` — `ProfilePresentation` 类型 + 四份实例
- `ccr-ui/src/configs/profileEditorAdapter.ts` — adapter 相关类型（仅类型）
- `ccr-ui/tests/fixtures/profiles.ts` — 三平台 typed DTO 夹具（父任务 `design.md`「视觉与响应式验收条件」定义的 7 条数据集，本任务先建 typed 部分）
- `ccr-ui/tests/profile-presentation.smoke.test.ts`
- `ccr-ui/tests/profile-credentials.smoke.test.ts`
- `ccr-ui/tests/profiles-vendor-key.smoke.test.ts`
- `ccr-ui/tests/platform-color-tokens.smoke.test.ts`

修改：

- `ccr-ui/src/styles/tokens.css` — 平台色角色扩展与色值更新
- `.trellis/spec/ccr-ui/frontend/theme-token-contracts.md` — 名称增量登记（R10）
- `ccr-ui/src/i18n/locales/zh-CN/*`、`en-US/*` — 新增 label key

不动：`src/configs/profiles.ts`、`src/features/**`、`src/components/**`。

## 依赖方向

`configs/` 不得 import `features/` 或 `components/`（`layering-contracts.md`）。四个新文件只 import `@/types`（含 `types/generated/`）与彼此，不引入其他方向的依赖。

## ProfilePresentation 实例

### fieldSlots

四个槽位在卡片视图是 2×2 网格，在表格视图分别对应 BASE URL 列、col3、col4 与卡片专有的第四项。表格六列宽度按设计稿固定为 `216px | minmax(200px,1fr) | 176px | 104px | 136px | 132px`，其中 col3 取 `fieldSlots[1]`、col4 取 `fieldSlots[2]`。

| 平台        | slot0                    | slot1 (col3) | slot2 (col4) | slot3             |
| ----------- | ------------------------ | ------------ | ------------ | ----------------- |
| claude      | `base_url`               | `model`      | `auth_mode`  | 见 R5 决策        |
| codex       | `base_url`               | `model`      | `auth_mode`  | `wire_api`        |
| grok        | `base_url_display`       | `model`      | `auth_mode`  | `reasoning_effort` |
| antigravity | `base_url`               | `model`      | `auth_mode`  | `region`          |

`slot` 取空串时由渲染层统一显示占位符，presentation 内不写占位文案。

Grok 的 slot0 用 `base_url_display`。该字段是展示专用的安全形式，可能省略 query 与 userinfo；`profiles-page-contracts.md` 明确禁止把它复制进写入路径，因此它只出现在 `ProfileDisplayRecord.slots`，不出现在 adapter 的表单初值中。

### project() 的其余输出

| 字段            | claude                            | codex                                        | grok                                             |
| --------------- | --------------------------------- | -------------------------------------------- | ------------------------------------------------ |
| `vendorKey`     | `toVendorKey(base_url)`           | `toVendorKey(base_url)`                       | `toVendorKey(base_url_display)`                  |
| `authKey`       | `auth_mode`（`subscription`/`api_key`） | `auth_mode`（五值）                       | `profile_kind === 'official' ? 'official' : auth_mode` |
| `badges`        | 空                                | `auth_source` / `openai_login_method` 有值时各一项 | `profile_kind` 一项                          |
| `sortKeys.usageCount` | `usage_count ?? 0`          | `usage_count ?? 0`                            | `0`（DTO 无该字段）                              |
| `searchText`    | `name + description + base_url + tags` 小写拼接 | 同左                          | `name + description + base_url_display + tags` 小写拼接 |

`searchText` 不含任何凭据字段。因为 `project()` 的入参已经过 `stripCredentials`，凭据字段在此处已不存在。

### R5 决策方法

从 `ClaudeProfile` DTO 已有字段中选。评估顺序：

1. `provider` — 供应商标识。区分度高，与统计卡的「供应商去重计数」语义连贯。
2. `effort_level` — 与 grok 的 `reasoning_effort` 同构，但 Claude profile 中该字段常为空。
3. `account` — 账号标识，可能含敏感信息（邮箱等），需确认展示形式后再考虑。

实施时读取真实 `~/.ccr/platforms/claude/profiles.toml` 统计三者填充率，填充率最高且不含个人可识别信息者胜出。结论与三个候选的实测数字写入 `notes.md`。

若三者填充率都低于 50%，选 `provider` 并在 `notes.md` 记录该 slot 大多数情况显示占位符，交由 rollout 阶段整体走查时确认可接受性。

## Token 设计

### 命名

```css
--color-platform-{key}          /* dot */
--color-platform-{key}-rgb      /* dot 的 rgb 分量 */
--color-platform-{key}-surface  /* 新增 */
--color-platform-{key}-border   /* 新增 */
--color-platform-{key}-text     /* 新增 */
```

当前存在：claude / codex / grok / gemini / opencode 的前两项。antigravity 一项都不存在。

新增名称统计：六平台 × 三角色 = 18，加 antigravity 的 dot 与 rgb = **20 个新名称**。这 20 个名称需按 R10 登记进 `theme-token-contracts.md` 的冻结段。

### 归属判定

平台色是随主题切换的语义值，属层一（`tokens.css` 的 `:root` 与 `[data-theme='dark']` 块）。不进 `@theme`（那是全主题恒定的常量），也不进 `@theme inline`（现有平台色 token 未做 Tailwind namespace 映射，本任务不新增映射面）。不需要 bridge。判定结论写入 `notes.md` 并同步到 `theme-token-contracts.md` 的登记条目。

### 暗色取值

直接取设计稿。`surface` = 设计稿 `bg`，`border` = 设计稿 `border`，`text` = 设计稿 `fg`。

| key         | dot       | surface   | border    | text      |
| ----------- | --------- | --------- | --------- | --------- |
| claude      | `#d97757` | `#33231b` | `#6b4028` | `#e8835b` |
| codex       | `#7cab82` | `#1f2a22` | `#3c5442` | `#93bf98` |
| grok        | `#a79bc4` | `#2b2637` | `#4b4463` | `#b3a8cc` |
| antigravity | `#98afc9` | `#212c38` | `#3d4d5e` | `#a8c0d8` |
| opencode    | `#735f52` | `#2a221d` | `#4a3d35` | `#c4b3a3` |

gemini 设计稿未给值，按现有 `--color-platform-gemini: #7d97b6` 用同一推导规则补齐。

`--color-platform-antigravity-rgb` = `152 175 201`（`#98afc9` 的分量）。

**gemini 与 antigravity 并存**：设计稿把 antigravity 列为独立平台并给出四件套，仓库 `config/platformDescriptors.ts` 则用 descriptor id `gemini` 承载 `rootPath: '/antigravity'`，`--color-platform-gemini` 已存在。本任务按设计稿新增 antigravity 的五个 token，与 gemini 的并存，不合并也不改名。同一概念两个名字是既有的命名不一致，记入 `notes.md` 上报，不在本任务解决。

### 明色推导

设计稿只有暗色。明色按统一规则从 dot 色推导，六个平台关系一致：

- `surface`：dot 色与页面底色按 12% 混合后取整为 hex 字面量，目标是浅淡有色底。
- `border`：dot 色与页面底色按 32% 混合后取整为 hex 字面量，比 surface 深。
- `text`：dot 色向暗侧调整，直到对同平台 `surface` 的对比度达到 4.5:1，取整为 hex 字面量。

混合只在推导阶段用工具算出数值，写进 `tokens.css` 的是最终 hex。CSS 中不出现 `color-mix()`。理由：`platform-color-tokens.smoke.test.ts` 直接读 `tokens.css` 文本解析色值计算 WCAG 对比度，`color-mix()` 在文本解析层无法求值。

### 变更溢出

`--color-platform-codex` 与 `--color-platform-grok` 的色值变化会影响所有既有消费点。实施第一步先执行：

```bash
rg -n "color-platform-" ccr-ui/src
```

逐一记录消费位置与确认结论。若某处变更后对比度不达标，在该处补局部修正，不回退 token 值。

## 测试

| 文件                                        | 覆盖                                                                                   |
| ------------------------------------------- | -------------------------------------------------------------------------------------- |
| `tests/profile-presentation.smoke.test.ts`  | AC1 AC2 AC3 AC4：结构断言 + 三平台 `project()` 投影 + Grok badge / display URL 约束     |
| `tests/profile-credentials.smoke.test.ts`   | AC5：sentinel 深度扫描，claude / codex 剥离、grok 原样                                  |
| `tests/profiles-vendor-key.smoke.test.ts`   | AC6：八类等价类                                                                        |
| `tests/platform-color-tokens.smoke.test.ts` | AC8 AC9：解析 `tokens.css`，六平台五 token × 明暗齐备 + 明色对比度 ≥ 4.5:1 + 无 `color-mix()` |

sentinel 深度扫描的实现：递归遍历 `stripCredentials` 的返回值（对象、数组、字符串），断言 sentinel 字符串不出现在任何叶子。sentinel 由 `crypto.randomUUID()` 生成，避免与固定测试串碰撞。

focused 命令：

```bash
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/profile-presentation.smoke.test.ts tests/profile-credentials.smoke.test.ts tests/profiles-vendor-key.smoke.test.ts tests/platform-color-tokens.smoke.test.ts
```

治理测试（R10）：

```bash
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/theme-switch.smoke.test.tsx tests/token-single-point.smoke.test.tsx
```

## 风险

- 明色平台色推导可能与首页现有明色观感冲突。若发生，优先保证对比度达标，观感差异记录到 `notes.md` 交由 rollout 阶段整体确认。
- `utils/*ProfileEditor.ts` 的现有表单模型与 typed DTO 字段名不完全一致（Grok 用 camelCase 表单、snake_case DTO）。`project()` 只读 DTO，不碰表单模型，因此本任务不受影响；差异由 `08-26-profile-editor` 的 adapter 吸收。
- `theme-token-contracts.md` 的冻结段是治理记录而非代码，改动它属于规格更新。若该文档要求另立 token-governance 任务，在 `notes.md` 记录并在父任务 rollout 前与用户确认，不擅自跳过登记。
