# GPUI 08：命令、同步与签到：执行计划

## Preconditions

02、03、04 完成。 用户须审阅父任务最新方案。若批准后发现需要改变范围/兼容行为，重新评审变化部分。

2026-10-28 依既有父任务“可消费工程/开放验收分开跟踪”规则继续：02 completed；03 单层实色 fallback 与有界 runtime 可消费；04 生命周期/命令策略已独立检查，新增焦点/跨平台恢复动作修复已有实际 red/green 回归，真实 fixture CAS/Cancel/Refresh 已在带源码身份的 baseline 原生复验。04完整新候选/全命令原生验收仍开放，不标完成。08 可独立接入现有受控进程、命令校验、同步/签到 owner；若发现所消费接口、数据或安全回归先修复，不豁免完整验收。用户已明确恢复全部既有实施批准，无需新建任务或重复批准。主会话唯一拥有共享 exports/manifests/三份 lock/shell；只用隔离假进程/假 HTTP fixture，不向真实服务同步/签到/刷新余额。

## Ordered Work

### 2026-11-05 continuation

用户继续既有 GPUI 重构；恢复现有 08，不新建任务。会话起点 `dev@5d069e63`，工作区干净。优先关闭命令订阅恢复与 foreground/help，再接入已有 `ccr_cli::services::install_service::InstallService` 的 llmusage 安装纵向链。命令实现者拥有共享 jobs owner 与 native jobs 文件；安装实现者只拥有新增 native install adapter/view；主会话唯一拥有 Services、exports、manifest/lock、shell、台账及规格；独立 check 在集成后进行。

1. 原子订阅加快照恢复，覆盖运行中离开/返回、终态返回、迟到事件、取消；foreground/help 保持受控 runner 和同一 allowlist。
2. 安装入口复用 domain 的 canonical PlanId、TTL/single-use、host 校验和 attempt 单例；先展示计划再明确确认。离页仅脱离视图，关闭应用必须等待取消/清理；fixture 禁止真实探测/安装。
3. 默认并行的 owner/adapter/控件回归、严格 Clippy、格式、敏感写入和 Tauri consumer 回归；构建匹配源码身份的 native 候选，只用隔离合成进程。日志位于 `outputs/desktop/continuation-20261105/`。
4. 分别记录 source、headless test 和 native 证据；完整 08、父任务与 DPI/跨平台/性能验收不因局部通过而关闭。

独立检查发现新安装入口实际消费的三个 owner 缺陷：detect 的无界 `.output()`/未等待 tree cleanup 与 UTF-8 字节截断；ring redactor 复用原偏移切已修改字符串；exec 无期限等待 reader。先保存 red 回归再在上述 CLI 原 owner 修复、跑合成 flood/timeout/held-reader 和 consumer 回归；不在 native adapter 掩盖错误，不扩大到无关命令。

同步/签到/余额的网络执行、写账户和 per-origin queue/throttle 尚未实现。研究只确认签到批量操作的全局并发限制，未证实 PRD 所述 per-origin queue/throttle；不能按现有能力计入验收。

2026-11-05 route slice：`/commands` 展示既有 `command_catalog`，空目录给明确空态，不可执行命令在启动前拒绝；收藏和最近命令仍留在页内且不执行。`/checkin` 只读已有 SQLite 账户和已保存余额，缺库不创建、不写账户、不联系提供商，缺失余额不填 0。`/sync` 只读规范 `sync_folders.toml`，缺文件保持未配置，不读旧 `sync.toml`、不写密码、不接触 WebDAV、不报告同步成功。纸面保持实色，侧栏仍为 13 项。PRD 8.1–8.6 不因这三段本地读取标完成。

1. 读取 curated specs、父任务设计和当前代码；记录修改边界及基础 fixture 状态。
2. 迁移命令工作台、工具安装、同步、签到及余额刷新。
3. 取消、超时、重试、并发/单例、离开页面、关闭窗口、订阅恢复等 lifecycle。 保留命令 allowlist、network/local/secret mutation 分级、WebDAV 密码生命周期和签到 per-origin 限流。
4. 先验证 service/adapter 的行为，再验证 native UI；失败/取消/冲突测试使用隔离 fixture。
5. 更新父任务功能台账，写 evidence.md 的实际命令、结果、截图、未运行项。
6. 通过本任务验收再交由 check 角色独立核对，遵守项目角色写入权限。

## Validation

- 目标 ccr-core/ccr-sync/ccr-checkin 测试
- just desktop-test；just desktop-test-ui；just secret-write-check
- just tauri-process-smoke；Tauri command/sync/checkin 回归
- controlled fake process/HTTP server 测取消、超时、限流与重试

上列新 desktop 命令以 01 的交付为前提，perf/package 以 10 的交付为前提。若命令或 fixture 尚不存在，先完成相应实现，不写“检查通过”。Rust 行为测试保持默认并行并 skip export_bindings。

## Completion

逐项对应 prd.md 验收，保存失败修复后的最终结果；不提交/发布、不 archive 未完成工作。父任务最终验收负责全量整合。
