# T06 只读聚合门禁与遗漏检查接入

状态：最终计划已获用户批准；任务启动和验收状态以 task.json 与实施记录为准。

## 目标
解决父报告 F08，完成 只读聚合门禁与遗漏检查接入。优先级 P2。

## 依据
- 父报告：../09-29-evergreen-harness-audit/research/audit-report.md
- 原始记录与退出码：../09-29-evergreen-harness-audit/research/check-results.md
- 前置依赖：T01、T02、T03、T04、T05、T08。

## 要求与验收
- T06-AC1：检查路径不重写版本、格式、图标、生产源码，不自动修复失败项。
- T06-AC2：受控命令 stub 测试能证明各 OS 的 ci 顺序、首个失败传播和新增安全/契约检查无遗漏；不依赖宿主安装完整工具链。
- T06-AC3：路径相关性包含新增检查文件；required job 名称与覆盖率门槛不变。
- T06-AC4：完整 just ci 的成功只表示其实际执行项通过，Rust/Tauri coverage 与 MSRV 等托管附加项单列。

## 范围边界
仅实施 design.md 的文件白名单和 implement.md 的检查。不得修改用户全局配置、降低门禁、提交、推送或发布。Trellis check 的可写自修角色仅限批准后的实现范围。

## 模型与工具
强模型设计检查边界和 required contexts；低成本模型按既定列表修改三个 OS recipe 与静态回归。不能自行删测试、降阈值或串行化恢复。

## 完成后的知识回写
dependency-governance.md 和 T02 共享说明记录五工具通用检查命令及副作用。
