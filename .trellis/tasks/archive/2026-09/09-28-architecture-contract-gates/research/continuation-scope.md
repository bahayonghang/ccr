# 2026-09-28 后续实施范围

用户在首轮实施交付后要求：继续完成后续任务。

## 本轮处理

1. 最小更新根 workspace 与独立 Tauri workspace 的 rustls 锁定依赖，处理上一轮 cargo audit 的 RUSTSEC-2026-0285；执行前核对官方公告数据库与可解析修复版本。不新增依赖或修改既有 feature/平台范围。允许必要的同系传递依赖升级，须逐项记录。
2. 先运行安全审计及依赖治理，再执行受影响检查和最终聚合。保留原失败日志，不降低门禁或改变并行策略。
3. 两份原受保护 .tmp 脚本的最小 lint 修改已向用户单独询问；收到明确授权前保持原字节，不移动、不删除、不忽略。
4. 独立 Trellis checker 核对剩余验收与新增变更；可在本机完成的行为验证继续执行，真实账户、跨平台和原生验收不得以替代测试标记通过。
5. 主会话负责更新汇总与证据索引；子代理只能写各自分配的代码或报告，不得覆写父报告、规划生成器或历史证据。
6. 原 Insights 状态保持不变。当前授权不包含推送、发布或删除用户文件；提交/归档仍须遵循原任务边界。

## 已知测试 lint 的后续修复

处理首轮 T09 检查报告保留的 4 项 Tauri all-targets Clippy 测试 lint。只调整测试结构和断言写法；全局环境锁覆盖范围、并发策略、生产业务与原断言语义不得削弱。测试源码由 implement_t09 单独负责，Cargo 检查在安全更新 owner 完成后串接，完整 suite 保留原并行执行。

## 代理检查路径

本轮再次尝试派发专用 trellis-check，平台返回 agent thread limit reached。复用已有代理做非作者检查，按报告明确独立范围；该路径不记录为专用角色派发成功。

## 独立复核新增验收

T08 AC1/design.md 的后端 patch 后 reread 不能由既有内存 merge 测试完全证明。本轮在 Codex 后端加入临时文件、共享生产持久化 helper、磁盘再次读取的 model-only 回归，断言原通知数组和非目标字段保留。测试不覆盖 State、IPC admission、cache invalidation 或 WebView 端到端。

T06 design.md 明确要求 Windows/Linux/macOS tree cleanup matrix。macOS 缺执行证据属于未完成验收，不能降级为可选测试。真实 provider OAuth 登录按 T11 design.md 不调用，属于独立未测范围。

## 最终聚合新增失败

第一次续作 just ci 在 doctor_timeout_terminates_parent_and_grandchild 缺少 grandchild.pid 时失败。修正范围限于 fix.rs 私有 helper 与测试：保留生产 spawn 后立即开始 deadline 的顺序；树清理测试先确认父/孙进程 ready，再测试同一等待回收逻辑；另测真实生产入口 deadline。保留受控延迟反例，原宿主启动延迟原因仍未确认。
