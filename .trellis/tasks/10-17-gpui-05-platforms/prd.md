# GPUI 05：平台设置与认证

## Goal

迁移各平台设置、认证、供应商及登录退出流程。

## Parent and Dependencies

父任务：[gpui-desktop](../10-17-gpui-desktop/prd.md)。需求映射：R4、R6、R9。

前置：04 完成。 父子关系不自动保证依赖完成。

## Requirements

- 迁移 Claude/Codex/Grok/OpenCode/Antigravity 等现有 descriptor 实际设置能力。
- 迁移认证状态、账户管理、登录/登出、OAuth/device/callback 等现有流程及供应商目录。
- 保留 managed lock、dirty-key patch、秘密与非秘密模板边界；按 capability 展示功能。

## Acceptance Criteria

- [ ] 5.1：每个平台只显示现有支持的能力，unsupported 不能误显示成功。
- [ ] 5.2：认证成功/拒绝/超时/取消、callback 冲突、窗口关闭均有测试；token 不进入 UI 日志或持久 UI 偏好。
- [ ] 5.3：设置保存只写 dirty keys，保留未知字段、managed locks 和 CAS；退出登录不误删无关配置。
- [ ] 5.4：供应商模板不保存 secret；账户更换后所有旧结果失效。
- [ ] 5.5：迁移台账平台设置与 auth 项可追溯到 native action 和测试。

## Out of Scope

不扩大到本任务未列的业务域；不删除旧表面、不改真实用户数据、不降低已有门禁。父任务声明的完整迁移范围由后续任务继续承接。

## Status

planning。技术待验证项由本任务检查闭环，不视作已完成。开始前读父任务 design.md、implement.md、当前子任务全部规划和 curated context。
