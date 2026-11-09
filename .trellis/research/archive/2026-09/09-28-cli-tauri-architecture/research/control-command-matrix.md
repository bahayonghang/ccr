# T11 控制命令与资源矩阵

日期：2026-09-28；基线：34d8a85e0e48b793733835e0304c8ed33940fcee。仅为规划，未运行原生并发测试。

## 当前事实与目标边界

- 当前 risk、concurrency、timeout_enforcement 来自 `ccr-ui/src/api/generated/command-manifest.json`；registry 为生成源。表中目标资源与策略是待实施设计，不表示当前实现已具有 owner 生命周期保证。
- `ccr-ui/src-tauri/src/commands/runtime_policy.rs:41-81` 将 permit 绑定 handler future；后台 start 返回后，后台作业可能仍在执行。由业务 owner 保持的 admission 需单独验证。
- command 与 usage 的 get/status 以及 install recent、OAuth port probe 已是 Parallel。当前缺陷限定到共享长 permit 下的控制排队；上述查询列入防回归。
- `codex_oauth_login_completed` 会交换 token 并提交账号，不能视为无副作用 status。install 没有独立 get/status 命令，使用已有 recent；OAuth 没有独立 login get/status 命令，标为 N/A，不为验收新增接口。
- owner 以已支持环境与实际执行资源为边界；不因为引入资源名称扩大并行能力。多个入口操作同一资源时共享 admission，控制只需短状态锁或消息通道，不需取得长期执行 permit。

## 逐命令矩阵

| ID | 资源 | 操作 | 当前 risk / concurrency / timeout | 目标策略 | 验证 |
| --- | --- | --- | --- | --- | --- |
| `execute_ccr_command` | command runner 执行资源 | foreground | process_execution / singleton / business_owned | 执行资源 admission 持到真实完成；不阻塞控制通道 | C01/C05 |
| `start_ccr_command_job` | command runner 执行资源 / job_id | start | process_execution / singleton / business_owned | 先取得 admission，再交给后台 job owner；handler 返回不释放 | C01/C05 |
| `get_ccr_command_job_status` | job_id snapshot | query/status | read_only / parallel / completion_aware | 保持 Parallel，只读 owner snapshot | C01/C06 |
| `cancel_ccr_command_job` | job_id cancel handle | control/cancel | process_execution / singleton / business_owned | 按 ID 直达 owner；不取得执行 permit | C01/C06 |
| `llmusage_install_detect` | 安装检测 | query | read_only / parallel / completion_aware | 保持现有 Parallel 查询及完成语义 | C02/C06 |
| `llmusage_install_probe_capabilities` | 主机安装能力 | query | read_only / parallel / completion_aware | 保持现有 Parallel 查询及完成语义 | C02/C06 |
| `llmusage_install_plan` | 安装计划及其受控进程资源 | foreground/prepare | process_execution / singleton / business_owned | 保留受控执行及确认；占用执行 permit 时仍允许控制调用 | C02/C06 |
| `llmusage_install_execute` | InstallService attempt slot / attempt_id | start | process_execution / singleton / business_owned | 保留 opaque capability；admission 由 attempt owner 持到真实清理 | C02/C05 |
| `llmusage_install_cancel` | attempt_id cancel handle | control/cancel | process_execution / singleton / business_owned | 按 ID 直达 InstallService；不取得执行 permit | C02/C06 |
| `llmusage_install_recent` | InstallService ring buffer | query/recent | read_only / parallel / completion_aware | 保持 Parallel；作为现有事件恢复接口 | C02/C06 |
| `llmusage_install_manual_catalog` | 安装手动指令目录 | query | read_only / parallel / completion_aware | 保持现有 Parallel 查询 | C02/C06 |
| `llmusage_install_check` | 检测结果与主机能力 | query | read_only / parallel / completion_aware | 保持现有 Parallel 查询及完成语义 | C02/C06 |
| `import_usage_v2` | 当前 usage 导入资源 | foreground | local_mutation / module_exclusive / completion_aware | 同步导入与后台导入共享 T06 admission，持到进程清理 | C03/C05 |
| `import_all_usage_v2` | 当前 usage 导入资源 | foreground | local_mutation / module_exclusive / completion_aware | 全量与单源导入共享 T06 admission，持到进程清理 | C03/C05 |
| `start_usage_import_job_v2` | 当前 usage 导入资源 / job_id | start | local_mutation / module_exclusive / completion_aware | 复用 T06 原子 admission/token；保留已有 job 的幂等复用 | C03/C05 |
| `get_usage_import_job_status_v2` | job_id snapshot | query/status | read_only / parallel / completion_aware | 保持 Parallel，只读 owner snapshot | C03/C06 |
| `cancel_usage_import_job_v2` | job_id cancel handle | control/cancel | local_mutation / module_exclusive / completion_aware | 直达 T06 owner；不取得 usage 执行 permit | C03/C06 |
| `codex_oauth_login_start` | 当前 OAuth login slot / listener | start | secret_mutation / module_exclusive / completion_aware | 先拥有 listener 与 admission，再发布 login_id；owner 持到终态清理 | C04/C05 |
| `codex_oauth_login_completed` | login_id / 账号提交资源 | control/complete，含秘密写入 | secret_mutation / module_exclusive / completion_aware | 交给 login controller 执行可取消 exchange；账号提交仍保留互斥与授权 | C04/C06 |
| `codex_oauth_login_cancel` | login_id cancel handle | control/cancel | secret_mutation / module_exclusive / completion_aware | 直达 login controller；不等待 exchange 的执行 permit | C04/C06 |
| `codex_oauth_submit_callback_url` | login_id callback slot | control/callback，含秘密写入 | secret_mutation / module_exclusive / completion_aware | 短临界区校验并投递 owner；重复或过期回调按状态拒绝 | C04/C06 |
| `codex_is_oauth_port_in_use` | 现有回调端口探测 | query/port | read_only / parallel / completion_aware | 保持 Parallel；不将端口占用等同于登录状态 | C04/C06 |
| `codex_release_oauth_port` | ProcessGateway 拥有的端口进程 | control/release | secret_mutation / module_exclusive / completion_aware | 控制通道可达；仅取消可证明归属的进程，保留现有报告与确认 | C04/C06 |

## 逐命令族验收

- C01 / command：foreground 执行 fixture 持住共享执行 barrier；对已有 job 调用 get_ccr_command_job_status、cancel_ccr_command_job，分别在 barrier 释放前收到 snapshot 和 cancel owner acknowledgement。覆盖 start_ccr_command_job 的后台 owner 与 execute_ccr_command 的 foreground 路径。
- C02 / install：用 llmusage_install_plan 或另一受控 foreground fixture 持住现有共享执行 barrier；llmusage_install_cancel 在 barrier 释放前到达既有 attempt owner；recent、detect、probe_capabilities、manual_catalog、check 各逐 ID 验证原有可达性。独立 get/status 为 N/A。
- C03 / usage：分别用 import_usage_v2 和 import_all_usage_v2 的受控同步导入 fixture 持住执行 barrier；cancel_usage_import_job_v2 必须在 barrier 释放前到达目标 owner，status 必须可读。测试可注入带已存在 job 的 owner，以验证 admission 前的控制路径；外部 llmusage 和 SQL 所属不改变。
- C04 / OAuth：在 codex_oauth_login_completed 的 token exchange 或受控 body read 设置 barrier；login_cancel、submit_callback_url 和 release_oauth_port 各自可到达负责校验/取消的 owner，不等待长期执行 permit。login_completed 自身的控制投递也须在无关 codex_auth 长处理下可达，其 token exchange/提交仍由 controller 串行管理；重复完成、过期回调返回既有类型的确定结果。port probe 保持可达；独立 login get/status 为 N/A。release 不得终止外部不归属进程。
- C05 / admission 生命周期：对 command、install、usage、OAuth 四类 start 分别使用 started/cleanup barrier，等待 start handler 已返回，再尝试相同资源第二次 start。第二次不得创建并发作业；按既有协议拒绝、等待或复用同一 job。首作业收到 cancel 或产生终态时，清理未结束仍持有 admission；只有实际完成和清理结束后才允许新作业。同步/后台 usage 入口使用同一资源约束。
- C06 / 权限与完成：逐 ID 比较原有 authorization、confirmation 和 audit；验证辅助窗口拒绝、缺少确认拒绝、错误或过期 ID 不影响当前作业。任何被允许的控制必须先通过原授权。取消应先被接收，最终 Cancelled 仍等待真实清理；cleanup failure 保留错误分类。
- 测试通过 owner acknowledgement 和 barrier 顺序判定；采用暂停时钟或明确有界看门狗。禁止先释放 barrier 再观察取消、任意 sleep 或仅以全部 Parallel 的 manifest 快照作为成功证据。

## 源码锚点

- registry 风险/并发推导：`ccr-ui/src-tauri/src/commands/handler_registry.rs:178-263`。
- command start/status/cancel：`ccr-ui/src-tauri/src/commands/command_exec.rs:1834-1875`。
- install start、事件转发与 cancel/recent：`ccr-ui/src-tauri/src/commands/install.rs:32-87`。
- usage start/status/cancel：`ccr-ui/src-tauri/src/commands/usage.rs:1509-1591`；同步入口 `ccr-ui/src-tauri/src/commands/usage.rs:1423-1505`。
- OAuth completion/cancel/callback/port：`ccr-ui/src-tauri/src/commands/codex_auth.rs:1436-1561`。
