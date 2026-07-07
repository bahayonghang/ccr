# 应用外壳与首页材质落地

## Goal

把子任务 07-07-ui-glass-tokens 建立的三档玻璃材质落到应用外壳(MainLayout 侧栏/顶栏/设置 Dock)与首页 Dashboard,并解决首页的引导与重点问题:让"当前状态 → 下一步行动"的动线更突出,弱化装饰性信息。

## Requirements

### R1 外壳材质(玻璃预算的主要消费者)

- 侧栏 `sidebar-glass`、顶栏 `topbar-glass` 迁移到 `.glass-chrome` / chrome 档令牌;这两处 + 一个活跃模态即为全屏玻璃预算(≤3)的标准分配。
- 主内容区、nav item、设置 Dock 卡片全部使用不透明表面;移除内容区内的 backdrop-blur 残留(含移动端遮罩以外的 `backdrop-blur-[2px]` 类)。
- 侧栏 resize 时(`will-change` 已有)不得出现模糊重绘卡顿;拖拽期间可临时降级为不透明。

### R2 首页 Dashboard 引导与重点(含截图复核结论)

- **消除双 Hero**:页头"运行概览"保留为唯一 display 级大标题;"处理阻塞项/一切就绪"卡降级为紧凑状态条(结论徽章 + 一行摘要 + 展开查看信号清单),不再使用巨型 display 字号。
- **消除重复文案**:"用量趋势"区 eyebrow 与标题文字完全相同→删除 eyebrow;"会话、请求和 Token 均来自本机用量归档。"出现两次→只保留一处。
- **行动队列去装饰性编号**:01/02/03/04 数字既不是快捷键也不是优先级——要么绑定真实快捷键并显示 ⌘1-4,要么改为图标+标题;第一项主行动用 accent 实心按钮层级,其余 ghost;空态(0 profile)给三步首次使用引导。
- **状态信号 affordance 区分**:"桌面后端已连接。"等状态 pill 视觉上像按钮——改为带 ✓/! 状态图标的清单行,去句号;可点击项与纯状态项样式明确分离。
- **统计 tile 语义化**:"事件 3/3"红色 tile 含义不明→改"错误 3 · 警告 3"分色;"本机 36.2% / 49.0%"标注 CPU/内存;数值等宽、标签用正文字体。
- **事件流治理**:按时间稳定倒序(截图中 04:49 PM 之后出现 10:43 PM);相同消息聚合为一条 ×N;FRONTEND 徽章降噪(淡色小徽章);消息两行截断 + tooltip 全文。
- **用量趋势图可读性**:柱状 hover 显示日期与数值,补 Y 轴刻度或峰值标注;"会话 0"时给出解释文案而非裸 0。
- Platform Matrix 行整行可点 + hover 反馈,CLI 版本徽章 loading 用骨架而非空白。

### R2b 信号质量门控

- 首页"阻塞"红色叙事只允许由影响核心功能的信号驱动(后端不可达、CLI 缺失、配置写入失败);前端 UI 日志错误(如 Failed to save Claude profile 重试类)归入事件流,不得同时驱动阻塞卡 + 红色 tile + 行动队列(截图中同一噪声三处放大);每个错误信号在首屏最多一个入口。

### R3 性能

- 首页不新增任何 backdrop-filter;现有 deferred/idle 加载策略(scheduleWhenIdle、keep-alive pause/resume)不回归。
- StageBackground/AnimatedBackground 若在首页路径上渲染装饰层,确认其透明度令牌在新对比度体系下不需要上调;禁止为了"玻璃感"给背景加动态 orb。

## Out of Scope

- Dashboard 数据逻辑(dashboardPresentation)、监控 feed 协议。
- 其他页面的材质迁移(子任务 3/4/5)。

## Acceptance Criteria

- [ ] DevTools Layers:Dashboard 路由下 backdrop-filter 元素仅侧栏 + 顶栏(打开模态时 +1)。
- [ ] 侧栏拖拽 resize 与页面滚动在 Performance 面板录制中无长帧(>32ms)连续出现。
- [ ] 首页在 0 profile / 正常 / 后端错误三种状态下截图:主行动按钮视觉层级最高,空态有创建引导。
- [ ] 首页只有一个 display 级大标题;"用量趋势"无重复 eyebrow/说明句;行动队列无装饰性编号。
- [ ] 仅前端日志错误存在时,首屏无红色阻塞叙事(信号门控生效);事件流倒序且重复条目聚合 ×N。
- [ ] 亮/暗主题下外壳玻璃有可感知的模糊+饱和材质(对比基线截图),reduced-transparency 下回退不透明。
- [ ] `bun run type-check && bun run lint` + 主题 smoke tests 通过。

## Dependencies

- 依赖 07-07-ui-glass-tokens 完成(material 令牌与 utility 就位)。
