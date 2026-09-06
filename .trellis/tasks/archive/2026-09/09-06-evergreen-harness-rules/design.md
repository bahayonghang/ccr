# 设计

## 机制

AGENTS 作为项目事实唯一来源；先纠错再在 CLAUDE 顶部使用不在代码段内的 @AGENTS.md。Claude scoped imports 若为必要规范用真实导入或明确按需读取，不将所有内容无条件重复加载。新增 docs/agents/harnesses.md 汇集工具差异、角色权限、模型分工与批准项结果；code_map 只加导航。既有专用 skill 通过此说明共用，不复制五份。纠正当前项目选择的集成方式与官方能力上限混淆，不为对齐而新增 hooks/custom agents。只更新列明的本仓库入口，Trellis 生成器上游不在范围；记录更新可能覆盖的本地说明。待 OMP 结果后填写当前接入行为，历史证据复核若未批准则保留为待办。

## 所有权与顺序

目录级现有说明同步范围：ccr-ui/CLAUDE.md 和 code_map.md 修正 Vue/Pinia/旧路由/上游依赖与 Anthropic 旧设计方向，引用本目录 AGENTS 和 DESIGN.md，不重新定义视觉规则；crates/AGENTS.md 与 code_map.md 补 ccr-usage 所有者，crates/ccr/src/CLAUDE.md 把旧单体 src 树改为 facade 与域crate导航，不重写产品架构。docs/agents/harnesses.md 同时提供 docs/en/agents/harnesses.md，遵守 docs/AGENTS.md 的语言镜像；沿用现有 agents 文档定位，由根说明链接，不新增产品导航栏。

已安装 Trellis 文件允许本地定制，依据 .agents/skills/trellis-meta/references/local-architecture/generated-files.md:64；不改 .trellis/.template-hashes.json 或 .version，不改上游/global npm/node_modules。四份 platform-map 同步本次 Grok/Kimi 说明，不覆盖其他已有内容。hooks-and-settings.md 已准确限定本项目交付机制，不修改。docs/agents/harnesses.md 记录这些本地定制及适用工具。

共享事实纠正可早做，最终收口等待已批准的 UI/CI/OMP 子项；不等待未批准P2。

## 工具分工

主线程强模型确定契约，明确方案后可由低成本档执行白名单修改，最终由独立强模型审阅。 工具边界见父任务 research/harnesses.md；优先 Codex/Claude 的 shell 与源码审查，Grok 的只读 plan/explore 不承担 shell 测试；Kimi/OMP 执行必须给完整任务内容。没有实际解析模型和价格证据时不宣称更便宜。

## 约束与撤回

不更改用户全局AGENTS/账户/默认模型，不写外部知识库，不复制全部规则，不新增文档/模型validator，不启动五套客户端计费运行。 仅撤回本次拥有的文件差异，保持其他任务变更；不创建兼容层。所有批准项通过父规则子任务回写适用工具和检查结果。
