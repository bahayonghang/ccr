# T08 前端依赖安全审计修复

状态：最终计划已获用户批准；任务启动和验收状态以 task.json 与实施记录为准。

## 目标
解决父报告 F01，完成 前端依赖安全审计修复。优先级 P1。

## 依据
- 父报告：../09-29-evergreen-harness-audit/research/audit-report.md
- 原始记录与退出码：../09-29-evergreen-harness-audit/research/check-results.md
- 前置依赖：无前置子任务。

## 要求与验收
- [x] T08-AC1：18 条当前公告不再触发 frontend-audit，现有 maxActiveExceptions=0 保留；实施时出现新公告继续如实处理。
- [x] T08-AC2：每个变更包具有旧/新版本、校验和、依赖路径和开发/运行时边界记录；不把工具链漏洞写成已证实的产品可利用漏洞。
- [x] T08-AC3：冻结安装、类型检查、lint、904 项 smoke 及覆盖率、构建和 audit policy 回归均通过。

本地实施证据：[research/implementation-results.md](research/implementation-results.md)。固定版本 hosted 工具链与原生客户端验证仍为 UNVERIFIED；父任务负责最终独立审查。

## 范围边界
仅实施 design.md 的文件白名单和 implement.md 的检查。不得修改用户全局配置、降低门禁、提交、推送或发布。Trellis check 的可写自修角色仅限批准后的实现范围。

## 模型与工具
强模型确认安全公告、依赖范围及兼容性；低成本模型可按批准的版本集合更新 lockfile、整理变更表并运行检查。禁止直接交给低成本模型全量升级。

## 完成后的知识回写
dependency-governance.md 保存五工具通用的公告—依赖链—验证规则；不将本轮原始日志导入个人知识库。
