# 故障排除指南

> 本页汇总 CCR 7.x 多平台配置库（`profiles.db`）、运行时路由与宿主权限的常见排查路径。首次上手请先阅读 [快速开始](/guide/quick-start)。

## 常见问题与解决方案

### 1. 配置库未初始化或提示 Profile 未找到

**现象**：切换或查看 Profile 时提示配置库不存在或目标名称未找到。

**解决方案**：

```bash
# 1. 初始化对应平台的 SQLite 配置库（如尚未初始化）
ccr claude profile init
ccr codex profile init
ccr grok profile init

# 2. 查看目标平台下已存在的 Profile 列表
ccr claude profile list
ccr codex profile list
ccr grok profile list

# 3. 通过 stdin 安全创建缺失的 Profile
printf '%s' "$ANTHROPIC_AUTH_TOKEN" | ccr claude profile create work \
  --base-url https://api.example.com \
  --auth-token-stdin
```

---

### 2. 手工修改或复制 `profiles.toml` 后未生效

**原因**：CCR 7.x 的权威活体存储已迁移至 `~/.ccr/platforms/<platform>/profiles.db`（SQLite）。手工创建或把备份 `profiles.toml` 复制回平台目录不会被 7.x 二进制读取。

**解决方案**：

```bash
# 方案 A：通过编辑器投影直接修改活体配置库
ccr claude profile open

# 方案 B：将外部 ExchangeV1 TOML 导入配置库（先 --dry-run 预演）
ccr claude profile import ./my-profiles.toml --dry-run
ccr claude profile import ./my-profiles.toml --on-conflict overwrite

# 修改后重新应用并校验
ccr claude profile switch <name>
ccr validate
```

---

### 3. 运行时状态不一致或残留 Codex `app-server` 进程

**现象**：切换 Profile 后宿主 CLI 仍读取旧路由、环境变量覆盖了宿主设置，或 Codex 后台残留旧 `app-server`。

**解决方案**：

```bash
# 1. 查看运行时总览与统一诊断报告
ccr current --verbose
ccr validate
ccr doctor --verbose

# 2. 检查当前 Shell 是否残留覆盖宿主设置的环境变量
env | grep -E 'ANTHROPIC_|OPENAI_|XAI_|CODEX_'

# 3. 针对 Codex：清理残留 app-server 并按需重放当前保存的 Profile
ccr codex fix
ccr codex fix --repair-runtime

# 4. 如需退出 Profile 模式回到官方登录态
ccr claude profile off
ccr codex profile off
ccr grok profile off
```

---

### 4. API 连通性失败或密钥失效

**现象**：宿主 CLI 报错连接超时、`401 Unauthorized` 或 `INVALID_API_KEY`。

**解决方案**：

```bash
# 1. 对当前激活 Profile 执行在线探活或定向测试
ccr doctor --online
ccr provider test <name> --verbose

# 2. 通过标准输入安全轮换密钥并重新应用
printf '%s' "$NEW_AUTH_TOKEN" | ccr claude profile edit <name> --auth-token-stdin
ccr claude profile switch <name>
```

> Grok 平台更新密钥时，请使用 `printf '%s' "$NEW_KEY" | ccr grok profile edit <name> --api-key-stdin` 或 `--set env_key=XAI_API_KEY`。

---

### 5. 目录权限错误或残留文件锁超时

**现象**：提示 `Permission denied` 或等待文件锁超时。

**解决方案**：

```bash
# 1. 修复配置目录与宿主设置文件权限
chmod 755 ~/.ccr ~/.ccr/platforms
chmod 600 ~/.claude/settings.json

# 2. 确认无正在运行的 CCR 进程后，清理陈旧锁文件
ls -la ~/.claude/.locks/
rm -rf ~/.claude/.locks/*
```

---

### 6. 跨设备或跨用户迁移配置

**原因**：CCR 默认使用当前用户主目录下的 `~/.ccr/`。不同机器或系统用户之间建议使用 `ExchangeV1` 导出/导入或 WebDAV 同步，而非手工拷贝数据库文件。

**解决方案**：

```bash
# 在源环境导出（分享给他人时务必加 --no-secrets）
ccr claude profile export --no-secrets -o /tmp/claude-profiles.toml

# 在目标环境预演并导入
ccr claude profile import /tmp/claude-profiles.toml --dry-run
ccr claude profile import /tmp/claude-profiles.toml --on-conflict skip
```

## 调试日志与问题反馈

```bash
# 单次开启 debug 级别日志执行诊断
CCR_LOG_LEVEL=debug ccr doctor --all-platforms --verbose

# 查看按 UTC 日期轮转的日志文件（默认保留 14 天）
ls -lh ~/.ccr/logs/

# 收集版本摘要供 Issue 反馈（请勿附上含明文密钥的宿主设置或备份文件）
ccr --version
ccr version
```

## 相关文档

- [配置示例索引](./index)
- [多平台配置指南](./multi-platform-setup)
- [快速开始](/guide/quick-start)
- [配置模型](/guide/configuration)
- [平台支持](/reference/platforms/)
