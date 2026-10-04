# T07 续作与父任务集成独立复核

日期：2026-10-04（America/Chicago）。基线：`dev / c50716b5ed1bcd1c5b4950a746a8621258d0e0b5`。本代理只写指定 independent 证据，不修改产品、审批、任务元数据或规范。重型检查由主会话与实施者执行；本代理复算现有原始回执，未重复运行测试。

结论：T07 批准的六行夹具修改、精确 JSON 忽略和传递环境读取规范符合已审查候选。六份局部 Cargo 回执、工作区测试和严格 lint 的日志一致性通过。T05 固定两个锁节点符合范围，当前 npm 审计仍为 6 high。T08 原 0/0 验收未满足；T01 精确版本新生成契约 FAILED，macOS、hosted 和 fresh-client 边界未验证。完整 `just ci` 最终回执待主会话提供，不宣称父任务已完成。

## Findings (fixed)

无本代理产品修复。未发现六行实现的机械问题。

已核对由所属实施者修复的原缺口：

- 文件：`crates/ccr/tests/managers/general.rs` 和 `legacy_registry.rs`。
- 原问题：六个写入测试使用显式文件路径，但共享保存操作仍会读取进程 `CCR_LOCK_DIR`，没有持有同一环境 mutex guard。
- 当前修复：general 新增 5 行、legacy_registry 新增 1 行；命名绑定为 `let _env = ...`，保持至测试作用域结束。去除这六行后与 HEAD 原文件逐字节换行规范化结果相同，断言和生产逻辑未改。候选 SHA-256 匹配 `a7082c421018a9b54167b3069ec3000cc21210bdebe3ca135cf6d7f52e775738`。

主会话另行将本轮 task metadata/docs 的 CRLF 新行规范为 LF，修复首次默认 `git diff --check` 报告。原始日志字节未修改；初次 exit 2 与后续默认检查 exit 0 分开记录。

## Findings (not fixed)

### 1. T08 当前配置未满足原 AC1

`frontend-audit-allowlist.json` 为 1 个 high 公告、1/1 例外；原 PRD/spec 为 0/0。当前 PRD 已追加日期快照，保留历史勾选并明确本轮未满足原 AC1。例外调整尚待用户明确答复，不能以当前审计退出 0 关闭原验收。

T08 本轮研究核对官方 braces tarball、全部 10 个安装文件及登记 patch 重建；报告未发现未登记 MAX_LENGTH 修改。Node/Bun 局部深度100/101/2000和8组普通样例通过；没有持久 braces 回归测试，完整前端覆盖率及固定工具链边界仍须独立记录。该项是安全政策与批准范围判断，本代理不修改政策。

### 2. T05 当前 audit 保留 6 high

实际批准更新只有 brace-expansion 5.0.9→5.0.12、fast-uri 3.1.7→3.1.8，每节点三个字段，共六个叶字段。原 327 解析条目未变，VSCE 仍为 3.9.2。audit before 为 7 high + 1 moderate / exit 1，after 为 6 high / exit 1。原始 stdout/stderr 哈希与 metadata 计数已独立匹配。

VSCE 4 副本 audit 0 不能代替当前实际锁的 FAIL，也不能覆盖历史 zero 回执。用户后续已明确批准major迁移；截至本报告核对的两节点产品快照，major候选尚未应用，需等待主会话CI源快照结束后由实施者处理。审批前官方源码复核见父任务 `research/resume-2026-10-04-independent-vsce4-review.md`。安全扫描 core API 运行、目标 Node 安装/打包/合成负例均 **PREPARED_NOT_RUN**。

### 3. T01 精确版本新生成契约 FAILED

用户已明确批准保留本地生成并调整验收。产品三文件的最小改动保留全部14项测试和断言，增加缺扩展 `SKIPPED_UNVERIFIED` 输出；双语说明均要求匹配版本生成、14项实际运行及原生加载/trust分别验收。

本机现存扩展14 pass / 99 assertions；仅源码夹具0 pass / 14 skip / 0 fail。两类结果已分开。原安装CLI0.6.17与项目0.7.0-beta.4不同、旧扩展缺生成来源的历史记录保留。

后续使用官方精确CLI/core 0.7.0-beta.4在自有临时目录生成新项目。安装、实际版本、help、init回执exit0；生成扩展SHA为`5fbd4cfe2461f5457c6ba93fb168ad0dbdfc0e76d6626fd0407eebc13baf8a35`。当前测试原字节在该临时项目运行，exit1、8pass/6fail/0skip/50assertions；14项全部实际执行。五份新回执日志bytes/SHA与真实产品before/after输入均独立匹配，registry原文hash和临时安装lock version/resolved/integrity一致。

源码第316–317行缓存输入只包含prd/info，643–655行显式材料化只读取prd/info，全文没有design.md或implement.md。六条首次失败均为design内容/路径断言缺失。当前事实支持“新生成扩展漏design/implement材料化与缓存输入”，不证明后续缓存/UTF8/预算断言全部执行；不能将六条失败都归因于各自测试名中的独立机制缺陷。

该受控fresh-generation契约由UNVERIFIED更新为 **FAILED**，调整后的AC3未完成。真实ignored扩展未改；原生OMP加载/trust仍为 **UNVERIFIED**。生成资产修复涉及已批准本地生成交付方案的维护机制，需主会话批准具体project patch范围，本代理不直接修生成扩展或放宽断言。

### 4. T07 外部验收边界仍未齐

macOS进程测试、当前最终SHA的hosted结果、fresh-client原生加载/trust保持 **UNVERIFIED**。历史Linux coverage/MSRV/Windows/Tauri回执保持原时间与源码边界；本轮局部测试不替代正式环境矩阵。历史 FileLockError 的精确交错和原因仍未查明。

### 5. 原始日志交付范围需要保留明确状态

本轮 stdout/stderr `.log` 受现有忽略规则影响。本代理核验本机实际文件；未暂存、强制纳入、提交或归档。若只交付JSON和报告，接收方无法仅凭版本库重算raw日志。主会话交付时应单列本机原始证据与仓库交付文件范围，保留用户指定的诊断大JSON精确忽略。

## 调用链与生命周期核对

`ConfigManager::save` → config file handler → `fileio::write_toml_opts`；`PlatformConfigManager::save` → `fileio::write_toml`；`SettingsManager::save_atomic` 直接进入 `write_guarded`，`restore` 进入 `save_atomic`。`guarded_write.rs:103–112` 选择 `LockManager::with_default_path`，`lock.rs:218–229` 读取进程 `CCR_LOCK_DIR`。显式配置文件路径和settings lock参数不消除该读取。

两个测试模块使用同一测试二进制中的 `support/env.rs`。fixture先持有静态mutex再设置环境，Drop在mutex仍存活时恢复两个变量。六个新绑定是命名局部变量，保存、读取、断言和随后创建资源的清理都在guard作用域内；无未等待后台写入。7个环境相关测试使用现有fixture互斥，其余10个用例继续默认并行。没有新增serial、线程覆盖、重试或阈值调整。

`test-fixtures.md`只增加一条传递读取规则，明确上述三个manager保存路径及五工具适用范围；与源码一致。

`.gitignore:108`精确锚定一个诊断JSON。当前文件大小1,164,069 bytes，SHA-256 `fd50a44aab9e7d9ab269425ceaa6c691004db9390147cf3be9ad273c7d43bd96`，与应用前后回执一致。只读取大小和hash，没有解析或输出JSON内容。

## Verification

| 项目 | 结果和边界 |
| --- | --- |
| T07六份Cargo回执 | PASS：12份原始log的SHA/bytes、14个输入before/after/current SHA、退出码、fixture cleanup与默认并行一致 |
| exact / all-features managers | 1 passed / 16 filtered；17 passed / 0 failed |
| platforms / sync_content / default managers | 35 passed；3 passed / 114 filtered；17 passed；全部0 failed |
| T07 clippy | PASS，all-targets/all-features、`-D warnings`，编译结束记录与exit0一致 |
| 源码与测试名 | suite/default日志17个实际测试名与两个源模块17个test完全相同 |
| 父任务 `just test` | PASS：41个result blocks，1867 passed / 16 ignored / 0 failed；stderr确认workspace/all-features/skip-export默认并行 |
| 父任务 `just lint-strict` | PASS：全workspace/all-targets/all-features，`-D warnings -D clippy::unwrap_used` |
| 父lint/test输入快照 | 仅5个UI/policy/justfile输入，前后与当前一致。不能扩展为整个Rust源码前后未变结论 |
| T07/parent context validate | PASS，均exit0 |
| workflow governance | PASS，exit0，serial-only 0 / target0 |
| T05 npm-ci / vscode-ci | PASS，原始日志hash一致；当前audit仍FAIL6high |
| T05 lock范围 | PASS：2节点×3字段，没有其他锁字段变化 |
| T01精确版本新生成 | FAILED：8pass/6fail/0skip/50assertions，CLI/core/project均0.7.0-beta.4；5份回执和生成源码hash匹配 |
| 默认 `git diff --check` | 初次FAIL2（任务文档CRLF新行）；主会话规范元数据后最终默认重验PASS0，见独立OMP final补充回执 |
| 原full CI失败 | 保留exit1，原log SHA独立匹配 `acaa41ae374ef61a8d44733e49e09e5ef59f0800fafc78febaec2195c1fe1cc2` |
| 当前full CI | 等待主会话最终回执；不由局部PASS推断 |
| TypeCheck | Rust由局部clippy/test与全workspace严格clippy完成编译；本代理未重跑独立check或前端type-check |

机器可读复核：`resume-2026-10-04-independent-verification.json`、`resume-2026-10-04-independent-parent-verification.json`、`resume-2026-10-04-independent-omp-final-verification.json`、`resume-2026-10-04-independent-omp-matched-verification.json`。两个验证脚本只读回执和源码，运行窄静态检查，拒绝覆盖已有证据文件。T01四份调整回执的原始日志bytes/SHA与before/after/current输入已核验一致；local14pass、absent14skip及明确marker、版本不匹配结论分别匹配实际输出。五份精确版本生成回执和首次FAIL另行核验并保留。最终CI核验应新增补充回执，保留首次检查记录。
