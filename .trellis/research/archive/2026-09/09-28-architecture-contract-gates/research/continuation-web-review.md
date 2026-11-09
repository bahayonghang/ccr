# Web 交互验收记录

日期：2026-09-28。状态：Web 合成 IPC 和无运行时错误状态检查通过。证据索引为 continuation-web-review.json。

## 实际结果

- 路由：http://127.0.0.1:49173/codex/settings。使用 Tabbit Browser、Vite 和官方 @tauri-apps/api/mocks。
- 在真实 Settings 表单把默认模型从 codex-before 改为 codex-after。仅发生一次 codex_update_settings，payload 的 settings 只有 model，原 confirmationToken 保留。
- 两项 notifications 数组逐项保持，未知推理值 future-effort 保持。保存按钮及四档推理标签没有原始翻译键。
- 点击源文件并确认“继续查看”后，真实 CodeMirror 可见；仅一次 raw read，合成路径为 /synthetic/config.toml，原文含实际换行。
- Web .cm-scroller 为 flex，gutter 与 content 顶部坐标相同。最终合成流程的 pageerror 和未知 mock command 均为 0。
- 移除全部 mock 并重新加载后，页面显示 Tauri runtime is unavailable for get_current_environment 和一个重试按钮，没有 pageerror。
- 本任务 scratch page 已关闭，Tabbit task 已 finish。没有操作用户原有页面。

## 验证边界

本记录验证实际浏览器前端与合成 IPC 交互。该流程没有写真实用户配置，没有调用原生 IPC，没有证明生产 Tauri CSP 或原生 WebView 行为。真实文件写回由独立后端显式临时路径 helper 测试覆盖；两种证据不能合并为原生端到端通过。本轮未采集截图。

## 保留的 harness 失败

早期入口 route 未匹配 Vite 查询参数；一次 payload 断言漏掉 confirmationToken；宽 CSS 选择器同时匹配隐藏 dialog。续接时旧 task 状态失效，旧页面附着失败；最终流程在新 scratch page 全程重做。一次保存后的 deepEqual 受跨 realm 原型影响失败；随后只读核对 JSON 值和请求次数，没有重复保存。这些日志保留，不将 harness 失败归为产品缺陷。
