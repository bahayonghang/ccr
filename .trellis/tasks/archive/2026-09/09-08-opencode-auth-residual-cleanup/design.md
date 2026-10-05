# 清理设计

具体锚点在父任务 research/codebase.md。仅三处源码所有权：
- crates/ccr-config/src/managers/tui_config.rs:121、138、211 及测试。
- crates/ccr-codex/src/utils.rs:62、253 及专属测试。
- crates/ccr-tui/src/tui/theme.rs:81、379、621、792 及过时注释。

删除专属代码前复查调用方；语言/主题/Usage fixture 改为 grok_auth，删除的测试不带走仍有效断言。未知 tab 测试使用任意未知字符串，不保留 OpenCode 迁移器。

默认六页顺序来自当前 DEFAULT_TAB_ORDER，主 TUI 不需要重新添加 Grok 页。删除旧枚举使旧配置整体 fallback，不能声称保留其旧语言/主题；仅有效配置保留原值。

更新 .trellis/spec/ccr-config/backend/backend-guidelines.md、ccr-codex/backend 的失效 OpenCodePaths 说明（限实际删除类型）、README.md:12。TUI 规范本子任务只更新 OpenCode 退休与独立 Usage 兼容文字；最终账号界面章节由 TUI 子任务负责，执行顺序避免文件重叠。

不动有效 OpenCode 配置/统计、Codex quota core、归档和无关 dirty。回退只限任务修改。
