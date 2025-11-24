# CCR Scripts 目录

本目录包含用于 CCR 项目开发和维护的辅助脚本工具。

## 脚本列表

### 1. git-commit.sh

**Git 提交助手脚本** - 自动生成符合 [Conventional Commits](https://www.conventionalcommits.org/) 规范的提交信息。

#### 功能特性
- ✅ 自动检测仓库状态（避免在 merge/rebase/detached HEAD 状态下提交）
- ✅ 智能推断提交类型（feat/fix/docs/test/refactor/chore/ci）
- ✅ 自动推断作用域（scope）（如 ui/web/core/docs）
- ✅ 检测项目语言（中文/英文）并生成对应语言的提交信息
- ✅ 基于变更内容智能分组建议（docs/tests/code/config）
- ✅ 支持 emoji 前缀（可选）
- ✅ 纯 Git 实现，不依赖构建工具

#### 使用方法

```bash
# 基本用法（交互式）
./scripts/git-commit.sh

# 自动暂存所有更改并提交
./scripts/git-commit.sh --all

# 添加 emoji 前缀
./scripts/git-commit.sh --emoji

# 跳过 git hooks
./scripts/git-commit.sh --no-verify

# 修改上一次提交
./scripts/git-commit.sh --amend

# 添加 Signed-off-by
./scripts/git-commit.sh --signoff

# 明确指定类型和作用域
./scripts/git-commit.sh --type feat --scope ui --emoji

# 组合使用
./scripts/git-commit.sh --all --emoji --signoff
```

#### 参数说明

| 参数 | 说明 |
|------|------|
| `--no-verify` | 跳过 git hooks（传递 --no-verify 给 git commit） |
| `--all` | 无暂存更改时，自动执行 `git add -A` 暂存所有更改 |
| `--amend` | 修改上一次提交而非创建新提交 |
| `--signoff` | 添加 Signed-off-by 行（传递 -s 给 git commit） |
| `--emoji` | 在提交类型前添加 emoji（✨, 🐛, 📝 等） |
| `--scope <scope>` | 明确指定提交作用域（如 ui, web, backend） |
| `--type <type>` | 明确指定提交类型（feat, fix, docs, refactor, test...） |
| `-h, --help` | 显示帮助信息 |

#### 提交类型推断规则

脚本根据暂存的文件自动推断提交类型：

- **docs**: 仅文档文件变更（README*, *.md, docs/*）
- **test**: 仅测试文件变更（tests/*, *_test.rs, *test.js 等）
- **ci**: 仅 CI/工作流文件变更（.github/workflows/*, .gitlab-ci.yml）
- **chore**: 仅配置/脚本文件变更（*.toml, *.json, justfile, scripts/*）
- **fix**: 源代码变更且 diff 中包含 "fix"/"bug"/"error"/"panic"
- **feat**: 其他源代码变更

#### 作用域推断规则

根据文件路径自动推断：
- `ccr-ui/frontend/*` → `ui`
- `ccr-ui/backend/*` → `web`
- `src/*` → `core`
- `tests/*` → `tests`
- `docs/*` → `docs`
- 其他 → 顶级目录名或 `repo`

#### 语言检测

脚本自动检测项目主要语言：
- 如果最近 50 条提交信息包含非 ASCII 字符 → 中文
- 否则 → 英文

#### 示例输出

```bash
$ ./scripts/git-commit.sh --all --emoji

Staged files:
  ccr-ui/backend/src/handlers/mcp.rs
  ccr-ui/frontend/src/views/McpManager.vue

Split suggestions (by concern):

  - code:
      ccr-ui/backend/src/handlers/mcp.rs
      ccr-ui/frontend/src/views/McpManager.vue

Draft commit message written to .git/COMMIT_EDITMSG:
-----------------------------------------------------------------
✨ web: 添加 MCP 服务器管理功能

- 变更类型: feat
- 影响范围: web
- 涉及文件:
  - ccr-ui/backend/src/handlers/mcp.rs
  - ccr-ui/frontend/src/views/McpManager.vue

注: 提交信息由 git-commit 辅助脚本自动生成, 如有需要可手动编辑。
-----------------------------------------------------------------
Running: git commit -F .git/COMMIT_EDITMSG
[dev a1b2c3d] ✨ web: 添加 MCP 服务器管理功能
 2 files changed, 150 insertions(+), 20 deletions(-)
```

---

### 2. version-sync.sh

**版本同步脚本（Bash 版）** - 将根 `Cargo.toml` 的版本号同步到所有子项目。

#### 同步目标

- `ccr-ui/backend/Cargo.toml`
- `ccr-ui/frontend/package.json`
- `ccr-ui/frontend/src-tauri/Cargo.toml`
- `ccr-ui/frontend/src-tauri/tauri.conf.json`
- `ccr-ui/frontend/src/components/MainLayout.vue`（侧边栏版本标识）
- `ccr-ui/frontend/src/layouts/MainLayout.vue`（侧边栏版本标识）

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

### 3. version-sync.ps1

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

1. **修改代码后提交**：
   ```bash
   # 方式 1：手动暂存 + 提交
   git add <files>
   ./scripts/git-commit.sh

   # 方式 2：自动暂存所有更改
   ./scripts/git-commit.sh --all
   ```

2. **修改版本号后**：
   ```bash
   # 编辑根 Cargo.toml 更新版本号
   vim Cargo.toml

   # 同步到所有子项目
   ./scripts/version-sync.sh

   # 提交更改
   ./scripts/git-commit.sh --all --type chore --scope repo
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

### git-commit.sh 实现要点

- **纯 Git 实现**：仅使用 `git` 和 POSIX 标准工具（`awk`, `sed`, `grep`）
- **状态检测**：通过检查 `.git/MERGE_HEAD`, `.git/rebase-apply` 等文件和目录判断仓库状态
- **文件分类**：使用 `case` 语句和路径匹配规则对文件进行智能分类
- **语言检测**：通过检查最近提交信息中的 Unicode 字符判断项目主要语言
- **消息生成**：自动生成标题和正文，写入 `.git/COMMIT_EDITMSG` 后调用 `git commit`

### version-sync.sh 实现要点

- **版本提取**：使用 `awk` 提取 `[package]` 区块，再用 `sed` 精确匹配 `version = "..."`
- **原子操作**：使用 `mktemp` 创建临时文件，确保更新失败不会破坏原文件
- **多格式支持**：同时支持 TOML（Cargo.toml）和 JSON（package.json, tauri.conf.json）
- **容错处理**：在缺少 `jq` 时自动降级为 `sed` 进行 JSON 解析

---

## 常见问题

### Q: git-commit.sh 是否支持 Windows？
A: 需要 Git Bash 或 WSL 环境。推荐使用 version-sync.ps1 进行版本同步。

### Q: 为什么 git-commit.sh 不自动运行测试？
A: 设计原则是仅负责提交信息生成，构建和测试应由 pre-commit hooks 或 CI 负责。

### Q: version-sync.sh 是否支持 monorepo？
A: 当前版本针对 CCR 项目结构优化，但可以通过修改脚本适配其他 monorepo 项目。

### Q: 如何自定义 emoji 映射？
A: 编辑 `git-commit.sh` 中的 `emoji_for_type()` 函数即可。

### Q: 如何添加新的文件分类规则？
A: 编辑 `git-commit.sh` 中的以下函数：
- `is_docs_file()` - 文档文件
- `is_test_file()` - 测试文件
- `is_ci_file()` - CI 文件
- `is_config_or_script_file()` - 配置/脚本文件

---

## 许可证

MIT License（与 CCR 主项目一致）

---

**最后更新**: 2025-01-24
**维护者**: CCR 开发团队
