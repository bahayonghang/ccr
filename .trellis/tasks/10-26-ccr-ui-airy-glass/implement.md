# 执行计划（AG01–03 已实施，AG04 继续验收）

## 启动评审
- [x] 用户批准最终范围及design.md适配决定；本轮继续该批准范围。
- [x] 复查工作树、研究和生效规范，保护并行GPUI/服务改动。
- [x] 子任务规划文件和上下文完整。
- [ ] AG01先收集基线：代码/脏文件指纹、环境、主题/字体、数据fixture、窗口、浏览器/WebView版本。
- [ ] 检查验证入口的隐含安装行为；未授权安装不得直接运行含bun install的聚合门，先运行已有依赖下等价子检查并明确聚合门NOT_RUN。

## 顺序
1. AG01：校准性能harness（当前React选择器、路由标识、滚动距离和错误路径）→成功基线和原始快照→同步新规范→最小令牌治理→外壳/首屏/浮层实底→四组合代表页。
2. AG02：公共控件→Profiles/配置/平台设置→资源/认证/同步/环境。保留dirty、权限、确认和滚动。
3. AG03：首页/图表/平台概览→Usage/会话/日志→动态命令/签到/托盘。核对未知/零/空、图例、keep-mounted及虚拟化。
4. AG04：从当前catalog导出覆盖表→前端/架构/构建/coverage→视觉和键盘→性能前后对照→原生→集成复核。

实现/check默认使用Trellis对应子代理；派发先声明当前子任务、文件所有权和验收标准。主会话负责决策及跨阶段协调。每阶段先窄检查再扩展，不循环重复无新增信息的全库检查。

## 验证入口
具体目标测试见research/validation-contracts.md与子任务计划。
- 根目录预检：just version-check，然后just fmt-check；不自动修复。
- ccr-ui：bun run test:smoke -- <目标文件>。
- ccr-ui：bun run type-check；bun run lint；bun run test；bun run build。
- ccr-ui：bun run test:smoke --coverage（保留既有阈值和默认并行）。
- ccr-ui Web：bun run dev:web -- --host 127.0.0.1 --strictPort。
- 交付：just frontend-check、just ui-check，检查其安装前置边界；ui-check不等于build/coverage/视觉完成。
- 最终：just ci；存在既有无关失败时单独归因，不能报告全绿。
- bundle budget需先满足sourcemap产物要求，使用package已有build:with-budget入口。
- 原生只为实际WebView/窗口/系统主题/DPI/托盘验证启动，使用隔离数据，不执行真实配置写入或用量重建。

## 每阶段交付
- [ ] 文件清单和R1–R8对应；没有无关产品修改。
- [ ] 外观守卫迁移原因与行为保护均有证据。
- [ ] 退出码、最终计数、截图、环境、未测项完整。
- [ ] 回滚只逆转本段diff，规范/令牌/测试一致回退。
- [ ] 不擅自提交、推送、安装或发行；本轮不归档未实施任务。

## 2026-10-26 结果

AG01–03 展示实现与 Web 检查已应用，最终验收保持开放。完整结果、命令和限制见 `../10-26-ccr-ag04-acceptance/evidence/integration-summary.md`；全路由见 `route-matrix.md`。未安装、提交、推送、发行或归档。
