# 设置系统重设计 — 技术设计

> 依赖：`07-28-color-system-rebuild` 的新值域与令牌。约束：testid 尽量保持；bootMessages 三处同步；i18n 无 `{|}`。

## 1. 页面结构（目标）

```
/settings
├── Hero（图标 + eyebrow/title/description + 摘要 pill：runtime/version/theme/locale/sidebar px）
└── grid
    ├── 左：section 导航（sticky，4 项，重设计选中态）
    └── 右：sections
        ├── appearance
        │   ├── Card「主题模式」→ 分段控件 light/dark/system（带解析结果指示）
        │   ├── Card「界面风格 Flavor」→ 3 项真实 token 预览卡
        │   ├── Card「强调色 Accent」→ 4 项实心按钮预览
        │   └── Card「字体」→ 行为不变，样式更新
        ├── language（1 Card，样式更新）
        ├── shell（1 Card，5 行，样式更新）
        └── diagnostics（1 Card，样式更新）
```

- 不新增 section、不改路由；`settings-section-{key}` 导航 testid 保留。
- 1365 行单文件保持单文件（与现状一致），样式块按新契约重写；如超 1600 行可抽 1 个 `settings/AppearancePreviewSwatch.vue` 子组件。

## 2. 外观区交互设计

- **主题分段控件**：`role="radiogroup"` + 三 `role="radio"`；`settings-theme-{light|dark|system}` testid 保留；system 选中时显示"当前解析：暗色/亮色"。
- **flavor 预览卡**：每项一个 button（`settings-flavor-{neutral|clay|catppuccin}`），内部 mini 预览：
  ```html
  <div class="flavor-preview" :data-preview-flavor="opt.value">
    <div class="fp-surface"><span class="fp-text">Aa</span><span class="fp-muted">Aa</span><i class="fp-accent"/></div>
  </div>
  ```
  预览样式用**作用域覆写**：在预览元素上内联/作用域设置目标 flavor 的 `--color-bg-base` 等令牌子集（从 tokens.css 锚点值静态复制到组件样式，注释标注"与 tokens.css 同步"），让三张预览并排呈现各自 flavor 的真实表面/文本/accent —— 不需要切换全局主题即可对比。catppuccin 预览按当前 resolvedTheme 显示 latte 或 mocha。
- **accent 预览**：每项显示一个实心按钮样例（`background: var(--accent 候选色); color: var(--contrast)`，同样作用域覆写）+ 名称；`settings-accent-{clay|sage|sky|mauve}` testid 保留。
- 选中态：border-accent 2px + 实心小圆点，替代现状的低对比描边。

## 3. i18n 变更

- 新增/重写键（双 locale + bootMessages 副本）：
  - `settings.appearance.flavorOptions.neutral.{label,description}`
  - `settings.appearance.flavorOptions.catppuccin.{label,description}`（说明 light→latte、dark→mocha 自适应）
  - `settings.appearance.accentOptions` 下四个保留键的描述重写
  - 主题分段控件 aria/提示文案
- 删除键：flavor `paper|graphite|latte|frappe|macchiato|mocha` 独立项、accent `sand|amber|rose|slate` 项。
- `bootMessages.ts` 的 settings 副本整段与语言包 diff 对齐。

## 4. dock 更新

- `MainLayout.vue` dock 的主题摘要：`flavor` 显示名映射改为 3 值（`settings.dock` 或复用 flavorOptions label）；样式按不透明表面契约。

## 5. 测试更新

- `app-settings.smoke.test.ts`：
  - 选项枚举更新：`settings-flavor-{neutral,clay,catppuccin}`、`settings-accent-{clay,sage,sky,mauve}`。
  - 持久化断言：写入 `ccr-flavor=catppuccin` 后 `data-flavor=catppuccin`、`data-resolved-flavor` 随主题解析。
  - 预览断言（AC2）：flavor 预览元素存在且其样式引用 token 变量。
- `main-layout-theme-stage.smoke.test.ts`：dock 摘要断言更新为 3 值显示名。
- 保留既有 testid 命名；任何删除的 testid 在测试中同步删除。

## 6. 风险

- bootMessages 副本遗漏 → 首屏旧文案：AC3 用键集合一致性测试兜底（若现有测试无此断言，新增一个轻量对比）。
- 预览卡静态复制的 token 子集与 tokens.css 漂移 → 组件注释 + AC2 断言缓解；不追求运行时动态解析（避免在设置页注入第二份 palette）。
