# 技术设计:应用外壳与首页材质落地

## 1. 影响面

- `ccr-ui/src/components/MainLayout.vue`(sidebar-glass/topbar-glass 类的样式源,约 589 行)
- `ccr-ui/src/styles/shell-critical.css`、`backgrounds.css` 中 shell 相关块
- `ccr-ui/src/views/DashboardView.vue` + `ccr-ui/src/components/dashboard/*`(5 个组件)
- `ccr-ui/src/styles/home.css`(卡片层级令牌微调)

## 2. 外壳迁移

- `sidebar-glass`/`topbar-glass` 现有自定义样式改为引用 chrome 档令牌(保留类名,避免测试/选择器破坏):`background: var(--material-glass-chrome-bg); backdrop-filter: var(--material-glass-chrome-blur); border-color: var(--material-glass-chrome-border); box-shadow: var(--material-glass-chrome-highlight), var(--material-glass-chrome-shadow);`
- 拖拽 resize 期间加 `body.is-resizing` 时侧栏 `backdrop-filter: none`(startResize/stopResize 已有状态位 isResizing,绑定 class 即可),规避拖拽全程重模糊。
- 移动端遮罩的 `backdrop-blur-[2px]` 保留(短时存在),但改为 `bg-black/55` 无 blur——遮罩本身已提供对比,省一层 GPU 拷贝。
- 设置 Dock:去掉渐变叠层中的低对比装饰(`from-accent-primary/12` 保留可,但底面改 `--surface-card-*` 不透明),状态点(session active 绿点)保留——它是真实语义状态。

## 3. 首页信息架构调整

现状:hero → readiness(8col)+actions(4col) → usage(8col)+signals(4col) → platform matrix。

调整为:
1. hero 行:标题左,右侧新增"就绪结论徽章"(ok/checking/error 三态,数据来自现有 backendStatus,点击滚动到 readiness 卡)。
2. 第一栅格行交换:**actions(8col)+ readiness(4col)**——行动优先,状态检查降为辅助。actions 内主行动用 `--color-accent-primary` 实心按钮(现有按钮体系),最多 1 个实心,其余 ghost。
3. usage + signals 行不动,但卡片标题降一级(--home-text-section → body 加粗),表面用不透明 card 档。
4. platform matrix:行整体 `cursor: pointer` + hover 背景 `--home-surface-card-hover`,版本徽章 loading 态用 12×48px 骨架条。

空态引导:`dashboardPresentation.actions` 为空或后端返回 0 profile 时,actions 卡渲染"三步引导"(创建 Profile → 配置 MCP → 导入用量),复用 EmptyState.vue 风格,不新造组件。

## 3b. 截图复核补充(2026-07-07)

- ReadinessLedger("处理阻塞项"卡)降级:display 标题改 section 级(--home-text-section),卡片高度压缩;信号 pill 改清单行(SIcon Check/AlertTriangle + 文案,无句号);阻塞判定接 R2b 门控——`buildDashboardPresentation` 中区分 signal severity(core/log),仅 core 级驱动"阻塞"叙事与红色 tile。
- DashboardUsageMovement:删除与标题重复的 eyebrow 与第二处来源说明;柱状图加 hover tooltip(日期+数值,纯 CSS/轻量实现,不引入图表库)。
- DashboardSignalStream:entries 按 timestamp 倒序排序后再渲染;相邻相同 message 聚合 `{message, count, lastAt}`;来源徽章淡化(text-muted 小写)。
- DashboardNextActions:移除 01-04 序号列;若保留数字则同时注册 ⌘1-4 快捷键并在 hover 显示;主行动实心。
- 统计 tile:标签行加语义注解(CPU/内存、错误/警告分列),数值 `--font-mono` + tnum。

## 4. 权衡

- 不把 readiness 结论做成常驻 toast/banner——首页已有 BackendStatusBanner 处理硬错误,徽章只做软状态。
- 栅格交换只动 CSS grid-column 与模板顺序,不动 dashboardPresentation 数据结构。
- hero 右侧徽章在 <1180px 折行到标题下方(现有断点体系)。

## 5. 回滚

MainLayout 材质迁移与 Dashboard IA 调整分两个 commit;各自可独立 revert。
