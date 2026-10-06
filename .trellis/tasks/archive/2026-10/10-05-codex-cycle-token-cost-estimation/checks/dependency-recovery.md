# 前端依赖门槛恢复记录

日期：2026-10-06。状态：SCOPED_RECOVERY_PASS / FULL_CI_PASS（主会话执行）。

用户已回复“批准，继续”，授权 [dependency-recovery-draft.md](dependency-recovery-draft.md) 的两份锁文件、3 个现有条目更新及完整 CI 复测。本文记录执行角色的限定修改和检查。完整 `just ci` 由主会话运行，本文不将局部检查作为 AC12 的完成证据。

## 实施范围

| 文件 | 条目 | 更新 |
| --- | --- | --- |
| `ccr-ui/bun.lock` | `postcss-selector-parser` | 7.1.4 → 7.1.6 |
| `ccr-ui/bun.lock` | `source-map-js` | 1.2.1 → 1.2.2 |
| `docs/bun.lock` | `source-map-js` | 1.2.1 → 1.2.2 |

对应 SHA-512 integrity 使用已核实并获批准的 [dependency-recovery-draft.patch](dependency-recovery-draft.patch)。`postcss-selector-parser` 的 cssesc / util-deprecate 依赖范围未变，`source-map-js` 未添加依赖。执行后反向 `git apply --check --unidiff-zero` 退出 0，补丁与工作区一致。

逐文件比较 `HEAD` 内容加上述替换与当前锁文件，确认差异只有 3 个条目；安装后的 package.json 版本与锁定版本一致。检查脚本读取已批准 patch，并核对每条 integrity；Bun frozen install 成功。没有另行下载 tarball 计算独立哈希。

`ccr-ui/package.json`、`docs/package.json`、`ccr-ui/patches/braces@3.0.3.patch` 和 `ccr-ui/scripts/frontend-audit-allowlist.json` 的 `git diff --exit-code` 退出 0。保留现有 braces 补丁、例外数量、到期规则和审计门槛。其他 Codex 功能改动及任务外修改均保留。

## 检查结果

Bun 版本为 `1.4.2 (744846f84)`。

| 命令 / 检查 | 执行目录 | 退出码 | 状态 / 证据 |
| --- | --- | --- | --- |
| `bun install --frozen-lockfile` | `ccr-ui` | 0 | PASS；[dependency-ui-frozen-install.log](dependency-ui-frozen-install.log) |
| `bun install --frozen-lockfile` | `docs` | 0 | PASS；[dependency-docs-frozen-install.log](dependency-docs-frozen-install.log) |
| 安装版本、批准 integrity、限定差异和未变项核对 | root | 0 | PASS；[dependency-scope-verification.log](dependency-scope-verification.log) |
| `git diff --check -- ccr-ui/bun.lock docs/bun.lock` | root | 0 | PASS；同上 |
| `just frontend-audit` | root | 0 | PASS；[dependency-frontend-audit.log](dependency-frontend-audit.log) |
| 完整 `just ci` | root，主会话 | 0 | PASS；[just-ci-06.log](just-ci-06.log)，16项步骤全部通过，12:30.169 |

审计输出：`frontend dependency audit passed: 1 reported advisories, 1/1 active exceptions`。该项使用已有有效例外；未新增例外。第五轮 CI 的原始失败保留在 [just-ci-05.log](just-ci-05.log)，未覆盖。上述限定检查未发生失败，无需重试。

## 角色和证据边界

实现角色读取原生注入的 saved hook，并补读未内联的 task / research 文件、UI frontend index 及 UI/docs AGENTS。产品修改只涉及本页所列两份锁文件。执行角色未运行 Cargo 或完整 `just ci`，没有提交、归档、push、PR、merge、release 或安装用户工具。交回主会话后进行独立检查及正式门槛复测。

真实账户、其他操作系统与 hosted CI 的状态沿用任务验证记录；依赖恢复没有增加这些证据。

## 主会话最终验收补记

独立限定范围检查完成后，主会话执行第六轮完整 `just ci`，退出 0。16 项步骤全部通过，用时 12:30.169。Frontend Audit 保留 1 reported advisory / 1/1 active exceptions；未新增例外。AC12 完成，正式结果见 [verification.md](verification.md)。上述结果由主会话取得，执行角色未运行完整 CI。
