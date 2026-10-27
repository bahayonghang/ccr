# AG03：观测、图表与运行状态页面：执行清单

## 执行前
- [ ] 父方案批准，前置任务证据完整，当前状态允许启动。
- [ ] 读取本任务上下文、父设计及research；重查dirty文件与验证前置，不静默安装。

## 工作步骤
- [ ] 建立数据与异步状态fixture，检查rendered page而非redirect/placeholder。
- [ ] 迁移首页/Insights及平台概览，保留布局密度、有界图表和底部状态栏。
- [ ] 迁移Usage/预算/价格和会话/日志，先固定底板，再校对数据色和legend/table。
- [ ] 迁移commands/checkin/tray展示；不改订阅、取消和恢复逻辑。
- [ ] 运行图表/会话/作业烟测，采集滚动/图表切换对照证据并交AG04。

## 验证
目标测试：tests/usage/apexcharts-style-contract.smoke.test.ts、tests/usage/usage-chart-stability.smoke.test.tsx，以及usage/agent-sessions/monitoring/commands/checkin/tray当前匹配smoke。实施前用rg --files核实当前匹配，不凭旧目录猜测测试名。

cwd ccr-ui：bun run test:smoke -- <实际目标文件>，随后bun run type-check、bun run lint、bun run test、bun run build；最终coverage和聚合门由AG04集中完成。AG01先before，AG03提供同数据after，AG04归并对照。

Web验证使用bun run dev:web -- --host 127.0.0.1 --strictPort；视口/主题/键盘/透明降级和性能门槛完全采用父verification.md。预览只处理隔离fixture与可读展示；不触发真实配置/认证/同步写入。原生结果独立记录，无法运行写NOT_RUN。

## 交付
- [ ] 每条AC都有测试/截图/记录引用，说明失败与未测项。
- [ ] 附变更文件、命令退出码/最终计数、环境指纹和回滚点。
- [ ] 父route矩阵/verification更新；不将规划校验通过当产品验收。
- [ ] 未获授权不提交、推送、安装、发行或归档。

## 2026-10-26 执行记录

本阶段展示实现已应用；定向与集成检查见 `../10-26-ccr-ag04-acceptance/evidence/integration-summary.md`。任务保持 `in_progress`，不将产品实施、Web 通过和桌面最终验收混为一项。完整原始日志和失败采样均保留；没有安装、提交、推送或归档。

AG04 仍未关闭的项目：原生 Windows DPI/托盘及其他平台、实际浏览器 200% 缩放、无 backdrop-filter 支持的真实引擎、真实数据 Usage/虚拟会话性能；现有确认框关闭后的焦点恢复失败和独立 i18n 检查漂移另列。已验证场景与未测场景不能相互替代。

## 2026-10-27 用户指定归档

按用户明确要求，相关实现与证据已拆分提交，任务树行政归档。`meta.final_acceptance=open`；未通过／未运行项目保持原始状态。旧文档中的 in_progress／未归档说明为2026-10-26历史记录，不是当前目录状态。最新说明见 `../10-26-ccr-ag04-acceptance/closure.md`。
