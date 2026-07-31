# 配色系统重构 — 技术设计

> 父任务 `07-28-ccr-ui-visual-redesign`；证据 `../07-28-ccr-ui-visual-redesign/research/diagnosis.md`。
> 原则：**令牌名不动，值域与几何重建** —— 消费者零改动即可获得不透明表面与实心文本；组件层剩余问题由子任务 B 迁移。

## 1. 目标架构

```
:root                      = neutral light（新默认）
[data-theme='dark']        = neutral dark
[data-flavor='clay'] (+dark)                      = clay 暖色 flavor
[data-resolved-flavor='latte']                    = catppuccin light 解析结果
[data-resolved-flavor='mocha']                    = catppuccin dark 解析结果
[data-accent='clay|sage|sky|mauve'] (+dark 变体)  = 4 个 accent
```

- 删除：`paper` / `graphite` flavor 块；`frappe` / `macchiato` ctp 调色板块；4-flavor 共享 Catppuccin 重映射块（拆成 latte/mocha 各自完整的重映射块）。
- `data-flavor` 存用户选择（`neutral|clay|catppuccin`），`data-resolved-flavor` 存解析结果（`neutral|clay|latte|mocha`）。三轴独立契约不变。

## 2. 新配色几何（锚点值，实施时以对比度测试为准微调）

几何公理：**暗色 elevated 逐级提亮；亮色桌面压暗、卡片最亮；所有表面令牌 100% 不透明；所有文本令牌 100% 不透明。**

### neutral dark（新默认暗色）

| 令牌 | 值 | 说明 |
|---|---|---|
| bg-base | `#131316` | 桌面基底（最暗） |
| bg-elevated | `#1a1b1f` | 侧栏/顶栏/工作区 |
| bg-surface | `#22242a` | 卡片（更亮） |
| bg-overlay | `#2c2f37` | 浮层/chip（最亮） |
| text-primary | `#f2f3f5` | ≥12:1 on surface |
| text-secondary | `#c9ccd3` | ≥7:1 |
| text-muted | `#9ba1ab` | ≥4.5:1 |
| text-ghost | `#6d727c` | 仅装饰/占位 |
| text-disabled | `#4f545d` | 免对比 |
| text-inverted | `#17181c` | 实心 accent 按钮文字 |
| border-subtle/default/strong | `rgb(235 238 245 / 14%|22%|34%)` | 卡片边界肉眼可辨 |
| accent(clay) | `#e8835b` / hover `#f0926c` / active `#d4744a` | 高彩度，非粉彩 |
| accent-primary-contrast | `#1d1207` | 实心按钮文字（新增） |

### neutral light（新默认亮色）

base `#e8e9ec` < elevated `#f2f3f5` < surface `#fbfcfd`；overlay `#dcdee3`；text `#191b20 / #3f434c / #5f646e / #878d98 / #b3b8c0`；inverted `#f7f8fa`；border `rgb(25 27 32 / 12%|19%|30%)`；accent `#cf6239`（对比文字 `#fff8f2`，对 ≥3.5:1）。

### clay flavor（保留，按同一几何重修）

- 暗色维持现有 base `#17120f` < elevated `#221b18` < surface `#2a221e` < overlay `#342b26`（几何已正确），文本改实心、border alpha 提到 14/22/34、accent 暗色提亮彩度（`#e8835b` 同 neutral，保持品牌一致）。
- 亮色维持现有暖纸面阶梯，文本/border 同步收紧。

### catppuccin 解析（latte / mocha）

- **修复 elevation 反转**：latte 改为 base=`mantle` < elevated=`base` < surface=`#fafbfe`（近白卡片）< overlay=`surface0`；mocha 沿用现 mocha 覆盖块几何（crust < base < surface0 < surface1），提升为唯二 Catppuccin 块。
- 文本改实心：直接引用 `ctp-text/subtext1/subtext0/overlay1`，去掉 alpha。
- stage 令牌在 latte/mocha 块内同样 100% 不透明。

### stage 语义层（全部 flavor 统一）

- `--color-stage-text-primary/secondary/muted/quiet` → `var(--color-text-primary/secondary/muted/ghost)`（100% 不透明，98/90/76/62% 的 alpha 全部移除）。
- `--color-stage-surface-soft/medium/strong` → `var(--color-bg-surface)` / `var(--color-bg-elevated)` / `var(--color-bg-overlay)`（不透明）。
- `--color-stage-chip-neutral-bg/border/text` → `var(--color-bg-overlay)` / `var(--color-border-default)` / `var(--color-text-secondary)`。
- `--stage-bg-mesh/aurora/orb/grid-*` 死令牌删除；`--stage-bg-noise-opacity` 随噪点层一并删除。

## 3. 玻璃与表面契约（新）

| 别名 | 新解析 | 用途 |
|---|---|---|
| `--surface-shell-*` | 不透明 `bg-elevated` + border-default + shadow-sm（blur: none） | 侧栏/顶栏 |
| `--surface-status-*` | 不透明 `bg-surface` + border-default（blur: none） | 页内 sticky 条 |
| `--surface-card-*` / `--surface-workspace-*` | 100% 不透明（去掉 /98%） | 内容卡 |
| `--surface-modal-*` | floating 档：`rgb(bg-elevated / 92%)` + `blur(12px)`（**去 saturate**）+ border-strong | modal/popover |

- `--material-glass-chrome-*` / `--material-glass-inline-*` 令牌保留名称但改为不透明配方（预算契约不变：floating 同屏 ≤1）；`--inner-glow` 暗色降为 ≤3% 或直接 `none`。
- `prefers-reduced-transparency: reduce` 块按新令牌集重写，仍含 `html:root[data-resolved-flavor='mocha']` 同级重置。
- 旧 `--glass-*` / `--liquid-glass-*` deprecated 令牌保持薄值不动（子任务 B 逐步迁移消费者）。

## 4. flavor / accent 精简与迁移

### 值域

```ts
type FlavorMode = 'neutral' | 'clay' | 'catppuccin'
type ResolvedFlavor = 'neutral' | 'clay' | 'latte' | 'mocha'
type AccentMode = 'clay' | 'sage' | 'sky' | 'mauve'
```

### 迁移表（read 时映射 + store 初始化时写回）

```ts
const FLAVOR_MIGRATION = { paper: 'neutral', graphite: 'neutral',
  latte: 'catppuccin', frappe: 'catppuccin', macchiato: 'catppuccin', mocha: 'catppuccin' }
const ACCENT_MIGRATION = { sand: 'clay', amber: 'clay', rose: 'clay', slate: 'sky' }
```

- `readStoredFlavor` / `readStoredAccent`：迁移表 → 白名单校验 → 非法回退 `neutral` / `clay`。
- `resolveFlavorMode`：`catppuccin` → light 解析 `latte`，dark 解析 `mocha`；其余直通。
- **`index.html` 首帧 IIFE 必须内联同一迁移表与解析逻辑**（无 import 能力），`theme-bootstrap.smoke.test.ts` 逐字锁定同步更新。
- `DEFAULT_FLAVOR = 'neutral'`；store `initializeTheme` 检测到存储值 ≠ 迁移值时写回 localStorage。

### accent 暗色值（高彩度方向锚点）

clay `#e8835b`、sage `#6fbf73`、sky `#6ea8e8`、mauve `#b78fe0`；各配 `*-contrast` 深色文字令牌。Catppuccin flavor 下 accent 映射改 ctp 高彩度色（clay→peach、sage→green、sky→blue、mauve→mauve，直接用而非 pastel 化）。

## 5. 氛围层收敛

- `StageBackground.vue`：模板删 `__halo × 2` / `__grain` / 洗色带，`__base` 改 `background: var(--color-bg-base)`（不透明）；样式表同步精简。`premium-pink/blue` 令牌消费随之消失 → 令牌删除。
- `AnimatedBackground.vue`：同款收敛（保留组件壳以免动 3 个挂载点；若挂载点改动更小也可直接删组件，实施时择一并在 implement 记录）。
- `backgrounds.css`：删除无消费者的 `.premium-background` / `.premium-bg-orb` / `.orb-*` / `.premium-bg-pattern` 块；`deferred-decorations.css` 引用同步清理；`base.css:311` 的 reduced-motion 引用同步删除。
- `base.css:19-20`：`[data-theme='dark']` 下 `-webkit-font-smoothing: auto; -moz-osx-font-smoothing: auto`。

## 6. 确定性 bug 修复

- `PlatformMcpView.vue` / `PlatformPluginsView.vue`：删 `onCardHover` 的 JS 内联背景，改 `:hover` CSS 类（`background: var(--color-bg-overlay)`）。
- `OutputStylesView.vue`：`bg-white` → `bg-bg-surface` 或补 `dark:` 守卫。

## 7. 新守卫测试 `tests/theme-contrast-contract.smoke.test.ts`

方案：静态解析（不依赖 jsdom 对 CSS var 的不完整支持）——

1. 读 `tokens.css` 文本，按已知选择器清单（`:root`、`[data-theme='dark']`、`[data-flavor='clay']`、`[data-theme='dark'][data-flavor='clay']`、`[data-resolved-flavor='latte']`、`[data-resolved-flavor='mocha']`、各 accent 块）抽取 `--color-*` 定义，实现一个仅覆盖这些块的微型级联解析器（`var()` 递归解析、hex/rgb 归一）。
2. 对 6 组 theme × resolved-flavor 组合计算 WCAG 相对亮度对比度：text-primary/secondary/muted vs bg-surface；accent vs accent-contrast；border-default vs bg-surface（相对亮度差阈值）。
3. 断言全部文本/表面令牌定义不含 `< 100%` 的 alpha（正则扫令牌定义行）。
4. 断言 `--color-stage-surface-*` 与 `--surface-card-*` 解析后不透明。

## 8. 兼容性与回滚

- 存储键不变（`ccr-theme/ccr-flavor/ccr-accent`）；迁移在读取侧完成，旧版本回滚后读到新值（`neutral/catppuccin`）会按旧白名单回退 clay/light —— 可接受，不丢数据。
- i18n 显示名（`settings.appearance.flavorOptions.*` 等）在本任务只新增 `neutral/catppuccin` 键、保留旧键不删（子任务 C 统一清理）。
- 回滚点：tokens.css 单文件可整体 git 还原；迁移逻辑独立函数可单独还原。

## 9. 受影响文件清单（预期）

- `ccr-ui/src/styles/tokens.css`（主战场）、`theme.css`（bridge 微调）、`base.css`、`backgrounds.css`、`deferred-decorations.css`
- `ccr-ui/src/utils/themeBootstrap.ts`、`ccr-ui/src/stores/shellPreferences.ts`（写回迁移）、`ccr-ui/index.html`（IIFE）
- `ccr-ui/src/components/common/StageBackground.vue`、`AnimatedBackground.vue`
- `ccr-ui/src/views/generic/PlatformMcpView.vue`、`PlatformPluginsView.vue`、`OutputStylesView.vue`
- `ccr-ui/tests/theme-contrast-contract.smoke.test.ts`（新）、`theme-bootstrap.smoke.test.ts`、`apple-glass-surface-contract.smoke.test.ts`、必要时的 `app-settings.smoke.test.ts`
