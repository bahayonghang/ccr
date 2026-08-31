# 全项目依赖审计与分批升级

## Goal

建立覆盖 CCR 全仓库的可复现依赖基线，识别过期依赖、安全风险、废弃 API、可安全升级项与存在 Breaking Change 的升级项，并形成按风险从低到高、每批均通过完整门禁后才继续的实施合同。

## Background

- 当前分支为 `dev`，创建任务前工作树干净；当前没有其他活动 Trellis 任务。
- 仓库包含根 Rust workspace、`ccr-ui/src-tauri` Rust workspace、Tauri 命令宏、React/Bun UI、npm 管理的 VS Code 扩展、VitePress 文档站，以及 GitHub Actions/工具链版本。
- `code_map.md` 与 `ccr-ui/package.json` 证明当前 UI 已是 React；用户提供说明中的 Vue 描述是旧信息，扫描与升级以当前清单、锁文件和源码为准。
- 本任务处于 `planning`；本阶段不得改产品依赖清单、锁文件或源码，也不得运行 `task.py start`。

## Requirements

### R1 — 全量依赖清单

- R1.1 盘点所有受版本控制的依赖清单、锁文件、工具链固定版本、GitHub Actions 引用与 pinned git revision。
- R1.2 区分 direct/runtime、direct/dev/build、transitive、platform-specific 与 repository tooling 依赖。
- R1.3 记录每套生态的权威清单、锁文件和安装/解析工具，避免混用 Bun、npm 与 Cargo 的更新结果。

### R2 — 风险与升级分类

- R2.1 为每个可更新的直接依赖记录当前约束/解析版本、候选目标版本、SemVer 跨度及证据来源。
- R2.2 单独列出已知漏洞、受影响版本范围、可利用性相关上下文、修复版本与无法确认的边界。
- R2.3 通过编译器/类型检查告警、源码使用点及上游迁移/发布说明识别废弃 API；仅凭版本过期不得宣称项目正在调用废弃 API。
- R2.4 将候选项分为：无需源码迁移的低风险更新、需要聚焦适配的中风险更新、包含 major/明确 Breaking Change/关键基础设施迁移的高风险更新、暂缓或无可用修复。
- R2.5 对 pinned git 依赖保留 commit 可追溯性；不得把“上游有新提交”等同于可安全升级。

### R3 — 分批升级合同

- R3.1 已确认的安全修复优先于普通版本更新；安全修复队列内部再按实施风险从低到高形成互不混杂、可独立验证和回滚的批次。安全问题不得被藏在普通版本更新中。
- R3.2 每批明确清单/锁文件变更、预期源码迁移、重点回归面、回滚点和进入下一批的门槛。
- R3.3 每完成一批先运行聚焦检查，再运行仓库完整门禁；完整门禁失败时必须在同一批内定位、修复并重跑，稳定前不得进入下一批。
- R3.4 任何新增依赖、远程发布、提交、推送或用户全局工具安装均不在隐含授权范围内。

### R4 — 证据与结果边界

- R4.1 报告分别列明 Passed、Failed、Skipped、Unavailable 与 UNVERIFIED；网络查询、锁文件解析和本地测试证据不得互相替代。
- R4.2 规划结论必须包含仓库文件/行号、命令与上游权威来源；动态版本/公告在实施前需刷新。
- R4.3 规划完成后必须再次获得用户对最新版规划摘要的明确批准，才能启动第一批升级。

## Key Decisions

- 2026-08-30：用户确认“安全修复优先于普通低风险更新”。执行顺序固定为：安全锁文件修复 -> 安全兼容升级 -> 安全 Breaking 升级 -> 普通 Low -> 普通 Medium -> 普通 High/Breaking。
- “安全优先”不降低验证标准：每个安全批次仍按其实施风险隔离，必须完成聚焦检查、安全复扫和完整 `just ci`；失败时停留在当前批次修复或回到 planning，不得用未验证的强制 override 换取表面零告警。
- 保持一个顺序执行的 Trellis 主任务，不创建可并行子任务；原因是各批共享锁图、版本漂移合同和全仓门禁，只有单一状态机能可靠表达“上一批稳定后才继续”。

## Acceptance Criteria

- [x] AC1：扫描矩阵覆盖根 Cargo workspace、Tauri Cargo workspace/命令宏、React/Bun UI、VS Code/npm、VitePress、GitHub Actions、Rust/Node/Bun 工具链与 pinned git 依赖，且列出任何明确排除项。（R1）
- [x] AC2：每个直接依赖均可追溯到当前约束/锁定版本，并被标记为 current、patch/minor 可升级、major/Breaking、vulnerable、deprecated-use、deferred 或 unavailable 之一；未知项显式保留。（R2、R4）
- [x] AC3：安全发现包含 advisory/来源、受影响范围、仓库实际解析版本和处置批次；无漏洞扫描能力或无锁文件的生态明确报告缺口。（R2.2、R4）
- [x] AC4：废弃 API 结论至少有编译/类型告警、源码使用点或官方迁移说明之一支撑，不将依赖“旧”误报为 API“废弃”。（R2.3）
- [x] AC5：`design.md` 定义安全优先顺序、实施风险模型、批次门禁、失败处理、回滚边界与动态证据刷新规则；`implement.md` 先列安全批次且在各队列内部按风险递增，并给出每批验证命令。（R3、R4）
- [x] AC6：每个实施批次的完成条件包含相关聚焦测试、`just ci` 完整门禁、依赖安全复扫和干净差异审计；任何失败都会阻止下一批。（R3.3）
- [x] AC7：规划阶段只修改本 Trellis 任务工件；未修改产品清单、锁文件或源码，未安装新工具，未启动任务实施。（R3.4、R4.3）

## Out of Scope

- 在规划获批前实际更新版本、重写锁文件、迁移源码或修复由升级触发的测试失败。
- 自动合并机器人、发布流程、提交、推送、PR 或 Release。
- 以依赖升级为名进行无关重构、功能变更或新增依赖。
- 将本地静态/测试结果宣称为生产环境、供应链发布者身份或未来版本稳定性证明。
