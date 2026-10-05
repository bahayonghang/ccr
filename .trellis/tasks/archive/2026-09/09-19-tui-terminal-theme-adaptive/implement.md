# Implement: TUI 终端主题自适应（auto 持久化选项）

按序执行；每步先过最窄验证再扩大（repo AGENTS.md 快速检查优先级）。

## Checklist

### 1. ccr-config：`TuiTheme::Auto` 成为默认值
文件：`crates/ccr-config/src/managers/tui_config.rs`
- [ ] 枚举加 `Auto`（`#[default]` + `serde(rename = "auto")`），更新 `as_str` / `toggled`（:42-65）
- [ ] `deserialize_theme_or_mocha` → `deserialize_theme_or_auto`：支持 `"auto"`，未知/非字符串回退 `Auto`（:88-105）；同步 `TuiConfig` 字段属性（:145）
- [ ] 测试更新：`default_theme_is_mocha` → auto（:323-329）；`unsupported_theme_falls_back_*` 断言改 `Auto`（:610-634）；新增 `theme = "auto"` 的 load/save round-trip 断言
- [ ] 验证：`rtk cargo test -p ccr-config`

### 2. ccr-tui：启动解析支持持久化 auto
文件：`crates/ccr-tui/src/tui/theme.rs`
- [ ] `resolve_startup_variant`（:226-237）按 design C2 表格重写；删除 `From<TuiTheme> for ThemeVariant`（:255-262），解析处显式 match
- [ ] 更新 `init_theme`（:207-224）文档注释：探测条件改为"env=auto 或持久化 auto"
- [ ] 测试（:659-693）：保留"持久化 Mocha/Latte 不调用探测器"断言；新增"持久化 Auto 调用探测器"、"探测失败回退 Mocha"、"env=auto + 持久化 Auto + 探测失败 → Mocha"
- [ ] 验证：`rtk cargo test -p ccr-tui`

### 3. Spec 同步
- [ ] `.trellis/spec/ccr-tui/backend/backend-guidelines.md`：重写 Startup Theme 场景契约（design.md「Spec / Test Impact」）
- [ ] `.trellis/spec/ccr-config/backend/backend-guidelines.md`：TuiTheme 章节补 auto 值/默认值/回退规则

### 4. 质量门
- [ ] `just fmt-check`
- [ ] `just lint-strict`
- [ ] `just test`

### 5. 手动冒烟（实现完成后）
- [ ] 亮色终端启动 `ccr` TUI → 呈 Latte；暗色终端 → Mocha（需 `tui.toml` 无固定 theme 或为 auto）
- [ ] `tui.toml` 写 `theme = "latte"` 后启动 → 固定 Latte 且首帧无探测延迟
- [ ] Ctrl+T 切换 → 持久化为可见变体

## Risky Files / Rollback Points

- `crates/ccr-config/src/managers/tui_config.rs`：默认值变更影响所有无 theme 键的存量配置 —— commit 1 独立，可单独回退。
- `crates/ccr-tui/src/tui/theme.rs`：解析语义变更 —— commit 2 依赖 commit 1 的类型，回退需按逆序。

## Pre-start Checks

- [x] 无其他活跃任务（`task.py current` 指向本任务）
- [x] `TuiTheme` / `toggled()` / `From` impl 调用点已全量 grep，无隐藏消费方
- [ ] 用户批准最终规划摘要后再 `task.py start`
