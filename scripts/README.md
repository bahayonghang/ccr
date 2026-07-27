# CCR Scripts 目录

本目录收纳仓库级维护脚本。工作区重构后，脚本与文档都应围绕当前目录布局展开：

- 可安装 CLI crate：`crates/ccr`
- 数据库相关 crate：`crates/ccr-db`
- 共享类型 crate：`crates/ccr-types`
- UI 工程根目录：`ccr-ui`
- 文档、脚本、示例目录：根 `docs/`、`scripts/`、`examples/`
- 汇总产物目录：根 `outputs/`（如存在）

> 注意：根 `Cargo.toml` 现在是 workspace manifest，不再是 `cargo install --path` 的直接目标；本地源码安装应使用 `cargo install --path crates/ccr`。

## 脚本列表

### `version-sync.sh`

用于维护 workspace crate 与 UI 版本标识的一致性。

#### 当前应对齐的目标路径

- `crates/ccr/Cargo.toml`
- `crates/ccr-types/Cargo.toml`
- `ccr-ui/package.json`
- `ccr-ui/src-tauri/Cargo.toml`
- `ccr-ui/src-tauri/tauri.conf.json`
- `ccr-ui/src/components/MainLayout.vue`

#### 用法

```bash
# 执行同步
./scripts/version-sync.sh

# 仅检查
./scripts/version-sync.sh --check
./scripts/version-sync.sh -c

# 输出详细信息
./scripts/version-sync.sh --verbose
./scripts/version-sync.sh -v

# 组合使用
./scripts/version-sync.sh -c -v
```

### `version-sync.ps1`

`version-sync.sh` 的 PowerShell 对应版本，面向 Windows 环境。

```powershell
.\scripts\version-sync.ps1
.\scripts\version-sync.ps1 -Check
.\scripts\version-sync.ps1 -Verbose
.\scripts\version-sync.ps1 -Check -Verbose
```

### `version-sync.Tests.ps1` / `version-sync.bats`

`version-sync` 的 Windows / Bash 测试套件，与脚本本体同目录维护，便于同步更新路径和测试入口。

### `check-doc-drift.ps1` / `check-doc-drift.sh`

用于检查 `ccr-ui/README.md`、Bun lock 策略和 Tauri manifest 事实是否继续一致。该脚本已接入根 `just version-check`，会阻断以下漂移：

- `ccr-ui/package-lock.json` 回归；
- README 中的前端版本、Bun-only 策略、Rust MSRV / Edition、运行模式与 manifest 不一致；
- README 保留旧 HTTP/Axios 双模式、旧 TypeScript/Rust/Tokio 版本或旧命令数量描述。

```bash
bash scripts/check-doc-drift.sh --verbose
```

```powershell
.\scripts\check-doc-drift.ps1 -Verbose
```

### `check-dependency-drift.ps1` / `check-dependency-drift.sh`

用于比较根 `Cargo.toml` 的 `[workspace.dependencies]` 与独立 Tauri manifest `ccr-ui/src-tauri/Cargo.toml` 中重复声明的依赖版本。该脚本已接入根 `just version-check`。

治理规则：

- 相同依赖重复声明且版本一致时通过；
- 非豁免版本漂移会阻断门禁；
- 豁免项必须在脚本内带有明确原因；
- 如果豁免项对应依赖消失，或版本已经对齐但豁免未删除，脚本会视为 stale allowlist 并失败。

```bash
bash scripts/check-dependency-drift.sh --verbose
```

```powershell
.\scripts\check-dependency-drift.ps1 -Verbose
```

### `check_release_security.py`

用于验证 release workflow 的 fail-closed 签名、SBOM、OIDC provenance、集中发布
和 updater 禁用策略。`just release-security-check` 会先运行对应单测，再检查真实
workflow；该 gate 已接入 `just ci-governance-check`。

```bash
python scripts/check_release_security.py check
python scripts/check_release_security.py preflight macos
python scripts/check_release_security.py write-tauri-config windows <temp-config>
python scripts/check_release_security.py checksums <asset-root> <SHA256SUMS>
```

`preflight` 只输出缺失的 secret/variable 名称，不输出值。临时 Tauri 配置只包含
签名 identity/thumbprint/timestamp policy，不写入证书、密码或 publisher token。

## 推荐流程

```bash
# 1. 修改对应 crate/UI 的版本号来源
vim crates/ccr/Cargo.toml

# 2. 检查同步目标
./scripts/version-sync.sh --check

# 3. 必要时执行同步
./scripts/version-sync.sh
```

## CI 建议

```yaml
- name: Check version consistency
  run: |
    chmod +x ./scripts/version-sync.sh
    ./scripts/version-sync.sh --check
```

## 维护提示

- 调整版本同步逻辑时，同时检查 Bash 与 PowerShell 版本。
- 若工作区目录再次调整，优先更新本文档中的 canonical path，再同步更新脚本实现。
- 若流水线产出需要落盘，统一收集到根 `outputs/`，避免散落在子目录。
- 新增/删除同步目标时，在两个脚本的 `SYNC_TARGETS` 配置表中同步修改。

## 未来展望：CLI 集成方案

当前版本同步逻辑由独立的 Bash/PowerShell 脚本实现。未来可考虑将其集成到 `crates/ccr` CLI 中，获得以下优势：

### 潜在优势

1. **单一实现**：无需维护两套脚本，避免跨平台逻辑 diverge
2. **类型安全**：Rust 的类型系统可在编译期捕获路径/格式错误
3. **更好的错误提示**：使用 `clap` 提供子命令帮助、参数验证、彩色输出
4. **与现有 CLI 集成**：例如 `ccr version sync`、`ccr version check`

### 设计草图

```rust
// crates/ccr/src/cli/version.rs
#[derive(Subcommand)]
pub enum VersionCommand {
    /// 检查版本一致性
    Check { verbose: bool },
    /// 同步版本到所有目标
    Sync { dry_run: bool },
}
```

### 迁移路径

1. 先在 Rust 中实现核心逻辑（文件解析、版本比较、原子写入）
2. 保留现有脚本作为轻包装器，调用 `ccr version --legacy`
3. 验证通过后，逐步弃用脚本，转向 CLI 子命令

### 当前建议

- 保持现有脚本用于 CI/CD 和日常使用
- 重大同步逻辑修改时，评估是否值得 Rust 重构
- 若 CLI 已有类似功能，优先复用 CLI 而非重复实现
