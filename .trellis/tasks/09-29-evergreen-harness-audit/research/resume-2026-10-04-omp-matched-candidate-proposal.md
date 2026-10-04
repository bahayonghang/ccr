# T01 版本限定 OMP 项目补丁候选

状态：只在自有临时生成目录演示，真实产品未应用；新增仓库交付范围待批准。

## 最小差异

精确 CLI 0.7.0-beta.4 官方生成扩展与本机已定制扩展只有 3 个差异块，共 **新增 16 行、删除 14 行**。候选没有复制整个扩展为仓库交付文件。

1. 定义统一的 PRD/design/implement/info 材料列表。
2. 缓存输入使用该列表，设计与实施文件变化触发刷新。
3. 材料化使用同一列表和已有可信路径解析、预算函数，保持 info 兼容。

候选保存为 [patch](resume-2026-10-04-omp-matched-candidate.patch)。SHA-256：`e70526c73eedb4562587a77895f8d5f32a01b896f5b7f9137ca791d50d35268e`。目标官方生成原文 SHA-256：`5fbd4cfe2461f5457c6ba93fb168ad0dbdfc0e76d6626fd0407eebc13baf8a35`。预期补丁后 SHA-256：`a9ef13beb1bb7214e2ada5cb43fea596065ab296b68a8da171220a77840c5851`。两个源码均为 LF，未引入换行差异；元数据见[候选核对](resume-2026-10-04-omp-matched-candidate-metadata.json)。

## 临时演示

对官方 CLI 临时生成目录执行 `git apply --check`，退出 0；应用同一 patch，退出 0。随后只复制当前工作区测试源码，执行同一 14 项契约：**14 pass、0 fail、99 assertions、退出 0**。具体命令、日期、输入 SHA、stdout/stderr 原始日志及哈希分别见 `resume-2026-10-04-omp-matched-patch-check.json`、`*-patch-apply.json`、[候选测试](resume-2026-10-04-omp-matched-candidate-test.json)。

该临时成功不覆盖原官方新生成 **8 pass / 6 fail**；两次结果独立保留。真实仓库源码、原 ignored extension 和其他代理产品文件未改。native loading/trust 仍为 UNVERIFIED。

## 建议新增交付范围

最小可直接审阅的交付为：

- 新增 `scripts/trellis/patches/omp-context-0.7.0-beta.4.patch`，内容为本次 30 行增删候选。
- 更新现有 `docs/agents/harnesses.md` 与英文镜像的生成步骤：核对项目与 CLI 精确版本、核对原始 extension SHA，执行补丁检查/应用，再实际运行全部 14 项。生成后的 extension 继续 ignored。已为补丁后 SHA 时跳过重复应用；其他版本或 SHA 不匹配时保留 UNVERIFIED 并停止，不自动恢复或覆盖。

如要求自动化 fail-closed 与重复运行保障，再批准新增 `scripts/trellis/apply-omp-context-patch.mjs` 及相应合成 fixture 检查。该工具应只处理固定版本/固定 patch/固定 extension 路径，拒绝版本或 hash 漂移与路径越界，原始 SHA 和已补丁 SHA 为唯一允许值；不调用真实 repo 的 init/update，不改用户 trust 或全局安装。该自动化工具尚未实施，本候选不预先扩大该范围。

新增 patch 跟踪与 setup 步骤均需明确批准。批准后的正式验证应包含精确版本官方来源/完整性、从新生成资产应用仓库 patch 后 14 项实际通过、已补丁重复操作与错误版本/hash 拒绝、缺扩展 SKIPPED_UNVERIFIED，以及最终独立差异审查。当前根 CI 的本机定制扩展 PASS 不替代该新交付验证。
