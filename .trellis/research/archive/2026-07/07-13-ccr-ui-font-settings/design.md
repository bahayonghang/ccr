# Design — ccr-ui 字体设置与 fallback

## 1. 架构与边界

复刻既有 theme/flavor/accent 偏好链路，新增一条「字体」偏好。**纯前端、纯 localStorage**，零 Rust/Tauri 改动。

```
index.html 引导脚本 ──(首帧前应用覆盖)──┐
                                        ▼
utils/fontPreferences.ts  ──apply──▶  :root inline style (--font-sans/brand/mono)
   ▲ read/persist/sanitize                ▲
   │                                      │ 依赖回退基座
stores/shellPreferences.ts (ref+setter)   tokens.css: --font-*-base
   ▲
views/AppSettingsView.vue「字体」卡片 (下拉+输入+预览)
   ▲
i18n: settings.appearance.typography.*
```

新增文件：`ccr-ui/src/utils/fontPreferences.ts`（也可并入 `themeBootstrap.ts`；独立文件更内聚，推荐独立）。

## 2. CSS 变量：override + fallback 机制（核心）

**目标**：覆盖时保留原栈作为回退尾；重置时精确回到原栈。

### tokens.css 重构（最小侵入）

把三条字体轨的字面栈抽成 `-base` 变量，原变量改为引用 base：

```css
--font-sans-base:
  "MapleBright", "SF Pro Text", "PingFang SC", "Microsoft YaHei UI",
  "Microsoft YaHei", sans-serif;
--font-brand-base:
  "SF Pro Display", "Segoe UI Variable Display", "PingFang SC",
  "Microsoft YaHei UI", "Microsoft YaHei", sans-serif;
--font-mono-base:
  "Cascadia Code", "Cascadia Mono", "SFMono-Regular", ui-monospace, "Consolas",
  "MapleBright", monospace;
--font-sans: var(--font-sans-base);
--font-brand: var(--font-brand-base);
--font-mono: var(--font-mono-base);
```

所有既有 `var(--font-sans/brand/mono)` 消费端**无需改动**——语义等价。

### apply（覆盖）

用户选界面字体 `UI`、代码字体 `CODE` 时，在 `document.documentElement` 上写**内联** custom property：

```js
root.style.setProperty("--font-sans", `${quoted(UI)}, var(--font-sans-base)`);
root.style.setProperty("--font-brand", `${quoted(UI)}, var(--font-brand-base)`);
root.style.setProperty("--font-mono", `${quoted(CODE)}, var(--font-mono-base)`);
```

- 内联样式特异性 > stylesheet `:root` 规则 → 覆盖生效。
- `var(--font-*-base)` 在 tokens.css 加载后惰性解析 → 引导脚本先跑也安全（首帧无字体时浏览器用系统字体，tokens.css 落地即补齐 base 尾）。

### reset（回退到系统默认）

值为空 / 哨兵 `system` → `root.style.removeProperty('--font-sans')`（brand/mono 同理）→ 解析回 stylesheet 的 `var(--font-*-base)`。

## 3. 数据契约

- localStorage keys：`ccr-font-ui`、`ccr-font-code`（对齐 `ccr-theme/ccr-flavor/ccr-accent` 命名族）。
- 值语义：空串或缺失 = 系统默认（无覆盖）；非空 = 用户字体族名（已净化）。
- `SupportedFontPreference = { ui: string; code: string }`（空串代表默认）。

### 净化 sanitizeFontFamily(input) → string

用户输入进入 CSS 值，必须净化（安全项，呼应仓库「masking/atomic/安全优先」基调）：

1. `trim()`；
2. 移除 `"` `'` `` ` `` `\` `;` `{` `}` `<` `>` `(` `)` 及控制字符（这些要么破坏引号串，要么可能构造注入/额外声明）；
3. 折叠内部多余空白；
4. 截断到 `MAX_FONT_NAME_LEN = 64`；
5. 结果为空 → 视为默认（不覆盖）。

`quoted(name)` = `"` + sanitized + `"`（净化后必不含双引号，引号串安全）。即便如此，`setProperty` 对非法值本就静默失败，是第二道防线。

## 4. UI 设计（AppSettingsView 外观区新增卡片）

置于 flavor / accent 卡片之后，沿用 `app-settings-card--tight` 视觉规格。品牌基调「克制/编辑式」，不引入新视觉分支。

每个控件一行：

- 左：标题 + 说明（`app-settings-row__copy`）。
- 右：预设 `<select>`（原生，含「系统默认」+ 预设项 + 「自定义…」）；选「自定义…」时展开一个 `<input type="text">` 键入字体名。若当前持久值不在预设列表中，下拉显示为「自定义」并回填输入框。
- 行下：预览条（`font-family` 内联为组合值）。
  - 界面字体预览串：`现代化 AI 工作台 AaBbGg 0123456789`
  - 代码字体预览串：`const x = () => { 0O il1 }`

预设清单（已确认）：

- 界面字体：系统默认 / MapleBright（内置）/ PingFang SC / Microsoft YaHei / Noto Sans SC / Source Han Sans SC / Inter / SF Pro / Segoe UI。
- 代码字体：系统默认 / Cascadia Code（内置）/ JetBrains Mono / Fira Code / SF Mono / Consolas / Menlo / Source Code Pro。

无障碍：`<select>`/`<input>` 关联 `<label>`；预览条 `aria-hidden` 或提供文本说明。

## 5. 首帧 FOUC 处理（index.html 引导脚本扩展）

在既有「主题预初始化」IIFE 内追加：

```js
var fUi = (localStorage.getItem("ccr-font-ui") || "").trim();
var fCode = (localStorage.getItem("ccr-font-code") || "").trim();
// 内联同款净化（与 util 保持一致的字符白名单，避免重复大逻辑：仅做基础 strip）
function clean(s) {
  return s
    .replace(/["'`\\;{}<>()]/g, "")
    .replace(/\s+/g, " ")
    .slice(0, 64)
    .trim();
}
fUi = clean(fUi);
fCode = clean(fCode);
if (fUi) {
  root.style.setProperty("--font-sans", '"' + fUi + '", var(--font-sans-base)');
  root.style.setProperty(
    "--font-brand",
    '"' + fUi + '", var(--font-brand-base)',
  );
}
if (fCode) {
  root.style.setProperty(
    "--font-mono",
    '"' + fCode + '", var(--font-mono-base)',
  );
}
```

`theme-bootstrap.smoke.test.ts` 中执行引导脚本的用例需同步扩展断言（见 implement.md）。

**权衡**：`#app-loader` 与 `.loader-text` 在内联 `<style>` 里硬编码了 MapleBright 栈，不读 CSS 变量。让其跟随需改写为 `var(--font-sans)`，但 loader 仅 ~300ms 且早于 Vue 挂载，收益低、且会把字体覆盖耦合进闪屏。**决定：loader 字体保持不变**，列入 Out of Scope。用户可见的应用界面（挂载后）字体正确。

## 6. Store 集成（shellPreferences.ts）

- 新增 ref：`uiFont`、`codeFont`（`readStoredFontPreference()` 初始化）。
- `initializeTheme()` 末尾追加 `applyFontsToDocument(uiFont.value, codeFont.value)`（复用现有「创建即应用」时机）。
- 新增 setter：`setUiFont(name)` / `setCodeFont(name)` → 净化 + persist + apply + 更新 ref。
- `return` 暴露 `uiFont/codeFont/setUiFont/setCodeFont`。

## 7. 兼容 / 迁移 / 回滚

- 纯新增能力，旧用户无 `ccr-font-*` 键 → 走默认（内置栈），行为与今日完全一致。**无迁移**。
- tokens.css 抽 `-base` 是等价重构；风险面仅在拼写。
- 回滚：删除新 util/store 分支/UI 卡片/i18n key，并把 tokens.css 三行 `var(--font-*-base)` 还原为字面栈即可，无数据残留（localStorage 冗余键无害）。

## 8. 关键权衡小结

| 决策        | 取                           | 舍                          | 理由                                   |
| ----------- | ---------------------------- | --------------------------- | -------------------------------------- |
| 覆盖机制    | 内联 `:root` custom property | 改写全部消费端 / 新增 class | 一处生效、易重置、零消费端改动         |
| 回退保留    | prepend + `var(--*-base)` 尾 | 直接替换 `--font-*`         | 缺字形自动回退，正是 Codex 行为        |
| 存储        | localStorage                 | Tauri 后端                  | 对齐既有 appearance 偏好，MVP 边界清晰 |
| 输入        | 预设 + 自定义文本            | 原生字体枚举                | 免原生依赖，跨平台零成本               |
| 安全        | 净化 + 引号包裹 + 长度限     | 直接注入                    | 防 CSS 注入/破栈                       |
| loader 字体 | 不动                         | 跟随覆盖                    | 收益/复杂度不成比例                    |
