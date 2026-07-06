# TUI 详情面板 key 掩码显示（前后各 4 位，便于核对）

## Goal

Claude/Codex profile 详情面板的 `token` 行从只显示 `configured` 改为附带掩码
key（前后各 4 位），让用户不打开 `profiles.toml` 就能核对当前 profile 用的是
哪把 key。同时把 Codex 详情中放错分组的 `token` 行归位。

## Confirmed Facts

- 现状（`crates/ccr-tui/src/tui/ui.rs`）：
  - `claude_profile_detail_lines`：`auth_mode = api_key` 且 `config.auth_token`
    非空 → `token: configured`；空 → `missing`；订阅模式 → `subscription`。
  - `codex_profile_detail_lines`：`openai_api_key`/`provider_env_key` 模式同上，
    其余 → `-`；且 `token` 行排在 **Activity 分组末尾**（Claude 在 Routing/Auth），
    分组语义错位。
  - `detail_value_style` 按值精确匹配 `configured` 上绿色。
- 仓库唯一掩码策略：`ccr_core::mask_sensitive`——长度 ≤10 全部 `*`；>10 显示
  前 4 + `...` + 后 4（如 `sk-a...cdef`）。已被 ccr-store history、CLI clear 等
  复用。
- `auth_token` 是 `Secret` 类型，仅 `.expose()` 可取值；Display/Debug 恒掩码。
- 仓库红线（CLAUDE.md）：改动涉及 secret 显示时必须保持掩码；
  `rust-security-reviewer` 子代理需参与审查。

## Requirements

- `token` 行在 key 已配置时显示 `configured (<masked>)`，`<masked>` =
  `ccr_core::mask_sensitive(token.expose().trim())`；Claude 与 Codex 一致。
- `missing` / `subscription` / `-` 三种状态的文案与配色保持不变。
- Codex 的 `token` 行从 Activity 分组移到 Routing/Auth 分组（与 Claude 对齐，
  放在 `auth_source` 之后）。
- `detail_value_style` 对 `configured (…)` 前缀仍显示 success 绿色（由精确匹配
  改为前缀匹配，其余取值规则不变）。
- 不新增第二套掩码函数/策略；若未来需要更长前后缀，只能改 `mask_sensitive`
  本体（本任务不改）。

## Acceptance Criteria

- [x] Claude/Codex 详情单测：长 key（>10 字符）渲染为
      `configured (xxxx...yyyy)`；短 key（≤10）渲染为 `configured (****…)`
      全星号形态；缺失/订阅/`-` 状态回归通过。
- [x] 安全单测：详情全部渲染行拼接后不包含完整 token 明文（用
      `sk-ant-test1234567890` 之类固定样本断言 `!contains`）。
- [x] Codex 详情行序单测：`token` 出现在 Routing/Auth 分组内、Activity 分组不再
      含 token 行。
- [x] `configured (…)` 行仍为 success 配色（`detail_value_style` 单测）。
- [x] `cargo test -p ccr-tui -- --test-threads=1`、`just fmt-check`、
      `just lint-strict` 全绿。
- [x] 交由 `rust-security-reviewer` 审查 diff（masking 显示面变更）。
      第一轮 needs-fix（mask_sensitive 多字节 panic，见
      research/security-review.md），修复后第二轮 approve。

## Out of Scope

- 修改 `mask_sensitive` 的前后缀位数或阈值（全局策略，动它影响日志/CLI 全部
  掩码输出）。
- Focus 摘要块、列表页、auth 子应用（Claude/Codex/OpenCode Auth tab）的 key
  显示——auth tab 已有自己的 email/id 掩码体系。
- 明文查看开关（如按键临时揭示完整 key）。

## Notes

- 改动集中在 `ui.rs` 两个 detail_lines 函数 + `detail_value_style`，预计 <50 行；
  PRD-only 轻量任务，无需 design.md/implement.md。
- 与子任务 `07-06-tui-profile-page-polish` 改同一文件，先做本任务再做 polish。
- 实施期例外：安全审查发现 `mask_sensitive` 按字节切片会对多字节 token panic
  （本任务使其可从 TUI 渲染路径触达），故将其改为按字符切片——前后缀位数与
  阈值策略未动，ASCII 输出逐字节不变，不属于 Out of Scope 禁止的策略变更。
  详见 `research/security-review.md`。
