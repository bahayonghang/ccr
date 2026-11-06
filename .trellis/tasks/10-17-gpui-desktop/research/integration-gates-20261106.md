# 当前整合门禁收据

基线为本轮工作区，不复用历史完整CI结论。Rust固定1.98.0及显式RUSTC；GPUI/root/Tauri target保持分离，默认测试并行，`--skip export_bindings`不变。未运行原生窗口交互。

## GPUI第一轮

- `just gpui-fmt`格式修复后，`just gpui-fmt-check`退出0。
- `just gpui-lint`首个实际失败：source layers的manual_flatten；仅调整等价Option遍历，再跑退出0，无lint suppression。
- `just gpui-test`：Python16、AccessKit3通过；lib198通过/4失败/1忽略。三个旧测试依赖的行为已变化：Tools前置Tab序列、Claude默认Workbench、合法Help终态自动写Recent。更新实际导航/数据断言后各窄测通过，没有取消非法草稿不写入、秘密不落盘等约束。
- 第四项是真实生产渲染栈溢出。trace进入detail→transcript→session operations→trash renderer（此时trash=None）。debug renderer预留所有分页/行builder的大frame；仅将分页、单行提成自然helper，同控件1MiB+poisoned锁红→绿，分页同配置亦通过。未增大产品/编译器栈，诊断日志全部移除。

## GPUI第二轮

- `just gpui-fmt-check`与`just gpui-lint`退出0。
- `just gpui-test`：Python16通过、AccessKit3通过、lib **202通过/0失败/1忽略**。既有1MiB守卫内 **19个实际控件子进程** 全部通过，保留CCR_LOCK_DIR毒化，覆盖新增表单、MCP、会话分页、监控、滚动用量、Grok概览与能力页。
- `native_shell`为 **8通过/1失败/1忽略**；`platform_routes_show_supported_local_status_without_secrets`仍等待旧只读字段，124.18s后自行timeout。此用例正在逐平台核对实际typed表单/来源状态；不能只换ID掩盖读取错误。尝试精确检查测试进程时它已自行退出，未停止其他进程。
- 因上述integration失败，后续`native_ui`/doc tests尚未执行；整个`gpui-test`仍为FAIL，不能用lib通过代替。

## GPUI最终本批门禁

- 旧平台用例已核对实际读取终态：Settings检查普通字段值，Authentication显式走tool-auth；不以disabled=None判断就绪。保留来源字节不变、未知能力拒绝、所有可见label/value不含秘密和跨平台隔离断言。仅修改该integration测试，无生产捷径。
- 窄native_shell **9通过/0失败/1既有忽略** 后，重新执行完整根`just gpui-fmt-check`、`just gpui-test`，均退出0；本批`just gpui-lint`已退出0，后续仅上述测试适配，没有生产修改。
- 最终完整`gpui-test`：Python **16**、AccessKit **3**、lib **202通过/0失败/1忽略**、native_shell **9通过/0失败/1忽略**、native_ui **2通过**、runtime_shutdown **2通过/1忽略**、doc tests **0项**。lib内19个1MiB+poisoned锁实际控件子进程全通过。忽略项均为既有显式fixture/helper，不是为修门禁新增skip。
- 这是当前已稳定功能批次的源码/fixture/headless门禁。未引用的native benchmark草稿不在此覆盖内；后续显式Source分支需要新的窄测、独立check与门禁。

## 其他

- 最新共享ccr-app严格all-target/all-feature Clippy退出0，包含cfg(test)性能数据生成器；其完整独立check另记。
- Tauri显式fixture的锁/认证摘要隔离发现、红绿与最终回归见performance-fixture-check及implementation收据；不是原生WebView或性能可比证据。
- 完整根`just ci`、当前候选`gpui-package`尚未运行。原生交互、OS/IME/视觉验收按用户决定延期，父任务/01/09保持in_progress。
