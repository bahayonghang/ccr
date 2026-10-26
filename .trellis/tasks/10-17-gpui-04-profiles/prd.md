# GPUI 04：配置与 Profiles

## Goal

接入配置目录、profile 生命周期、CAS、原子写与冲突交互。

## Parent and Dependencies

父任务：[gpui-desktop](../10-17-gpui-desktop/prd.md)。需求映射：R4、R6、R9。

前置：02、03 完成。 父子关系不自动保证依赖完成。

## Requirements

- 迁移配置目录读取、创建/编辑/删除、profile 切换、应用/停用、备份恢复和已有 converter 能力。
- 保留 profiles.db 单一 owner、resource version/CAS、冲突反馈、原子写与字段级 patch 语义。
- 配置列表/编辑器/保存结果构成真实闭环，支持取消编辑和返回。

## Acceptance Criteria

- [ ] 4.1：列出 -> 编辑 -> 校验 -> 保存 -> 重读 -> 应用/停用的 fixture 测试通过；UI 与实际当前 profile 一致。
- [ ] 4.2：两个客户端从相同 revision 保存时，一个成功另一个冲突，不丢字段、不静默覆盖。
- [ ] 4.3：无效输入、磁盘失败、备份失败、apply 失败保持原状态并反馈可恢复原因。
- [ ] 4.4：敏感字段遮蔽，日志/截图/报错不含 secret；删除/恢复遵循现有确认语义。
- [ ] 4.5：迁移台账 config/profile/converter 项有 native 对应和回归证据。

## Out of Scope

不扩大到本任务未列的业务域；不删除旧表面、不改真实用户数据、不降低已有门禁。父任务声明的完整迁移范围由后续任务继续承接。

## Status

已批准并实施中；02完整验收、03可消费工程与用户指定核心流程已独立通过。03未完成的UIA/IME/延期矩阵继续跟踪，按父implement.md依赖判定继续04，不重新请求实施批准。技术待验证项仍由本任务检查闭环，不视作已完成。
