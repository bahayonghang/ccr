# GPUI 09：执行环境与桌面集成

## Goal

接入 Local/WSL/SSH、系统窗口、托盘与退出生命周期。

## Parent and Dependencies

父任务：[gpui-desktop](../10-17-gpui-desktop/prd.md)。需求映射：R5、R6、R8、R9。

前置：04、05、06、07、08 完成。 父子关系不自动保证依赖完成。

## Requirements

- 接入现有 Local/WSL/SSH 环境，保持 capability、路径解析、SSH trust 与 SFTP 行为。
- 迁移托盘、隐藏/恢复、退出确认、窗口状态、系统打开与通知等现有 shell 能力。
- 区分 OS 支持与环境能力，完善用户数据与新 UI 偏好隔离、并存及关闭清理。

## Acceptance Criteria

- [ ] 9.1：Windows WSL detect/read/write 路径 fixture 通过；非 Windows 不出现伪 WSL 入口。
- [ ] 9.2：SSH 首次信任、key changed、认证失败、断线、重连、取消均有边界测试，不静默接受 host key。
- [ ] 9.3：环境切换后缓存/编辑 draft/job 显示无串环境，Local-only 功能给出原因而非读取错误来源。
- [ ] 9.4：托盘与关闭行为可配置且与已有语义对应；退出后 owned process、task、watcher 有清理证明。
- [ ] 9.5：三 OS native shell 能力表明确 pass/unsupported/not_run，unsupported 需说明产品影响，不能当完成。

- [ ] 9.6 (R5/AC9)：本域遵循父 [轻盈玻璃合同 V1–V4](../10-17-gpui-desktop/research/airy-glass-plan.md)，复用 03 唯一主题/材质层；V3 本域场景在 light/dark 及 glass/solid 下有原生证据。阅读/编辑/图表/日志/确认保持实色，无新增 blur，既有业务与焦点/虚拟化不回归；未具备的模式/平台如实标记。

## Out of Scope

不扩大到本任务未列的业务域；不删除旧表面、不改真实用户数据、不降低已有门禁。父任务声明的完整迁移范围由后续任务继续承接。

## Status

in_progress。技术待验证项由本任务检查闭环，不视作已完成。开始前读父任务 design.md、implement.md、当前子任务全部规划和 curated context。9.1 实机读写、真实环境切换、进程树退出清理、跨 OS 原生壳与视觉收据仍未齐备；验收框保持开放。
