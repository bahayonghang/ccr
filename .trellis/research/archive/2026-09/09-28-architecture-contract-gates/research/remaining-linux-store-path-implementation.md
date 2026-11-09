# ccr-store 路径恢复测试的平台修正

## 结果

仅修改 `crates/ccr-store/src/sessions/providers.rs` 中 `restores_codex_live_jsonl_under_tempfile_with_windows_path_shapes` 一个测试。所有平台继续验证原路径与 canonical 路径可恢复；Windows 继续验证反斜杠路径可恢复；Unix 验证替换后的路径不存在且恢复返回错误。生产路径规则、依赖、规范与测试并行设置均未修改。

## 根因与责任归属

- 原测试在 `providers.rs:2646` 将临时目录路径中的所有斜杠替换为反斜杠，随后无平台条件地要求恢复成功。独立 WSL 编译并运行原 exact 测试，在原 `providers.rs:2647` 重现失败；此前原路径和 canonical 成功断言均已执行通过。
- Unix 使用斜杠分隔目录。替换后的字符串是包含反斜杠的相对文件名，未指向已创建的临时文件。新增 Unix 断言已经实测路径不存在，恢复调用返回错误。
- `AgentSessionProviderRegistry::restore_source` 在 `providers.rs:590-615` 从私有归档记录重建 provider source，然后调用 `validate_stored_source`。该 owner 在 `providers.rs:768-783` 对本机 root 和 physical_path 进行 canonicalize，再检查根目录包含关系与 provider shape。
- Tauri 在 `ccr-ui/src-tauri/src/services/agent_sessions.rs:645` 保存原生 physical_path；在 `:672-688` 以 `PathBuf::from(&stored.file_path)` 传回 registry。现有功能是本机会话浏览，未定义将其他操作系统路径重写为当前系统路径的迁移契约。因此修正测试的平台预期即可，无需改变生产路径解释规则。

## 源码与历史证据边界

- HEAD：`34d8a85e0e48b793733835e0304c8ed33940fcee`，分支 `dev`。
- 修改前 providers.rs SHA-256：`9f9c8188b81cd587d8703a22988202f546f75a9ffac001f0cc9d1bc52b85b0ec`；原 bytes 与 HEAD 相同，目标文件原 git status 为空。完整原 bytes/base64 保存在 `remaining-linux-store-path-before.json`。
- 修改后 providers.rs SHA-256：`93d816299b3d24abba4cbe02597d0cd91d5d258a865ea84a3fe79267c39c22c0`。
- 比较完整原 bytes 与修改后 bytes，确认该测试之前、之后的字节完全相同；生产代码未改。
- 历史 `remaining-linux-workspace-quality-after-octal.json/log` 保留失败：80 通过、1 失败、2 忽略。旧 `49801fe0931bcdf156ce4718d5fae5ca844b4ff29988661318ce556a4d292a7d` 冻结仅包含 268 个路径，其中没有 providers.rs。当前 hash 不回填为旧运行的源码身份。
- 新 `remaining-linux-store-path-repro.json/log` 绑定修改前源文件重新编译原 exact 测试：exit 101，0 通过、1 失败、82 过滤；该次运行源码无漂移。

## 验证

| 检查 | 结果 | 收据 |
| --- | --- | --- |
| WSL Ubuntu 24.04，原 exact 测试 | exit 0；1 通过、82 过滤 | `remaining-store-path-linux-exact.json/log` |
| Windows，原 exact 测试 | exit 0；1 通过、83 过滤 | `remaining-store-path-windows-exact.json/log` |
| Linux ccr-store lib 全套，默认并行 | exit 0；81 通过、2 个既有忽略、0 过滤 | `remaining-store-path-linux-lib.json/log` |
| Linux ccr-store all-targets/all-features Clippy，-D warnings | exit 0 | `remaining-store-path-linux-clippy.json/log` |
| rustfmt --edition 2024 --check providers.rs | exit 0 | `remaining-store-path-format.json/log` |
| git diff --check -- providers.rs | exit 0 | `remaining-store-path-diff-check.json/log` |

每个绿色收据都记录相同的修改后源码 hash，运行内 source_changes 均为空。保留原测试名称、原临时夹具、原成功断言与全部既有测试；未 skip 整个测试，也未把既有两个忽略标为新通过。

## 后续边界

交 root 与 check_t01 复核。root 需要将 providers.rs 加入新的源码冻结后运行正式完整门禁。本轮未运行 aggregate、exporter 或 macOS；历史 exporter 访问冲突的原因仍未查明，不能由本次测试修正推定解决。未提交、推送、归档或改变任务状态。
