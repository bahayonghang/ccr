# GPUI 02：共享应用服务与运行时

## Goal

提取被两端消费的框架无关编排、任务边界和类型契约，保持 Tauri 行为。

## Parent and Dependencies

父任务：[gpui-desktop](../10-17-gpui-desktop/prd.md)。需求映射：R4、R7、R9。

前置：01 完成。 父子关系不自动保证依赖完成。

## Requirements

- 建立最小框架无关 ccr-app 编排 crate，首条真实链只做配置列表读取；已有领域 API 不重新封装一整层同名方法。
- 定义 typed request/result、environment/generation key、长期 Tokio runtime、有界队列及 app-owned jobs 生命周期。
- 为后续迁移提供按域提取方法；Tauri 当前业务行为与 wire/export 保持。

## Acceptance Criteria

- [x] 2.1：配置列表来自真实 fixture 的 ccr-config API，Tauri 与 GPUI 结果一致。
- [x] 2.2：ccr-app 依赖图无 ccr umbrella、tauri、gpui 或 WebView；GPUI render 中无同步文件/DB/进程/网络调用。
- [x] 2.3：受控慢任务时 UI 可继续响应；环境切换后的旧响应丢弃；关闭页面后订阅回收。
- [x] 2.4：进度 10Hz 合并、容量 128、终态必达；背压/取消/晚到结果均有确定性测试。
- [x] 2.5：Tauri 既有 fixture 与 bindings 漂移检查通过，无不必要 schema 或业务行为改变。

## Out of Scope

不扩大到本任务未列的业务域；不删除旧表面、不改真实用户数据、不降低已有门禁。父任务声明的完整迁移范围由后续任务继续承接。

## Status

2026-10-18：本子任务 AC2.1–2.5 通过独立检查与核心原生续验，completed，保留目录供后续上下文引用，不归档。01/父任务开放的平台/性能验收独立跟踪，未因 02 完成而通过。实际命令、失败修复、原生收据及范围见 evidence.md 与 research/check-final.md。
