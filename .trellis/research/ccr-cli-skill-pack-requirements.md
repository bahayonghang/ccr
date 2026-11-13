# ccr-cli 技能包的结构与分发要求

研究票 #170（地图 #167）。代码基线 `origin/dev`@`16cbc6fa`。qiaomu-meta 以本机安装的 `qiaomu-meta-skill` 2.8.1 为准，只读它的 `SKILL.md`、`references/`、`scripts/`，没有运行它的任何脚本。网页资料查阅于 2026-11-12 至 11-13。下文对 qiaomu-meta 只写转述和判断，不抄原文。

## 摘要

1. `skills/ccr-cli/` 的最小文件集是 9 个文件：`SKILL.md`、`README.md`、`manifest.json`、`agents/interface.yaml`、`evals/trigger_cases.json`，以及 `reports/` 下的 `skill-ir.json`、`trigger-eval.json`、`prior-art-research.md`、`creation-handoff.md`。规范本身只要求 `SKILL.md`，其余 8 个是 qiaomu-meta 的校验脚本要求的。
2. qiaomu-meta 的创作与本地门禁可以直接用在仓库子目录上；它自带的发布器 `publish_skill.py` 假定「一个技能 = 一个仓库，包在仓库根」，与 `skills/ccr-cli/` 放在 ccr 仓库里的前提冲突，不应对这个目录运行。
3. 一个技能包只能有一个 `SKILL.md`，所以三个平台不能拆成三个技能目录放在同一个包里；按平台或主题拆分只能落在 `references/`。
4. 反过来约束 CLI 的要求有六项：技能要教的命令都有稳定的 `--json`（目前 profile 的 `switch` 和顶层 `export`、`import` 没有）；破坏性和批量命令有 `--dry-run`（目前 profile 动作和 `import` 没有）；非交互环境下不挂起并有统一的确认开关；退出码和错误流可依赖；`--help` 是参数的唯一事实来源；任何输出（含 JSON）保持密钥遮蔽并写明备份位置。

## 1. qiaomu-meta 对新技能包的要求

### 1.1 成熟度模式决定门禁

四档模式逐级加码：Scaffold（个人试验）、Production（团队复用）、Library（共享基础设施）、Governed（公开发布，或涉及账号、密钥、网络、文件写入）。它要求选「够用的最轻一档」，缺证据的项一律写 `missing evidence`，不许用计划冒充证据。

ccr-cli 技能包公开发布，教的命令会改写含密钥的配置，按它自己的升级规则落在 Governed。Governed 在 Library 之上再要求：回滚边界、信任边界、密钥扫描、公开声明必须有证据支撑。

### 1.2 校验脚本强制的产物

以下来自 `scripts/validate_skill.py` 与 `scripts/release_check.py` 的代码，是硬约束而不是建议。

| 项 | 要求 | 不满足时 |
|---|---|---|
| 根文件 | `SKILL.md`、`README.md`、`agents/interface.yaml`、`manifest.json` | 失败 |
| frontmatter | `name`、`description` 非空 | 失败 |
| 入口唯一 | 包内除根目录外不得再有名为 `SKILL.md` 的文件；内嵌示例用 `SKILL.example.md`，夹具用 `SKILL.fixture.md` | 失败 |
| 链接 | `SKILL.md` 里指向 `.md` 的相对链接必须存在 | 失败 |
| `agents/interface.yaml` | `interface.display_name`、`short_description`、`default_prompt`；`compatibility.adapter_targets` 非空 | 失败 |
| 同上 | `adapter_targets` 含 `openai`、`claude`、`generic`、`agent-skills-compatible` | 警告 |
| `manifest.json` | `name`、`version`（`X.Y.Z`）、`owner`、`updated_at`、`status`、`maturity_tier` | 失败 |
| 同上 | 有 `release_gates`；`context_budget_tier` 为 `production` 时 `SKILL.md` 不超过 14 000 字节 | 警告 |
| 证据报告 | `maturity_tier` 为 `production`、`library`、`governed` 时必须有 `reports/skill-ir.json`、`reports/trigger-eval.json`、`reports/prior-art-research.md`、`reports/creation-handoff.md`；IR 的名称和版本与 manifest 一致；触发报告全部通过 | 失败 |
| README | 含字面量 `npx skills add`、`你可以直接这样说`、`validate_skill.py`、`Troubleshooting`；manifest 声明了 `upstream_inspiration` 时 README 要原样出现 | 警告 |
| 触发样例 | `evals/trigger_cases.json` 存在，且 `should_trigger`、`should_not_trigger`、`near_neighbor` 三个桶都非空 | 警告 |
| 脚本 | `scripts/*.py` 使用 `argparse` 或带内部模块标记 | 警告 |

`release_check.py` 把「校验有任何警告」也判为阻断，所以上表的警告项在发布检查里等同必需。它另外检查：manifest、IR、触发报告的版本一致；密钥扫描；`git diff --check`；当前分支不是 `main` 或 `master`；工作树干净（本地阶段只警告）；带 `--run-tests` 时在包内跑 `python3 -m unittest discover -s tests`；带 `--install-check` 时做隔离安装。没有 `reports/output-evidence.json`（且其中记录模型实跑或盲评通过）时，输出证据一项保持警告。

推断（读代码得出，未实跑）：包内没有 `tests/` 目录时带 `--run-tests` 会阻断，不带则该项是警告。

### 1.3 触发评测

`scripts/trigger_eval.py` 是关键词概念匹配的冒烟测试，不调用模型。`evals/trigger_cases.json` 需要：

- `positive_concepts`：概念名到短语列表的映射。
- `description_required_concepts`：`description` 必须命中的概念。
- `recommended_threshold`（默认 0.34）与可选的 `negative_patterns`。
- 三个桶，元素是字符串或 `{text, family}`。`near_neighbor` 与 `should_not_trigger` 一样期望不触发。

不写 `positive_concepts` 和 `description_required_concepts` 时会退回到它自己的默认概念（技能、素材、创作动作），ccr-cli 的 `description` 不可能命中，所以这两个键必须自己定义。它只证明 `description` 的用词覆盖了样例，不证明模型真的会选中这个技能；后者要按第 2.3 节的方法另做，做不了就记为 `missing evidence`。

### 1.4 输出评测

技能涉及账号操作、生成代码或发布物，或质量依赖规则时需要。每个用例写清提示语、输入文件、不用技能的基线输出、用技能的输出、可检查的断言、需要人判断的备注。包内没有输出评测的运行器；只有 `reports/output-evidence.json` 记录了通过的模型实跑或盲评，发布检查才记通过，静态夹具不算。

### 1.5 Skill IR

`scripts/export_skill_ir.py` 把包导出为平台中立的 `reports/skill-ir.json`，Production 以上默认要求。它读取的输入决定了其他文件该怎么写：

- `manifest.json` 的 `intent` 对象（`job_to_be_done`、`target_users`、`inputs`、`outputs`、`exclusions`）。不写时目标用户默认成 qiaomu 自己的操作者，所以 ccr-cli 要显式填。
- `agents/interface.yaml` 的 `compatibility`（`canonical_format`、`adapter_targets`、`activation`、`execution`、`trust`、`permissions`、`degradation`）与 `gates`。
- `SKILL.md` 里标题恰为 `Router Rules`、`Compact Workflow`、`Gate Ladder`、`Output Contract` 的二级小节的列表项；用别的标题时 IR 的 workflow 部分为空。
- `evals/trigger_cases.json` 的三个桶。

IR 不得含密钥、本机绝对路径、未测试平台的「已支持」声明。

### 1.6 先例研究与交付说明

- `reports/prior-art-research.md`：查询词、查阅日期、skills.sh 与 SkillsMP 两个目录的结果（安装量与仓库 star 分开记，都不当评分）、候选对比表、每个候选学到的机制、舍弃项、原创部分、缺失证据。要求先去重、回到 GitHub 源头核对，不执行候选的脚本。
- `reports/creation-handoff.md`：点名 2 到 4 个实际读过的参考技能，按保留、改造、舍弃、原创四类说明取舍，每条优势标注为设计优势、已验证优势或假设，并出现当前版本号。

### 1.7 README 与默认署名

README 是给安装者看的产品页，不是 `SKILL.md` 的复制：一句话价值、一行安装命令、3 到 5 个自然语言触发例子、带验证命令的前置条件、输出示例、配置、至少 3 行的 Troubleshooting、风险与副作用、致谢、License。中文优先，不出现私有域名、密钥、本机路径。

默认约定对 ccr 不适用，创建时必须显式覆盖：

- 技能名默认加 `qiaomu-` 前缀；ccr 要用 `ccr-cli`。
- 默认写入 qiaomu 的版权与社交链接；规则允许「另有所有者时」替换。
- 发布器会向 README 注入 qiaomu 的个人介绍区块，并把头像与二维码复制进 `assets/qiaomu-profile/`；非 qiaomu 的包用 `--skip-qiaomu-profile` 关闭。

### 1.8 发布与 npx 安装

规则：只在用户明确要求时发布；禁止直推默认分支；已发布的版本号不可复用；只有远端默认分支版本、GitHub Release、技能发现、干净安装都核对过才算发布完成。

`scripts/publish_skill.py` 的流程（读代码）：核对 frontmatter 与 manifest 的名称和版本 → 检查 `git`、`gh`、`npx` → 确定仓库（参数、`origin`，否则用技能名新建）→ 补 MIT `LICENSE`、README、个人介绍区块 → 功能分支、`git add -A`、提交、推送 → 向默认分支开 PR，检查通过后合并 → 建 `v<版本>` 的 Release → `npx skills add <owner>/<repo> --list` 验证可发现 → 隔离目录里 `npx --yes skills add <owner>/<repo> --skill <name> --yes`，检查 `.agents/skills/<name>/SKILL.md` 存在。

它假定技能包位于仓库根。用在 `skills/ccr-cli/` 上有四处冲突（读代码推断，未实跑，也没有跑 `--dry-run`）：

| 冲突 | 依据 |
|---|---|
| 已发布阶段从默认分支的仓库根读 `manifest.json`，ccr 根目录没有这个文件，会阻断 | `release_check.py` 的 `remote_default_version` |
| 它建的 Release 标签是 `v<技能版本>`，与 ccr 自己的 CLI 版本标签（`v6.5.0` 等）同一命名空间 | `publish_skill.py`；`git tag --list` |
| 源目录的 `origin` 与目标仓库一致时就地操作，`git add -A` 不带路径会暂存整个工作树，PR 直接指向默认分支，绕过 ccr 的 `dev` → `main` 流程 | `publish_skill.py`；[git-add 文档](https://git-scm.com/docs/git-add) |
| 默认在技能目录内再建一份 `LICENSE` 并注入个人介绍区块 | `publish_skill.py` 的 `prepare_package` |

结论：qiaomu-meta 的创作流程和本地门禁（`validate_skill.py`、`trigger_eval.py`、`export_skill_ir.py`、`release_check.py --phase local`）可用于 `skills/ccr-cli/`；发布走 ccr 自己的 PR 流程，不运行 `publish_skill.py`。它要求的「发现与干净安装」证据改用下面的命令手工取得，取不到就记 `missing evidence`：

```bash
npx skills add bahayonghang/ccr --list
npx skills add bahayonghang/ccr --skill ccr-cli
```

## 2. Agent Skills 规范

### 2.1 目录与命名

- 技能是一个至少含 `SKILL.md` 的目录；`scripts/`、`references/`、`assets/` 可选。（[规范](https://agentskills.io/specification)）
- `name`：1 到 64 个字符，只能是小写字母、数字、连字符，不能以连字符开头或结尾，不能有连续连字符，**必须与父目录名相同**。所以 `skills/ccr-cli/` 对应 `name: ccr-cli`。（同上）
- Anthropic 另外禁止 `name` 含保留词 `anthropic`、`claude`，也不能含 XML 标签；建议用动名词或名词短语，避免 `helper`、`tools` 这类泛名。`ccr-cli` 合规；不要把名字展开成带 `claude` 的全称。（[Anthropic 最佳实践](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices)）
- `description`：1 到 1024 个字符。可选字段：`license`、`compatibility`（最多 500 字符，写环境要求）、`metadata`（字符串到字符串的映射，版本号要加引号）、`allowed-tools`（实验性）。（规范）
- `SKILL.md` 建议不超过 500 行、正文不超过约 5000 token；引用文件从 `SKILL.md` 直接链接，只隔一层；超过 100 行的参考文件开头放目录。（规范；Anthropic 最佳实践）

### 2.2 一个目录放多个技能

规范只定义技能目录**内部**的内容，不规定技能目录放在哪里，也没有「技能集合」的格式。（[客户端实现指南](https://agentskills.io/client-implementation/adding-skills-support)）多技能布局的规则来自客户端与安装器：

- 客户端在技能容器目录里找「含 `SKILL.md` 的子目录」。同名冲突时项目级覆盖用户级；同一级内先到或后到取其一，并应告警。校验从宽：`name` 与目录名不一致只警告、照常加载，缺 `description` 才跳过。（同上）
- `npx skills` 安装器在仓库里搜索的容器目录包括仓库根（有 `SKILL.md` 时）、`skills/`、`.agents/skills/`、`.claude/skills/` 等，每个容器向下最多三层；较浅层的 `SKILL.md` 会遮蔽它下面的；同名只保留先找到的；标准位置一个都没找到时才递归全仓搜索。清单里**没有** `.github/skills/`。也可在根目录 `skills.sh.json` 的 `paths` 里追加容器目录。（[skills CLI：creating-skills](https://github.com/vercel-labs/skills/blob/main/docs/creating-skills.md)）
- 安装来源可以是 `owner/repo`、指向仓库内技能目录的 `/tree/<分支>/skills/<名>` 链接、`owner/repo@标签`；`--skill` 选单个技能，`--list` 只列出不安装，`-g` 装到用户级，默认以符号链接指向一份规范副本。（[skills CLI：commands](https://github.com/vercel-labs/skills/blob/main/docs/commands.md)）
- Claude Code 的项目技能只从 `.claude/skills/<name>/` 加载；`<plugin>/skills/<name>/` 是插件布局。frontmatter 里不认识的字段会被静默忽略。（[Claude Code skills 文档](https://code.claude.com/docs/en/skills)）

对 ccr 的含义：

- `skills/ccr-cli/SKILL.md` 是安装器的标准扁平布局，`npx skills add bahayonghang/ccr` 能直接发现。
- 顶层 `skills/` 一旦存在，安装器在标准位置就有结果，不会再递归到 `.github/skills/`，面向仓库开发的两个技能不会被推给最终用户。这是从上面的搜索规则推出的，未实测。
- 顶层 `skills/` 不是任何客户端的项目级加载目录，在仓库里工作的 Agent 不会自动加载它，与 `.github/skills/` 互不干扰。
- qiaomu-meta 的「入口唯一」比规范严：规范和安装器都允许 `skills/` 下并列多个技能，但每个技能目录各自是一个包。以后要拆分，只能在 `skills/` 下加兄弟目录（如共享基础技能加分域技能），不能在 `skills/ccr-cli/` 里嵌套。

### 2.3 description 怎样划触发边界

- 启动时只加载每个技能的 `name` 和 `description`（每个约 50 到 100 token），模型据此决定是否读入全文，所以触发完全由 `description` 承担。（客户端实现指南；[优化 description](https://agentskills.io/skill-creation/optimizing-descriptions)）
- 写法：同时写「做什么」和「什么时候用」，带用户会说的具体关键词；避免空泛的一句话。（规范）用第三人称，不用「我可以」「你可以」。（Anthropic 最佳实践）围绕用户意图而不是内部实现，把适用场景列全，包括用户没点名该领域的情况。（优化 description）两处措辞建议略有出入（第三人称陈述对「Use this skill when…」的祈使句），共同点是不用第一、第二人称叙述；「陈述能力 + Use when…」两者都满足。
- 误触发时在 `description` 里写明不做什么，或划清与相邻能力的边界；不要把失败样例里的词直接塞进去。（优化 description）
- Claude Code 的技能清单把 `description` 与 `when_to_use` 合计截断在 1536 个字符，最重要的用例要放最前。（Claude Code skills 文档）
- 评测方法：约 20 条真实口吻的查询，8 到 10 条应触发，8 到 10 条不应触发；最有价值的反例是共享关键词但实际需要别的东西的「近似」查询；每条跑 3 次，触发率以 0.5 为界；按约 60/40 分训练集与验证集，只用训练集的失败改写，用验证集选最终版本。（优化 description）
- 有副作用的流程，Claude Code 可用 `disable-model-invocation: true` 让技能只能由用户显式调用。这是 Claude Code 专有字段，不在开放规范里，对其他客户端无效。（Claude Code skills 文档）
- 批量、破坏性、高风险操作推荐「先出计划，校验计划，再执行，最后核对」的流程。（Anthropic 最佳实践）

ccr-cli 的 `description` 需要划开的相邻请求（据地图 #167 的范围，属于建议，待创建时用触发样例验证）：仓库开发类请求（归 `.github/skills/`）；直接手改 Claude Code、Codex、Grok 自身的配置文件；账号快照（`auth`）与 Profile 的区分；桌面端与 TUI 的操作。

## 3. 公开先例：教 Agent 操作某个 CLI 的技能

三个先例都是直接读仓库里的 `SKILL.md` 和 README 得到的。没有查询 skills.sh 与 SkillsMP 目录，安装量、star、许可证未采集，这部分按 qiaomu-meta 的说法是 `missing evidence`，留给创建会话补。

| | Google Workspace CLI（`gws`） | Lark CLI（`lark-cli`） | Cloudflare `wrangler` |
|---|---|---|---|
| 位置 | CLI 仓库顶层 `skills/`，一技能一目录 | CLI 仓库顶层 `skills/` | 独立技能仓库的 `skills/wrangler/` |
| 拆分 | 一个共享技能 `gws-shared`（认证、全局参数、输出格式、安全规则），每个 API 服务一个技能，另有助手与配方技能；服务技能把共享技能列为前置 | 一个共享技能 `lark-shared`（身份、认证、输出契约、高风险操作），每个业务域一个技能，另有工作流技能 | 单个技能，约 90 行（估计），按工作阶段组织（看项目、取文档、改、预览、验证），不是命令手册 |
| 参数细节 | 技能只列资源与方法，参数让 Agent 用 `gws <服务> --help` 和 `gws schema <方法>` 现查 | 执行前先看参考文档或 `--help`；有 `schema` 自省命令 | 明确不依赖记忆中的示例，以项目内 `wrangler --help`、配置 schema 和官方文档为准；取不到就说明缺口，不编造语法 |
| 破坏性命令 | 写入和删除前必须向用户确认；破坏性操作优先 `--dry-run`；不输出密钥 | 写入和删除前确认用户意图；命令支持时用 `--dry-run` 预览；退出码 10 表示「高风险确认门禁」而不是错误，用户明确同意后才带确认参数重试；不明文输出密钥 | `secret put`、`secret delete` 按部署对待（立即生效）；`deploy --dry-run` 做验证；生产写入与删除必须符合用户明确表达的意图；回滚前先查其局限 |
| JSON 输出 | 依赖：默认输出 JSON，成功、错误、下载元数据都是结构化 JSON；退出码 0 到 5 分类 | 依赖：默认 JSON；成功信封含 `ok`，错误走 stderr 并带类型、代码、提示；以 `ok` 或退出码判断成败 | 不依赖：全文没有 `--json` |
| 安装 | `npx skills add` 整仓，或用 `/tree/main/skills/<名>` 装单个 | 整仓 `npx skills add larksuite/cli -y -g` | 未查 |
| frontmatter | `metadata.version` 与 CLI 版本一致；声明依赖的可执行文件 | `metadata.requires.bins`；顶层 `version` | 只有 `name`、`description`；`description` 写明适用条件并把相邻情形指给另一个技能 |

来源：[gws-shared](https://github.com/googleworkspace/cli/blob/main/skills/gws-shared/SKILL.md)、[gws-drive](https://github.com/googleworkspace/cli/blob/main/skills/gws-drive/SKILL.md)、[gws README](https://github.com/googleworkspace/cli/blob/main/README.md)；[lark-shared](https://github.com/larksuite/cli/blob/main/skills/lark-shared/SKILL.md)、[lark-cli README](https://github.com/larksuite/cli/blob/main/README.md)；[wrangler 技能](https://github.com/cloudflare/skills/blob/main/skills/wrangler/SKILL.md)。两个 README 里技能数量前后不一致，这里不引用数字。

可带走的做法：

- 认证、全局参数、输出契约、安全规则集中在一处，其余内容只引用它。单技能时对应 `SKILL.md` 正文；参考文件只放分平台或分主题的细节。
- 技能不抄参数表，指向 `--help` 或自省命令。这样技能副本落后于 CLI 版本时不会教错。
- 破坏性命令用三层处理：技能里要求先向用户确认；CLI 提供 `--dry-run` 预览；CLI 自己用可识别的信号拦截（Lark 的专用退出码）。前一层只靠 Agent 自觉，后两层要 CLI 支持。
- 两个依赖 JSON 的先例都把「默认 JSON、成败看字段或退出码、错误有固定结构」写成契约；不依赖 JSON 的那个把正确性押在现查文档上。

拆分粒度的判断：`gws` 与 `lark-cli` 按域拆，是因为各自覆盖几十到上百个 API。ccr 是三个平台共用同一组 profile 动作，更接近 `wrangler` 的单技能形态，也正好满足 qiaomu-meta 的入口唯一规则。

## 4. 与本仓库现状的贴合

- 仓库已跟踪的技能只有 `.github/skills/ccr-gate-recovery/SKILL.md` 和 `.github/skills/ccr-ui-visual-workflow/SKILL.md`，各自只有一个 `SKILL.md`，frontmatter 只有 `name` 与 `description`，面向在仓库里干活的五套工具。`docs/agents/harnesses.md` 规定正文以 `.github/skills/` 为跟踪来源，各客户端装到自己的本地目录。
- `.claude/`、`.agents/`、`skills-lock.json` 都在 `.gitignore` 里，属于本地安装产物。顶层 `skills/` 目前不存在，新增后是仓库里第一个面向最终用户的技能目录。
- 仓库根已有 MIT `LICENSE`。安装器只复制技能目录，许可证不会跟着走；在 frontmatter 写 `license: MIT` 可以让安装后的副本带上许可信息，不必在技能目录里再放一份。
- 版本：`manifest.json` 要求独立的 `X.Y.Z`。它是否跟随工作区版本（`just version-sync` 的范围）没有现成约定，是待定项。
- qiaomu-meta 的脚本是 Python 3（可选 PyYAML），不在 `just ci` 里。`reports/` 下的 JSON 是脚本生成物且要入库，谁在什么时候重新生成没有现成约定，是待定项。

## 5. 结论

### 5.1 `skills/ccr-cli/` 的最小文件集

```text
skills/ccr-cli/
├── SKILL.md                    # name: ccr-cli；description 划触发边界；正文是路由与最小流程
├── README.md                   # 面向安装者的产品页，含 npx 安装命令与 Troubleshooting
├── manifest.json               # 名称、版本、owner、maturity_tier、intent、release_gates
├── agents/
│   └── interface.yaml          # 展示名、默认提示语、目标平台、权限与信任边界、门禁
├── evals/
│   └── trigger_cases.json      # 自定义概念 + 应触发 / 不应触发 / 近邻 三个桶
└── reports/
    ├── skill-ir.json           # export_skill_ir.py 生成
    ├── trigger-eval.json       # trigger_eval.py 生成，须全部通过
    ├── prior-art-research.md   # 先例研究
    └── creation-handoff.md     # 交付说明
```

分层看：规范只要 `SKILL.md`；qiaomu-meta 校验不失败要前 4 个；发布检查不阻断要加 `evals/trigger_cases.json`；`maturity_tier` 为 `production` 及以上要再加 4 份报告。公开分发的包属于后者，所以是 9 个。

不在最小集里、按需再加的：

- `references/`：`SKILL.md` 超过 14 000 字节或 500 行时，把分平台、分主题的细节挪过去。三个平台加导入导出，很可能第一版就用得上。
- `reports/output-evidence.json` 与输出评测用例：没有时发布检查保持警告，对外不能声称输出质量已验证。
- `tests/`、`scripts/`：包里没有自带脚本时不需要。
- `LICENSE`、`assets/qiaomu-profile/`：不需要，见 1.7 与第 4 节。

创建时必须显式告诉 qiaomu-meta 的三件事：技能名用 `ccr-cli` 而不是 `qiaomu-` 前缀；所有者是 ccr 维护者；不运行 `publish_skill.py`，分发走 ccr 仓库自己的 PR 流程。

### 5.2 反过来约束 CLI 的要求

现状一栏是在基线上的抽查，不是逐命令审计。

| 要求 | 来源 | 现状 |
|---|---|---|
| 技能要教的每条命令都有 `--json`，字段稳定，stdout 只有 JSON | `gws`、`lark-cli` 都把结构化输出写成契约；技能不该教 Agent 解析给人看的文本 | `ccr claude profile` 的动作里除 `switch` 外都带 `--json`（`crates/ccr-cli/src/cli/subcommands/claude.rs`）；Grok 的 profile `switch` 同样没有，Codex 未逐项核对；顶层 `export`、`import` 没有（地图 #167 现状基线）。各动作的 JSON 是否共用一种成败结构未核对 |
| 破坏性与批量命令有 `--dry-run` | 三个先例都用它做预览；Anthropic 的「计划、校验、执行」流程；Governed 档要求回滚边界 | 只出现在 `clean`、会话清理、Codex 修复与历史同步等维护命令上；profile 的各动作和顶层 `import`（合并或整体替换）没有 |
| 非交互环境不挂起，确认开关统一 | Agent 无法回答终端里的确认提示；技能只能教一条规则 | 有全局 `-y/--yes` 与各命令的 `--force`，但 `--force` 的含义不统一（跳过确认、覆盖同名、删除前先恢复运行态都叫这个名字）；stdin 不是终端时各命令的行为未逐一核对 |
| 成败可由退出码或 JSON 字段判断，错误走 stderr | `gws` 的分类退出码；`lark-cli` 的 `ok` 字段与专用确认退出码 | `.trellis/spec/ccr-cli/backend/cli-output-presentation.md` 记录了「部分既有错误分支返回 `Ok`，退出码仍为 0」这一保留行为；技能要教的命令需要逐个确认不在此列 |
| `--help` 完整且是参数的唯一事实来源 | `wrangler` 与 `gws` 的技能都不抄参数表；安装到用户机器的技能副本会落后于 CLI | 未核对各子命令帮助文本的完整性 |
| 任何输出（含 `--json`）保持密钥遮蔽；破坏性修改的备份位置能从输出里读到 | 先例的「不输出密钥」规则只有 CLI 默认遮蔽才守得住；Governed 档要求技能能说明如何回滚 | 遮蔽与改前备份是仓库既有约束；备份路径是否出现在 `--json` 输出里未核对 |

两条不改 CLI 行为、但约束命令面演进的要求：

- 技能教的是规范入口 `ccr <平台> profile …`。旧入口隐藏后仍要可用并给出迁移提示，因为已安装的技能副本不会随 CLI 升级。
- 技能需要一个判断 CLI 版本是否够用的办法（`ccr --version` 加 `compatibility` 字段，或在 `metadata` 里声明最低版本），否则教到尚未发布的命令时没有降级路径。

## 6. 未核实与待定

- 没有运行 qiaomu-meta 的任何脚本；1.2、1.8 中标注为推断的结论只来自读代码。
- 没有实测 `npx skills add bahayonghang/ccr`；2.2 的发现行为来自安装器文档。
- 先例没有经过 skills.sh 与 SkillsMP 检索，流行度、许可证、维护状况未采集。
- 没有逐命令审计 ccr 的 `--json`、确认与退出码行为。
- 待地图决定：技能版本是否跟随工作区版本；`reports/` 生成物由谁、何时重新生成；是否需要模型实跑的触发与输出证据，还是接受 `missing evidence`；如果以后想要 qiaomu-meta 完整的发布门禁，是否把技能镜像到独立仓库。
