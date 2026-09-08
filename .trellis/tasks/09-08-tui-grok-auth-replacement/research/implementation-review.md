# 实施与静态审查记录

日期：2026-09-08。基线 `dev @ d881f699`；初始工作树仅有本父任务及三个子任务的未跟踪规划文件。

## 授权与证据边界

本会话用户明确要求实施现有父任务及关联子任务。沿用规划中记录的限制：仅源码、必要测试源码及文档维护与静态审查；不运行测试、构建、lint、just ci、TUI 或真实账号验证，不读取/修改真实认证和配置，不探测/控制当前 Grok Build，不安装、不新增依赖、不提交或发布。

四个成员的 `task.py validate` 均通过，说明 JSONL 引用结构有效；不代表行为验收。父任务和三个子任务均已启动，TUI 在服务 DTO/API 契约冻结后接入。行为 AC 保持未勾选，实施交付与验证完成分开记录。

## 清理子任务静态审查

实现与独立 `trellis-check` 已完成，无待修复发现。涉及 `tui_config.rs`、Codex `utils.rs`、TUI `theme.rs`、README 和 config/Codex 规范。

- 删除旧 tab 变体、过滤及孤立路径/配色，活跃源码无这些旧类型/helper 悬空引用。
- 六页默认顺序、合法自定义排序与独立 Usage 过滤仍在；混合 fixture 保留语言/主题/排序断言。
- 未知标识测试源码断言整份默认回退且首次读取不写源文件。含旧 `opencode_auth` 的配置适用该路径，包括语言/主题回退。
- 有效 OpenCode 配置/session/usage 消费者与 Codex quota core 保持。
- 限定清理文件的 `git diff --check` 通过；测试、编译及 lint 未运行。

## 服务与 TUI 集成

服务及 TUI 源码已交付；独立审查发现已修正并完成最终定点复核，限定范围内无遗留确定性生产阻断。

独立服务审查指出的生产路径问题均已修正并完成静态复核：

- 回存写入报错但回读字节相同，仅能证明本地可见，不能保证持久化。切换/登出必须在此分支停止，不继续覆盖或删除 runtime，也不回滚已可见的新保存库。
- outgoing 不能仅按 scope map key 与身份匹配；必须先验证其实际 issuer/client/auth_mode 等凭据结构，再自动回存。
- Windows DACL 应保持既有保护/继承状态；新增 secret owner-only 需求不能改变普通 preserve_mode 的行为。

TUI 审查已推动修正并完成最终定点复核：来源列表按单行截断分页；别名输入与服务 32 位规则对齐；窄屏优先显示操作状态、认证未验证与 profile 路线；登出确认明确全部 scope/未保存凭据影响；断连清除旧成功摘要，退出保留持久化警告及结果未知提示。极小详情区域也分别说明操作成功/失败与刷新失败；限定范围内无剩余确定性生产缺陷。

父级同步了现有中英文 Grok/TUI 命令文档，移除“无账号快照/仅登出/不解析 token”的过时声明，并说明保存与切换的前提、scope 范围和独立登出的影响。

本会话只读复核[固定官方 GrokAuth 定义](https://github.com/xai-org/grok-build/blob/72a61251fcffb464bcc687aeb5a998e5a98ec0c9/crates/codegen/xai-grok-shell/src/auth/model.rs#L40)：`expires_at` 为可选日期时间，已知元数据有具体类型，`user_id` 是原生必填字符串。实现保留计划的缺失身份显式保存能力，但缺少该字段的目标在写入 runtime 前拒绝；不合成身份，未知扩展字段仍保留。

## 需求—子任务—源码映射

| 需求 | 子任务 | 静态入口与实现 |
| --- | --- | --- |
| R1/R2 退休标识与六页 | cleanup | `ccr-config/src/managers/tui_config.rs` 默认顺序/load；`ccr-codex/src/utils.rs`；`ccr-tui/src/tui/theme.rs` |
| R3 保存/列表/覆盖 | service + TUI | `grok_auth_service/accounts.rs` read_snapshot/save_current；`grok_auth/app.rs` Source/Name/Confirm |
| R4 scope 局部切换/回存 | service | accounts.rs switch_account、credential、matching、save_store、persist |
| R5 删除与独立登出 | service + TUI | delete_account/off_checked/off_inner → `application/auth_off.rs::grok_auth_off_locked`；局部 typed confirmation |
| R6 响应式交互/状态 | TUI | `grok_auth/app.rs` 后台结果收集与 stale；`ui.rs` 列表/详情/弹窗/footer；主 `tui/app.rs` Busy 导航与 tick |
| R7 秘密/锁/权限/冲突 | service + core | opaque revision + Secret；accounts.rs operation/native/leaf lock 顺序与 CAS；core lock.rs 非截断、atomic_writer.rs 写前 DACL |
| R8 profile/MCP 隔离 | service + TUI | 账号路径无 profile_off；TUI 只读 inspect_activation_state；服务测试源码使用真实平台目录的旁路文件断言 |
| R9 当前文档/入口/测试源码 | 三子任务 + 父级 | 当前 spec、README、中英文 grok/tui 文档；main.rs launcher 保持进入统一 Grok Auth 页；独立测试源码已维护 |

## 已维护的回归源码与剩余证据

服务夹具涉及只读保存、官方锁占用、捕获后原生刷新、A/B round trip、身份 enrichment/歧义、其他 scope 与配置文件不变、删除/登出、损坏数据、CAS、回存失败、持久化未确认和操作锁串行。core 夹具包含锁 holder 内容、新 secret DACL 及当前用户 SID。TUI 夹具包含默认取消、后台调用次数/Busy、断连、刷新失败、别名选择、多个来源、主壳空间分配与各尺寸双语帧。

这些均为测试源码，未执行、未编译。最后两项专门故障夹具已补齐并通过独立静态复核：Windows 句柄允许 CAS 读取但禁止 DELETE sharing，覆盖 runtime 实际替换失败后原 runtime 与最新 outgoing 保留；DACL 设置失败注入位于真实写前权限边界，仅限 Windows 测试构建且按 fixture 目录隔离，覆盖同步既有目标与异步新目标的无 payload/临时文件清理。源码齐备不能当作对应行为验证已完成。

源码格式整理仅调用已安装 rustfmt，限定本次拥有的 `.rs` 文件并使用 `--edition 2024 --config skip_children=true`；不属于测试、构建、lint 或 fmt-check 运行。

## 验证状态

| 证据 | 状态 |
| --- | --- |
| 四任务 JSONL 结构检查 | PASS |
| 清理子任务源码与独立静态审查 | PASS（静态） |
| 服务及故障源码最终独立静态审查 | PASS（静态） |
| TUI 最终独立静态审查 | PASS（静态） |
| 父级集成源码/接口/范围与 git diff --check | PASS（静态） |
| 测试、构建、lint、just ci | SKIPPED（用户要求） |
| 实际 UI 帧、文件权限/锁竞争、真实账号切换有效性 | UNVERIFIED |
| 当前 Grok Build、真实凭据/config、安装/push/发布 | 未操作 |

## 交付状态

父任务和三个子任务的源码实施、必要文档/测试源码维护及静态审查已交付。用户随后明确要求“请提交所有改动并归档任务”，已授权全部本地提交及四任务归档。归档表示本轮源码交付收尾，不表示动态行为验收通过；行为 AC 保持未勾选，测试/构建/lint 仍为 SKIPPED。后续双主题实际效果、Windows/Unix 文件行为及真实新会话认证仍需要验证证据。未授权 push/发布。
