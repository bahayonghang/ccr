# 前端依赖门槛恢复方案

状态：DRAFT / NOT_APPLIED。日期：2026-10-06。现有 Codex 功能实现没有修改下列依赖文件；该方案扩大到前端和文档锁文件，需要单独批准后执行。

2026-10-06审批更新：用户回复“批准，继续”，批准本方案两份锁文件的3条目补丁及完整CI复测。以上DRAFT状态为批准前历史；实际实施和复测结果将记录于`dependency-recovery.md`。

## 已确认失败与来源

`just-ci-05.log` 完整执行20:25.175后，Frontend Audit退出1。此前13项门槛PASS；后两项已单独补跑PASS。正式 `just ci` 结果仍为FAIL，AC12未完成。

| package | 已有版本 | 公告 | 核实的补丁版本 |
| --- | --- | --- | --- |
| postcss-selector-parser | ccr-ui 7.1.4 | [GHSA-rj75-hqrm-r3gf](https://github.com/advisories/GHSA-rj75-hqrm-r3gf)，moderate，2026-10-05更新 | 7.1.6 |
| source-map-js | ccr-ui/docs 1.2.1 | [GHSA-68fv-2mgg-jv7q](https://github.com/advisories/GHSA-68fv-2mgg-jv7q)，high，2026-10-05更新 | 1.2.2 |

`git diff --exit-code -- ccr-ui/package.json ccr-ui/bun.lock docs/package.json docs/bun.lock`退出0。`git show HEAD:ccr-ui/bun.lock`包含相同7.1.4/1.2.1条目。失败来自既有锁定依赖的当前审计结果。`frontend-audit-allowlist.json`和阈值未修改。

已通过官方npm registry读取两补丁版本的version、dependencies和dist.integrity：7.1.6依赖仍为cssesc ^3.0.0、util-deprecate ^1.0.2；1.2.2没有新增dependencies。补丁版本满足现有父依赖版本范围。

## 拟修改

仅更新3个现有锁条目，见同目录 `dependency-recovery-draft.patch`。该文件为零context diff；`git apply --check --unidiff-zero`检查通过，产品锁文件未应用patch：

1. `ccr-ui/bun.lock`：postcss-selector-parser 7.1.4→7.1.6、source-map-js 1.2.1→1.2.2。
2. `docs/bun.lock`：source-map-js 1.2.1→1.2.2，与同一已确认公告对齐。

两份package.json和审计allowlist保持原值。不添加依赖、不增加忽略项、不修改门槛。执行角色拥有这两份锁文件；本任务其他产品文件保留已验证实现。当前patch仅为可审阅资料，尚未应用到产品锁文件。

## 批准后的验证

- 两目录分别`bun install --frozen-lockfile`，确认版本/完整性与限定diff。
- 先`just frontend-audit`，失败保留并核实；不得新增例外来消除失败。
- 然后完整`just ci`，保留默认并行、独立binding、覆盖率和其他门槛。正式PASS后再完成AC12。
- 如果工具引入额外锁变更，先审查原因和范围，保持原有braces patch；不能使用全面升级替代该3条目方案。

批准依赖恢复不包含提交、归档、push、PR、merge、release或安装。
