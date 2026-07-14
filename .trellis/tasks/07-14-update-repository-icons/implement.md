# Implementation Plan

## Preconditions And Review Gate

- [ ] 用户已审阅 `prd.md`、`design.md` 和本计划，并明确要求开始实施。
- [ ] 执行 `python ./.trellis/scripts/task.py start 07-14-update-repository-icons`，状态进入 `in_progress` 后再修改生产代码/资产。
- [ ] 使用 `trellis-before-dev` 重新加载相关规范与当前 dirty-tree 边界。
- [ ] 记录实施前 `git status --short`，保护现有 `AGENTS.md`、依赖清单/lockfile 和其他无关改动。

## 1. Promote Brand Sources

- [ ] 将参考包的四个生产 SVG 同步到 `branding/` 对应文件；不复制 `glyph.svg`、preview、独立生成脚本或 exports 目录。
- [ ] 更新 `branding/README.md` 的设计语义、配色、source role、output ownership 和 fallback 说明。
- [ ] 校验四个生产 SVG 与参考源逐字节一致，并解析为合法 XML。

Verification:

```powershell
git diff --no-index -- branding/app-icon.svg ref/ccr-icon-redesign/sources/app-icon.svg
git diff --no-index -- branding/display-logo.svg ref/ccr-icon-redesign/sources/display-logo.svg
git diff --no-index -- branding/master.svg ref/ccr-icon-redesign/sources/master.svg
git diff --no-index -- branding/vscode-icon.svg ref/ccr-icon-redesign/sources/vscode-icon.svg
```

## 2. Replace The Legacy Fallback

- [ ] 在 `ccr-ui/scripts/generate_icons.py` 中把旧蓝/橙 prism/circuit Pillow renderer 改为 Dual Runtime Router renderer。
- [ ] 删除 `REFERENCE_ICON_PNG`、`load_reference_icon()` 及所有根 `icon.png` 输入依赖。
- [ ] 保留 CairoSVG primary path、Pillow fallback、现有输出 API 和命令接线。
- [ ] 让 app/display variant 的 tile inset 与对应 SVG 一致；小尺寸采用高分辨率绘制再下采样。
- [ ] 检查 Android background/foreground 生成策略与新深色 tile/透明边缘相容。

Focused verification:

```powershell
rg -n "0E9FF3|FF5800|BRAND_BLUE|BRAND_ORANGE|REFERENCE_ICON_PNG|load_reference_icon|circuit|prism" branding ccr-ui/scripts/generate_icons.py
cd ccr-ui
bun run icons:generate
bun run icons:ensure
```

预期：`rg` 无旧品牌实现命中；本机 Cairo DLL 缺失时生成命令仍成功。

## 3. Regenerate Every Owned Asset

- [ ] 运行一次生成器，刷新 Web、docs、VS Code、Tauri、Windows、Android 和 iOS 的全部已提交派生资产。
- [ ] 用脚本检查所有 PNG 的预期尺寸、RGBA/透明外缘，检查 ICO size set 和 ICNS 可读性。
- [ ] 检查 `ccr-vscode/resources/icons/ccr.svg` 包含 `currentColor`，其余复制型 SVG 与品牌源一致。
- [ ] 对任务拥有的输出计算 SHA-256，第二次运行生成器后重新计算；确认零漂移。
- [ ] 对派生资产扫描旧蓝 `#0E9FF3`、旧橙 `#FF5800` 和旧品牌文案，不允许残留。

Rollback point: 若任一平台资产不合规，回滚本任务的品牌源、生成器和派生输出作为整体，不能保留半套新品牌。

## 4. Visual And Packaging Verification

- [ ] 按 `ccr-ui-visual-workflow` 从 `ccr-ui/` 启动 `bun run dev:web -- --host 127.0.0.1 --strictPort`。
- [ ] 在 `http://127.0.0.1:5173/` 检查 titlebar/display logo、新 favicon、浅色/深色背景下的清晰度，并留存必要截图证据。
- [ ] 构建/预览 docs 中英文首页，确认 `/logo.svg` 和 favicon 解析成功。
- [ ] 构建并打包 VS Code 扩展，确认 VSIX 包含 `icon.png` 与 `resources/icons/ccr.svg`；检查 Activity Bar glyph 的主题适配边界。
- [ ] 对 16、24、32、64、128、512 和 1024 px 样本进行视觉抽查，重点检查内层双色路径是否粘连、CLI chevron 是否丢失、外层 C 是否被裁切。

## 5. Verification Ladder

从最窄检查逐步升级：

```powershell
cd ccr-ui
bun run icons:ensure
bun run build

cd ../docs
bun run build
bun run audit

cd ../ccr-vscode
npm run lint
npm test
npm run package

cd ..
just version-check
just fmt-check
just frontend-check
just ci
git diff --check
```

- [ ] 若最终 `just ci` 被任务开始前已存在的无关依赖改动阻断，记录确切失败与归属，不顺手修改无关文件。
- [ ] 检查 `git status --short` 和显式路径 diff，确认未覆盖用户现有改动，未把 `ref/` 或构建产物纳入任务。

## 6. Completion Review

- [ ] 逐项映射 AC1-AC7 到命令输出、哈希检查和视觉证据。
- [ ] 使用 `trellis-check` 完成规范、复用、跨表面和 dirty-tree 一致性审查。
- [ ] 如本次迁移形成可复用的品牌生成约束，按 Phase 3 更新相应 Trellis spec；否则记录无需更新的理由。
- [ ] 用户确认后再进入提交与 Trellis finish-work，不在 planning 阶段提交或归档。
