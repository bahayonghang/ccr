# CCR UI 视觉系统重设计：中性高对比配色 + 设置系统

## Goal

解决暗色模式"泛白"（奶雾、低对比、发灰）问题：重构整个配色系统为中性高对比现代风（新默认），同步重设计设置系统 UI，并保留精简后的 flavor/accent 个性化能力。

## Background

- 用户在三张暗色截图（Profiles 管理 / Overview / Settings，Catppuccin Macchiato）上反馈整体泛白，要求重新设计整体样式（包括设置系统）并重构配色系统。
- 根因审计已固化到 `research/diagnosis.md`：11 类根因，核心是不透明契约被三条旁路打破（壳层 chrome 玻璃、247 处模板 alpha 表面、stage-soft 58% 令牌）+ 全局背景底雾 + 半透明文本令牌 + 粉彩 accent 配白字 + Catppuccin 映射反转 elevation 几何。
- 用户已确认三项设计决策：
  1. **设计语言**：转向中性高对比现代风（类 Linear/Vercel 的中性灰阶 + 单一高彩度品牌色），暖色 clay 降级为普通 flavor。
  2. **flavor/accent 精简**：flavor 7 → 3，accent 8 → 4，旧值做存储迁移。
  3. **氛围层大幅收敛**：侧栏/顶栏改不透明；玻璃仅保留浮层；背景光晕/洗色带/噪点删除；清理死代码。
- 本任务是父任务，不直接实施；三个子任务各自独立可验证，依赖顺序在子任务 PRD 中显式声明。

## 子任务地图

| 子任务 | 交付物 | 依赖 |
|---|---|---|
| `07-28-color-system-rebuild` | tokens.css 配色几何重建、flavor 7→3 / accent 8→4 精简与迁移、氛围层收敛、白色背景 bug 修复、对比度守卫测试 | 无，最先实施 |
| `07-28-surface-contract-migration` | 组件面迁移：247 处 alpha 表面、11 处直写透明文本、~30 处 inset 白高光、按钮文字色、壳层/模态表面契约接入 | 依赖 color-system-rebuild 的新令牌与契约 |
| `07-28-settings-redesign` | `/settings` 页面重设计（新 flavor/accent 集合、真实 token 预览）、i18n/bootMessages/dock 同步、settings 相关 smoke 更新 | 依赖 color-system-rebuild；可与 surface-contract-migration 并行 |

## Requirements

### R1. 配色系统重构（子任务 A）

- 新默认 flavor `neutral`：中性灰阶、暗色 elevated 逐级提亮、表面 100% 不透明、文本实心且满足对比度下限。
- flavor 精简为 `neutral`（默认）/ `clay` / `catppuccin`（light→latte、dark→mocha 自适应），accent 精简为 `clay`（默认）/ `sage` / `sky` / `mauve`；旧存储值按迁移表映射，首帧 IIFE 与 themeBootstrap 双实现同步修改。
- 氛围层收敛：StageBackground/AnimatedBackground 删除 halo/洗色带/噪点层；chrome 档玻璃改不透明；floating 档保留但提高不透明度、降低 saturate；清理 `backgrounds.css` 死代码与 `--stage-bg-*` 死令牌。
- 修复 R1 类确定性 bug（JS 白背景、`bg-white` 无暗色守卫）。
- 新增 token 对比度守卫 smoke 测试，把"文本/表面/边框对比度下限"变成可执行契约。

### R2. 组件面迁移（子任务 B）

- 内容区模板 alpha 表面（`bg-bg-*/N`）迁移到不透明语义表面；直写透明文本改实心令牌；卡片 inset 白高光移除或收敛进令牌。
- 实心按钮文字由白色改为 `--color-text-inverted`；`text-white` 存量审计收敛。
- 壳层（侧栏/顶栏/模态）接入新表面契约。

### R3. 设置系统重设计（子任务 C）

- 重设计 `/settings` 页面布局与外观区：flavor/accent 选择器使用真实 token 渲染的预览（不再是脱离实际的渐变 swatch），主题模式切换改为分段控件。
- i18n（双 locale + bootMessages 副本）、MainLayout settings dock、全部 settings 相关 smoke 测试同步更新。

### R4. 兼容与守护

- `data-theme` / `data-flavor` / `data-accent` 三轴独立性保持；`ccr-theme`/`ccr-flavor`/`ccr-accent`/`ccr-font-*` 存储键不变（仅值域迁移）。
- 字体系统（`--font-*-base` 轨道、预设块、净化器）不动。
- 旧 flavor/accent 存储值在 web 与 Tauri 首帧都安全迁移，不闪退、不丢用户选择语义。
- reduced-transparency 无障碍回退在新契约下继续完整生效。

## Acceptance Criteria

- [x] AC1: 三个子任务各自的 AC 全部通过（见各子任务 prd.md）。
- [x] AC2: 视觉验证矩阵通过：Overview / Profiles / Settings 三条路由 × light/dark × 3 个 flavor，截图 + 计算样式断言（`data-theme`/`data-flavor`/`data-resolved-flavor`/`data-accent` 与关键 token 值），证据记录到父任务 `research/visual-verification.md`。
- [x] AC3: 泛白量化收敛：暗色下内容区不存在 < 98% 不透明度的常驻表面（modal 浮层除外）；文本令牌 100% 不透明；全局背景无 halo/洗色带/噪点层；`rg` 扫描无 `255 255 255` 无守卫残留（白名单：accent 上文字、mask）。
- [x] AC4: `cd ccr-ui && bun run type-check && bun run lint && bun run test:i18n` 通过；全部相关 smoke 套件（theme-bootstrap / app-settings / apple-glass-surface-contract / font-preferences / main-layout-theme-stage + 新增对比度守卫）通过。
- [x] AC5: `just ui-check` 通过；最终 `git diff` 不触碰本任务范围外的文件（Rust 后端、CLI、docs 站除外事项需在子任务中显式声明）。

## Out Of Scope

- 平台业务页面（Claude/Codex/OpenCode/Gemini 各 SettingsView）的功能性重构（仅在子任务 B 中做表面契约迁移）。
- 新增偏好维度（如密度、字号、自定义主题色盘）。
- 设计系统文档站（`ccr-ui/design-system/`）重写；如有引用过期，仅在 AGENTS.md/相关 README 做最小同步。
- Rust/Tauri 后端变更（`DesktopShellPreferences` 字段不变）。
