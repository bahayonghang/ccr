# 技术设计：外观设置页重排与预览一致性

## 1. 前后差异表（AC1 的直接产物）

实施第一步填完，每行给出「保持不变 / 仅换版式 / 行为改变」三选一。
分类为「行为改变」的行必须写明改了什么。

| UI 块                      | 现状                                       | 分类     | 说明                                                                                          |
| -------------------------- | ------------------------------------------ | -------- | --------------------------------------------------------------------------------------------- |
| 主题三选项 `ThemeOption`   | 已存在，独立呈现为纵向列表                 | 仅换版式 | 仍用既有 `ThemeOption` 与 props；改为同一张卡内三列栅格，选中态加粗边框 + CSS 勾标（非仅靠颜色） |
| `system` 解析结果说明      | 已存在                                     | 保持不变 | 仍在 `theme === 'system'` 时显示 `resolvedHint`                                               |
| flavor 两张卡 `FlavorCard` | 已存在，独立呈现                           | 仅换版式 | 仍用既有 `FlavorCard` 与 props；去掉第二张卡头，并入同一张卡，选中态加粗边框 + 勾标           |
| flavor 预览色条            | 已存在，取值来自 `flavorPreviewStyle`      | 保持不变 | 仍走 `FLAVOR_PREVIEW_TOKENS`；本任务不改正文取值                                              |
| UI 字体下拉 + 自定义输入   | 已存在，`data-testid="settings-font-ui"`   | 保持不变 | 下拉、自定义展开与 `settings-font-ui` 均保留                                                  |
| 代码字体下拉 + 自定义输入  | 已存在，`data-testid="settings-font-code"` | 保持不变 | 下拉、自定义展开与 `settings-font-code` 均保留                                                |
| 混排预览                   | 已存在于各下拉正下方                       | 仅换版式 | 收到字体卡底部共享预览：中英混排 + 等宽数字/金额样例                                          |
| 留空回退提示 callout       | 已存在                                     | 仅换版式 | 保留 callout，文案对齐设计稿（留空回退、缺字形逐级回退）                                      |
| 主题卡与 flavor 卡合并     | 已在同一 `app-settings-card` 但双卡头      | 仅换版式 | 合并为单一「明暗与底色族」卡头，flavor 改为卡内分隔后的子标题                                 |
| 预览一致性机制             | 不存在                                     | 行为改变 | 新增 `flavor-preview-consistency.smoke.test.ts`，解析四面作用域并与预览表比对                 |

预期结论：多数行为「保持不变」或「仅换版式」。
若填表后发现「行为改变」的行只有一两条，说明本任务的实际增量确实小，
这是正确的结论，不需要为了让任务显得饱满而扩大范围。

## 2. 改动范围

| 文件                                                         | 改动                                 |
| ------------------------------------------------------------ | ------------------------------------ |
| `ccr-ui/src/features/configs/settings/AppearanceSection.tsx` | 分区合并与版式                       |
| `ccr-ui/src/features/configs/styles/app-settings.css`        | 版式对应样式                         |
| `ccr-ui/tests/flavor-preview-consistency.smoke.test.ts`      | 新增：预览取值与 `tokens.css` 一致性 |

`ccr-ui/src/features/configs/lib/flavorPreview.ts` **不改**——其 20 个取值在令牌子任务后仍然正确，
本任务只为它加守护。若填表后发现取值确有错，才改，并把该文件加入本表。

不改 `themeBootstrap.ts`、`fontPreferences.ts`。

## 3. 预览一致性机制

### 3.1 问题

`FLAVOR_PREVIEW_TOKENS` 是 `tokens.css` 的手工镜像：

```ts
neutral: {
  light: { base: '#e8e9ec', elevated: '#f2f3f5', surface: '#fbfcfd', text: '#191b20', muted: '#5f646e' },
  dark:  { base: '#131316', elevated: '#1a1b1f', surface: '#22242a', text: '#f2f3f5', muted: '#9ba1ab' },
},
clay: {
  light: { base: '#ebe1d0', elevated: '#f5eee1', surface: '#fefaf2', text: '#31241c', muted: '#715d4c' },
  dark:  { base: '#17120f', elevated: '#221b18', surface: '#2a221e', text: '#f3eadf', muted: '#b9a695' },
}
```

`tokens.css` 改了，这里不会跟着改，设置页预览会静默偏离真实呈现。

### 3.2 方案：测试守护，不做运行时求值

不改为运行时 `getComputedStyle` 求值——那需要在 DOM 里插带 `data-theme` / `data-flavor` 的探针元素，
改动面远大于收益，且在 SSR / 测试环境下要额外兜底。

改为新增一个解析型测试：

1. 读 `ccr-ui/src/styles/tokens.css` 文本。
2. 按四个作用域选择器切块：`:root`、`[data-theme='dark']`、`[data-flavor='clay']`、`[data-theme='dark'][data-flavor='clay']`。
3. 从每块中取 `--color-bg-base` / `--color-bg-elevated` / `--color-bg-surface` /
   `--color-text-primary` / `--color-text-muted` 的取值。
4. 与 `FLAVOR_PREVIEW_TOKENS` 的对应项逐个比对（大小写不敏感）。

注意作用域的继承关系：`[data-flavor='clay']` 只覆盖部分令牌，未覆盖的继承自 `:root`。
测试需要按「块内有定义则取块内值，否则取上层值」的规则解析，不能假设每块都定义了全部五个令牌。
实施时先读实际文件确认哪些令牌在哪块中被定义。

失败时的报错要指出是哪个 flavor、哪个主题、哪个令牌不一致，方便修复。

### 3.3 与令牌子任务的关系

令牌子任务只改边框与圆角取值，不改 `--color-bg-*` 与 `--color-text-*`。
因此这个测试在令牌子任务合入后应当直接通过。
若不通过，说明令牌子任务超出了它自己声明的范围，属于需要回头处理的问题。

## 4. 分区合并

设计稿把主题与 flavor 放进同一张卡。合并只是容器与栅格的改变：

```
[卡：明暗与底色族]
  [明暗：三个 ThemeOption]
  [system 解析结果说明]
  [分隔]
  [底色族：两张 FlavorCard]

[卡：界面与数据字体]
  [界面字体下拉 + 自定义输入]
  [数据字体下拉 + 自定义输入]
  [混排预览]
  [留空回退提示]
```

`ThemeOption` / `FlavorCard` 的内部实现与 props 不动，只改它们的容器与外层样式。
既有 `data-testid` 全部保留，`app-settings-view.smoke.test.tsx` 不应回归。

## 5. 选中态非颜色可辨（R7）

选中的 `ThemeOption` 与 `FlavorCard` 除强调色外，另加边框加粗与一个勾选图标。
灰度模拟下依靠边框与图标判断。

## 6. 回滚

```bash
git checkout -- ccr-ui/src/features/configs/settings/AppearanceSection.tsx \
  ccr-ui/src/features/configs/styles/app-settings.css
rm -f ccr-ui/tests/flavor-preview-consistency.smoke.test.ts
```

一致性测试可单独保留——它不依赖版式改动，独立有价值。
</content>
