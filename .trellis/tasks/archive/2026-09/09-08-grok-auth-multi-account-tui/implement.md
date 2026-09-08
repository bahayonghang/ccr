# TUI 执行计划

- [x] 切换范围和实施授权已确定；服务 DTO/API 冻结后启动，集成服务实现和清理 theme 结果。
- [x] 读取 TUI/backend/i18n/分页与父协议研究；确认当前主壳 global-key、mouse、tick、退出顺序。
- [x] 用 service snapshot 构建账号列表/选择与详情，删除旧 logged_in-only UI 状态。
- [x] 实现局部 modal：来源、保存名、覆盖/切换/删除/登出确认；复用几何和输入，不改全局删除弹窗语义。
- [x] 接入 spawn_blocking、Busy 和结果通道，处理 executor 不可用、陈旧确认、服务错误/刷新错误。
- [x] 在主 app 必要分发处协调弹窗取消、Busy 导航/退出；更新所有 Grok 构造 fixture 与退出总结，保留 warnings/结果未知。
- [x] 实现三类布局/小屏降级、i18n、现有主题色、footer；修正窄屏关键反馈与输入可见性，实际渲染及双主题效果未执行验证。
- [x] 更新 TUI 规范并独立静态审查；发现已修正，最终定点复核无遗留确定性生产缺陷。

## 验证

用户要求本轮不测试：以下命令全部不执行，记 SKIPPED（用户要求）；只维护必要测试源码并静态审查。不构建、启动 TUI、使用真实账号或操作当前 Grok Build。

```powershell
rtk cargo test -p ccr-tui grok -- --test-threads=1
rtk cargo test -p ccr-tui -- --test-threads=1
rtk cargo clippy -p ccr-tui --all-targets --all-features -- -D warnings
rtk cargo test -p ccr-cli --test dispatch_routing -- --test-threads=1
rtk proxy just fmt-check
```

新增测试名含 grok 或明确纳入全量；中英文/主题全局状态使用现有隔离约定。这些为后续验证准备，本轮不执行父 just ci。无真实账号/UI操作。

交付接口变化只通过父任务与服务任务协调，不直接改其持久化/锁文件。回退限本任务代码与规范。

源码实施与静态复核已交付；仅对自有文件运行 rustfmt 源码格式化和静态 diff 检查，未执行上述验证命令。PRD 行为 AC 保持未勾选，任务保留 in_progress，未提交/归档。
