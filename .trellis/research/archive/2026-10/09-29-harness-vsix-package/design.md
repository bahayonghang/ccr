# T05 设计

## 责任与方案
将扩展运行文件和必要发布说明定义为受控允许范围。拒绝 .serena、.abcoder、工具状态目录、个人 *.local.*、源码映射及 AGENTS/CLAUDE/code_map 等内部说明。保留 package.json、dist/extension.js、资源图标、许可证和 README/CHANGELOG。以 vsce ls 和最终 VSIX 清单做双重检查，不读取或提交真实本地配置内容。

## 文件白名单
- `ccr-vscode/.vscodeignore`
- `ccr-vscode/package.json`
- `ccr-vscode/package-lock.json`（用户后续批准的两个传递补丁）
- `ccr-vscode/justfile`
- `ccr-vscode/scripts/check-package-files.mjs`（新增）
- `ccr-vscode/scripts/check-package-files.test.mjs`（新增）
- `.trellis/spec/ccr-vscode/frontend/extension-surface-contracts.md`

## 兼容与回滚
保持既有命令、共享接口和数据安全规则；所列新增脚本均为检查器或测试。提交实施前重新核对当前文件和基线。回滚只撤销本任务差异，不 reset/clean 或覆盖他人工作。生成/ignored 文件的交付边界必须由父任务强模型复核。

## 依赖
无前置子任务。父子层级不代替执行顺序。

## 已批准依赖补丁

fast-uri 3.1.6 → 3.1.7、undici 7.29.0 → 7.29.1 均满足已核验的父依赖范围。只改两个传递节点及其完整性元数据，不批量升级；重新运行 npm ci、npm audit、just vscode-ci、just vscode-coverage 和最终包内容检查。保持 package.json 依赖范围和 70% 门槛。

## 2026-10-04 继续实施范围

用户在未完成清单后要求继续实施。追加两个已复核解析目标：brace-expansion 5.0.9 → 5.0.12、fast-uri 3.1.7 → 3.1.8。仅更改 package-lock.json 对应节点的 version/resolved/integrity。若当前公告要求不同目标，先报告新范围。保留 undici 7.29.1、manifest 和现有门槛。

## 2026-10-04 已批准 VSCE 4 迁移

用户明确批准新增主版本方案。仅更新 `ccr-vscode/package.json` 的 VSCE 声明为 `^4.0.0`，以及 `ccr-vscode/package-lock.json` 的必要依赖闭包。采用已复核候选，保留其他直接依赖和 extension contributions；核对默认 secret/.env 阻断及 lazy keyring 路径。固定 Node 24.20.0 执行安装、扫描负例、扩展 CI、原覆盖门槛和最终 VSIX；父任务最后运行真实完整 CI。扫描夹具限自有临时目录，使用合成内容，不访问账户/PAT 或执行 publish/login。
