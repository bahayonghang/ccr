# Implement：Grok TUI 顺序与空态迁移指引

> 开始前：仅在用户审阅本计划并再次明确批准后运行 `task.py start`。实现前加载 `trellis-before-dev`，读取 ccr-config、ccr-tui、ccr-cli backend checklist。

## 1. 调整并锁定默认顺序

- [ ] 将 `DEFAULT_TAB_ORDER` 改为 Codex Profile -> Claude Profile -> Grok Profile -> 现有 Auth 顺序。
- [ ] 把默认顺序测试改为完整向量断言，而不是只检查 Grok 索引。
- [ ] 复核完整自定义顺序原样保留、旧不完整顺序仅补项、保存 round-trip 测试。
- [ ] 验证：`cargo test -p ccr-config tui_config -- --test-threads=1`

## 2. 修复 TUI Profile 空态

- [ ] 将成功读取但列表为空的提示改为 `ccr {platform} profile create --help`。
- [ ] 将第二条提示改为创建后按 `r` 重载；读取失败空态保持不变。
- [ ] 增加英文/中文 Grok 空态回归测试，并覆盖 Claude/Codex 的通用生成行为。
- [ ] 增加 `ccr platform init`、`ccr add` 负断言；测试结束恢复英文语言。
- [ ] 验证：`cargo test -p ccr-tui -- --test-threads=1`

## 3. 修复 CLI 帮助发现面

- [ ] 在 `PlatformAction` 上隐藏 `switch/current/info/init/profile`，保留 enum、参数和 dispatch 迁移错误臂。
- [ ] 改写根/platform 自定义帮助，推荐 `ccr platform list`、`ccr current` 及显式 Claude/Codex/Grok profile 帮助。
- [ ] 更新 help 集成测试：direct/help-path 等价、退休命令不再可见、支持入口仍可见。
- [ ] 增加 `ccr platform init grok` 回归：仍解析、非零退出、返回 legacy + Grok profile 迁移指引。
- [ ] 验证：`cargo test -p ccr --test commands -- --test-threads=1`
- [ ] 验证：`cargo test -p ccr-cli --test dispatch_routing -- --test-threads=1`

## 4. 全量验证与审查

- [ ] 记录 `ccr-ui/src/types/generated/usage/DailyTrendDto.ts` 实现前内容哈希，验证后确认未变；不纳入 diff。
- [ ] `just fmt-check`
- [ ] `just lint-strict`
- [ ] `just test`
- [ ] `just ci`
- [ ] `git diff --check`
- [ ] 检查最终 diff 只包含任务文件和计划中的 config/TUI/CLI/test/spec 文件。
- [ ] 运行 `rg -n "ccr platform (init|switch|current|info|profile)|ccr add" crates/ccr-tui/src/tui/ui.rs crates/ccr-cli/src/cli/help_config.rs`，确认本任务覆盖面无旧推荐。

## 5. 规范与收尾

- [ ] 更新 `.trellis/spec/ccr-config/backend/backend-guidelines.md` 的内置 TUI 顺序。
- [ ] 若帮助隐藏/legacy 解析不变量尚未被规范记录，补充 `.trellis/spec/ccr-cli/backend/backend-guidelines.md`；不写入临时实现细节。
- [ ] 按 Trellis finish-work 流程提交、归档和记录 journal；不 push。

## 回滚点

- 默认顺序改动没有落盘迁移，可独立回退。
- CLI 变体只能撤销隐藏属性，不能删除兼容解析臂。
- 若最终 gate 产生无关 generated diff，恢复验证副作用并重新检查，不得覆盖用户原有修改。

## 明确不做

- 不恢复 legacy platform 路由。
- 不修改真实 Grok/CCR 配置与凭据。
- 不在本任务中机械替换 `docs/` 或其他平台历史指引。
