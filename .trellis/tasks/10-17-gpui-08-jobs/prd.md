# GPUI 08：命令、同步与签到

## Goal

迁移命令任务、安装、同步与签到，保留取消、超时与密钥边界。

## Parent and Dependencies

父任务：[gpui-desktop](../10-17-gpui-desktop/prd.md)。需求映射：R4、R6、R7、R9。

前置：02、03、04 完成。 父子关系不自动保证依赖完成。

## Requirements

- 迁移命令工作台、工具安装、同步、签到及余额刷新。
- 取消、超时、重试、并发/单例、离开页面、关闭窗口、订阅恢复等 lifecycle。
- 保留命令 allowlist、network/local/secret mutation 分级、WebDAV 密码生命周期和签到 per-origin 限流。

## Acceptance Criteria

- [ ] 8.1：合法命令执行有真实进度与终态，非法参数/不支持环境在启动前拒绝。
- [ ] 8.2：取消前启动、重复 start、终态后迟到事件、进程树清理失败及窗口关闭都有 fixture 测试。
- [ ] 8.3：秘密只传入 domain 生命周期，不进事件/日志/历史；同步冲突与失败不报告成功。
- [ ] 8.4：签到多账户刷新沿用 per-origin queue/throttle，避免 UI 并发扩大请求量。
- [ ] 8.5：台账所有任务动作对应 native 操作及状态恢复路径，S2 输入延迟达标。

## Out of Scope

不扩大到本任务未列的业务域；不删除旧表面、不改真实用户数据、不降低已有门禁。父任务声明的完整迁移范围由后续任务继续承接。

## Status

planning。技术待验证项由本任务检查闭环，不视作已完成。开始前读父任务 design.md、implement.md、当前子任务全部规划和 curated context。
