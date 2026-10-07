# Auth CLI 提示展示迁移

## 目标

将 Codex、Claude、Grok Auth 的结果、字段和建议迁移到简洁符号与分组，完成父任务 R2–R4、R6、R7 及 AC1、AC4、AC7。

## 背景

Codex save 的邮箱与提示标题均使用 info，已知账号仍给名称占位符（save.rs:57-73）。Auth info 调用为 127 处。JSON/取消已在部分 handler 中单独处理，应保留（grok/auth.rs:49-77）。证据和完整清单在父 research。

## 要求与范围

- 覆盖 Codex Auth 的 save/list/current/switch/delete/rename/import/export/sync/repair/update/off，人类输出中适用的展示分支。
- 覆盖 Claude Auth save/list/current/switch/delete/off 与 Grok Auth 全部人类分支。
- 使用既有服务结果；不增加认证检查、进程检查、读取凭据或网络请求。
- 保存成功默认建议查看账号；失败给当前分支的恢复动作；已知名称安全传参，无需要填写的命令占位符。
- 保留运行中警告、导出加密说明、确认、覆盖、取消和部分成功语义。
- 保留 JSON DTO、流、退出码、各平台脱敏和存储行为。发现既有退出码问题只记录。

## 验收标准

- [x] save 首行、缩进邮箱和下一步符合父 AC1；缺少邮箱/描述不打印空字段。captured binary 断言及父实际 Windows 18 格 save/缺字段 screen 均通过；normal/NO_COLOR 首行 ✓，dumb 为成功:。
- [x] 成功建议最多两项，不默认建议切换回当前保存账号；错误建议与实际分支一致。依据：当前源码独立复查及已执行的 Auth 展示断言，进程环境限制另见 verification.md。
- [x] 覆盖恢复命令能传入普通/前导连字符名称，不泄露密码和密钥；必要覆盖说明完整。普通与前导连字符的 Codex/Claude 建议均在 2026-10-07 进程断言中通过；全流程隔离验收仍未完成。
- [x] 未登录、不支持存储、API Key、Provider Key、未知、空列表、取消、失败和部分成功没有错误成功标签。已产出的状态由隔离 binary 验证；部分导入由既有 renderer 3 项与实际失败/取消/运行中分支共同支持。Unknown 当前无 native producer，仅源码分支复查，runtime 为 NOT_RUN。
- [x] 现有 JSON 可解析且保持结构和值；实际失败退出码与迁移前一致。依据：路径隔离后的展示 22 项及既有 Grok/Claude/Codex 8/10/14 项回归；未运行完整加密密码交互。
- [x] 合成 binary fixture 验证消息与业务结果；各受影响 Auth 文件有迁移处置记录。路径独立门槛通过后已解除临时 Windows ignore；历史未隔离回执保留。

## 依赖与状态

依赖 C1 共享接口；接口主体及补充局部门槛已完成，联合门槛由父任务记录。C2 已迁移并完成独立源码与路径安全复查。用户于 2026-10-07 批准最小路径扩展，修复后展示 22 项与 Grok/Claude/Codex 8/10/14 项实际 Windows binary 回归通过。父实际 Windows 18 格 save/缺字段 screen 已独立核对。C2 保持 in_progress；父完整 native 总体验收、full CI 与交付由主会话收口。每条验收依据分别采用源码、renderer、binary 和原生回执；Unknown 无 native producer、完整加密密码交互、其他 OS、hosted 与真实账号仍为 NOT_RUN。详见 checks/verification.md。commit、发布和归档未授权。
