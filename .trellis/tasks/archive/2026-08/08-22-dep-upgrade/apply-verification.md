# `@apply` 验证记录（段 2 / Tailwind v4，AC5）

> 任务：`08-22-dep-upgrade` 段 2。日期：2026-08-23。
> 结论先行：**活样式（编译图内）`@apply` 现存 0 处**；648 处 `@apply` 全部位于 25 个 `.vue` SFC 的 `<style>` 块，自 2026-08-23 React 基座落地起不在 vite 编译图内，属死代码。逐文件移交清单见第 b 节。

## a. 活样式验证

### 现状测量（与 prd/design 记载的「另有 2 处 .css 内 @apply」不符，偏差登记）

```text
$ grep -rnE "^\s*@apply\b" ccr-ui/src --include="*.css" --include="*.ts" --include="*.tsx"
（无匹配）

$ grep -rln "@apply" ccr-ui/src --include="*.vue" | wc -l
25
```

prd.md 记载「648 处 @apply（集中在 25 个文件）+ 2 处 .css 内 @apply」。当前 `.css`/`.ts`/`.tsx` 内 `@apply` 指令为 0 处；`src/styles/utilities.css:8` 仅有一处注释文字提及 `@apply`（非指令）。2 处 .css 内 `@apply` 在 react-foundation 批次前已被移除，本任务无活样式 `@apply` 需要加 `@reference`。

### 展开验证（补偿性证据）

活样式无 `@apply` 可取代表规则，改用「工具类展开 → 产物命中 → 运行时计算值」三层抽检验证 v4 生成链路无静默失效。产物：`dist/assets/index-Dz-Pw-Iy.css`（202.43 kB）。

| 来源 | 代表规则 | 展开属性 | 产物命中 |
| --- | --- | --- | --- |
| `core.css` @theme inline | `.font-bold` | `font-weight:500`（压缩语义） | `.font-bold{--tw-font-weight:500;font-weight:500}` |
| `core.css` @theme inline | `.text-text-primary` | `color:rgb(var(--color-text-primary-rgb))` | 命中（逐字） |
| `core.css` @theme inline | `.p-4` | `padding:var(--space-4)` | `.p-4{padding:var(--space-4)}` |
| `core.css` @theme inline | `.rounded-lg` | `border-radius:var(--radius-lg)` | `.rounded-lg{border-radius:var(--radius-lg)}` |
| `core.css` @theme inline | `.py-2.5` | `calc(var(--spacing)*2.5)` | `.py-2\.5{padding-block:calc(var(--spacing) * 2.5)}` |
| `core.css` @theme | `.animate-fade-in` | `animation:var(--animate-fade-in)` + `@keyframes fade-in` | 两者均命中 |
| `core.css` @utility | `.transition-interactive` | 6 项 transition-property + 默认时长/缓动 | 命中（逐字） |
| `core.css` @utility | `.duration-fast` | `transition-duration:var(--duration-fast)` | 命中 |
| `components/surfaces.css` | `.surface-shell,.liquid-glass` | background/backdrop-filter/border/box-shadow | 命中（逐字） |
| `core.css` @custom-variant | `.dark:hidden` | `:where([data-theme=dark],[data-theme=dark] *)` | `.dark\:hidden:where([data-theme=dark],[data-theme=dark] *){display:none}` |

运行时抽检（`bun run dev` → http://127.0.0.1:15173 无头加载，注入探针元素读计算值）：

```text
font-bold → font-weight "500"；font-semibold → "500"
bg-bg-base → rgb(232, 233, 236)（light token）；data-theme=dark 下 → rgb(19, 19, 22)（dark token）
text-text-primary → rgb(25, 27, 32)；text-accent-primary → rgb(207, 98, 57)
rounded-lg → "8px"（= --radius-lg）；p-4 → "16px"（= --space-4）
transition-interactive → transition-property 含 color/background-color/border-color/box-shadow/transform/opacity
dark:hidden → light 下 "block"，data-theme=dark 下 "none"
```

## b. 移交清单（25 个 `.vue` 文件，648 处 `@apply`）

事实陈述：以下文件自 2026-08-23（react-foundation 批次 1-2 提交 `81d6aa9b`/`8e262ce2`）起不在编译图内——`index.html` 入口为 `/src/main.tsx`，从入口可达的 import 闭包不含任何 `.vue` 文件；其 `<style>` 块中的 `@apply` 不参与构建，v4 下也不存在「静默失效」问题（不构建即不产出空规则）。归属子任务按 `08-22-react-foundation/path-mapping.md` 交叉引用；每个文件迁移落位时的义务：**为其样式文件加 `@reference`**（相对路径指向 `src/styles/index.css` 所在层）。

| 文件 | @apply 数 | 迁移目标路径（path-mapping.md） | 归属子任务 | 义务 |
| --- | --- | --- | --- | --- |
| `ccr-ui/src/components/codex/CodexAccountCard.vue` | 3 | src/features/codex/CodexAccountCard.tsx | 08-22-views-codex | 迁移落位时为其样式文件加 @reference |
| `ccr-ui/src/components/common/MarketplacePagination.vue` | 7 | src/ui/MarketplacePagination.tsx | 08-22-shell-port | 迁移落位时为其样式文件加 @reference |
| `ccr-ui/src/components/MainLayout.vue` | 4 | src/shell/MainLayout.tsx | 08-22-shell-port | 迁移落位时为其样式文件加 @reference |
| `ccr-ui/src/components/sync/SyncAccountDialog.vue` | 25 | src/features/sync/SyncAccountDialog.tsx | 08-22-views-sync-tools | 迁移落位时为其样式文件加 @reference |
| `ccr-ui/src/components/sync/SyncInfoSidebar.vue` | 12 | src/features/sync/SyncInfoSidebar.tsx | 08-22-views-sync-tools | 迁移落位时为其样式文件加 @reference |
| `ccr-ui/src/components/sync/SyncOperationOutputPanel.vue` | 28 | src/features/sync/SyncOperationOutputPanel.tsx | 08-22-views-sync-tools | 迁移落位时为其样式文件加 @reference |
| `ccr-ui/src/components/ui/AsyncStatePanel.vue` | 3 | src/ui/AsyncStatePanel.tsx | 08-22-design-system | 迁移落位时为其样式文件加 @reference |
| `ccr-ui/src/components/ui/Button.vue` | 18 | src/ui/Button.tsx | 08-22-design-system | 迁移落位时为其样式文件加 @reference |
| `ccr-ui/src/components/ui/Card.vue` | 2 | src/ui/Card.tsx | 08-22-design-system | 迁移落位时为其样式文件加 @reference |
| `ccr-ui/src/views/AppSettingsView.vue` | 72 | src/features/configs/AppSettingsView.tsx | 08-22-views-profiles-config | 迁移落位时为其样式文件加 @reference |
| `ccr-ui/src/views/ClaudeCodeProfilesView.vue` | 4 | src/features/platform/profiles/ClaudeCodeProfilesView.tsx | 08-22-platform-unify（收敛为薄壳） | 迁移落位时为其样式文件加 @reference |
| `ccr-ui/src/views/ClaudeCodeView.vue` | 56 | src/features/claude/ClaudeCodeView.tsx | 08-22-views-claude | 迁移落位时为其样式文件加 @reference |
| `ccr-ui/src/views/CodexAuthView.vue` | 1 | src/features/platform/auth/CodexAuthView.tsx | 08-22-platform-unify（收敛为薄壳） | 迁移落位时为其样式文件加 @reference |
| `ccr-ui/src/views/CodexProfilesView.vue` | 4 | src/features/platform/profiles/CodexProfilesView.tsx | 08-22-platform-unify（收敛为薄壳） | 迁移落位时为其样式文件加 @reference |
| `ccr-ui/src/views/CodexSessionsView.vue` | 44 | src/features/codex/CodexSessionsView.tsx | 08-22-views-codex | 迁移落位时为其样式文件加 @reference |
| `ccr-ui/src/views/CodexSettingsView.vue` | 8 | src/features/platform/settings/CodexSettingsView.tsx | 08-22-platform-unify（收敛为薄壳） | 迁移落位时为其样式文件加 @reference |
| `ccr-ui/src/views/CodexSlashCommandsView.vue` | 16 | src/features/codex/CodexSlashCommandsView.tsx | 08-22-views-codex | 迁移落位时为其样式文件加 @reference |
| `ccr-ui/src/views/CodexView.vue` | 63 | src/features/codex/CodexView.tsx | 08-22-views-codex | 迁移落位时为其样式文件加 @reference |
| `ccr-ui/src/views/CommandsView.vue` | 72 | src/features/platform/commands/CommandsView.tsx | 08-22-platform-unify（收敛为薄壳） | 迁移落位时为其样式文件加 @reference |
| `ccr-ui/src/views/GeminiCliView.vue` | 59 | src/features/gemini/GeminiCliView.tsx | 08-22-views-secondary-platforms | 迁移落位时为其样式文件加 @reference |
| `ccr-ui/src/views/grok/GrokProfilesView.vue` | 4 | src/features/platform/profiles/GrokProfilesView.tsx | 08-22-platform-unify（收敛为薄壳） | 迁移落位时为其样式文件加 @reference |
| `ccr-ui/src/views/HooksView.vue` | 22 | src/features/claude/HooksView.tsx | 08-22-views-claude | 迁移落位时为其样式文件加 @reference |
| `ccr-ui/src/views/OpenCodeView.vue` | 55 | src/features/opencode/OpenCodeView.tsx | 08-22-views-secondary-platforms | 迁移落位时为其样式文件加 @reference |
| `ccr-ui/src/views/SkillsMigrationView.vue` | 23 | src/features/claude/SkillsMigrationView.tsx | 08-22-views-claude | 迁移落位时为其样式文件加 @reference |
| `ccr-ui/src/views/SyncView.vue` | 43 | src/features/sync/SyncView.tsx | 08-22-views-sync-tools | 迁移落位时为其样式文件加 @reference |

合计 25 个文件 / 648 处，与 prd.md 的 648 计数一致；归属列与 path-mapping.md 全部命中（0 未映射）。

## c. 偏差说明（相对 AC5 原文「25 个文件加 @reference」）

AC5 原文要求「25 个 `@apply` 文件的样式生效验证记录落盘，无静默失效项」，implement.md 段 2 写为「25 个文件逐个加 `@reference`」。本任务按预批准偏差执行，理由：

1. **前提已消失**。design.md §3 写该方案的依据是「25 个 .vue 文件在编译图内」。react-foundation 批次 1-7 落地后，入口为 `main.tsx`，编译图内已无任何 `.vue` 文件。给不参与构建的 SFC `<style>` 块加 `@reference` 不会产生任何产物变化，属于对死代码的无效编辑；且本任务边界明确禁止改 `.vue` 文件。
2. **「静默失效」在该前提下不可能发生**。v4 的静默失效指「组件级样式文件参与构建但缺 `@reference`，`@apply` 展开为空规则」。不参与构建的文件不产出任何规则，无失效可言。
3. **义务不灭失，改为按归属子任务追踪**。第 b 节逐文件登记了目标路径、归属子任务与「迁移落位时为其样式文件加 @reference」义务；各视图子任务迁移对应文件时，其样式文件（`.tsx` 内联样式或 `.css`）一旦进入编译图并使用 `@apply`，必须补 `@reference`，届时按第 a 节方法做代表规则展开核对。
4. **活样式链路已验证无静默失效**。第 a 节以 10 条产物命中 + 7 项运行时计算值证明 v4 生成链路（@theme inline / @theme / @utility / @custom-variant / 组件类）展开正确。

因此本文件以「活样式验证（0 处 + 补偿性展开验证）+ 25 文件移交清单 + 偏差说明」三节满足 AC5 的意图：无静默失效项、义务逐文件可追溯。
