# T10 最终检查汇总

日期：2026-09-28。代码已实施；完整根 CI 未通过，任务不进入 completed 或 archived。

## 检查角色与证据

专用 trellis-check 派发因 agent thread limit reached 阻断。gates owner 与 specs owner 对非本人切片进行交叉审查；root 承担跨任务集成检查。root 的 clean/list/桌面环境夹具/TUI 断言补丁均由非作者复核。该汇总不把交叉审查改称专用 checker 已运行。

- cross-check-gates.md：16 个门禁/生成事务/日期/锁测试文件，独立运行 27 项治理与 25 项绑定/日期测试。
- cross-check-specs.md：39 个 AC、33 个证据组、9 个 P1；独立核对 128 份产物、128 个 selector、25 条链接。新增 A14 逐例区分文件读取、内存合并、序列化和 null 字段断言。
- root-integration-review.md：逐项记录跨入口 owner、集成修复、原始失败与最终门禁边界。
- research/requirements-evidence.json 与 evidence-mapping-validation.json：可机读的源码/日志 SHA、测试位置和 AC/P1 映射。

## 验收条款

| 条款 | 实际结果 | 边界 |
| --- | --- | --- |
| AC1 | 完整 Windows just tauri-ci exit 0；406 行为测试与 2 guard 通过；包含严格 bin Clippy、bindings、inventory。真实 Windows just fixture 注入 desktop 失败使 aggregate 非零。 | 最新根 just ci 的完整 Test 与 Release Build 通过，Security Audit 因既有 rustls 公告失败；不授予 aggregate 成功。Linux/macOS aggregate 只检查编排定义。 |
| AC2 | 9 个 P1 关联已执行的原始局部反例与修后行为测试、源码/差异指纹；39 AC /33 组可追溯。 | fix_commit=null；本轮没有提交授权。局部红例不外推原生 UI、所有旧故障路径或跨平台。 |
| AC3 | 规范路径、owner、registry/manifest 计数、链接与 context 检查通过；此前 pricing/analytics/Insights 规范内容按字节保留。 | 规范导航检查不替代产品行为验收。 |
| AC4 | 正式主工作区 frontend-check 的 5 条旧脚本错误保留；隔离授权源副本的同命令通过。原 .tmp 文件 SHA 和 Insights 生命周期保持原样。 | 隔离成功不覆盖原工作区失败；没有删除、移动或新增忽略来绕过检查。 |

## 当前限制

Windows doctor fixture 曾缺失 grandchild.pid；定向复跑与临时诊断下完整 CLI 344 项通过，但原因未查明。临时诊断已撤除，fix.rs 与 HEAD 原字节一致。最终同源 workspace 测试及 binary 指纹由父 integration-report.md 与 research/root-delivery-after-utc-ci.json 记录。

macOS、Linux pinned Rust 1.98 完整 CI、原生 WebView/CSP、真实远程账户/OAuth、交互 TUI 未验收。额外 Tauri all-targets Clippy 的旧问题仍保留。Windows 旧进程启动异常和首次 binding drift 不能因后续成功而视为原因已修复。

## 交付决定

实现与定向验证已落盘，完整 aggregate 与原生验收保持开放。保留所有原始失败，保留父子任务 in_progress，不提交、不推送、不归档。最终结果见父任务 integration-report.md、execution-ledger.json 和 final-delivery-verification.json。

最终补充：Usage 两个 UTC 夹具的 4 行修改通过非作者审查，完整 ccr-usage 65 项通过；最终 root aggregate 的所有 workspace 测试及 Release 构建通过。Security Audit 报既有 rustls 0.23.40 的 RUSTSEC-2026-0285，Cargo.lock 与 HEAD 原字节一致，未扩大范围升级依赖。完整 CI 及原脚本 lint 仍未通过。
