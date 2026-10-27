# AG04：全路由视觉、可访问性与性能验收：执行清单

## 执行前
- [ ] 父方案批准，前置任务证据完整，当前状态允许启动。
- [ ] 读取本任务上下文、父设计及research；重查dirty文件与验证前置，不静默安装。

## 工作步骤
- [ ] 导出当前全路由，逐条对账AG02/03记录，验证实际loader和参数fixture。
- [ ] 执行目标烟测后扩到全前端/coverage/build/architecture；先检查命令安装前置。
- [ ] 执行主题/视口/状态/降级/键盘视觉矩阵及实际alpha和blur计数。
- [ ] 复核AG01已校准且成功采before的perf harness，按相同方法采after并比较，失败回交对应子任务。
- [ ] 使用隔离环境完成原生窗口/DPI/系统主题/托盘检查；记录不可用平台。
- [ ] 对账规范与变更范围，输出每个R1–R8的证据索引和最终PASS/FAIL/NOT_RUN。

## 验证
目标测试：全部前端smoke/coverage与父implement.md门禁；既有目标域检查先于聚合门。实施前用rg --files核实当前匹配，不凭旧目录猜测测试名。

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
