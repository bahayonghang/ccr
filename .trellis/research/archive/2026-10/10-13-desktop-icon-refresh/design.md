# Design

## Asset Ownership
内置 ImageGen 候选保存在任务预览目录；获批结果复制到 branding/ 作为版本控制的 PNG 生产源，并保存提示词。生产链路不依赖 home 或任务归档目录。

既有 SVG 消费 URL 保持稳定，使用内嵌同一生成图像的 SVG 兼容载体。PNG/ICO/ICNS 从 PNG 源确定性缩放，无需 Cairo。旧几何 fallback 随来源策略替换；README 与品牌契约同步说明新的来源。Activity Bar 保持 currentColor 单色矢量，与选定的新标记一致，不保留旧汇流形状。

## Visual Direction
放弃旧汇流概念。用户已选定 A：两个相互旋转的蓝色环形片段。蓝色为唯一品牌色，冷白/浅蓝为基底。内置 ImageGen 的生产准备编辑保留符号形状与位置，仅将背景延伸为干净的全画幅浅色方形源。标准圆角和透明外角由确定性导出 mask/clipPath 定义；图像覆盖整个 1024 画布，mask 限制外缘，不再次缩小标记。

## Compatibility and Risks
保留既有目标尺寸、路径、透明外角；Tauri 显式加入已生成的 ICNS。移动端仅同步既有图标。粗线条及留白保证小尺寸识别，实检 16/32 px。SVG 内嵌资产，禁止绝对本机路径。造型/背景修改再次使用 ImageGen，缩放和编码仅作导出。

## Rollback
品牌源、生成器、配置、全部派生资源必须一起回滚，避免源输出不一致。
