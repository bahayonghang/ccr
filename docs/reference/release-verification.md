# Release 签名与来源验证

正式 release 只发布已完成平台签名、VSIX publisher 签名和 GitHub build
provenance 的产物。`SHA256SUMS` 只证明传输完整性，不能替代发布者身份验证。

## 下载后验证

先验证 GitHub attestation 与集中 checksum：

```bash
gh attestation verify <artifact> --repo bahayonghang/ccr
sha256sum -c SHA256SUMS --ignore-missing
```

macOS 应同时通过代码签名、Gatekeeper 与 notarization ticket：

```bash
codesign --verify --deep --strict --verbose=2 "CCR Desktop.app"
spctl --assess --type execute --verbose=2 "CCR Desktop.app"
xcrun stapler validate "CCR Desktop.app"
xcrun stapler validate CCR_Desktop.dmg
```

Windows 应使用系统信任链和 RFC3161 时间戳验证可执行文件与安装包：

```powershell
signtool verify /pa /all /v .\ccr.exe
signtool verify /pa /all /v .\CCR_Desktop.msi
Get-AuthenticodeSignature .\ccr.exe
```

VSIX 必须和同一 release 中的 manifest、`.p7s` 一起验证：

```bash
npx --yes @vscode/vsce@3.7.1 verify-signature \
  -i ccr-vscode-<version>.vsix \
  -m extension.signature.manifest \
  -s extension.signature.p7s
```

任何一步失败都应停止安装或更新，并保留当前已安装版本。

## 受保护 release 环境

仓库 workflow 只引用以下 secret/variable 名称，不保存值：

| 平台 | Secret / variable |
| --- | --- |
| Apple | `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` |
| Windows | `WINDOWS_CERTIFICATE_BASE64`, `WINDOWS_CERTIFICATE_PASSWORD`, `WINDOWS_CERTIFICATE_THUMBPRINT`, `WINDOWS_TIMESTAMP_URL` (variable) |
| VSIX | `VSCE_PAT`, `VSIX_SIGN_TOOL_PATH` (variable) |

VSIX job 只在带 `vsix-signing` label 的受控 self-hosted Linux runner 上运行。
`VSIX_SIGN_TOOL_PATH` 必须指向由 publisher 管理的可执行 sign-tool。签名、验证、
SBOM、OIDC attestation 或 Marketplace 发布任一失败时，GitHub Release job 不会运行。

## Updater 状态

当前桌面应用没有启用 Tauri updater。`just release-security-check` 会阻止在没有
“签名 manifest + provenance 验证失败时保持原版本”回归测试的情况下加入 updater
依赖或配置。开发包不能复用正式 release 名称，也不能成为 updater 输入。

## 轮换、吊销与失败回滚

身份轮换时，在受保护 environment 更新 secret/variable，保留旧公钥或证书链以验证
历史 release，并用新身份先完成隔离 release 验证。身份疑似泄露时立即禁用 environment、
吊销证书或 publisher token，并停止 tag 发布。

失败的 release 不得改成同版本 unsigned 产物，也不得覆盖原 tag。修复后使用新版本和
新 tag；若 Marketplace 已发布而 GitHub Release 失败，应记录 partial publication，停止
updater 元数据并完成 GitHub 侧验证后再恢复发布。
