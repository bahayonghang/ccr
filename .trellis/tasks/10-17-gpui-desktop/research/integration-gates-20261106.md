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
- 当前候选`gpui-package`尚未运行。原生交互、OS/IME/视觉验收按用户决定延期，父任务/01/09保持in_progress。

## 根完整 CI（HEAD 65a5fe1f）

- 第一次因晚到的 Tauri fixture 格式差异失败；单独格式修复提交 `f4fb6c5`，随后 fmt-check 退出0。
- 第二次首个实际失败为 Codex trash 的两处 `unnecessary_sort_by`；等价改为 `sort_by_key(Reverse(...))`，严格 Clippy 和 fmt-check 通过，提交 `65a5fe1`。
- 第三次 `just ci` 完整退出 **0**，耗时 **19:12.165**；日志 `outputs/desktop/integration-20261106-ci-rerun2.log`。版本、格式、harness、严格 Clippy、workspace check/test、release build、audit/governance、Tauri CI、frontend check/audit/coverage、VSCode CI 均通过。
- 该轮运行期间 Root/Tauri 源码与锁文件冻结；同时开发中的独立 native Benchmark/System Info/Codex diagnostics 不由此收据覆盖。后续共享 OAuth 清理入口若修改 Root，需要重新取得 CI 收据。

## 扩展原生批次（HEAD 57abc91 + 当前工作区）

- 纳入 Benchmark source/共同 UTC 默认窗口、System info、Codex diagnostics、SSH CLI detect、账户原值回填/普通 Session 表单和缺钥保护。
- `just gpui-fmt-check`、`just gpui-lint` 退出0。第一次完整 `just gpui-test` 为 lib **227通过/1失败/4忽略**；26个实际1MiB子项已通过，整个门禁仍失败。
- 唯一失败为旧 OAuth 控件用例：新预检正确拒绝 fixture 主机探测，旧测试仍等待启动。该测试本来已把待登录状态预置到临时端口；修正后先点击 Start 并验证预检结束且没有 owner，再直接启动这个已捕获的临时端口 owner，继续通过真实 Cancel 控件检查 join、密文 pending 删除、端口可重新绑定、callback/URL 清空。未恢复隔离模式的固定1455探测、未注入虚假成功、未增超时或跳过取消覆盖。精确回归1/1通过，仅调整测试，不修改生产守卫。
- 随后重新完整 `just gpui-test` **退出0**，日志 `outputs/desktop/integration-20261106-gpui-current-test-rerun1.log`：Python16、AccessKit3、lib **228通过/0失败/4忽略**、native_shell **9通过/0失败/1忽略**、native_ui **2通过**、runtime_shutdown **2通过/1忽略**、doc0。lib中 **26个实际1MiB+poisoned锁子进程** 全通过；忽略项为显式大数据/隔离helper，另有实际执行收据，不是为本次修门禁新增skip。
- 这是当前扩展批次的源码/fixture/headless验收。最终成对全域读回、交付包与共享改动后的新完整CI继续单独记录；后续S2源码不由本轮通过结果覆盖。
- 随后用同一已编译artifact显式执行大数据用例 `runtime::benchmark::tests::generated_pairs_load_through_native_domain_consumers --exact --ignored`，**1/1通过，234.60s**，日志 `outputs/desktop/integration-20261106-native-pairs-final.log`。最终共享validator/sidecar guard下，S0/S1/S2的Profiles、全量分页Sessions/summary/detail、100k usage各分组/趋势/记录、Settings/Auth/监控读取及DB原hash保持断言通过。运行时长是功能核验成本，不是启动或原生性能样本；未因重复provider discovery慢而绕过真实consumer。
