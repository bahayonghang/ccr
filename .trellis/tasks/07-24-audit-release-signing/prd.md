# Release 签名与 Provenance

> 父任务：`07-24-audit-remediation` ｜ 覆盖：P2-14 ｜ 报告 Epic E8

## Goal

把 release pipeline 从"仅 checksum"提升到"identity attestation"：平台代码签名 + notarization + provenance，让用户可区分官方 artifact 并抗篡改。

## 背景 / 证据（已核实）

- `ccr-ui/src-tauri/tauri.conf.json` — macOS `signingIdentity: null`、Windows `certificateThumbprint: null`
- `.github/workflows/release.yml` — 生成 SHA256（如 `:188-190` VSIX sha256sum），但未观察到代码签名、notarization、provenance

## Requirements

- [ ] macOS：signing + notarization（Developer ID + notarytool）
- [ ] Windows：Authenticode 签名（配置 `certificateThumbprint` 或 signtool 步骤）
- [ ] VSIX：publisher 签名
- [ ] GitHub OIDC / Sigstore provenance（SLSA）
- [ ] 更新链路：验证签名/provenance 后再更新（配合 updater）
- [ ] （可选）CycloneDX/SPDX SBOM 随 release 发布

## Acceptance Criteria

- [ ] macOS/Windows/VSIX artifact 全部签名并可验证
- [ ] CLI artifact 附 provenance
- [ ] release 文档说明验证方式
- [ ] 自动更新在验证通过前不应用（若已启用无提示更新，先关闭直至链路完成——对齐报告 §13 发版建议）

## Out of Scope

- 不把证书、私钥、PAT 或 publisher credential 写入仓库
- 不把 unsigned development artifact 标记或发布为正式 release
- 不以 checksum、fixture 或 dry run 替代真实平台/Marketplace 签名验收

## Notes

- **需外部证书流程**：Apple Developer ID、Windows code signing cert、VSCode Marketplace publisher，涉及密钥管理与 secrets 配置，非纯代码任务
- 报告 §11 指出 repo config 显示 null 但外部发布流程可能另有配置——落地前先盘点 release secret/process inventory
- 2026-07-27 现场盘点：使用 keyring OAuth 回读确认远程 `release` environment 已存在并仅允许 `v*` tag，但 environment/repository secrets 与 variables 均为 0；本机 12 个 Apple/Windows/VSIX 身份环境变量也全部缺失。`main`/`dev` strict branch protection 已验证，但仓库仍没有可用于真实发布的签名身份。因此只能证明 workflow、校验脚本、托管回归和文档的仓库侧闭环，不能把真实签名 artifact 验证宣称为已通过
- 优先级 P2，可在 P1 发版阻断组完成后进行

## Key Decision

- 2026-07-26 用户选择严格端到端验收：本子任务和父任务必须保持未完成，直到提供相应证书、publisher 权限和可读写 Actions secrets 的 GitHub 凭据，并验证真实签名 artifact 与 provenance；仓库侧 workflow/fixture 通过不构成归档条件。

## Verification checkpoint (2026-07-27)

- 仓库侧已实现受保护 `release` environment 的 fail-closed DAG：CLI/Tauri
  平台签名、VSIX sign-tool + Marketplace 发布、集中 SBOM/checksum、GitHub
  OIDC provenance，且 GitHub Release 只在前述步骤全部成功后创建。
- `just release-security-check`、`actionlint 1.7.12`、
  `just ci-governance-check`、`just vscode-ci`、`just ui-check`、中英文
  docs build/audit 与最终 `just ci` 已通过；最终 CI 的 12 个步骤全部为绿。
- 首次聚合 CI 的 104 个 smoke 文件 / 464 个测试本身全部通过，但 Vitest
  worker teardown 出现两次瞬态 `onUserConsoleLog` rejection；focused、独立
  full smoke 及最终完整 CI 均未复现，因此未引入无证据的 runner workaround。
- `just version-check` 被不属于本任务的并行 `7.0.0` doc-drift 阻塞；未修改
  或暂存对应版本元数据。
- 远程 `release` environment 已存在，custom deployment branch policy 为
  tag `v*`；environment/repository secrets 与 variables inventory 均为 0。
  `main`/`dev` 均启用 strict required checks、admin enforcement，并禁止
  force-push/deletion。PR #43（head `94eda6d0`）的四条 required contexts、
  Tauri Linux/Windows/macOS 和 gateway coverage 全部通过，但该 PR 不执行
  tag release，不能替代真实签名验收。最新 `v6.5.0` release 早于本整改，
  只有 checksum 资产，其 artifact digest 的 attestation API 返回 404。
  没有真实 Apple/Windows/VSIX 身份或新 pipeline 签名 artifact，因此所有
  Acceptance Criteria 继续保持未勾选，本任务不得归档。
