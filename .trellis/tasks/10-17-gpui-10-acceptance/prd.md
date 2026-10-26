# GPUI 10：性能、三平台与交付验收

## Goal

执行完整功能矩阵、性能对比、原生视觉验收及独立打包门禁。

## Parent and Dependencies

父任务：[gpui-desktop](../10-17-gpui-desktop/prd.md)。需求映射：R1 至 R9，父任务 AC1 至 AC8。

前置：01 至 09 全部完成，台账无无人负责项。 父子关系不自动保证依赖完成。

## Requirements

- 完成逐命令/route 台账和真实端到端场景，不以视觉相似替代功能。
- 对完整应用执行性能合同、三平台原生视觉/输入/可访问性验收。
- 独立本地打包、desktop-bench/perf/package、版本/依赖/CI 路由治理和文档。

## Acceptance Criteria

- [ ] 10.1：父任务 AC1 至 AC8 有实际证据，command/route 台账无 pending/unmapped，功能缺口不得以壳完成掩盖。
- [ ] 10.2：同机 release 的所有性能预算通过并保留原始数据；无跨 OS 指标误比较。
- [ ] 10.3：Windows/macOS/Linux 的 native 窗口、中文输入、DPI、键盘与可访问性记录完整。
- [ ] 10.4：本地安装包身份/产物路径与 CLI/Tauri 不冲突，干净用户目录可启动；不自动发布或使用签名密钥。
- [ ] 10.5：desktop-ci/audit 与完整 just ci 通过，既有 coverage/parallelism/hosted context 名保持；文档如实列支持范围。

## Out of Scope

不扩大到本任务未列的业务域；不删除旧表面、不改真实用户数据、不降低已有门禁。父任务声明的完整迁移范围由后续任务继续承接。

## Status

planning。技术待验证项由本任务检查闭环，不视作已完成。开始前读父任务 design.md、implement.md、当前子任务全部规划和 curated context。
