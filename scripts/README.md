# CCR Scripts 目录

本目录包含用于 CCR 项目开发和维护的辅助脚本工具。

## 脚本列表

### 1. version-sync.sh

**版本同步脚本（Bash 版）** - 将根 `Cargo.toml` 的版本号同步到所有子项目。

#### 同步目标

- `ccr-ui/backend/Cargo.toml`
- `ccr-ui/package.json`
- `ccr-ui/src-tauri/Cargo.toml`
- `ccr-ui/src-tauri/tauri.conf.json`
- `ccr-ui/src/components/MainLayout.vue`（侧边栏版本标识）
- `ccr-ui/src/layouts/MainLayout.vue`（侧边栏版本标识）

#### 使用方法

```bash
# 直接运行同步
./scripts/version-sync.sh

# 仅检查版本一致性，不执行同步
./scripts/version-sync.sh --check
# 或
./scripts/version-sync.sh -c

# 显示详细输出
./scripts/version-sync.sh --verbose
# 或
./scripts/version-sync.sh -v

# 组合使用
./scripts/version-sync.sh -c -v
```

#### 参数说明

| 参数 | 说明 |
|------|------|
| `--check`, `-c` | 仅检查版本一致性，不执行同步 |
| `--verbose`, `-v` | 显示详细输出 |

#### 示例输出

```bash
$ ./scripts/version-sync.sh

♻️  开始同步版本到 UI 文件...
  - 后端: 3.6.1 -> 3.6.2
  - 前端: 3.6.1 -> 3.6.2
  - Tauri Cargo.toml: 3.6.1 -> 3.6.2
  - Tauri tauri.conf.json: 3.6.1 -> 3.6.2
  - 前端 MainLayout (components): 3.6.1 -> 3.6.2
  - 前端 MainLayout (layouts): 3.6.1 -> 3.6.2
✅ 同步完成
```

```bash
$ ./scripts/version-sync.sh --check

✅ 版本一致性检查通过
```

#### 使用场景

1. **发布新版本前**：确保所有组件版本号一致
2. **CI/CD 集成**：在构建前检查版本一致性
3. **开发过程中**：根版本号变更后同步到所有子项目

---

### 2. version-sync.ps1

**版本同步脚本（PowerShell 版）** - Windows 平台使用的版本同步工具。

#### 功能特性

与 `version-sync.sh` 功能完全相同，但适配 Windows PowerShell 环境。

#### 使用方法

```powershell
# PowerShell 中直接运行
.\scripts\version-sync.ps1

# 仅检查版本一致性
.\scripts\version-sync.ps1 -Check

# 显示详细输出
.\scripts\version-sync.ps1 -Verbose

# 组合使用
.\scripts\version-sync.ps1 -Check -Verbose
```

#### 参数说明

| 参数 | 说明 |
|------|------|
| `-Check` | 仅检查版本一致性，不执行同步 |
| `-Verbose` | 显示详细输出 |

---

## 使用建议

### 日常开发流程

**修改版本号后**：
```bash
# 编辑根 Cargo.toml 更新版本号
vim Cargo.toml

# 同步到所有子项目
./scripts/version-sync.sh

# 提交更改
git add -A && git commit -m "chore: bump version to x.y.z"
```

### CI/CD 集成

在 GitHub Actions 或 GitLab CI 中检查版本一致性：

```yaml
# .github/workflows/ci.yml
- name: Check version consistency
  run: |
    chmod +x ./scripts/version-sync.sh
    ./scripts/version-sync.sh --check
```

---

## 技术实现细节

### version-sync.sh 实现要点

- **版本提取**：使用 `awk` 提取 `[package]` 区块，再用 `sed` 精确匹配 `version = "..."`
- **原子操作**：使用 `mktemp` 创建临时文件，确保更新失败不会破坏原文件
- **多格式支持**：同时支持 TOML（Cargo.toml）和 JSON（package.json, tauri.conf.json）
- **容错处理**：在缺少 `jq` 时自动降级为 `sed` 进行 JSON 解析

---

## 常见问题

### Q: version-sync.sh 是否支持 monorepo？
A: 当前版本针对 CCR 项目结构优化，但可以通过修改脚本适配其他 monorepo 项目。

### Q: version-sync.sh 是否支持 Windows？
A: Windows 用户请使用 `version-sync.ps1` PowerShell 版本。

---

## 许可证

MIT License（与 CCR 主项目一致）

---

**最后更新**: 2026-03-02
**维护者**: CCR 开发团队
