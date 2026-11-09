# 2026-10-04 绑定漂移复验

首轮完整 CI 的失败记录保持有效：`just tauri-bindings-check` 当时报告 6 个生成文件漂移，并恢复了原字节。

同日稍后的串行再生成比较到 0 个文件差异。正式 `just tauri-bindings-check` 退出 0，输出 `TypeScript bindings match the worktree baseline`。生成目录在命令后没有未提交差异。

这次通过不改写首轮失败。漂移原因未查明，没有生成可提交的产品补丁。
