# 实施计划：切换可靠性修复

## 前置

- 基线 dev @ 797c1877；先读 implement.jsonl 上下文；遵守 spec（auth safety、atomic-writer、error-freeze、TUI 边界）。
- 分析先行：D5 复现矩阵完成后才进入修复实现。

## 步骤（按序，每步独立提交粒度）

1. D5 切换不可用复现分析（只写 research/switch-failure-analysis.md，不改代码）。
   完成标准：候选 1-6 各有「成立/不成立/未验证」结论与证据（命令+输出或代码锚点）。
2. D1 同步入口强化：新鲜度定向（runtime 较新 → 快照；快照较新且 current_auth → 经 commit_plan 写回 runtime）、多匹配规则、缺失 account_id NoOp、内容短路断言；接入 TUI reload/r 键；配额路由：fetch_account_quota_inner 对 current_auth 以 runtime 为源并双落盘；失败 warn 不阻断。
3. D2 切换序列重排：switch_account 开头换出同步（D1 新鲜度定向）+ 末尾写后校验；测试 A rotate → switch B → A 快照新值；失败路径无半写。
4. D3 修复触发扩展：should_repair_tokens 加 refresh_token_invalidated；重试语义与「需重新登录」文案；合成测试。
5. D4 身份规则：多匹配/缺失测试固定行为。
6. D6 窗口边界文档。
7. 端到端合成两场景：(a) save A → 模拟轮换 → CCR 观察点 → login B（写 runtime）→ switch A → quota stub（拒绝被消费 token）成功；(b) 模拟轮换后无观察点直接 login B → 观察点呈现「需重新登录」。
8. 回归：10-05 既有测试与 ccr-tui 矩阵不回归。

## 验证命令（按序）

```bash
cargo test -p ccr-codex --all-features
cargo test -p ccr-tui --all-features
cargo test -p ccr-cli --all-features   # 若触及 CLI
just lint-strict
just test
```

原生手工验证（另记 checks/verification.md）：真实切换 + 配额刷新 + R 修复；未做项标 NOT_RUN。

## 评审门

- 每步完成后检查 spec 合规、无 secrets 日志、原子写保持。
- 全部完成后 3.3 spec 写回 + 3.4 提交。

## 回滚点

- 步骤 2-5 各自独立提交，可单独 revert；步骤 3 依赖步骤 2。
