# 修正 ccr-ui 字体选项并分析衬线字体显示发虚

## Goal

修正 ccr-ui 界面字体预设与本机实际字体族不匹配的问题，并解释 `Source Han Serif SC VF` 全局应用后小字号界面偏虚的原因；在用户确认选项策略后实施最小修改并验证。

## Confirmed Facts

- 字体偏好功能由提交 `535e282d` 引入，界面字体写入 `--font-sans` 与 `--font-brand`，代码字体写入 `--font-mono`。
- 当前预设包含 `Source Han Sans SC`，但本机没有该 family；本机可见相关 family 包括 `Source Han Sans CN` 与 `Source Han Serif SC VF`。
- `Source Han Serif SC VF` 的 OpenType family 名准确，Windows 字体注册值为 `SourceHanSerifSC-VF.ttf`。
- `Source Han Serif SC VF` 是思源宋体可变字体，不是思源黑体；其 `wght` 轴为 250–900，默认实例为 250。
- 字体文件没有 `gasp` / `fpgm` / `prep` / `cvt` 等 hinting 表，65,535 个 glyph 均无指令。
- 真实页面运行在 150% 缩放（DPR 1.5），大量 UI 文本为 11px / 12px / 14px；根样式使用 `-webkit-font-smoothing: antialiased`，正文继承 `-0.016em` 字距。
- 浏览器计算样式对正文请求 `font-weight: 400`、eyebrow 请求 `500`，没有证据表明应用错误地固定在 250 字重。
- Tauri 主窗口 `transparent: false`，Windows ClearType 已开启；二者不是本次主因。
- 当前工作区原有无关改动 `ccr-ui/src-tauri/Cargo.toml`，本任务不得触碰。
- 用户确认采用方案 1：保留 `Source Han Sans SC`，新增 `Source Han Sans CN` 与 `Source Han Serif SC VF`。

## Requirements

- 调整 `UI_FONT_PRESETS` 时只修改字体选项数据，不改持久化、fallback 或 CSS 变量链路。
- 保留 `Source Han Serif SC VF` 的真实 family 拼写，不使用文件名或 PostScript name 代替。
- 保留 `Source Han Sans SC`，并在同一界面字体预设列表新增 `Source Han Sans CN` 与 `Source Han Serif SC VF`。
- 增加针对最终预设名的 smoke 断言，避免再次出现选项名称漂移。
- 在真实设置页验证选项可选择、刷新保持、根 CSS 变量正确。
- 不通过全局加粗、强制 variation axis 或改全部字距来掩盖单一字体的渲染特性。

## Acceptance Criteria

- [x] 已区分字体设置链路问题与宿主字体渲染问题。
- [x] 已用字体表、系统配置和浏览器计算样式解释发虚原因。
- [x] 用户确认最终字体选项策略。
- [x] 最终 family 出现在界面字体下拉框并可持久化。
- [x] 原有字体选项仅按确认范围变化。
- [x] `font-preferences`、`app-settings` 和字体栈契约 smoke tests 通过。
- [x] 真实设置页完成桌面/浏览器验证。

## Out of Scope

- 下载、安装或打包字体。
- 为单一衬线字体重写全局字号、字重、字距或颜色系统。
- 原生枚举全部已安装字体。
- 修改与本任务无关的工作区改动。

## Open Questions

- 无。方案 1 已确认。
