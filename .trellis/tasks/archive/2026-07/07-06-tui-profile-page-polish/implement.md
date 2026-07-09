# implement — TUI profile 页面体验修复

前置：先读 implement.jsonl 所列 spec；本任务在
`07-06-tui-key-masked-display` 之后执行（同文件串行）。

## 步骤

1. **R1 显示宽度截断**（独立可回滚）
   - `crates/ccr-tui` 若未直接依赖 `unicode-width` 则在 Cargo.toml 显式声明
     （workspace 树已有该 crate，锁文件不引入新版本）。
   - `ui.rs`：`truncate_text`/`pad_text`/`column_widths` 改用
     `UnicodeWidthStr::width`/`UnicodeWidthChar::width`；截断逐字符累计宽度，
     预留 1 列给 `…`；宽度为奇数余量时允许短 1 列（不得溢出）。
   - `usage` 模块的 `truncate` 同步修复（位置以子任务 A 落地后为准）。
   - 单测：CJK/ASCII/混合/emoji 样本 × 多个宽度断言 `width(out) <= w` 且尾部
     `…`；ASCII 回归样本不变。
   - 验证：`cargo test -p ccr-tui -- --test-threads=1`

2. **R2 提示去冗余**
   - `profile_meta_strings`：删除 keys 行（保留 Selected/Profiles/Legend）。
   - `render_profile_status_strip`/`last_apply_message`：strip 内容改为仅
     apply 结果/toast；不再拼接 `footer_text`。
   - 更新相关字符串断言单测。
   - 验证：`cargo test -p ccr-tui -- --test-threads=1`

3. **R3 标签改名**
   - 三个 `*_profile_detail_lines` 中 `detail_line("usage_count", …)` →
     `detail_line("switch_count", …)`；更新断言。

4. **R4 Focus 去重**
   - `profile_summary_strings` 收敛为 Name/Status/最近 apply 结果；
     `render_profile_context_workspace` 中 Wide 视口 Focus 高度 7 → 4（Compact/
     Standard 分支同步核对）。
   - 单测：Focus 行集合断言。
   - 验证：`cargo test -p ccr-tui -- --test-threads=1`

5. **收尾回归**
   - `just fmt-check` → `just lint-strict`。
   - 手工冒烟：宽/窄两种终端宽度下查看含中文描述 profile 列表与详情。

## Review gates

- 步骤 1 后：`rg 'chars\(\)\.count\(\)' crates/ccr-tui/src/tui/ui.rs` 确认截断
  路径无字符计数残留（其他用途逐一核对）。
- 全部完成后跑 trellis-check（对照 backend-guidelines 的 selection/分页红线）。

## 回滚点

- 四个 R 各自独立 commit，可单独 revert；R1 是纯函数替换，优先合入。
