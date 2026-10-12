# 重做 CCR 应用图标

## Goal

把 CCR 应用图标换成一张新标记。标记要让人看出产品功能：一个入口管理 AI CLI 配置，Claude Runtime 与 Codex Runtime 并列，profile 切换决定正在生效的路线。视觉以饱和蓝色为底，图形向上，在 16px 仍能认出。生产文件走现有品牌源和生成器，铺到桌面、网页、文档和 VS Code。

## Background

当前生产标记在 `branding/app-icon.svg`。桌面、网页、文档、VS Code 的图标都由 `ccr-ui/scripts/generate_icons.py` 从 `branding/*.svg` 生成。

Windows 上如果没有 Cairo，`generate_icons.py` 的 `render_native_brand_master` 会按写死的几何出图，不读取 SVG 路径。只改 SVG 时，这台机器导出的 PNG、ICO、ICNS 仍会是旧标记。

用户已否决暖灰底、旧图标的字母 C、双弧、尖括号，以及只表示“向上”的日轮、柱状和单曲线。新标记不从旧图形派生，但要表达统一入口、双 Runtime 和 profile 路由。功能依据是根 `README.md` 与 `docs/index.md`：统一管理 AI CLI 配置与运行时状态，Claude Runtime 与 Codex Runtime 显式并列。

## Confirmed facts

- 品牌源文件：`branding/master.svg`、`branding/app-icon.svg`、`branding/display-logo.svg`、`branding/vscode-icon.svg`。
- `vscode-icon.svg` 必须保持单色 `currentColor`，供视图容器使用。
- 生成器覆盖根目录 `icon.png`、`ccr-ui/src-tauri/icons/`、`ccr-ui/public/icons/`、`ccr-ui/src/assets/`、`ccr-vscode/`、`docs/public/`。
- 图标内不放文字、版本号或字标。
- 方向稿只作为外形参考。生产文件仍是 SVG，再由生成器导出位图。
- 色板：饱和蓝底，白色图形。底色取 `#1A73E8` 到 `#2F8CFF` 这一段，不用灰、不用褐、不用琥珀。

## Rejected

暖灰三稿已废弃，文件留在 research 里只作记录：

- `research/candidate-a-notch.jpg`
- `research/candidate-b-slab.jpg`
- `research/candidate-c-bracket.jpg`

## Candidates

日轮、单曲线、三根柱只表示向上，不表示路由，已废弃：

- `research/candidate-dawn.jpg`
- `research/candidate-ascent.jpg`
- `research/candidate-steps.jpg`

当前候选都用饱和蓝底和白色路线，图形含义不同：

| 代号 | 功能含义 | 参考图 |
| --- | --- | --- |
| 汇流 | 两条运行时收成一条正在生效的路线 | `research/candidate-merge.jpg` |
| 分轨 | 一个入口分出 Claude 与 Codex 两条向上的运行时 | `research/candidate-split.jpg` |
| 选定 | 多份 profile 中选定一份，并把它送上生效路线 | `research/candidate-select.jpg` |

## Requirements

- 选定一个候选后，用干净的 SVG 重画该标记，不把生成位图直接当作生产源。
- `master.svg`、`app-icon.svg`、`display-logo.svg` 使用同一标记。应用图标和展示图标可以在圆角和边距上不同，图形本身相同。
- `vscode-icon.svg` 是该标记的单色剪影。
- 图标以蓝色为底，图形为白。不出现旧标记的 C、双弧或尖括号。
- `render_native_brand_master` 画出和 SVG 相同的标记，使无 Cairo 的 Windows 导出与 SVG 一致。
- 跑通 `ccr-ui/scripts/generate_icons.py`，刷新它拥有的全部出口。
- 16px 和 32px 导出仍能看出一个完整标记。

## Acceptance Criteria

- [x] 用户选定汇流，且 `branding/*.svg` 只包含该标记。
- [x] `vscode-icon.svg` 仅使用 `currentColor`，轮廓与所选标记一致。
- [x] `generate_icons.py` 在本机成功结束，连续两次输出哈希相同。
- [x] Tauri、网页、文档、VS Code 的图标文件都来自这次生成。
- [x] 32px 与 512px PNG 中标记完整，圆角外角透明，底色为 `#1A73E8`。

## Out of Scope

- 改市场终端页面、组件或界面色板。
- 加字标、动效图标或发布宣传图。
- 改版本号、包元数据或扩展命令。
- 改 `ccr-ui/ref/` 下的参考仓库。

## Decision

生产标记用汇流。两条白色路线在饱和蓝底上收到顶部的一个圆点，表示多份配置收成一条正在生效的路线。几何以 `branding/app-icon.svg` 为准，Pillow 回退画出同一组坐标。
