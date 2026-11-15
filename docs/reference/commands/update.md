# update - 更新 CCR

从官方 GitHub 仓库编译并更新已安装的 `ccr` 二进制文件，支持分支选择、更新预览与实时编译进度输出。

## 用法

```bash
ccr update [OPTIONS] [BRANCH]
```

## 参数与选项

- `[BRANCH]`：目标 Git 分支（默认：`main`，可选：`dev`）
- `-c, --check`：仅预览将要执行的 `cargo install` 命令，不执行实际编译与覆盖安装

## 系统要求

- 已安装 Rust 1.90+ 与 Cargo 工具链（在 `PATH` 中可用）
- 已安装 `git` 且网络可访问 `https://github.com/bahayonghang/ccr`

底层等效安装命令：

```bash
cargo install --git https://github.com/bahayonghang/ccr ccr --force
```

> 若你在本地仓库工作区中源码构建安装，请运行 `cargo install --path crates/ccr`。

## 示例

### 1. 预览更新命令（推荐先执行）

```bash
ccr update --check
ccr update -c
```

### 2. 从默认稳定分支（`main`）更新

```bash
ccr update
```

### 3. 从开发分支（`dev`）更新

```bash
ccr update --check dev
ccr update dev
```

> [!WARNING]
> `dev` 分支包含在途开发变更，稳定性可能不及 `main`，建议仅在测试或验证新特性时使用。

## 推荐升级闭环

更新可执行文件不会删除 `~/.ccr/` 下的 `config.toml` 或各平台 `profiles.db`，但在跨大版本升级前建议先导出配置快照，并在升级后运行校验与体检：

```bash
# 1. 导出平台配置快照（可选；默认含内联密钥并按 0600 权限落盘，外发共享或入库请加 --no-secrets）
ccr claude profile export -o ~/backups/claude-$(date +%Y%m%d).toml
ccr codex profile export -o ~/backups/codex-$(date +%Y%m%d).toml

# 2. 执行更新
ccr update

# 3. 验证新版本与配置健康度
ccr --version
ccr current
ccr validate
ccr doctor
```

> 注：未传 `--no-secrets` 时，`profile export` 默认包含内联密钥（以仅属主可读写的 `0600` 权限写入文件），仅用于本机私有备份；如需分享或提交至版本库，请附加 `--no-secrets`。

## 故障排查与版本回退

### 常见问题

- **提示找不到 `cargo`**：确认已通过 `rustup` 安装工具链并加载 `$HOME/.cargo/env`。
- **编译失败**：先执行 `rustup update stable` 升级到最新稳定版 Rust 工具链后重试。
- **更新后 `ccr --version` 仍显示旧版本**：使用 `which ccr` 检查 `PATH` 中是否存在多份二进制，确认 `~/.cargo/bin` 优先级正确。

### 安装指定标签或提交以回退版本

```bash
# 安装指定 Release 标签
cargo install --git https://github.com/bahayonghang/ccr ccr --tag <tag> --force

# 或安装指定 Commit
cargo install --git https://github.com/bahayonghang/ccr ccr --rev <commit-sha> --force
```

## 相关命令

- [`version`](./version) - 查看详细版本摘要或简短版本号
- [`validate`](./validate) - 校验注册表与三平台配置
- [`doctor`](./doctor) - 运行统一环境体检
- [`claude`](./claude) / [`codex`](./codex) / [`grok`](./grok) - 导出或导入平台配置库
