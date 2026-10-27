# AG02：公共控件、配置与资源页面

## 目标
将公共控件与所有操作型页面统一到实底阅读层和轻盈玻璃外壳，保留原编辑与安全行为。

## 前置与范围
AG01令牌、外壳、四组合代表页和基线可用。

公共UI、Profiles、平台设置/认证/源编辑、MCP/Agents/Plugins/Hooks等资源，converter、sync、WSL/SSH与Skillport迁移引导。平台首页与动态观测由AG03负责。

## 验收标准
- [ ] AC1：所有操作路由在覆盖表有实际页面标识/状态证据，没有只改平台首页。
- [ ] AC2：控件normal/hover/pressed/focus/disabled/error、dialog和危险确认在四组合可辨识。
- [ ] AC3：草稿保存/取消/失败保留、CAS/dirty patch、遮罩、权限门与确认流程不变。
- [ ] AC4：公共UI保持单实现和层级边界；不新增重复弹窗或CSS主题分支。
- [ ] AC5：长中文/英文、800×600、缩放、移动导航下主要操作可达。

## 不在范围
不改后端数据、IPC、权限和业务机制，不处理并行GPUI改动，不新增主题体系，不安装/提交/推送/发行。

## 需求追溯与状态
父任务：../10-26-ccr-ui-airy-glass/prd.md；对应R2 R3 R5 R6 R8。父design.md和verification.md的兼容/验收合同适用于本任务。已批准实施；展示实现和检查记录已补齐，当前保持in_progress，最终验收缺口见AG04 integration-summary。

## 2026-10-27 收尾

用户已授权提交与归档；本次归档不补勾未满足的验收项。保留最终验收 open 和原始缺口，见 AG04 closure.md。
