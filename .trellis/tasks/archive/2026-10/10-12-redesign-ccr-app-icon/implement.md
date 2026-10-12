# 实施顺序

1. 按 design.md 的坐标改四份 SVG 和 `branding/README.md`。
2. 让 `render_native_brand_master` 用同一坐标画平涂蓝底和白色汇流。
3. 在 `ccr-ui` 运行 `bun run icons:generate`。
4. 看 32px 和 512px。路线要在圆角方块内，顶部是一个点，底色是 `#1A73E8`。
5. 再跑一次生成，比较输出哈希。然后跑 `bun run icons:ensure`。
