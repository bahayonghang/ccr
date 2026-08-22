# version - 查看版本信息

显示当前安装的 CCR 版本信息。

## 用法

```bash
# 详细版本信息
ccr version

# 简短版本号
ccr --version
ccr -V
```

## 两种入口的区别

### `ccr version`

给人看。

会输出：

- 当前版本号
- 作者
- 包描述
- 常用帮助入口
- 核心任务入口

适合手工确认当前安装状态。

### `ccr --version` / `ccr -V`

给脚本和 CI 用。

只输出一行简短版本号，例如：

```bash
$ ccr --version
ccr 5.9.4
```

适合：

- Shell 脚本
- CI 环境变量
- 自动化日志

## 常见场景

### 1. 手工确认当前安装

```bash
ccr version
```

### 2. 脚本里读取版本号

```bash
VERSION=$(ccr --version | awk '{print $2}')
echo "Current CCR version: $VERSION"
```

### 3. 更新后验证

```bash
ccr update
ccr --version
ccr --help
```

### 4. Issue / 排障信息收集

```bash
ccr version
ccr --version
```

建议同时附上 `ccr --help` 或相关子命令帮助页，便于确认当前命令面。

## 相关命令

- [update](./update) - 更新 CCR
- [platform](./platform) - 管理平台
- [codex](./codex) - 管理 Codex 多账号
- [grok](./grok) - 管理 Grok Build profile 与官方会话登出
