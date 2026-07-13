# 字体设置不生效诊断

## 结论

设置链路正常；截图中的界面字体没有产生预期视觉变化，是因为预设名 `Source Han Sans SC` 与本机实际安装字体族 `Source Han Sans CN` 不一致，浏览器按设计静默回退到 `MapleBright`。`SF Mono` 在本机存在，代码字体变量也已正确应用；它与默认 `Cascadia Code` 在小字号英文样本上的差异不明显。

结构性原因是预设列表只提供跨平台字体名字，不枚举或验证本机字体。不可用的选项仍可被选择、保存和显示，页面也没有“已回退”状态。

## 可重复证据

### 1. 设置链路 smoke tests

```powershell
cd ccr-ui
bunx vitest run --config vitest.smoke.config.ts tests/font-preferences.smoke.test.ts
```

结果：1 个文件、6 个测试全部通过。覆盖 sanitize、持久化、CSS 变量写入、重置和首帧恢复。

### 2. Windows 可见字体族

使用 `System.Drawing.Text.InstalledFontCollection` 检查截图中的两个选择：

- `Source Han Sans SC`: 不存在
- `SF Mono`: 存在
- 最接近的已安装字体族：`Source Han Sans CN`

读取 `C:\Windows\Fonts\SourceHanSansCN-Regular.otf` 的 OpenType name table 进一步确认：family/name ID 1 为 `Source Han Sans CN`，PostScript name 为 `SourceHanSansCN-Regular`。

### 3. 真实设置页运行态

生产构建：`bun run build:web`，通过 `http://127.0.0.1:4174/settings` 验证。

选择 `Source Han Sans SC` + `SF Mono` 后：

- 下拉值正确。
- `--font-sans`: `"Source Han Sans SC", var(--font-sans-base)`
- `--font-brand`: `"Source Han Sans SC", var(--font-brand-base)`
- `--font-mono`: `"SF Mono", var(--font-mono-base)`
- 正文计算栈以 `Source Han Sans SC` 开头，随后是 `MapleBright` 等 fallback。
- 代码预览计算栈以 `SF Mono` 开头，随后是 `Cascadia Code` 等 fallback。
- 刷新后两个下拉仍保持选中，证明 localStorage/首帧恢复正常。

将界面字体改为自定义 `Source Han Sans CN` 后，`--font-sans` 和正文计算栈立即改用该真实字体族名。

## 为什么测试会通过

`tests/font-preferences.smoke.test.ts` 验证的是字符串持久化和 CSS custom property 的组成，不验证宿主系统是否真的存在该字体族。浏览器 `document.fonts.check()` 对 `Source Han Sans SC` 也返回 `true`，因为 fallback 能完成渲染；它不能证明首选系统字体被采用。

## 假设结果

1. **确认**：`Source Han Sans SC` 预设与本机字体族名不匹配，触发静默 fallback。
2. **确认**：`SF Mono` 已安装且变量已应用，视觉差异不明显不等于未生效。
3. **排除**：`--cp-mono`、`--palette-mono`、`--font-family-mono` 等主要别名最终均引用 `--font-mono`，未发现代码字体通道断链。
4. **排除**：下拉事件、CSS 变量写入、持久化或首帧恢复失败。

## 修复方向

1. **推荐，最小修复**：保留 `Source Han Sans SC`，同时加入 `Source Han Sans CN`，并用文案区分不同字体包暴露的家族名；不要为修复当前 Windows 安装而全局替换 `SC`。
2. **更可靠**：对预设做本机可用性检查，不可用项显示“将回退”，避免下拉框的选中状态被误解为字体已采用。
3. **体验完整但成本更高**：在 Tauri 后端枚举本机字体，只显示可用项，或对不可用项显示“将回退”。这超出原字体任务明确的 MVP 范围。

当前 workaround：界面字体选择“自定义”，输入 `Source Han Sans CN`；代码字体 `SF Mono` 可保留。

## `Source Han Serif SC VF` 显示偏虚

### 结论

`Source Han Serif SC VF` 已经正确应用。偏虚主要来自字体与使用场景不匹配，而不是设置链路故障：它是思源宋体（衬线正文排版字体），细横与粗竖反差明显；CCR 将它用于大量 11–14px 的深色 UI 文本，在 150% 缩放与灰度抗锯齿下，细笔画落在半像素并被摊成灰边。

### 字体文件证据

- family：`Source Han Serif SC VF`；中文 family：`思源宋体 VF`。
- TrueType variable font，只有 `wght` 轴：250–900，默认 250。
- 无 `gasp`、`fpgm`、`prep`、`cvt `、`hdmx`、`VDMX`、`LTSH` 表。
- `maxSizeOfInstructions = 0`；65,535 个 glyph 的 hinting instruction 数为 0。

缺少小字号 hinting 不代表字体文件损坏，但意味着栅格化器无法针对 11–14px 将细笔画主动对齐到设备像素；衬线字比无衬线字更容易显软。

### 运行态证据

- `devicePixelRatio = 1.5`，11px 文本对应 16.5 个设备像素，天然存在半像素采样。
- 页面根样式计算为 `-webkit-font-smoothing: antialiased`，`text-rendering: optimizelegibility`。
- 正文与多数说明文字为 400，eyebrow 为 500；未发现应用把所有文字错误固定为 250。
- 全局 `--tracking-normal = -0.016em`；16px 时为 `-0.256px`，并作为绝对计算值继承给部分 12/14px 文本，进一步压缩衬线细节。
- 暗色界面大量使用 secondary/muted 文本色，降低了细笔画的局部对比度。

### 已排除

- Windows ClearType：已开启（`FontSmoothing=2`、`FontSmoothingType=2`）。
- Tauri 透明窗口：主窗口配置为 `transparent: false`。
- 字体没有应用：根变量与正文计算栈均以 `Source Han Serif SC VF` 开头。

### 建议

1. **推荐**：界面字体使用 `Source Han Sans CN` 或 `Noto Sans SC`；它们的无衬线字形更适合密集小字号 UI。
2. 若仍要提供 `Source Han Serif SC VF`，应作为明确标注的“衬线/阅读型”预设保留，不把它当作 `Source Han Sans SC` 的等价替代。
3. 不建议为了该字体全局加粗或强制 `font-variation-settings: 'wght' ...`，否则会覆盖现有 400/500/560/620 字重层级。
4. 是否移除 Windows 下的 `-webkit-font-smoothing: antialiased` 应作为独立视觉实验验证；它可能改善子像素锐度，但会影响所有字体和平台，不应混入本次最小选项修复。

## 实施结果

- 保留 `Source Han Sans SC`，新增 `Source Han Sans CN` 与 `Source Han Serif SC VF`。
- `font-preferences.smoke.test.ts` 锁定三个 family 均存在且预设数组无重复。
- 字体契约明确 SC、CN 与 Serif 是不同的 OS-visible family，不得互相替换。
- 浏览器 `/settings` 验证三个选项同时可见；此前作为自定义值保存的 `Source Han Serif SC VF` 会自动归类为正式选项。
- 选择 `Source Han Sans CN` 后刷新保持，输入框不再展开，`--font-sans` / `--font-brand` 与正文计算栈均正确。

验证结果：目标 smoke 48/48、i18n 23/23、type-check、lint、生产构建均通过；lint 仍报告一个与本任务无关的既有 `DashboardSignalStream.vue:61` raw-text warning。
