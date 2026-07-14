# ccr-ui 字体设置与 fallback

## Goal

在 ccr-ui「全局设置 → 外观」中，让用户自定义**界面字体**与**代码字体**，并像 Codex 一样保留**回退栈**：用户选中的字体被 prepend 到现有内置字体栈头部；当该字体缺失或缺字形（尤其 CJK）时，浏览器沿栈自动回退，界面永不出现豆腐块或整段无字形。

User value：高频使用者可按本机已装字体与个人偏好统一工作台观感（正文/标题走界面字体，代码/数值走代码字体），而不牺牲既有的 MapleBright / Cascadia 排版底座与稳定性。

## Confirmed Facts（已通过代码勘察确认）

- 字体三轨全部是 CSS 变量，定义在 `ccr-ui/src/styles/tokens.css` `:root`：
  - `--font-sans`（正文，MapleBright 栈）、`--font-brand`（标题/展示）、`--font-mono`（代码/数值，Cascadia 栈）。
- Tailwind `font-sans/brand/mono` 直接映射到这三个变量（`ccr-ui/tailwind.config.ts`）；`--font-brand` 在 15+ 组件/视图中被使用，覆盖它对标题生效。
- 既有 theme/flavor/accent 偏好是**纯 localStorage**，不写 Tauri 后端：`DesktopShellPreferences` 仅含 `confirm_before_exit/close_to_tray/open_panel_on_tray_click`。字体应复刻此模式。
- 偏好链路模板：`utils/themeBootstrap.ts`（read/persist/apply）→ `stores/shellPreferences.ts`（ref + setter）→ `views/AppSettingsView.vue`（UI）→ `index.html` 内联脚本（首帧防 FOUC）→ `i18n/locales/{zh-CN,en-US}.ts`（`settings.appearance.*`）。
- 无任何现成系统字体枚举命令（后端仅 `codex_auth.rs` 偶然命中 "font" 字样）。
- 测试底座：`tests/theme-bootstrap.smoke.test.ts`（含一条会执行 `index.html` 引导脚本并断言 `:root` 结果的用例）、`tests/settings-i18n.smoke.test.ts`（zh/en key 平价）、`tests/app-settings.smoke.test.ts`。

## Decisions（已与用户确认）

- **两个控件**：界面字体 → 覆盖 `--font-sans` + `--font-brand`；代码字体 → 覆盖 `--font-mono`。对齐 Codex 双字段模型。
- **预设下拉 + 自定义输入 + 实时预览**；**不做**原生系统字体枚举（font-kit / queryLocalFonts 超出 MVP）。
- **纯 localStorage**，不改 Tauri 后端 / ccr-config。
- **全局字体**（非按明暗主题分别设定）。

## Requirements

1. 外观区新增「字体 / Typography」卡片，含界面字体、代码字体两个控件。
2. 每个控件 = 预设下拉（含「系统默认」= 不覆盖，回到内置栈）+ 可手动键入任意字体名。
3. 每个控件下方有**实时预览**：界面字体预览用中英混排 + 数字串；代码字体预览用含 `0O il1{}=>` 的代码片段（等宽可辨识）。
4. 选中字体后立即全局生效（无需重启），并 prepend 到对应内置回退栈头部：
   - `--font-sans` / `--font-brand` → `"<界面字体>", <各自内置栈>`
   - `--font-mono` → `"<代码字体>", <内置 mono 栈>`
5. 选择「系统默认」或清空 → 移除覆盖，回到内置栈。
6. 偏好写入 localStorage 并在启动首帧前应用，无字体闪烁（FOUC）。
7. 用户键入值需**净化**后再进 CSS 变量（防 CSS 注入 / 破坏引号串），并限制长度。
8. 新增 i18n 文案在 zh-CN / en-US 双语齐备。

## Acceptance Criteria

- [ ] 设置页可分别设定界面字体与代码字体；切换后当前页正文、标题、代码/数值区字体即时变化。
- [ ] 选中一个本机**不存在**的字体名后，界面仍正常渲染（回退到内置栈，无豆腐块）——可通过断言计算后的 `--font-sans` 形如 `"Bogus", <base>` 且 DOM 可见文本无异常验证。
- [ ] 选择「系统默认」后，`--font-sans/brand/mono` 恢复为内置栈（覆盖被移除）。
- [ ] 刷新/重启后偏好保持，且首帧即为目标字体（引导脚本已应用），无可见字体跳变。
- [ ] 代码字体仅影响 `--font-mono` 通道，不改变正文；界面字体仅影响 `--font-sans/brand`，不改变代码区。
- [ ] 恶意输入（含 `"`、`;`、`}`、`<`、超长串）被净化，不产生 CSS 注入且不破坏样式。
- [ ] `just frontend-check-quick` 通过：typecheck + lint + 新增/扩展的 smoke 测试（字体 util、i18n 平价、引导脚本断言）全绿。

## Out of Scope

- 原生已装字体枚举（font-kit crate / `queryLocalFonts()`）。
- 字号 / 字重 / 行高 / 字间距的自定义（仅字体族）。
- 按明暗主题分别配置字体。
- 将字体偏好同步到 Tauri 后端 / WebDAV / 多设备。
- `#app-loader` 启动闪屏字体（硬编码 300ms splash，保持不变；见 design.md 权衡）。

## Open Questions

- （已解决）预设下拉字体清单 → 采用 design.md §4 推荐清单（界面 9 项 / 代码 8 项）。规划阶段无剩余阻塞项。
