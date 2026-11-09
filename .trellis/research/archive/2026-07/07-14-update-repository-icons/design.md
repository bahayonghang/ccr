# Technical Design

## 1. Boundary And Ownership

本任务保留现有单向资产流水线：

```text
ref/ccr-icon-redesign (local design input)
                 |
                 v
branding/*.svg (committed source of truth)
                 |
                 v
ccr-ui/scripts/generate_icons.py
                 |
                 +-- Web/Tauri UI assets
                 +-- Tauri desktop/mobile bundle assets
                 +-- docs logo/favicon
                 +-- VS Code marketplace/activity assets
                 +-- root icon.png reference
```

`ref/` 不是运行时依赖，也不进入生成命令。生产生成器只能读取 `branding/`，避免开发机本地参考目录影响构建。

## 2. Source Roles

| Source | Purpose | Primary consumers |
| --- | --- | --- |
| `branding/master.svg` | 可编辑的完整设计主稿 | 设计维护，不直接由运行时引用 |
| `branding/app-icon.svg` | 小尺寸和应用入口优化版 | favicon、Marketplace、Tauri/移动端 raster |
| `branding/display-logo.svg` | 展示面版本 | CCR UI titlebar、docs 首页 logo |
| `branding/vscode-icon.svg` | 24x24 `currentColor` 单色 glyph | VS Code Activity Bar / view container |

新设计包中的 `glyph.svg` 不加入生产真源：现有仓库没有对应职责，且 `vscode-icon.svg` 已承载单色 glyph 合同。

## 3. Generator Strategy

### 3.1 Primary renderer

保留 CairoSVG 路径，用于安装了 Cairo 系统库的环境，并继续从 SVG 直接栅格化。

### 3.2 Windows fallback

不能删除 fallback：当前 Windows 主机即使安装 Python `cairosvg` 包，也因缺少 `cairo-2`/`libcairo-2` 动态库而导入失败。

将现有 Pillow native renderer 原位改写为 Dual Runtime Router 几何：

- 深色圆角 tile 及轻量边框；
- cream 外层 C；
- clay/sage 内层路径；
- cream CLI chevron；
- `app` 与 `display` variant 分别保持 48/32 的 tile inset；
- 高分辨率绘制后 Lanczos 下采样，保证 16/24/32 px 边缘质量。

生成器不得读取根 `icon.png` 或任何下游输出作为 fallback。这样 SVG 与 native renderer 都由版本控制中的代码/真源定义，生成过程无循环依赖。

### 3.3 Drift protection

实现后在同一环境连续执行两次生成。第一次刷新所有派生物；记录相关文件 SHA-256；第二次执行后再次计算并比较。任何漂移都视为生成器缺陷。

## 4. Output Mapping

### Web and docs

- `ccr-ui/src/assets/favicon.svg`
- `ccr-ui/public/icons/icon.svg`, `icon.png`, `logo.svg`, `logo.png`
- `ccr-ui/src/assets/logo.png`
- `docs/public/logo.svg`, `favicon.svg`, `favicon.png`

### VS Code

- `ccr-vscode/icon.svg`, `icon.png`
- `ccr-vscode/resources/icons/ccr.svg`

`ccr-vscode/package.json` 路径保持不变。彩色 Marketplace 图标和 `currentColor` Activity Bar 图标使用不同源，避免在 VS Code 主题中出现固定低对比颜色。

### Tauri and platform bundles

- `ccr-ui/src-tauri/icons/` 顶层 PNG、ICO、ICNS 和 Windows Square/Store 图标
- `ccr-ui/src-tauri/icons/android/` launcher、round、foreground 与背景色配置
- `ccr-ui/src-tauri/icons/ios/` 全部 AppIcon 尺寸

虽然新设计包只附带桌面/Tauri 顶层导出，Android/iOS 仍由仓库生成器从同一 `app-icon.svg` 重新派生，防止旧图标残留。

## 5. Compatibility

- 不修改 `ccr-ui/src-tauri/tauri.conf.json`、`ccr-vscode/package.json` 和现有 HTML/Vue/Markdown 引用路径。
- SVG 保留 `viewBox`、可访问标题/描述；VS Code SVG 保持 `currentColor`。
- Raster 输出保留现有文件名、尺寸、RGBA 透明外缘、ICO size set 和 ICNS 格式。
- Android adaptive icon 的背景色应与新品牌 tile/透明策略一致，并通过小尺寸检查避免裁切。

## 6. Visual Verification

按 `ccr-ui-visual-workflow` 使用 Web 预览验证 `/` 路由的 titlebar/display logo，并检查浏览器实际加载的新 favicon；不为此默认启动 Tauri shell。文档站首页另做构建/预览检查。原生安装包图标通过文件结构、格式和必要时的打包产物验证，不把 Web 预览当作原生壳验证替代品。

## 7. Rollback

品牌源、生成器和全部派生资产必须作为一个原子变更回滚。禁止只回滚 `branding/` 或只回滚二进制输出，否则下一次预构建会再次覆盖并造成跨表面不一致。

## 8. Risks

| Risk | Mitigation |
| --- | --- |
| Windows 实际走 fallback，结果与 SVG 不一致 | 以提供的 16-1024 px exports/preview 为视觉基准，分别检查小尺寸与大尺寸 |
| 只替换桌面图标，移动端仍残留旧品牌 | 继续运行现有 Android/iOS 全量导出函数并检查旧色值 |
| VS Code 彩色图标在 Activity Bar 失去主题适配 | `resources/icons/ccr.svg` 独立使用 `vscode-icon.svg` 和 `currentColor` |
| 生成器读取自身输出导致机器间漂移 | 删除根 `icon.png` fallback 输入，连续运行两次并比较哈希 |
| 当前工作树已有无关改动 | 按显式路径审查 diff，不整理、不回滚、不纳入本任务结论 |
