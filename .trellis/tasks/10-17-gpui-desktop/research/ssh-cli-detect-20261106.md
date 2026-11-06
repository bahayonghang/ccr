# Native SSH CLI detect consumer

## 最小行为边界

读取实际 consumer 链：`ccr-ui/src/features/sync/SshManagementView.tsx:185` → `commands/ssh.rs:792` → `ssh/sftp.rs:87` → `platform/ssh.rs:257`。这里检测的是已连接远端的 CLI PATH，不是私钥 inventory，也不是 WebDAV Sync。原生已有连接 owner 是 `ccr-app::environments::ssh::Workspace`，consumer 是 Settings 的 `RemoteView`；因此没有在 WebDAV runtime 新建错误入口，也没有修改 Tauri 源/命令契约。

固定映射沿用现有实现：Claude→claude，Codex→codex，Antigravity（内部 gemini 标识）→agy，OpenCode→opencode。只显示 Installed / Not found 和实际返回的单个路径，不编造版本或 Grok 支持。

## 实施与安全合同

- `crates/ccr-app/src/environments/ssh/detect.rs:19` 新增 opaque `CliSession`（无公开字段/Debug），以及固定 `name/installed/path` 的 `CliStatus`。`cli_session` 在入队前捕获已验证连接的 host ID/session UUID；`detect_cli` 复用原 serial owner，在执行前校验同一个 session。相同主机重连也不能重绑定旧请求。
- 只复用 `SshTarget::ssh_command` 的 strict known-host 参数和已有 OpenSSH capability。四条固定 `command -v '…' || true` 指令不接受 renderer 提供的命令；不运行版本、私钥、CLI 安装或本地 fallback。
- PATH 输出上限4096 bytes，严格 UTF-8、绝对单行路径或空值；异常输出不发布部分成功行。传输/取消失败清 active session；旧 authority 被拒绝时不误清新连接。
- `ccr-desktop/src/runtime/environments.rs:105` 在 admission 捕获 authority；同 RequestKey.environment 和 action host 不一致先拒绝。Fixture/Benchmark 拒绝真实 transport；仅测试显式注入原有内存 transport。任务仍由既有 bounded executor 持有。
- `features/environments/remote/detect.rs` 提供明确点击、busy/cancel、静态安全错误和四行结果。host/file/连接动作清结果；`Shell.go` 只在离开 Settings 委托清 CLI，检测中请求取消并递增 generation。selected host、已有编辑器和其他 SFTP 草稿不被这处委托删除；迟到 generation/host 结果被拒绝。
- 两条 SSH control 测试已追加既有 Windows1MiB guard，保留所有其他 owner 的23项。
- `.trellis/spec/ccr/backend/ssh-security.md` 与 app `execution-environment.md` 补充固定命令、连接 authority、无部分行、fixture 与生命周期合同。

## 实际失败、根因与修复

第一次 native focused：新两条 CLI control 均通过；原 `rendered_host_actions_reach_isolated_transport_and_write_remote_document` 因确认保存控件不可见失败。Hunt 源码和安全 bounds 复现表明：旧测试请求1200×1800，但 headless display实际夹至1200×1080；Save位于 y998..1030，可点击；新增CLI区把原尾部 Confirm推到 y1112..1144，超过视口。不是 SSH owner、信任或写入失败。

最小修复是把同一个 document confirmation 区域放到 selected file / Load 之后、200px编辑器之前，保留原按钮、确认/取消、文档 snapshot 与后端调用。没有调整 display、窗口或 timeout；测试增加确认 visible/viewport bounds 断言。临时 bounds probe 已移除。第一次诊断误查了没有 test snapshot 的 Textarea accessibility ID，改为只取现有 button/window bounds 后取得上述证据，没有改生产框架。

## 真实验证收据

Windows，Rust1.98.0，默认测试并行，`--locked --offline`，保留 `--skip export_bindings`。Native artifact：`ccr_gpui-4a39d1c098e6c17e.exe`。

| 检查 | 实际结果 |
| --- | --- |
| Root `cargo test -p ccr-app --lib environments::ssh` | 26/26 PASS；新增固定命令/strict argv、无连接、同主机重连、断开、异常输出、取消，以及原信任/SFTP回归 |
| Desktop `cargo test --lib features::environments::remote` | 最终3/3 PASS；两个新控件/迟到结果用例 + 原保存、备份、草稿用例 |
| 当前 artifact `runtime::environments::ssh_tests` | 1/1 PASS；原 fixture host store 拒绝连接，扩展实际 DetectCli fixture/env mismatch 拒绝，无 known_hosts 初始化 |
| 当前 artifact 既有1MiB guard | 1/1 wrapper PASS，内部25个 exact children；包括两条新SSH控件，RUST_MIN_STACK=1048576和 child-only poisoned lock root，25.31s |
| own局部 rustfmt / diff-check | PASS |
| Desktop strict `clippy --lib --tests -- -D warnings` | SSH/System无lint；唯一source失败是peer `runtime/checkin/operations/credentials.rs:220,266` 两处 expect_used，已报owner，未代改 |

Root第一次 PATH cargo 调用实际命中了 mbx shim，因外置 cache权限未开始Rust检查；之后使用主会话统一的 pinned真实toolchain和既有root target，通过授权升级运行上述Root26项。没有创建新cache、改HOME、clean或改变target配置。

全部 SSH 通过显式内存 transport；没有真实 SSH/socket/keyscan/身份文件读取。没有启动原生窗口，没有真实远端 acceptance，也不把 headless 或1MiB证据称为原生通过。原生验收仍 user-deferred。未提交；独立 review 与共享改动后的完整CI由主会话继续。
