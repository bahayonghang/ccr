# 更新全仓库 CCR 图标

## Goal

将 `ref/ccr-icon-redesign` 中的 Dual Runtime Router 图标系统提升为 CCR 仓库的正式品牌资产，并通过现有生成流水线同步到桌面应用、Web UI、文档站和 VS Code 扩展，保证所有既有品牌图标入口使用同一套可再生源文件。

## Background

- 仓库已经以 `branding/master.svg`、`branding/app-icon.svg`、`branding/display-logo.svg` 和 `branding/vscode-icon.svg` 作为品牌单一真源，并由 `ccr-ui/scripts/generate_icons.py` 生成下游资产。
- 新设计包提供与现有四个真源一一对应的 SVG，以及 16-1024 px PNG、ICO、ICNS、Tauri 图标集和视觉预览；四个新 SVG 与当前真源的 SHA-256 均不同。
- 新图标的核心语义是统一入口外层 C、Claude clay 路径、Codex sage 路径和 CLI `>` 汇聚方向；主色为 `#17120F`、`#2A221E`、`#F3EADF`、`#E79A77`、`#7CAB82`。
- 新 PNG 导出均为 RGBA，具有透明外缘，并覆盖现有 Tauri 桌面/Windows 所需尺寸。
- 当前生成器在 `ccr-ui/scripts/generate_icons.py:30-34` 和 `:114-188` 保留旧蓝/橙品牌的 Pillow fallback；本机缺少 Cairo 系统库，CairoSVG 实际不可用，因此 fallback 是 Windows 上的真实执行路径，不能只替换 SVG 后继续沿用旧 renderer。
- 当前生成器以根目录 `icon.png` 作为 fallback 输入（`ccr-ui/scripts/generate_icons.py:23,190-205`），同时又在 `:286` 覆盖该文件，存在输入/输出循环，必须在本次品牌迁移中消除。
- 2026-06 的归档任务 `06-08-replace-ccr-icon-assets` 已建立同类生成边界；本次任务复用该架构，但以新的 Dual Runtime Router 设计替换上一版 circuit-link/prism 品牌。

## Requirements

### R1. 正式品牌源

- 用 `ref/ccr-icon-redesign/sources/` 中的 `master.svg`、`app-icon.svg`、`display-logo.svg` 和 `vscode-icon.svg` 更新 `branding/` 下对应文件。
- `branding/README.md` 必须描述新的图形语义、配色、源文件职责、生成资产范围和跨平台 fallback 约束，不再保留上一版 circuit-link/prism、蓝/橙品牌说明。
- `ref/ccr-icon-redesign/` 仅作为本地设计输入和验收基准；不把整套预览与重复导出作为生产资产提交。

### R2. 可重复的跨平台生成

- 更新 `ccr-ui/scripts/generate_icons.py`，使 CairoSVG 可用和不可用两条路径都生成新的 Dual Runtime Router 图标。
- 删除旧蓝/橙常量、旧几何 renderer 以及对生成输出 `icon.png` 的循环 fallback 依赖。
- 保持现有命令 `cd ccr-ui && bun run icons:generate`、`bun run icons:ensure` 和预构建接线不变。
- 同一次生成必须刷新所有既有下游 SVG、PNG、ICO、ICNS、Windows 方形图标、Android launcher 和 iOS AppIcon，不允许只覆盖设计包中预生成的 Tauri 子集。
- 连续运行两次生成器后，第二次不得产生内容漂移。

### R3. 既有消费表面一致

- Web/Tauri UI：favicon、titlebar/display logo、公共 icon/logo 资源使用新设计。
- Tauri 打包：`tauri.conf.json` 引用的 PNG/ICO 及仓库已提交的 ICNS、Windows、Android、iOS 图标全部由新真源派生。
- 文档站：中英文首页共用的 `/logo.svg` 以及 favicon SVG/PNG 使用新设计。
- VS Code：Marketplace `icon.png` / `icon.svg` 使用彩色应用图标；Activity Bar 和 view container 的 `resources/icons/ccr.svg` 保持 `currentColor` 单色语义并使用新 glyph。
- 根目录 `icon.png` 继续作为生成的视觉参考，不作为生成输入。

### R4. 兼容性与变更边界

- 不改变 package/manifest 路径、版本、publisher、Tauri bundle 配置、VS Code contribution ID 或应用行为。
- 不新增原本不存在的品牌展示位置，不改页面布局、文案或 README 截图。
- 不修改 `ref/` 中的输入资产，不触碰与图标任务无关的既有工作树改动。

## Acceptance Criteria

- [ ] AC1: `branding/` 的四个生产 SVG 与 `ref/ccr-icon-redesign/sources/` 对应文件内容一致，且品牌 README 已更新为 Dual Runtime Router 语义。
- [ ] AC2: 在本机无 Cairo DLL 的条件下，`bun run icons:generate` 成功，生成器中不再存在旧蓝/橙常量、旧 prism/circuit renderer 或根 `icon.png` fallback 输入。
- [ ] AC3: 所有当前已提交的品牌派生资产都被生成器覆盖；第二次生成后相关文件哈希保持不变。
- [ ] AC4: PNG 尺寸、RGBA/透明边缘、ICO 多分辨率和 ICNS 可读性通过脚本检查；VS Code 单色 SVG 仍使用 `currentColor`。
- [ ] AC5: Web 预览中 titlebar logo 与浏览器 favicon 使用新图标；文档首页 logo 和 VS Code 资源完成对应的静态/打包验证。
- [ ] AC6: `bun run icons:ensure`、UI build、docs build/audit、VS Code lint/test/package 和最终仓库门禁按 `implement.md` 通过。
- [ ] AC7: 最终 diff 仅包含任务规划、四个品牌源、品牌生成器/说明和由生成器拥有的下游资产；现有无关改动未被覆盖或清理。

## Out Of Scope

- 继续修改或重新设计用户提供的新图标。
- 更新页面布局、主题 token、应用文案或产品截图。
- 重命名 CCR、修改发行元数据或调整平台支持范围。
- 提交 `ref/ccr-icon-redesign` 的预览、独立生成脚本和重复 exports。
