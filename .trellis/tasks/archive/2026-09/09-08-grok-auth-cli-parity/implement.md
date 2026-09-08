# 实施与检查

1. 用户确认规划后运行 task validate 并 start；实现和检查使用 Trellis 子代理，主线程负责整合文档与验收。
2. 读取 ccr-cli backend-guidelines/test-fixtures/auth-off 和 ccr 入口测试规范。先运行现有 grok_auth 服务测试，记录旧代码基线。
3. 在隔离 CCR_ROOT/GROK_HOME/HOME 下增加真实 CLI 红灯回归，然后实现 enum/dispatch/handler/DTO。不要运行真实账户 save/off/switch。
4. 更新 help_config.rs、docs/reference/commands/grok.md、英文对应文件、auth-off.md。新增 CLI 集成测试文件，并按现有模块约定注册。
5. 验证 AC1–AC6：单/多/无 scope、重名/force、空库、删除取消、错误退出码、A/B 回存切换、损坏和并发回归、secret 输出及 MCP/profile 保持。
6. 执行 `cargo test -p ccr-cli grok_auth -- --test-threads=1`、入口 commands 的 grok/help 过滤测试（先确认 target 注册），`just fmt-check`、`just lint-strict`、最终 `just ci`。支持的外部命令经 RTK。无关失败单独报告。
7. 独立检查 diff 和验收映射，记录 PASS/FAIL/SKIPPED/UNVERIFIED；交付源代码、验证结果和安装边界。不自动提交、安装或推送。
