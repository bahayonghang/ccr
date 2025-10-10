# version - 显示版本信息

显示 CCR 的版本信息。

## 用法

```bash
ccr version
# 或使用别名
ccr ver
# 或使用标准选项
ccr --version
ccr -V
```

## 示例输出

```bash
$ ccr version
CCR (Claude Code Configuration Router) v0.3.1

Build Information:
  Commit: a1b2c3d
  Build Date: 2025-01-10
  Rust Version: 1.75.0

Repository: https://github.com/bahayonghang/ccr
License: MIT
```

## 输出信息

- **版本号**: 当前 CCR 版本
- **Commit**: Git commit 哈希
- **Build Date**: 编译日期
- **Rust Version**: 编译使用的 Rust 版本
- **Repository**: GitHub 仓库地址
- **License**: 开源协议

## 使用场景

### 1. 检查当前版本

确认使用的 CCR 版本：

```bash
ccr version
```

### 2. 问题报告

提交 issue 时提供版本信息：

```bash
$ ccr version
CCR v0.3.1
```

### 3. 验证安装

安装或更新后验证：

```bash
# 安装后
cargo install --git https://github.com/bahayonghang/ccr
ccr version

# 更新后
ccr update
ccr version
```

### 4. 脚本中检查版本

```bash
#!/bin/bash
VERSION=$(ccr version | head -n1 | awk '{print $NF}')
echo "Current CCR version: $VERSION"

# 版本比较
if [[ "$VERSION" < "v0.4.0" ]]; then
  echo "Please update CCR"
  ccr update
fi
```

### 5. 环境验证

在部署流程中验证版本：

```bash
# 检查是否安装
if ! command -v ccr &> /dev/null; then
  echo "CCR not installed"
  exit 1
fi

# 检查版本
ccr version
```

## 版本号格式

CCR 使用语义化版本号：

```
v<major>.<minor>.<patch>
```

示例：`v0.3.1`
- `0`: 主版本号（不兼容的 API 变化）
- `3`: 次版本号（向后兼容的新功能）
- `1`: 修订号（向后兼容的问题修复）

## 版本历史

查看完整的版本历史：

```bash
# 查看 GitHub Releases
https://github.com/bahayonghang/ccr/releases

# 查看更新日志
cat /path/to/ccr/CHANGELOG.md
```

## 构建信息

详细的构建信息包括：

### Commit Hash

Git commit 的短哈希，用于追踪具体的代码版本：

```bash
$ ccr version | grep Commit
Commit: a1b2c3d
```

### Build Date

二进制文件的编译日期：

```bash
$ ccr version | grep "Build Date"
Build Date: 2025-01-10
```

### Rust Version

编译时使用的 Rust 版本：

```bash
$ ccr version | grep "Rust Version"
Rust Version: 1.75.0
```

## 脚本集成

### Bash 脚本

```bash
#!/bin/bash

# 获取版本号
VERSION=$(ccr version | head -n1 | awk '{print $NF}')
echo "CCR Version: $VERSION"

# 提取主版本号
MAJOR=$(echo $VERSION | cut -d'.' -f1 | tr -d 'v')
MINOR=$(echo $VERSION | cut -d'.' -f2)
PATCH=$(echo $VERSION | cut -d'.' -f3)

echo "Major: $MAJOR, Minor: $MINOR, Patch: $PATCH"
```

### Python 脚本

```python
#!/usr/bin/env python3
import subprocess
import re

# 获取版本
result = subprocess.run(['ccr', 'version'], capture_output=True, text=True)
version_line = result.stdout.split('\n')[0]

# 提取版本号
match = re.search(r'v(\d+\.\d+\.\d+)', version_line)
if match:
    version = match.group(1)
    print(f"CCR Version: {version}")

    major, minor, patch = version.split('.')
    print(f"Major: {major}, Minor: {minor}, Patch: {patch}")
```

## 版本比较

### 检查最小版本

```bash
#!/bin/bash

REQUIRED="0.3.0"
CURRENT=$(ccr version | head -n1 | awk '{print $NF}' | tr -d 'v')

if [ "$(printf '%s\n' "$REQUIRED" "$CURRENT" | sort -V | head -n1)" = "$REQUIRED" ]; then
  echo "CCR version OK: $CURRENT"
else
  echo "CCR version too old: $CURRENT (required: $REQUIRED)"
  exit 1
fi
```

### 检查特定版本范围

```bash
#!/bin/bash

MIN_VERSION="0.3.0"
MAX_VERSION="0.5.0"
CURRENT=$(ccr version | head -n1 | awk '{print $NF}' | tr -d 'v')

if [[ "$CURRENT" > "$MIN_VERSION" && "$CURRENT" < "$MAX_VERSION" ]]; then
  echo "CCR version in range: $CURRENT"
else
  echo "CCR version out of range: $CURRENT"
fi
```

## 从源码构建时的版本

如果从源码构建，版本信息来自：

1. **Cargo.toml**: 版本号定义
   ```toml
   [package]
   version = "0.3.1"
   ```

2. **Git Tags**: 发布标签
   ```bash
   git tag -a v0.3.1 -m "Release v0.3.1"
   ```

3. **Build Script**: 构建时嵌入信息
   ```rust
   // build.rs
   println!("cargo:rustc-env=GIT_HASH={}", git_hash);
   println!("cargo:rustc-env=BUILD_DATE={}", build_date);
   ```

## 诊断信息

版本命令也可用于诊断：

```bash
# 检查二进制位置
which ccr

# 检查版本
ccr version

# 检查是否最新
ccr update --check

# 完整诊断
echo "=== CCR Diagnostic Info ==="
echo "Binary: $(which ccr)"
echo "Version: $(ccr version | head -n1)"
echo "Config: ~/.ccs_config.toml"
echo "Settings: ~/.claude/settings.json"
ccr validate
```

## 问题报告模板

提交 issue 时包含版本信息：

```markdown
### Environment

- CCR Version: [output of `ccr version`]
- OS: [e.g., Ubuntu 22.04, macOS 14.0]
- Rust Version: [output of `rustc --version`]

### Description

[Describe your issue here]

### Steps to Reproduce

1. [Step 1]
2. [Step 2]
...

### Expected Behavior

[What you expected]

### Actual Behavior

[What actually happened]
```

## CI/CD 集成

### GitHub Actions

```yaml
- name: Check CCR Version
  run: |
    ccr version
    echo "CCR_VERSION=$(ccr version | head -n1 | awk '{print $NF}')" >> $GITHUB_ENV

- name: Verify Version
  run: |
    echo "Using CCR version: $CCR_VERSION"
```

### GitLab CI

```yaml
check_version:
  script:
    - ccr version
    - export CCR_VERSION=$(ccr version | head -n1 | awk '{print $NF}')
    - echo "CCR Version: $CCR_VERSION"
```

## 相关命令

- [update](./update) - 更新到最新版本
- [init](./init) - 查看工具能力
- [validate](./validate) - 验证功能正常
