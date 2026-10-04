# T08 设计

## 责任与方案
本轮 18 条公告来自 4 个传递开发依赖：stylelint → colord/ajv→fast-uri；eslint/cosmiconfig → js-yaml；jsdom → undici。先锁定修复版本与 registry 校验和，优先更新兼容的 lockfile 解析；只有现有范围无法取得修复版本时才局部改 package.json。不得扩大 allowlist、降低 audit 级别或创建 package-lock。版本目标在批准后实施时依据当时公告再确认，不能仅根据今天的最低安全范围硬编码。

## 文件白名单
- `ccr-ui/package.json`
- `ccr-ui/bun.lock`
- `.trellis/spec/ccr/backend/dependency-governance.md`

## 兼容与回滚
保持既有命令、共享接口和数据安全规则；所列新增脚本均为检查器或测试。提交实施前重新核对当前文件和基线。回滚只撤销本任务差异，不 reset/clean 或覆盖他人工作。生成/ignored 文件的交付边界必须由父任务强模型复核。

## 依赖
无前置子任务。父子层级不代替执行顺序。
