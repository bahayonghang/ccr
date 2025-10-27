# sync 命令 - WebDAV 配置同步

## 📋 概述

`sync` 命令提供基于 WebDAV 协议的配置文件云端同步功能，支持两种同步模式：

**同步模式：**
- 📁 **目录同步（统一模式）** - 同步整个 `~/.ccr/` 目录及所有平台配置（推荐）
- 📄 **文件同步（传统模式）** - 同步单个 `~/.ccs_config.toml` 文件（向后兼容）

CCR 会根据您的配置结构自动检测使用哪种模式。默认支持坚果云，也兼容其他标准 WebDAV 服务。

## 🎯 功能特性

- ☁️ **WebDAV 同步** - 支持将配置文件或目录同步到 WebDAV 服务器
- 📁 **递归目录同步** - 自动上传/下载整个目录树及所有子目录
- 🚫 **智能文件过滤** - 自动排除临时文件（.bak, .tmp, .lock）和系统文件（.DS_Store）
- 🔒 **安全认证** - 支持基本认证（Basic Auth）
- 🔄 **双向同步** - 支持上传（push）和下载（pull）
- 🌰 **坚果云优化** - 默认配置适配坚果云，开箱即用
- ✅ **状态检查** - 实时查看同步配置和远程文件状态
- 🔄 **自动模式检测** - 根据配置结构自动选择目录或文件同步模式

## 📚 子命令列表

### sync config - 配置同步

交互式配置 WebDAV 连接信息。

```bash
ccr sync config
```

**配置项说明：**

| 配置项 | 说明 | 默认值 | 示例 |
|--------|------|--------|------|
| WebDAV 服务器地址 | WebDAV 服务的 URL | `https://dav.jianguoyun.com/dav/` | - |
| 用户名/邮箱 | 登录账号 | - | `user@example.com` |
| 密码/应用密码 | 认证密码 | - | 坚果云应用密码 |
| 远程路径 | 配置目录或文件在服务器上的路径 | `/ccr/` | `/backup/ccr/` 或 `/backup/config.toml` |

**坚果云应用密码获取方式：**

1. 登录坚果云网页版
2. 点击右上角头像 → 账户信息
3. 进入"安全选项"
4. 点击"添加应用"
5. 生成应用密码（请妥善保存）

**配置示例：**

```bash
$ ccr sync config

配置 WebDAV 同步
═══════════════════

请输入 WebDAV 服务器信息
💡 坚果云用户请使用应用密码，而非账户密码

ℹ WebDAV 服务器地址:
  默认: https://dav.jianguoyun.com/dav/
  请输入: [直接回车使用默认值]

ℹ 用户名/邮箱 *
  例如: user@example.com
  请输入: myemail@example.com

ℹ 密码/应用密码:
  💡 坚果云: 账户信息 -> 安全选项 -> 添加应用 -> 生成密码
  请输入: ****************

ℹ 远程路径
  默认: /ccr/
  请输入: [直接回车使用默认值]

─────────────────────────────────────────────────────────

▶ 测试 WebDAV 连接

✓ WebDAV 连接测试成功

▶ 保存同步配置
✓ 同步配置已保存

可用命令:
  ccr sync status    # 查看同步状态
  ccr sync push      # 上传配置到云端
  ccr sync pull      # 从云端下载配置
```

---

### sync status - 查看同步状态

显示当前同步配置和远程文件状态。

```bash
ccr sync status
```

**输出示例：**

```bash
同步状态
═══════════

  状态: 已启用
  同步模式: 目录同步（统一模式）
  WebDAV 服务器: https://dav.jianguoyun.com/dav/
  用户名: user@example.com
  远程路径: /ccr/
  本地路径: ~/.ccr/
  自动同步: 关闭

▶ 检查远程内容
✓ 远程配置目录存在
```

---

### sync push - 上传配置到云端

将本地配置文件或目录上传到 WebDAV 服务器。

```bash
ccr sync push [--force]
```

**参数说明：**

- `--force`, `-f` - 强制覆盖远程配置，跳过确认提示

**使用示例：**

```bash
# 普通上传（会提示确认）
ccr sync push

# 强制上传（不提示确认）
ccr sync push --force
```

**执行流程：**

1. **自动检测同步模式**：
   - 如果是统一模式：递归上传 `~/.ccr/` 目录及所有子目录
   - 如果是传统模式：上传单个 `~/.ccs_config.toml` 文件

2. **智能文件过滤**：
   - 自动排除临时文件（.bak, .tmp, .lock）
   - 排除系统文件（.DS_Store, Thumbs.db, desktop.ini）
   - 排除版本控制目录（.git, .gitignore）
   - 排除锁文件目录（.locks）
   - 排除备份目录（backups）

3. **确保远程目录存在**：
   - 自动创建远程父目录（如果不存在）
   - 递归创建所有必需的子目录

4. **上传内容**：
   - 如果远程内容存在且未使用 `--force`，询问是否覆盖
   - 递归上传所有文件和子目录

---

### sync pull - 从云端下载配置

从 WebDAV 服务器下载配置文件或目录到本地。

```bash
ccr sync pull [--force]
```

**参数说明：**

- `--force`, `-f` - 强制覆盖本地配置，跳过确认提示

**使用示例：**

```bash
# 普通下载（会备份本地配置并提示确认）
ccr sync pull

# 强制下载（不提示确认）
ccr sync pull --force
```

**执行流程：**

1. **自动检测同步模式**：
   - 如果是统一模式：递归下载 `/ccr/` 目录及所有子目录到 `~/.ccr/`
   - 如果是传统模式：下载单个 `~/.ccs_config.toml` 文件

2. **智能文件过滤**：
   - 应用与 push 相同的排除规则
   - 自动跳过临时文件和系统文件

3. **安全备份**：
   - 提示将覆盖本地配置（除非使用 `--force`）
   - 自动备份当前本地内容

4. **递归下载**：
   - 从云端下载所有文件和子目录
   - 覆盖本地配置

**⚠️ 安全提示：**

- pull 操作会自动备份本地配置（标签：`before_pull`）
- 备份文件位于与配置文件相同的目录
- 格式：`.ccs_config.toml.before_pull_YYYYMMDD_HHMMSS.bak`

---

## 🔧 配置文件格式

同步配置存储在 `~/.ccs_config.toml` 的 `[settings.sync]` 节中：

```toml
[settings.sync]
enabled = true
webdav_url = "https://dav.jianguoyun.com/dav/"
username = "user@example.com"
password = "your_app_password"
remote_path = "/ccr/"  # 统一模式为目录路径，传统模式为文件路径
auto_sync = false  # 预留字段，暂未实现
```

---

## 💡 使用场景

### 场景 1: 多设备配置同步

**需求：**在多台设备间同步 CCR 配置

**步骤：**

```bash
# 设备 A（主设备）
ccr sync config          # 配置 WebDAV
ccr sync push            # 上传配置

# 设备 B
ccr sync config          # 使用相同的 WebDAV 配置
ccr sync pull            # 下载配置
ccr list                 # 验证配置
```

### 场景 2: 配置备份

**需求：**定期备份配置到云端

**步骤：**

```bash
# 首次配置
ccr sync config

# 每次修改配置后
ccr add                  # 添加新配置
ccr sync push            # 备份到云端
```

### 场景 3: 多平台配置同步

**需求：**同步多个 AI CLI 平台的配置（Claude、Codex、Gemini 等）

**步骤：**

```bash
# 设备 A（主设备）- 配置多平台
ccr platform init claude
ccr platform init codex
ccr platform init gemini

# 添加各平台的配置
ccr platform switch claude
ccr add                      # 添加 Claude 配置

ccr platform switch codex
ccr add                      # 添加 Codex 配置

# 上传整个 ~/.ccr/ 目录（包含所有平台配置）
ccr sync config              # 配置 WebDAV
ccr sync push                # 递归上传所有平台配置

# 设备 B - 同步所有平台配置
ccr sync config              # 使用相同的 WebDAV 配置
ccr sync pull                # 下载整个 ~/.ccr/ 目录
ccr platform list            # 验证所有平台配置
```

### 场景 4: 恢复配置

**需求：**从云端恢复丢失的配置

**步骤：**

```bash
ccr sync config          # 配置 WebDAV
ccr sync pull            # 从云端恢复
```

---

## 🌐 支持的 WebDAV 服务

### 坚果云（推荐）

- 免费额度：1GB 存储 + 3GB/月 流量
- 速度快，国内访问稳定
- 官方文档：https://help.jianguoyun.com/?p=2064

**配置：**
- 地址：`https://dav.jianguoyun.com/dav/`
- 认证：邮箱 + 应用密码

### 其他 WebDAV 服务

理论上支持所有标准 WebDAV 服务，包括：

- **Nextcloud / ownCloud** - 自建私有云
- **Box.com** - 企业网盘
- **pCloud** - 个人云存储
- **4shared** - 免费网盘

配置时只需修改 `webdav_url` 为对应服务的 WebDAV 地址。

---

## ⚠️ 注意事项

### 安全性

1. **密码存储**
   - 密码明文存储在本地配置文件中
   - 建议使用应用专用密码，而非主账户密码
   - 不要将配置文件提交到公开的 Git 仓库

2. **网络传输**
   - 使用 HTTPS 确保传输加密
   - 避免在不安全的网络环境下同步

### 使用建议

1. **首次使用**
   - 先运行 `ccr sync status` 确认配置正确
   - 使用 `ccr sync push` 首次上传现有配置

2. **多设备同步**
   - 避免同时在多设备修改配置
   - pull 前建议先 push 保存本地更改

3. **定期备份**
   - 虽然有云端同步，仍建议定期使用 `ccr export` 导出配置

---

## 🐛 故障排除

### 问题：认证失败

**错误信息：** `认证失败：用户名或密码错误`

**解决方法：**
1. 确认用户名（邮箱）正确
2. 坚果云用户确认使用的是应用密码，而非账户密码
3. 检查密码是否包含特殊字符导致输入错误

### 问题：网络连接失败

**错误信息：** `网络错误: ...`

**解决方法：**
1. 检查网络连接
2. 确认 WebDAV 服务器地址正确
3. 尝试在浏览器访问 WebDAV 地址
4. 检查防火墙/代理设置

### 问题：文件不存在

**错误信息：** `文件不存在: /ccr/.ccs_config.toml`

**解决方法：**
1. 首次使用时正常，需要先 `push` 上传配置
2. 确认远程路径配置正确
3. 使用 `ccr sync status` 检查配置

---

## 📖 相关命令

- [`export`](./export.md) - 导出配置到本地文件
- [`import`](./import.md) - 从本地文件导入配置
- [`init`](./init.md) - 初始化配置文件

---

## 🔮 后续计划

- [ ] 自动同步模式（修改后自动 push）
- [ ] 冲突检测与合并
- [ ] 版本历史管理
- [ ] 加密传输支持
- [ ] 更多 WebDAV 服务的预设配置
