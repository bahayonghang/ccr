# rust-security-reviewer 审查记录（2026-07-06）

## 本任务 diff 审查

- 第一轮:**needs-fix** —— `ccr_core::mask_sensitive` 按字节切片
  （`&value[..4]` / `&value[len-4..]`），多字节 UTF-8 token 会在 char boundary
  panic;本任务让该路径首次可从 TUI 渲染面触达（用户可编辑 profiles.toml →
  TUI 崩溃,DoS）。
- 修复:`crates/ccr-core/src/utils/mask.rs` 改为按字符计数/切片
  （`chars().count()` 阈值 + `take(4)` / `skip(n-4)`），ASCII 输出逐字节不变,
  未改 PRD 红线中的前后缀位数/阈值策略;新增
  `test_mask_sensitive_multibyte_no_panic`。
- 第二轮:**approve** —— panic 路径关闭,ASCII 行为无回归,明文仅经
  `expose()` → `mask_sensitive` 单一通道,全部消费方无字节长度依赖。

## 审查带出的同类隐患（本任务未修,需单独拍板）

同一 bug class（display 路径按字节定长切片,多字节输入 panic）,均为存量代码:

1. **HIGH** `crates/ccr-cli/src/sync/commands.rs:271-272` —— WebDAV URL 截断
   `&webdav_url[..47]`,URL 来自用户可编辑 sync 配置,非 ASCII 路径段跨界即
   panic（`ccr sync` 显示面 DoS）。最值得跟进。
2. **MEDIUM** `crates/ccr-cli/src/commands/claude/auth/current.rs:118-123`
   `mask_uuid` —— 第二套字节切片掩码实现;`account_uuid` 是否有上游 ASCII
   校验需确认。建议改走 `mask_sensitive`。
3. **MEDIUM** `crates/ccr-cli/src/commands/codex/auth/current.rs:245-253`
   `mask_account_id` —— 同上。
4. **INFO** CLI display 代码普遍存在 `&s[..N]` 定长字节截断
   （profile/switch.rs、list.rs、sessions_cmd.rs、provider_cmd.rs、quota.rs、
   stats.rs 等）,建议统一 `truncate_chars` 辅助后逐步收敛。
