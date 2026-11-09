# Auth 展示迁移设计

使用父 design 的语义与环境契约，不引入 Auth 专用输出系统。

## 文件责任

- `crates/ccr-cli/src/commands/codex/auth/*.rs`、`commands/claude/auth/*.rs`、`commands/grok/auth.rs`：仅展示相关行及 import；mod 文件只在必要注册变化时修改。
- `crates/ccr/tests/commands/output_presentation.rs`（新增）及 commands.rs 注册；补充现有 grok_auth/claude_profile/codex_profile 的 Auth 回归。
- domain services、DTO、错误返回与参数定义不属于编辑范围。shared auth-off printer 只改变展示，不变业务结果。

## 分支展示

| 分支 | 结果/字段 | 建议 |
| --- | --- | --- |
| save 成功 | 一个成功结果、既有描述/邮箱/订阅字段 | auth list |
| switch 成功 | 成功结果及既有当前信息 | auth current |
| 重复名称失败 | 错误原因、覆盖后果 | 使用已校验名称的 save --force -- NAME |
| 未登录 | 既有未登录说明 | 原有官方登录命令 |
| 不支持存储 | 既有错误/警告、store/reason | 既有官方登录或设置说明，不执行设置 |
| list 有数据 | 原有表格、普通统计 | auth current；名称未知时 switch --help |
| 正常空状态 | 普通空状态说明 | 适用的 save --help 或登录 |
| current 未保存 | 警告与既有信息 | save --help |
| API Key / Provider Key | 既有模式、必要说明 | 不宣传保存 OAuth，必要时 list |
| rename 成功 | 成功结果，旧名/新名 | auth list |
| delete 成功 / 取消 | 成功 / 中性取消 | 成功后可 list，取消不追加成功建议 |
| import / export | 既有成功/部分结果、数量、加密说明 | 查看账号或既有 import/帮助入口 |
| sync / repair / update / off | 按已有结果分类，保留限制和风险 | 只保留适用且已存在的下一步 |

表中的 NAME 是实现计划参数槽，不打印到建议。import、repair 的失败和部分成功以现有结果对象/分支为准，不从输出文字推断整体成功。

## 兼容性与回退

已存在 error 后 Ok 的分支保留返回值。记录该缺陷，避免新增测试错误地认可业务成功；展示测试可以断言基线退出码和错误文本同时存在。

复用项目合成 env/账号夹具，禁止真实凭据验证。回退仅恢复 handler 和测试，不能恢复/覆盖真实 auth 数据。
