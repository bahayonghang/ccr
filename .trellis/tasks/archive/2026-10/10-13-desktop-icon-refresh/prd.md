# CCR 蓝色现代品牌图标与桌面资源更新

## Goal
使用内置 ImageGen 交付结合 CCR 多 AI CLI 配置管理、快速切换功能的蓝色、简洁、现代图标，优先保证 Tauri 桌面识别度。用户已要求创建任务并授权助手设计；最新反馈要求放弃旧图标造型，以蓝色为基色，审美参考 Apple、Anthropic、OpenAI。

## Background
当前品牌是蓝底白色双线汇流。`branding/README.md` 与 `ccr-ui/scripts/generate_icons.py` 管理统一源和输出。`ensure-icons.cjs` 在 prebuild 调用生成器，单独替换桌面 PNG 会被覆盖。Tauri 配置已有 PNG/ICO，ICNS 已生成但未列入 bundle.icon。

## Requirements
1. 以蓝色为基色，采用冷白/浅蓝底与简洁独立的蓝色几何标记；参考顶级科技品牌的简洁轮廓、留白与几何秩序，不复制商标。放弃旧汇流造型及暖白、墨黑、琥珀配色。
2. 使用真实 ImageGen 结果作为品牌源，并在仓库保存原始结果与提示词以供追溯。
3. 优先更新 Tauri PNG、ICO、ICNS 和 Windows tiles；通过现有链路同步 Web、文档、扩展品牌图片及既有移动端派生资源。
4. 保持消费路径、尺寸与桌面透明外角；Activity Bar 保留 currentColor 单色矢量标记。
5. 更新生成链路及资产所有权说明，保证 prebuild 不恢复旧图标。

## Acceptance Criteria
- 最终图标以蓝色为基色、简洁现代，建立 CCR 独立识别，与配置管理/切换的关联通过抽象几何表达，不使用分支、节点或网络图。
- 原始 ImageGen 输出、提示词、生产源均在仓库可定位。
- 16/32 px 与 512/1024 px 视觉检查通过；PNG/ICO/ICNS 可解码，尺寸及透明外角符合契约。
- 连续生成两次所有品牌资产哈希一致，icons:ensure 不回退。
- 完成资产契约规定的 generate、ensure、build、frontend-check、diff 检查，记录结果或真实环境阻碍。

## Out of Scope
业务功能、页面布局、主题系统、平台商标、功能图标库、发布安装包及修改已安装应用。

## Key Decisions
最新用户反馈优先于之前设计方案。浅色基底采用冷白/浅蓝，蓝色作为唯一品牌色。旧候选不再用于交付。生成图像作为生产来源，兼容 SVG 消费路径；先提供新候选，规划获批后再修改产品资源。

## Approved Design
用户明确选择附件 `exec-8d07966f-4178-40d1-ab77-ce7211825df8.png`（A：双环几何），并指示“采用这个，继续”。此为现有替换范围与候选的实施批准，无阻塞问题。内置 ImageGen 已保留该标记并将背景扩展为干净的全画幅浅色方形源，消除原透明边界的杂边；导出层应用标准圆角、外边距和透明角。
