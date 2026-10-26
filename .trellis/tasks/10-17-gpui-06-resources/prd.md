# GPUI 06：MCP 与扩展资源

## Goal

迁移 MCP、Skills、Agents、Hooks、Plugins、命令与提示词资源。

## Parent and Dependencies

父任务：[gpui-desktop](../10-17-gpui-desktop/prd.md)。需求映射：R4、R6、R9。

前置：02、03、05 完成。 父子关系不自动保证依赖完成。

## Requirements

- 迁移 MCP、Skills、Agents、Plugins、Hooks、Slash commands、system prompts、output styles、statusline 现有业务。
- 列表、搜索、详情、编辑与已有安装/卸载流程；复用 ccr-skills/ccr-store/ccr-types。
- 按能力区分平台和环境，统一资源编辑状态而不强迫不同配置格式共享写法。

## Acceptance Criteria

- [ ] 6.1：台账内所有资源动作均有真实 service 连接，安装/卸载反馈与结果重读一致。
- [ ] 6.2：无效资源、冲突、缺失工具、安装失败、取消、权限不足状态明确；不可访问环境不回退本地。
- [ ] 6.3：资源路径和归档输入继续通过现有边界，不能从 UI 跳过验证直接写任意路径。
- [ ] 6.4：虚拟化长列表键盘选择/编辑焦点保持，长名称不遮挡主操作。
- [ ] 6.5：原有领域 fixture 和 Tauri 资源回归通过。

## Out of Scope

不扩大到本任务未列的业务域；不删除旧表面、不改真实用户数据、不降低已有门禁。父任务声明的完整迁移范围由后续任务继续承接。

## Status

planning。技术待验证项由本任务检查闭环，不视作已完成。开始前读父任务 design.md、implement.md、当前子任务全部规划和 curated context。
