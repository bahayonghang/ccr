# GPUI 02 实施证据

日期：2026-10-18。当前阶段：completed，AC2.1–2.5 已独立验收。下文保留实施中间态与失败历史，以末尾最终验收为准。

## 恢复与依赖

- 用户已批准父任务与全部子任务，本轮明确要求继续后续实施；没有新建任务、重新立项或重复请求已批准实施的授权。
- 恢复 git status：退出 0，分支 dev，HEAD 仍为 1dfe776b；保留上轮全部工程/脚本及未跟踪规划文件，没有回退其他参与者的修改。
- `python .trellis/scripts/get_context.py`、`--mode phase --step 2.1 --platform codex`：退出 0，原 active 为 01；已恢复本阶段路由。
- 完整读取 02 implement/check manifests、prd/design/implement，以及 01 已核对配置链源图；原 manifest 五条引用各自保留，追加源图引用。
- `python .trellis/scripts/task.py start .trellis/tasks/10-17-gpui-02-services`：退出 0，planning -> in_progress。01 外部平台、完整基线/性能和原生矩阵验收仍开放；不借开始 02 将其判通过。
- 依赖判定见父 implement.md 2026-10-18 补充：已验证的工程/接口依赖支撑后续代码；与该代码无关的外部验收继续独立跟踪，最终所有 AC 仍须满足。

## 写入分工与边界

- services_domain：共享配置读取编排、必要的 path-injected current resolution helper/CLI 委托、Tauri wire adapter 与无全局 HOME 的 fixture；唯一 Cargo manifests/locks 写入 owner。
- services_runtime：ccr-app runtime/types/tests、GPUI runtime/app/真实只读配置视图；不写 manifests、领域持久化或 Tauri DTO。
- 主会话：协调、任务/上下文/父台账/证据、原生操作、最终集成；各文件只有一位当前产品写入负责人。独立 check 在明确交接后接手自修权限。
- 保留 registry fallback、snapshot revision、行顺序、空值默认、Secret 既有脱敏与 Tauri wire/export 位置；不修改 profiles 数据库 schema、业务 SQL 或冻结 CcrError variants。
- 新服务只有真实两端共同消费的配置列表编排；不创建一层同名领域 API 包装。阻塞 I/O 由 bounded worker 执行，UI render 不读文件/DB/网络或等待进程。

## 验收记录

实施中。下列条目记录实际命令和结果；两位 worker 的 research evidence 与独立检查完成后继续补齐。单项通过不代表完整功能或原生验收通过。

### 主会话：领域 API 生成原生 fixture

- 显式执行 `cargo test -p ccr-app --locked --offline --test config_read seed_native_fixture -- --exact --ignored --nocapture`。这是只在显式调用时生成采样数据的工具，不是通过 ignore 跳过功能验收测试；常规五项配置测试仍单独运行。
- 首次 sandbox 退出 **1**：mbx 缓存 lock 拒绝访问；日志 `outputs/desktop/services/native-seed.{stdout,stderr}.log`。仅申请既有 cache 访问后相同命令退出 **0**，1 个生成用例通过，完整日志带 `native-seed-authorized` 前缀。
- 子进程只设置 CCR_APP_FIXTURE_OUTPUT 和 CCR_LOCK_DIR；HOME/USERPROFILE 未修改。生成器验证输出为仓库 outputs/desktop/services 下的新直系目录，拒绝覆盖；通过 ConfigManager/PlatformConfigManager API 生成真实 SQLite catalog 与 registry，不复制 SQL。
- 输出 `outputs/desktop/services/native-fixture/`：profiles.db、config.toml、fixture-manifest.toml。输入参数 profiles.toml 是既有 ConfigManager 的逻辑路径，实际持久化 owner 使用同目录 profiles.db，不需要造一个 TOML 文件。
- revision、BLAKE3 写入 fixture manifest；另保存 SHA256/大小和命令回执至 `outputs/desktop/services/native-seed-receipt.json`。仅含虚构凭据，原生呈现仍须脱敏。
- 尚未使用此 fixture 启动 GPUI，不将数据生成成功计为原生 UI 或性能通过。

### 后续原生与检查进展

- 上述 seed 只是生成阶段；随后已实际启动真实 GPUI fixture，列表/当前项回退/脱敏详情/键盘选择/筛选与慢读取期间的文本选区已有截图，见 [native-services.md](research/native-services.md)。首次退出采集包装未可靠传播原生码，已改用实际 Process.ExitCode launcher，不将包装的 0 判为 native pass。
- 慢读取窗口正常关闭时出现 AccessKit invalid HWND panic；严密 launcher 一次复跑 native 0/空 stderr 只说明竞态不是每次发生。独立源码研究及专职补丁/确定性回归正在处理，关闭验收仍开放。
- 独立检查实际根 `just test` 退出 0：47 suites 共 2223 passed、0 failed、25 ignored、33 filtered，未把 ignored 算通过；严格 lint、version/fmt、绑定漂移最终复验及 Tauri profile 37 项测试/命令清单均退出 0。完整命令、首轮失败修复和日志见 [check-final.md](research/check-final.md)。AccessKit 补丁完成后须复验受影响 desktop；无理由不重复无关根全量门禁。
- 新共享服务与运行时契约已写 `.trellis/spec/ccr-app/backend/`；02 implement/check 各 7 条、03 各 6 条 curated context 校验退出 0。CLI/Codex fixture 规范同步由独立检查 owner 负责。
- 主会话补查 tempfile 官方 registry：退出 0，候选/选用均 3.27.0，MSRV 1.63，默认 getrandom；原始选取字段和来源保存在 `outputs/desktop/services/tempfile-registry.json`。这补齐 domain-evidence 形成时尚未单独查询的已有 dev dependency，不改变依赖版本。

### 最新用户执行范围与原生中断

- 用户明确要求继续 Computer Use，只验证核心功能，不做放大缩小等非核心矩阵；后续按真实读取/筛选、失败重试、取消退出和编辑/CAS/应用生命周期执行。缩放/DPI未运行，不伪记通过，也不撤销既有实施授权。
- AccessKit 窄补丁已产生确定性 stock fail/fixed 3 项 pass 和 debug 产物；最终命令层接线/独立复核仍在收敛，见 `research/accesskit-close-fix.md` 与 `research/check-final.md`。
- 补丁后两次 native 启动分别在首次 Computer Use 调用时收到 Esc stop；第二次发生在用户明确恢复请求之后。遵守停止边界，没有重置工具绕过或继续窗口输入；两次仅清理了确认 exe/fixture argv/PID 的本次测试进程，实际 -1 / wrapper 1 均记 interrupted。不是关闭成功或补丁失效证据。
- 02 继续 in_progress，补丁后的核心原生退出复验尚未完成；03–10 尚未开始产品实施。所有原生细节、截图和收据指向 `research/native-services.md`，不因结束当前回合而归档或判完成。

### 本回合最终一致状态

- Windows adapter 仅精确 vendor accesskit_windows 0.34.0；保留来源、许可证、stock 真实失败与修复后 3 项回归。生产锁仅该包由 registry 改 path，其余记录不漂移。独立检查核实四处坐标 COM 接口传播标准 UIA 元素不可用错误，不伪造坐标、不禁用 accessibility。
- mbx 自动迁移桌面 target 的一次副作用已按精确源/目标与原产物 hash 恢复，保留 `outputs/desktop/services/accesskit-close/target-recovery.json`。桌面命令现在通过官方状态和文件身份确认 shim 后，仅在子进程选用项目工具链并配对 Cargo/rustc/rustdoc；普通 Cargo/stub 退出码和全部 target/env 拒绝规则保留，不改全局配置。
- 正常 PATH 的最终 `just desktop-check`、`just desktop-test`、`just desktop-lint`、`just desktop-build-debug` 全部退出 **0**。测试包含 Python 12 项（含原 21 种环境覆盖与 native stub 23）、adapter 3 项、应用单测 3 项、实际 Kit headless 2 项。首次默认命令 E0514 编译器配对失败保留；不使用 clean 或升级全局工具链掩盖。
- 最终 debug 产物 `ccr-desktop/target/debug/ccr-gpui.exe`，SHA256 `94acabe375bc032da8cf868934eae6fb2de4392ab55cd7440849e3e9b84fc741`；最终独立锁 SHA256 `95c30e2ec5d68b3547a3cb219aac7898d867ff906c809af1591dc1b3dfbc41af`。此产物替代原生中断尝试的旧 DD53 产物，**没有新的 patched native pass**。
- 详细命令、原始失败和日志见 [accesskit-close-fix.md](research/accesskit-close-fix.md) 及独立 [check-final.md](research/check-final.md)。02 接线不再有缺失接口的中间态；实现者全部冻结，写权交回 checker。补丁后 desktop Rust 1.95、新核心原生退出复验、后续子任务及最终全根 just ci 仍未完成。

### 2026-10-18 核心续验

- 用户再次明确恢复 Computer Use 并保留已批准实施授权。新 JS kernel 成功恢复；并发键鼠/最小化期间遵守输入 guard，用户明确恢复 CCR 窗口并暂停输入后再继续。没有操作无关窗口或保存其内容。
- 最终 94ac 产物实际完成 core-resume-normal/error/cancel 三个 run，原生退出码均 **0**、stderr 均空、无 ccr-gpui 残留。真实读取/当前项回退、空与命中筛选、脱敏详情、缺失 catalog 错误、补入合成 DB 后 Refresh 恢复，以及 30s 慢读未完成时关闭取消全部有实际观察。
- 慢读 run 总生命期 28.52s < 人工延迟 30s，退出前最后截图仍 reading；结合确定性 runner/drop/join 测试验证关闭取消，不伪造终态或把这视为取消延迟预算测量。
- 原始退出/日志保存在 `outputs/desktop/services/native/core-resume-{normal,error,cancel}/`，汇总 core-resume-summary.json；核心截图 windows-core/。详见 native-services.md 新节。此前中断记录保留，不改写为成功。
- 补丁后 desktop MSRV 复验也已退出 **0**：子进程 RUSTUP_TOOLCHAIN=1.95，经普通 just desktop-check/helper 执行 locked all-targets；Cargo/rustc 实际路径同属已装 1.95 toolchain，版本均 1.95.0。日志 check-desktop-msrv.log，未安装工具/改全局配置/移动 target。
- 此节补齐上一阶段待验证的核心退出与 desktop 1.95；缩放/DPI仍 user-deferred，其他 OS、性能、最终 just ci 与父任务 AC 不据此判通过。

### 最终验收与交接 03

- 独立 trellis-check 已核对三个实际 native receipt、相同 94ac 产物、空 stderr、无 HOME 改写，并检查真实截图；结合领域/Tauri 兼容、bounded runtime、取消/清理与 MSRV 回归，AC2.1–2.5 通过，详见 research/check-final.md。
- task.json 与 prd.md 同步 completed；保留原目录供后续 curated context 引用，没有 archive。03 已经通过正常 task.py start 激活并由独立文件 owner 开始实施。
- 父迁移台账仅将 list_configs 标 verified_core（Local/Claude，02 原生核心范围）；完整 Profiles 路由、写入与其他业务项不据此判完成。父任务跨平台、性能、最终全量门禁继续开放。
