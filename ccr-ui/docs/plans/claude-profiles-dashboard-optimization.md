# Claude Profiles Dashboard — 顶部空白优化方案

> 目标文件：`ccr-ui/src/views/ClaudeCodeProfilesView.vue`、`ccr-ui/src/components/claude/ClaudeProfilesOverview.vue`
> 设计基线：Anthropic-like 编辑式工作台 / 暖中性色 / 高密度 power-user / 明暗双主题
> 版本：v1 · 针对 `PageHeaderCard` + `ClaudeProfilesOverview` 的顶部 dashboard 区域

---

## 1. 问题诊断（Root Cause）

对照截图，顶部 dashboard 的「空白」不是单一区域，而是 **四类结构性浪费** 叠加造成的：

```
┌────────────────────────────────────────────────────────────────────────────┐
│ [icon] Claude Profiles 管理                         [返回][刷新][添加 Profile] │
│        直接管理 CCR Core 中的 Claude profiles…                              │
│        [CLAUDE CONTROL CONSOLE] [Provider 分组 18]                          │
│ ────────────────────────────────────────────────────────────────────────── │
│ 当前 PROFILE        ░░░ 大片空白区 A ░░░   anyrouter official_relay @… API… │  ← ① 横向空带
│ anyrouter2 ●当前已激活                     ┌──────────┐  ┌──────────┐        │
│ AnyRouter 备用服务1 (github_5962,floorp)   │ PROFILES │  │ PROVIDERS│        │
│                                           │    25    │  │    18    │        │
│           ░░░ 空白区 B ░░░                  └──────────┘  └──────────┘        │  ← ② 左下死区
│                                           ┌──────────┐  ┌──────────┐        │
│                                           │ MODELS   │  │  ACCESS  │        │
│                                           │    7     │  │    17    │        │
│                                           └──────────┘  └──────────┘        │
│ 自定义 Endpoint 25  带标签 17  缺少主模型 18  缺少账号 8                       │  ← ③ 冗余 ribbon
└────────────────────────────────────────────────────────────────────────────┘
                      ░░░ 空白区 D：外层 padding-bottom 偏大 ░░░
```

### ① 横向空带 A（最致命）

**现象**：`当前 PROFILE` eyebrow 与 `anyrouter / official_relay / @github_5962 / API Key` chips 之间有一条 **30%–45% 宽度的横向空白带**。

**原因**：`ClaudeProfilesOverview.vue:5` 使用 `lg:flex-row lg:items-start lg:justify-between`，把左侧文字组与右侧 chips 推到左右两端；但左侧文字只占约 35% 宽度，右侧 chips 占约 25%，中间强制产生 40% 空白。

### ② 左下死区 B

**现象**：左列（`xl:grid-cols-[1.55fr_1fr]`，左侧 1.55fr）只有三行文字（eyebrow → name → description，约 110px 高），而右列 2×2 tile 网格约 200px 高，左下角多出约 90px 垂直空白。

**原因**：两列高度由内容决定但不互相约束；左列密度远低于右列。

### ③ 冗余 ribbon C

**现象**：底部 4 个 ribbon chip（Endpoint / 带标签 / 缺少主模型 / 缺少账号）与上方 4 个大 tile 在视觉上构成两层「4 分格」，造成层级复读。

**原因**：`overviewTiles` 与 `ribbonItems` 分别渲染成两个独立 section，数据维度却同属「数据健康度」。

### ④ 外层 padding 残余 D

**现象**：`PageHeaderCard.__body` 的 `margin-top: 1.25rem` + 内部 `space-y-3.5` + ribbon 下方 section 的 `py-3`，三层 padding 累计产生 ~60px 底部留白。

---

## 2. 设计原则（Guardrails）

按 `ccr-ui/CLAUDE.md` 的品牌语言：

1. **Power First** — 高密度扫读优先：每一行都承载有效信息，禁止纯装饰空白。
2. **编辑式层级** — 靠排版规模对比（1.6rem 标题 / 1.1rem 数值 / 0.72rem eyebrow）建立层次，而非靠空白拉开距离。
3. **横向节奏** — dashboard 顶部改为「横向编辑 ticker」结构，消灭 2 列 grid 造成的高度不对称。
4. **克制材质** — 延续现有暖中性色 + 轻度半透明；不引入新的渐变 / 发光 / mascot。
5. **双主题等价** — 所有新样式必须明暗双主题达到 4.5:1 对比度。

---

## 3. 方案 A / B / C 对比

| # | 思路 | 视觉 | 空白消除 | 工作量 | 取舍 |
|---|------|------|---------|--------|------|
| **A** | 只修 chips 布局（chips 贴到 name 同行，去掉 justify-between） | 最小改动 | 仅解决 ① | XS | 无法解决 ② ③ |
| **B** | 把左侧 card 填实（加 endpoint / model / tags 明细行） | 对称更好 | 解决 ① ② | S | 2 列 grid 依然存在，chips 与 tiles 仍有张力 |
| **C ✅ 推荐** | 重构成 **3 横带编辑式**：身份带 + 指标带 + 健康带 | 完全重构 | 解决 ① ② ③ ④ | M | 推翻 2 列 grid；最符合「编辑式工作台」气质 |

**推荐方案 C**。以下详述。

---

## 4. 推荐方案 C：横向编辑三带（Editorial Triple-Band）

### 4.1 新布局骨架

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ [icon] Claude Profiles 管理         [eyebrow·CONSOLE] [Provider 18]          │
│        直接管理 CCR Core…                       [返回] [刷新] [添加 Profile]  │
├──────────────────────────────────────────────────────────────────────────────┤
│ 【身份带 Identity】                                                           │
│ ● anyrouter2 │ 当前已激活 · API Key · anyrouter/official_relay · @github_5962 │
│   AnyRouter 备用服务1 (github_5962,floorp)                                   │
│   ENDPOINT  https://…/v1 ✓   MODEL  sonnet-4.6 → haiku-4.5   TAGS 17         │
├──────────────────────────────────────────────────────────────────────────────┤
│ 【指标带 Metric Ticker — 单行 8 格】                                          │
│ PROFILES    PROVIDERS   MODELS     ACCESS    ENDPOINT   TAGGED   NO-MODEL  NO-ACCT │
│   25          18          7          17         25        17       18        8   │
│  23✓/2✗      18 / 0↯    7/3       8🔒/9🗝     自定义     带标签   缺主模型   缺账号 │
└──────────────────────────────────────────────────────────────────────────────┘
```

三带特点：
- **身份带**：一条信息浓缩行 + 描述行 + 一条 mono 元数据行（endpoint / model / tags）。宽度占满，无 2 列 grid。
- **指标带**：8 个相同结构的指标单元，单行横向排布；primary 4 格字号 1.5rem、secondary 4 格字号 1.15rem，靠 typography 建立主次而非尺寸。
- **抛弃的东西**：2 列 grid / 2×2 tiles / 底部 ribbon / 右上角浮动 chips。

### 4.2 断点策略

| 断点 | 指标带 | 身份带 |
|------|--------|--------|
| `≥ 1280px` | 单行 8 格 | 三行完整 |
| `960–1279px` | 2 行 × 4 格 | 三行完整 |
| `640–959px` | 2 行 × 4 格 | 第三行元数据 wrap |
| `< 640px` | 横向 scroll（overflow-x: auto） | 三行，chips wrap |

禁止在桌面端降级为 2×4 grid — 必须保持「单行 ticker」节奏。

### 4.3 视觉规格（沿用现有 token）

```css
/* 身份带 */
--identity-name-size: 1.55rem;       /* 现有 1.35rem，略放大增加权重 */
--identity-name-weight: 620;
--identity-meta-size: 0.82rem;
--identity-meta-mono: var(--font-mono);

/* 指标带·primary */
--metric-primary-value: 1.5rem;      /* 现有 1.45rem 近似，保留 */
--metric-primary-label: 0.7rem;      /* uppercase + letter-spacing 0.22em */
--metric-detail: 0.74rem;

/* 指标带·secondary（原 ribbon） */
--metric-secondary-value: 1.1rem;
--metric-secondary-label: 0.68rem;

/* 分隔 */
--band-divider: 1px solid rgb(var(--color-border-default-rgb) / 18%);
--metric-divider: 1px solid rgb(var(--color-border-default-rgb) / 14%);  /* 指标之间细竖线 */
```

所有颜色、圆角、阴影使用现有 CSS 变量，无需新增 token。

### 4.4 身份带 —— 精确结构

```html
<section class="cpd-identity">
  <div class="cpd-identity__head">
    <span class="cpd-identity__dot" />                <!-- 状态点 -->
    <h2 class="cpd-identity__name">anyrouter2</h2>
    <span class="cpd-identity__status">当前已激活</span>
    <span class="cpd-identity__sep">·</span>
    <span class="cpd-identity__chip">API Key</span>
    <span class="cpd-identity__chip">anyrouter</span>
    <span class="cpd-identity__chip">official_relay</span>
    <span class="cpd-identity__chip">@github_5962</span>
  </div>

  <p class="cpd-identity__desc">AnyRouter 备用服务1 (github_5962,floorp)</p>

  <dl class="cpd-identity__meta">
    <div><dt>ENDPOINT</dt><dd class="mono">https://anyrouter.top/v1</dd><span class="ok">✓ 自定义</span></div>
    <div><dt>MODEL</dt><dd class="mono">sonnet-4.6</dd><span class="arrow">→</span><dd class="mono">haiku-4.5</dd></div>
    <div><dt>TAGS</dt><dd>{{ tags.length }}</dd></div>
  </dl>
</section>
```

关键点：
- `cpd-identity__head` 用 `flex-wrap: wrap; gap: 0.5rem; align-items: baseline`，让 name + status + chips 在**同一行**自然排布，彻底消灭空白带 A。
- 第三行 `cpd-identity__meta` 是 `display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr))`，自动填满宽度。
- 当没有 current profile 时，身份带降级为一条 hint：`<p>尚未激活 profile — 请从下方列表中选择</p>`。

### 4.5 指标带 —— 精确结构

```html
<section class="cpd-metrics" role="list">
  <!-- primary 4 -->
  <article class="cpd-metric cpd-metric--primary" role="listitem">
    <span class="cpd-metric__label">PROFILES</span>
    <span class="cpd-metric__value">{{ summary.totalProfiles }}</span>
    <span class="cpd-metric__detail">
      <strong class="ok">{{ summary.enabledProfilesCount }}</strong> 已启用
      · <strong class="warn">{{ summary.disabledProfilesCount }}</strong> 已停用
    </span>
  </article>
  <article class="cpd-metric cpd-metric--primary">… PROVIDERS …</article>
  <article class="cpd-metric cpd-metric--primary">… MODELS …</article>
  <article class="cpd-metric cpd-metric--primary">… ACCESS …</article>

  <!-- secondary 4（原 ribbon） -->
  <article class="cpd-metric cpd-metric--secondary">… ENDPOINT 25 · 自定义 …</article>
  <article class="cpd-metric cpd-metric--secondary">… TAGGED 17 · 带标签 …</article>
  <article class="cpd-metric cpd-metric--secondary">… NO-MODEL 18 · 缺主模型 …</article>
  <article class="cpd-metric cpd-metric--secondary">… NO-ACCT 8 · 缺账号 …</article>
</section>
```

样式要点：
```css
.cpd-metrics {
  display: grid;
  grid-template-columns: repeat(8, minmax(0, 1fr));
  gap: 0;                                     /* 零 gap，靠竖线分隔 */
  border-radius: 1.25rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 35%);
  background: linear-gradient(180deg,
    rgb(var(--color-bg-elevated-rgb) / 70%),
    rgb(var(--color-bg-surface-rgb) / 60%));
  overflow: hidden;
}
.cpd-metric {
  position: relative;
  padding: 0.9rem 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
  min-width: 0;
}
.cpd-metric + .cpd-metric::before {
  content: '';
  position: absolute; left: 0; top: 18%; bottom: 18%;
  width: 1px; background: rgb(var(--color-border-default-rgb) / 22%);
}
.cpd-metric--secondary .cpd-metric__value { font-size: 1.1rem; opacity: 0.85; }
@media (width < 1280px) {
  .cpd-metrics { grid-template-columns: repeat(4, minmax(0, 1fr)); }
  .cpd-metric--secondary:nth-child(n+5) { border-top: 1px solid rgb(var(--color-border-default-rgb) / 18%); }
}
@media (width < 640px) {
  .cpd-metrics { grid-template-columns: repeat(8, minmax(140px, 1fr)); overflow-x: auto; }
}
```

### 4.6 动效

- 身份带：`animation-delay: 80ms` + `animate-slide-up`（复用现有）
- 指标带：每个 metric 以 `animation-delay: calc(120ms + (var(--i) * 30ms))` 交错淡入
- 切换 profile 时，身份带 name 用 `view-transition-name` 做 shared-element transition（可选增强）
- 完整支持 `@media (prefers-reduced-motion: reduce)` — 全部改为静态

---

## 5. 具体代码改动清单

### 5.1 `ClaudeProfilesOverview.vue` 重写

**删除**：
- L3 的 `grid gap-3 xl:grid-cols-[minmax(0,1.55fr)_minmax(0,1fr)]` 外层 grid
- L4–43 的整个左侧 `<section>`（当前 profile 卡）
- L45–77 的 `<dl>` 4 tiles
- L80–89 的 ribbon `<section>`

**新增**（按 4.4 / 4.5 结构）：
- `<section class="cpd-identity">` — 身份带
- `<section class="cpd-metrics">` — 合并后的指标带（8 格）

**computed 保留**：`currentProfileName / currentProfileChips / overviewTiles / ribbonItems`。
**computed 新增**：`identityMeta`（组合 endpoint / model 对 / tags count）。

### 5.2 `ClaudeCodeProfilesView.vue` 调整

**L1796-1799** 的覆盖样式简化：
```css
.claude-profiles-view .page-header-card__body {
  padding-top: 1rem;                          /* 原 1.25rem */
  border-top: 1px solid rgb(var(--color-border-default-rgb) / 18%);  /* 原 22% 稍弱化 */
}
```

其他不动。

### 5.3 i18n key 新增

`src/locales/zh-CN.ts` / `en-US.ts` 在 `claudeProfiles` 下新增：
```ts
identityEndpointLabel: 'ENDPOINT',
identityEndpointCustom: '自定义',
identityEndpointOfficial: '官方',
identityModelLabel: 'MODEL',
identityTagsLabel: 'TAGS',
identityNoProfile: '尚未激活 profile — 请从下方列表中选择',
metricsSecondaryEndpoint: 'ENDPOINT',
metricsSecondaryTagged: 'TAGGED',
metricsSecondaryNoModel: 'NO-MODEL',
metricsSecondaryNoAccount: 'NO-ACCT',
```

保留旧 key（`overviewProfilesLabel` 等）直至确认无回滚需求。

---

## 6. 迁移步骤（推荐顺序）

1. **新建分支** `feature/claude-profiles-dashboard-bands`
2. **先改骨架** — 在 `ClaudeProfilesOverview.vue` 实现身份带 + 指标带基本结构，用硬编码数据跑通布局。
3. **接线 computed** — 把 summary / currentProfile 数据映射到新结构，验证空状态（无 profile / 无 endpoint）。
4. **接 i18n** — 补 zh-CN / en-US，`translateWithFallback` 覆盖。
5. **样式细化** — 实现 4.3 规格的 token、竖线分隔、断点降级。
6. **动效** — `animate-slide-up` + stagger delay + reduced-motion fallback。
7. **双主题验证** — dark / light 下手动检查对比度，必要时加一条 dark 特化规则。
8. **smoke test** — 补 `ccr-ui/tests/claude-profiles-overview.smoke.test.ts` 最小用例：
   - 无 profile 时显示 fallback hint
   - 有 profile 时显示身份带三行
   - 指标带渲染 8 个 `role="listitem"`
9. **视觉回归截图** — 1280 / 960 / 640 三档 viewport 各截一张。
10. **PR 自检**：`just frontend-check` + `just lint-strict`。

---

## 7. 预期收益

| 指标 | 现状 | 优化后 |
|------|------|--------|
| dashboard 顶部高度 | ~380px | ~230px（**-40%**） |
| 非信息空白面积占比 | ~28% | <8% |
| 一屏内可见 profile 卡片数 | 3–4 | 5–6 |
| 维度断点 | 2 列 grid 在 1024-1280 过渡不自然 | 8→4→2 线性降级 |
| 视觉层级复读 | tiles + ribbon 重复 | 单层 ticker，primary/secondary 靠 typography 区分 |

---

## 8. 风险与回滚

- **风险 R1**：部分 profile 缺 endpoint / model，身份带第三行显得稀疏 → **对策**：缺项降级为 `—` 并配 muted 色。
- **风险 R2**：指标带 8 格在窄屏拥挤 → **对策**：已在 4.5 指定 `< 1280` 改 4 格、`< 640` 改横向 scroll。
- **风险 R3**：Codex / Droid 共用同款 overview → **对策**：本次仅改 Claude 的 Overview 组件；其他平台下一轮同构。
- **回滚**：`ClaudeProfilesOverview.vue` 单文件改动，可直接 revert 单 commit。

---

## 9. 开始实施

确认方案后，建议执行顺序：

```bash
git checkout -b feature/claude-profiles-dashboard-bands
# Step 2–4：实现核心结构
# Step 5–7：样式与动效细化
just frontend-check
# Step 8：smoke test
bun --cwd ccr-ui run test
```

> 实施过程中若需要调整节奏（例如只做 Phase 1：身份带重构，Phase 2：指标带合并），在 PR description 中标注即可。
