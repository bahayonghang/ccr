# 执行计划:应用外壳与首页材质落地

## Checklist

1. [ ] 确认前置:07-07-ui-glass-tokens 已合入(material 令牌 + utility 存在)。
   - 验证:`rg "material-glass-chrome" ccr-ui/src/styles/tokens.css`。
2. [ ] MainLayout:sidebar-glass/topbar-glass 迁移 chrome 档;isResizing 时禁用 blur;移动遮罩去 blur。
   - 验证:DevTools Layers 面板 = 2 个 backdrop-filter;拖拽侧栏录制无长帧。
3. [ ] 设置 Dock 与 nav item 表面改不透明档。
   - 验证:亮色 clay 截图,Dock 边界清晰。
4. [ ] DashboardView:栅格交换(actions 8col 主位)+ hero 就绪徽章。
   - 验证:三种 backendStatus 状态截图(可用 web 预览 mock)。
4b. [ ] ReadinessLedger 降级为紧凑状态条 + 信号严重度门控(design.md §3b):仅 core 级信号驱动阻塞叙事,状态 pill 改清单行。
   - 验证:仅注入前端日志错误时首屏无红色阻塞叙事;core 错误时恢复。
5. [ ] DashboardNextActions:去 01-04 装饰编号(或绑定 ⌘1-4)+ 主/次行动按钮层级 + 空态三步引导。
   - 验证:0 profile 场景截图;i18n 中英文案齐全。
6. [ ] DashboardPlatformMatrix:整行可点 + hover + 版本徽章骨架。
   - 验证:键盘 Tab 可聚焦整行,Enter 触发导航(a11y)。
7. [ ] Usage Movement:去重复 eyebrow/说明句 + 柱状 hover tooltip + "会话 0"解释文案;Signal Stream:倒序 + 相同消息聚合 ×N + 徽章降噪;两卡降为二层级表面。
   - 验证:截图确认无重复文案;事件流时间序正确。
8. [ ] 全量验证:`bun run type-check && bun run lint`,三个主题 smoke,frontend-check-quick。
9. [ ] 截图对比(亮/暗 × clay/mocha)入 research/;review gate。

## Rollback

commit 划分:①外壳材质 ②首页 IA;视觉问题 revert ①,交互问题 revert ②。
