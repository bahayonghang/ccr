# GPUI 平台卡片 · Windows 原生验收

设计和验收对应 [规格 #225](https://github.com/bahayonghang/ccr/issues/225)、[配置方案 #226](https://github.com/bahayonghang/ccr/issues/226) 和 [Antigravity 工作台 #227](https://github.com/bahayonghang/ccr/issues/227)。[四平台原生卡面摘图](four-platforms.webp)

## 方案目录

使用同一份 27 项合成目录，在 125% 系统缩放下从侧栏进入各平台。原版与改版的 1930 × 980 逻辑像素截图处于相同滚动位置。较宽的卡内成对排列字段，较窄时逐行显示；卡尾「应用」和「启用并应用」一直有独立底色和轮廓。标题沿用用户选定的字族，升至 17px 半粗，缺省值与标签比真实值更轻。[旧宽窗](before-lightwide-claude.webp) · [新宽窗](after-lightwide-claude.webp) · [960 × 700 深色、20px 字号](after-darklarge-claude.webp)

四个平台沿用同一款斜向磨砂玻璃材质，各用独立色板。静态渐变从 95% 不透明度到 92%，上沿加 1px 内亮边。未逐卡采样背景或调用系统模糊。[Claude 琥珀橙](after-light-claude.webp) · [Codex 清蓝](after-light-codex.webp) · [Grok 石墨紫灰](after-light-grok.webp) · [Grok 暗色](after-dark-grok.webp)

在 1280 × 800 的浅色窗口先预览 B，用键盘聚焦 C 的独立按钮并按 Enter。详情仍显示 B，回执与磁盘的 current_config 都是 C。在 960 × 700 深色下从 B 的详情按 Esc 返回，再按 Enter 打开同一张卡。另一份干净目录中，鼠标移入并按下「启用并应用 D-Disabled」，释放后 SQLite 的当前记录为 D，enabled 为 1。鼠标移入前后的像素差异局限于按钮区域。[C 按钮的焦点](after-light-focus.webp) · [C 已应用、B 仍预览](after-light-applied.webp) · [按钮的常态、悬停与按下](button-states.webp) · [D 的操作结果](after-lightpointer-pointer-applied.webp)

20px 字号下的虚拟列表测试保留了 10,000 条数据及滚动、详情、焦点、Tab、应用等断言。卡片增高后，被测卡片的中心在滚动视口底边外约 4px，按中心模拟点击未能打开详情。测试改点卡片与视口交集的中心，并先断言详情确实打开，再验证返回焦点。

## Antigravity 工作台

当前状态和最近 30 天用量使用极光青，在宽窗并排，窄窗与大字号叠放。填入独立的合成用量库后，第一次大窗口复核发现请求标签与读数相隔约 1,550 逻辑像素。调整后的汇总区在 14px 基准字号下最多 460px，数值右对齐。实际打开工作台、用键盘刷新后，原有状态、用量与导航均可用。保存的示例显示合成的 101 次请求、100 tokens 和 0.12 美元；另用缺少用量库的夹具查验了原有提示。[改造前同尺寸](before-lightwide-antigravity.webp) · [调整后的宽窗](revised-lightwide-antigravity.webp) · [窄窗深色、大字号](revised-darklarge-antigravity.webp)

最终 release 从真实侧栏重走四个平台。在 Antigravity 页面打开期间，将独立夹具的请求数由 101 更新为 137，键盘激活「刷新状态」后，汇总和按日读数均更新到 137，焦点保留在刷新按钮。release 的 SHA-256 为 `3e42adb3c6c79119ab2d39455185a2864cb7f36f0e7e593986f7f6bcd180b79b`。[刷新后的原生窗口](release-antigravity-refreshed.webp)

## 对比度与复核

数值依次为文字、按钮边界和键盘焦点的最小对比度。涵盖实际主题色、纸面在光场上的合成、渐变两端和按钮的常态、悬停、按下。对应回归位于 GPUI 主题测试。

| 平台 | 浅色 | 深色 |
| --- | --- | --- |
| Claude | 6.69 / 3.87 / 12.77 | 5.90 / 3.52 / 5.99 |
| Codex | 5.86 / 3.45 / 12.10 | 5.81 / 3.51 / 6.17 |
| Grok | 6.67 / 3.70 / 11.60 | 6.38 / 3.82 / 6.37 |
| Antigravity | 5.74 / 3.43 / 12.72 | 6.27 / 3.74 / 5.86 |

依据 Design Atlas 的 `style:airy-glass` 和项目既有设计合同。Rust 源目录无 Web 静态扫描器支持的文件，`slop_check` 返回 2；按原生清单人工检查层次、真实内容、宽窄布局、控件状态和焦点。独立视觉复核在补入 Antigravity 数据后要求缩短读数距离，最后一轮结论为 `ship`。截图均来自 Windows 实例，macOS 和 Linux 未作原生视觉验收。[各次构建指纹、文件比对与门禁](checks.json)

最终同一版本的 `just gpui-test-ui` 41 项通过、1 项按仓库设置忽略，`just gpui-ci` 通过，主库 410 项通过、4 项忽略，包含 release 构建。`just ci` 17/17 步通过，用时 15 分钟，含前端 959 项测试和 VS Code 扩展打包检查。
