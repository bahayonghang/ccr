# GPUI 08：命令、同步与签到：设计

## Change Boundary

- ccr-desktop/src/features/jobs/、install/、sync/、checkin/，及各自 runtime adapter
- crates/ccr-app/src/jobs/、sync/、checkin/
- ccr-ui/src-tauri/src/commands/command_exec.rs、install.rs、sync.rs、checkin.rs、checkin_jobs.rs 的实际共享提取
- crates/ccr-cli/src/services/install_detect.rs、install_ring_buffer.rs、install_exec.rs：仅修复新入口实际消费的无界探测、UTF-8/多字段脱敏 panic 与 reader cleanup 无期限等待，不扩展为全仓进程重构

这些是预期边界，实施者先按职责定位具体函数，不能为方便而扩大重构。父任务 [architecture](../10-17-gpui-desktop/research/architecture.md) 提供源码证据；完整边界见父任务 design.md。

## Mechanism

复用 ccr-core ManagedProcess、ccr-sync、ccr-checkin；UI 仅发 typed action。runner 完成 process wait/reap/reader cleanup 才写 terminal，cancel action 只请求取消。snapshot query 用于先订阅后恢复，关闭 UI 不 drop 中途持久化操作。

命令页原子订阅与快照来自同一共享 owner，通知溢出后恢复最新有界状态；重返页面自动恢复观察，隐藏页不轮询，渲染更新最多 10Hz。GPUI foreground 与 background 使用同一可恢复流式 runner，仅请求策略模式不同；这不承诺复刻 Tauri one-shot foreground 的输出封顶失败和 wire 形状。帮助只构造经过 allowlist 验证的固定 `help <target>`。

llmusage 安装直接消费已有 `ccr_cli::services::install_service::InstallService`，不重复提取薄包装到 ccr-app。Services 持有 application-lifetime InstallOwner 安全投影；canonical PlanId、120 秒 TTL、single-use、host fingerprint 和 attempt 单例继续由原 domain 持有。原生页先展示计划再显式确认；tracked app job 排空事件，关闭时请求 domain 取消，并等待 attempt slot 实际释放后才报告终态。页面订阅只读取有界安全投影，离页不释放执行 owner。生产 fixture adapter 禁止真实主机探测和安装，测试只通过原 domain 的 test-support 合成执行器注入。

## Contract and Compatibility

继承父任务的 typed request/environment/generation、secret masking、CAS、原子写和 job terminal 契约。UI 仅负责呈现和 action；领域 owner 负责数据。每个新增入口补相应失败/取消/空结果，不添加通用动态 dispatch。

## Rollback

保留 Tauri/CLI 入口。按域提取和 native 接入分别形成可审阅变更，契约回归失败先修复；不通过迁移数据库或删旧功能绕过失败。无法验证的 OS 或能力在 evidence.md 记未运行，不标完成。
