# 08-26-profile-registry-tokens notes

## R5 Claude slot3

实测本机 `~/.ccr/platforms/claude/profiles.toml`（29 个 profile 表，只统计填充率，不记录字段值）：

| 字段 | 非空数 | 填充率 |
| --- | --- | --- |
| `provider` | 27 / 29 | 93.1% |
| `account` | 27 / 29 | 93.1% |
| `effort_level` | 1 / 29 | 3.4% |

选定 **`provider`** 作为 Claude slot3。理由：设计评估顺序第一位；与统计卡供应商去重语义连贯；`account` 可能含邮箱等个人可识别信息，不进入呈现层；`effort_level` 几乎为空，多数情况只会显示占位符。

## Token 治理

`theme-token-contracts.md` 原句要求「Further additions still require a dedicated token-governance task」。本子任务 R10 即该次 +20 名称的治理登记：已写入冻结段（unique-name union 432 → 452），归属层一 `tokens.css` 明暗块，不进 `@theme` / `@theme inline`。不另立独立治理任务。

明色三角色由 `--color-bg-surface` `#fbfcfd` 与各平台 dot 按 12% / 32% 混合后取整 hex，`-text` 向暗侧调整至对 `-surface` ≥ 4.5:1。暗色 claude/codex/grok/antigravity/opencode 取设计稿；gemini 暗色三角色由 `#7d97b6` 与 `#22242a` 按 22% / 40% 混合，text 向亮侧调至 4.5:1。

`--color-platform-codex` `#5b8a62` → `#7cab82`，`--color-platform-grok` `#716b80` → `#a79bc4`，同步 `-rgb`。

## `--color-platform-*` 消费点

| 位置 | 用途 | 确认 |
| --- | --- | --- |
| `src/styles/tokens.css` | 定义 | 本任务扩展四角色 |
| `src/styles/core.css` | `@theme inline` 映射 claude/codex/grok/gemini 的 dot | 仍指向 `-rgb`；色值变化为决策 1 预期溢出。三角色与 antigravity 不进 `@theme inline` |
| `src/styles/theme.css` | `--platform-*` 别名 | 跟随新 dot，可接受 |
| `src/styles/utilities/utilities.css` | 平台色文字 / 背景工具类 | 跟随新 dot，可接受 |
| `src/styles/components/profiles-page.css` | `--cp-icon-*` | 跟随新 dot，可接受 |
| `src/styles/components/codex-auth-shared.css` | Codex 认证控件强调 | 跟随新 Codex dot，可接受 |
| `src/features/usage/styles/dashboard-platform-matrix.css` | 用量矩阵强调色 | 跟随新 dot，可接受 |
| `src/features/usage/styles/dashboard-usage-movement.css` | 用量运动色点 | 跟随新 dot，可接受 |
| `src/features/grok/home/GrokHomeCards.tsx` | hover `color-mix` 6% | 跟随新 Grok dot，可接受 |
| `src/features/grok/GrokView.tsx` | 标题强调色 | 跟随新 Grok dot，可接受 |
| `src/ui/agent-icons.tsx` | 图标色 | 跟随新 dot，可接受 |
| `src/shell/MainLayoutNav.tsx` | OpenCode 导航点 | OpenCode dot 未改 |
| `src/features/usage/dashboard/DashboardView.tsx` | OpenCode icon class | OpenCode dot 未改 |
| `src/utils/claudeProfiles.ts` | provider 元数据 cssVar | 仅引用变量名，可接受 |

列宽在 `fieldSlots.columnWidth` 使用 rem（13.5 / 11 / 6.5 / 8.5），等价设计稿 216 / 176 / 104 / 136 px，避免 `hardcode-px-rgba.smoke.test.ts` 的 src ts px 门禁。

## 命名不一致

`rg -l "smoke.test" ccr-ui/src` 命中的是既有注释（`shellPreferences.ts` / `eventBridge.ts` / `types/checkin.ts`），不是测试文件。本任务新增测试均在 `ccr-ui/tests/`。
