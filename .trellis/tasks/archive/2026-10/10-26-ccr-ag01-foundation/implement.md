# AG01：轻盈玻璃规范、令牌与应用外壳：执行清单

## 执行前
- [ ] 父方案批准，前置任务证据完整，当前状态允许启动。
- [ ] 读取本任务上下文、父设计及research；重查dirty文件与验证前置，不静默安装。

## 工作步骤
- [ ] 修改视觉前先校准现有perf harness：React选择器、页面标识、滚动距离、错误路径；成功采before后才能进入令牌迁移。
- [ ] 审计工作树与验证命令隐含安装，记录before和路由登记。
- [ ] 完成新DESIGN适配与暗色/状态/图表底板候选配色，先算对比度；同步AGENTS/sidecar及spec。
- [ ] 记录token名字/消费者基线；登记唯一纸面别名，保留共享实底角色。
- [ ] 迁移外壳/首帧/flavor预览/macOS底色；浮层取消二次blur。
- [ ] 定向迁移旧外观守卫，补合成/运行时层数/降级；跑主题与shell烟测及四组合代表页。

## 验证
目标测试：tests/theme/、tests/quality/overlay-single-implementation.smoke.test.ts、tests/ui/confirm-interaction.smoke.test.tsx、tests/shell/cache-route.smoke.test.ts、tests/shell/main-layout-route-presence.smoke.test.ts。实施前用rg --files核实当前匹配，不凭旧目录猜测测试名。

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
