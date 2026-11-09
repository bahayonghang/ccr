# T01 设计

## 责任与方案
保留现有 ContextBudget、UTF-8 边界、trustedRoots 和角色 JSONL 隔离。将现存 design.md、implement.md 纳入与 prd.md 相同的受限读取路径。保留旧 info.md 兼容，不让 info.md 代替复杂任务的设计和实施文档。测试断言文档语义标记及预算行为，避免把 ## PRD 旧标题作为内容完整性的唯一证明。精确放行并交付 .omp/extensions/trellis/index.ts；该文件当前没有外部运行时 import，必须在实施时复核依赖闭包。其他工具目录继续保持本地生成策略，T02 负责说明。

## 文件白名单
- `.gitignore`
- `.omp/extensions/trellis/index.ts`
- `scripts/trellis/omp-context.test.ts`

## 兼容与回滚
保持既有命令、共享接口和数据安全规则；所列新增脚本均为检查器或测试。提交实施前重新核对当前文件和基线。回滚只撤销本任务差异，不 reset/clean 或覆盖他人工作。生成/ignored 文件的交付边界必须由父任务强模型复核。

## 依赖
无前置子任务。父子层级不代替执行顺序。
