# 实施计划

按顺序实施，每步验证。改动 CodexProcessService 前先读 `.trellis/spec/ccr-codex/backend/codex-app-server-cleanup.md`。

1. 服务层检测：daemon.pid 解析 + `find_managed_daemon` 交叉核对（ccr-codex）。验证：`cargo test -p ccr-codex codex_process_service` 新增用例（文件缺失 / 坏 JSON / pid 不在目标集 / 命中）。
2. 服务层重启：`restart_codex_daemon` 四态 + ManagedProcess 超时（ccr-codex）。验证：假 CLI fixture（成功 / 非零 / 超时回收 / 缺失）单测；`cargo test -p ccr-codex`。
3. CLI 接线：switch.rs 交互询问 + `--restart-daemon` + 非交互降级 + 文本样式（与 10-06 展示迁移协调）。验证：`just lint-strict`；真实守护进程下手工跑 `ccr codex auth switch` 观察询问与重启。
4. TUI 接线：overlay 确认 + 后台重启 + toast + EN/ZH。验证：`just test`；TUI 文本检查（EN/ZH 尺寸矩阵）。
5. Spec 更新：codex-app-server-cleanup.md 增补守护进程检测与重启合同（触发条件、签名、信号边界），必要时更新 spec index 描述。
6. 运行时验证：守护进程运行中切换 → 确认 → `codex app-server daemon version` 新 pid；新 codex 会话 creator_account_id 新账号；`ccr codex fix --dry-run` 行为不变。
7. 全量门禁：`just ci`（至少 `just lint-strict` 加 `just test`）。

回滚：纯代码与 spec 变更，无持久状态，还原提交即可。检查门：每步后按 check.jsonl 上下文做质量检查；归档前完成 spec 更新。
