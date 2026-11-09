# T04 设计

## 责任与方案
在解析边界处理 LF / CRLF 和 UTF-8 BOM，保留缺失字段、空值和未闭合 frontmatter 的失败行为；不批量重写工作区换行。取消将 ignored .claude/skills 目录视为全新检出的必备共享资产，改为校验已跟踪的共享规则引用。通过真实文件夹具调用检查器验证，避免测试只重复实现。

## 文件白名单
- `scripts/quality/check-copilot-assets.mjs`
- `scripts/quality/check-copilot-assets.test.mjs`（新增）
- `docs/guide/github-copilot-workspace.md`
- `docs/en/guide/github-copilot-workspace.md`

## 兼容与回滚
保持既有命令、共享接口和数据安全规则；所列新增脚本均为检查器或测试。提交实施前重新核对当前文件和基线。回滚只撤销本任务差异，不 reset/clean 或覆盖他人工作。生成/ignored 文件的交付边界必须由父任务强模型复核。

## 依赖
无前置子任务。父子层级不代替执行顺序。
