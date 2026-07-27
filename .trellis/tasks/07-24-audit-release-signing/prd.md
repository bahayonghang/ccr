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
- 2026-07-26 现场盘点：当前 `gh` 凭据调用 Actions secrets inventory 返回 HTTP 403，仓库内也没有可验证的 Apple/Windows/VSIX 签名身份；因此只能先证明 workflow、校验脚本和文档的仓库侧闭环，不能把真实签名 artifact 验证宣称为已通过
- 优先级 P2，可在 P1 发版阻断组完成后进行

## Key Decision

- 2026-07-26 用户选择严格端到端验收：本子任务和父任务必须保持未完成，直到提供相应证书、publisher 权限和可读写 Actions secrets 的 GitHub 凭据，并验证真实签名 artifact 与 provenance；仓库侧 workflow/fixture 通过不构成归档条件。

## Verification checkpoint (2026-07-27)

- 仓库侧已实现受保护 `release` environment 的 fail-closed DAG：CLI/Tauri
  平台签名、VSIX sign-tool + Marketplace 发布、集中 SBOM/checksum、GitHub
  OIDC provenance，且 GitHub Release 只在前述步骤全部成功后创建。
- `just release-security-check`、`actionlint`、`just ci-governance-check`、
  `just vscode-ci`、`just ui-check` 与中英文 docs build/audit 已通过。
- `just version-check` 被不属于本任务的并行 `7.0.0` doc-drift 阻塞；未修改
  或暂存对应版本元数据。
- 远程 `release` environment 查询为 HTTP 404，仓库 secrets inventory 为
  HTTP 403。没有真实 Apple/Windows/VSIX 身份或签名 artifact，因此所有
  Acceptance Criteria 继续保持未勾选，本任务不得归档。
