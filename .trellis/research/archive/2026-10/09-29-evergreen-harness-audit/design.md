# 父任务设计

## 责任边界
CCR CLI/TUI 入口在 crates/ccr；共享 Rust 逻辑仍留在现有领域 crate。ccr-usage 保持只读 SQLite 投影归属，Tauri llmusage_adapter 只负责 CLI sync、事件和 DTO。本计划不改变这些接口。

工具规则的共享入口是 AGENTS.md。CLAUDE.md 的真实 import 保留。双语 harness 页区分官方能力、当前机器集成、Git 交付和已运行证据。工具 schema 不相互复制。

## 改造策略
### T01 OMP 上下文完整性与可复现交付
保留现有 ContextBudget、UTF-8 边界、trustedRoots 和角色 JSONL 隔离。将现存 design.md、implement.md 纳入与 prd.md 相同的受限读取路径。保留旧 info.md 兼容，不让 info.md 代替复杂任务的设计和实施文档。测试断言文档语义标记及预算行为，避免把 ## PRD 旧标题作为内容完整性的唯一证明。精确放行并交付 .omp/extensions/trellis/index.ts；该文件当前没有外部运行时 import，必须在实施时复核依赖闭包。其他工具目录继续保持本地生成策略，T02 负责说明。

### T02 五套 harness 共享说明与角色权限对齐
AGENTS.md 保持共享规则入口，CLAUDE.md 保持真实 @AGENTS.md import。修正直接 Rust 命令为默认并行且跳过 export_bindings。分开列出官方能力、当前机器文件、Git 交付和原生运行证据。Kimi 优先描述项目自定义 agent 的实际存在；对未生成项目 agent 的工作区保留显式 pull 技能 fallback。新增静态契约检查只读取已跟踪的共享事实和受控夹具，不强制个人 dot 目录存在，不伪造 hook 已加载。只读 reviewer 与可写 trellis-check 分开，模型预算与工具权限分开。

### T03 Dependabot Bun 生态配置修复
仅将 /ccr-ui 和 /docs 的生态映射改为 Bun；/ccr-vscode 继续 npm，两个 Rust lockfile 继续 cargo。在现有治理脚本加入 packageManager、lockfile 和 Dependabot 目录的对照，不增加独立 CI 引擎。保留原调度、标签和安全策略。

### T04 Copilot 检查的跨平台换行兼容
在解析边界处理 LF / CRLF 和 UTF-8 BOM，保留缺失字段、空值和未闭合 frontmatter 的失败行为；不批量重写工作区换行。取消将 ignored .claude/skills 目录视为全新检出的必备共享资产，改为校验已跟踪的共享规则引用。通过真实文件夹具调用检查器验证，避免测试只重复实现。

### T05 VSIX 打包范围与本地文件排除
将扩展运行文件和必要发布说明定义为受控允许范围。拒绝 .serena、.abcoder、工具状态目录、个人 *.local.*、源码映射及 AGENTS/CLAUDE/code_map 等内部说明。保留 package.json、dist/extension.js、资源图标、许可证和 README/CHANGELOG。以 vsce ls 和最终 VSIX 清单做双重检查，不读取或提交真实本地配置内容。

### T06 只读聚合门禁与遗漏检查接入
沿用 just 和现有工作流。将 just ci 的 version-sync、fmt 移至显式维护命令；ci 仅检查，保留 version-check、fmt-check。聚合检查接入 frontend-audit、OMP、harness 和 Copilot 契约及最终包清单，并对 root/Tauri 两个 lockfile 进行审计。UI 构建检查使用已提交图标，图标生成保留独立命令。缺少审计工具时给出前置要求，不在只读聚合命令中安装全局工具。维持默认测试并行、现有 coverage 阈值和 required context 名称。

### T07 历史 CI 失败复验与诊断证据
先复验历史失败对应现有测试与正式 OS/coverage 门禁。当前主干测试已通过，不预先改变生产逻辑。若失败重现，只在已有测试增加有界诊断字段：doctor outcome、退出码、timed_out、stdout/stderr 字节计数及异步清理状态；敏感输出必须脱敏。超过诊断范围的产品修复重新提交变更计划。Tauri 9 条 audit warning 记录依赖链、target 可达性、责任人与后续处置，不新增 ignore。

### T08 前端依赖安全审计修复
本轮 18 条公告来自 4 个传递开发依赖：stylelint → colord/ajv→fast-uri；eslint/cosmiconfig → js-yaml；jsdom → undici。先锁定修复版本与 registry 校验和，优先更新兼容的 lockfile 解析；只有现有范围无法取得修复版本时才局部改 package.json。不得扩大 allowlist、降低 audit 级别或创建 package-lock。版本目标在批准后实施时依据当时公告再确认，不能仅根据今天的最低安全范围硬编码。


## 审批与模型
强模型持有安全、上下文边界、测试契约和最终判断。低成本模型只接受固定文件、预先确定的改动、具体 AC 与命令。未知根因、依赖重大升级和角色权限不能自行下放。现有 trellis-check 为可写角色，只能在用户批准实施后调用。

## 依赖图
T08、T01、T03、T04、T05 可独立实施；T02 等待 T01 的 OMP 契约；T06 等待 T01–T05、T08 的检查可用；T07 承接 T06 后的综合证据。T07 的旧失败调查可先读，正式关闭需等待最终修订。

## 兼容与副作用
保持 Root Workspace Required、Vue and Docs Required、Tauri Linux Required、VS Code Required 的 context 名称。Vue 命名是既有保护兼容，不能随 React 迁移无条件改名。保持 70%/85% 门槛、默认并行和 export_bindings 的独立生成契约。源码只读检查允许编译、缓存和受控夹具，不代表零磁盘写入。

## 交付与回滚
T01 只精确交付已被跟踪测试依赖的 OMP runtime；不放开整个 .omp 或其他个人工具目录。T02 明确其余本地生成资产的状态和引导，不把本机存在当成全新检出已安装。不得编辑 .template-hashes.json、全局 Trellis 或 node_modules。每个子任务的回滚仅限其差异。

## 后续批准：T05 扩展依赖补丁

用户另行批准 fast-uri 3.1.7 与 undici 7.29.1 两个兼容补丁，范围增加 ccr-vscode/package-lock.json。验收增加 npm audit 和 70% 扩展覆盖率，保留原打包检查及既有依赖范围。
