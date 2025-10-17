# sync 命令 - WebDAV 配置同步

## 📋 概述

`sync` 命令提供基于 WebDAV 协议的配置文件云端同步功能，默认支持坚果云，也兼容其他 WebDAV 服务。

## 🎯 功能特性

- ☁️ **WebDAV 同步** - 支持将配置文件同步到 WebDAV 服务器
- 🔒 **安全认证** - 支持基本认证（Basic Auth）
- 🔄 **双向同步** - 支持上传（push）和下载（pull）
- 🌰 **坚果云优化** - 默认配置适配坚果云，开箱即用
- ✅ **状态检查** - 实时查看同步配置和远程文件状态

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
| 远程文件路径 | 配置文件在服务器上的路径 | `/ccr/.ccs_config.toml` | `/backup/config.toml` |

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

ℹ 远程文件路径
  默认: /ccr/.ccs_config.toml
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
  WebDAV 服务器: https://dav.jianguoyun.com/dav/
  用户名: user@example.com
  远程路径: /ccr/.ccs_config.toml
  自动同步: 关闭

▶ 检查远程文件
✓ 远程配置文件存在
```

---

### sync push - 上传配置到云端

将本地配置文件上传到 WebDAV 服务器。

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

1. 检查远程文件是否存在
2. 如果存在且未使用 `--force`，询问是否覆盖
3. 确保远程目录存在
4. 上传本地配置文件

---

### sync pull - 从云端下载配置

从 WebDAV 服务器下载配置文件到本地。

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

1. 提示将覆盖本地配置（除非使用 `--force`）
2. 自动备份当前本地配置
3. 从云端下载配置文件
4. 覆盖本地配置

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
remote_path = "/ccr/.ccs_config.toml"
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

### 场景 3: 恢复配置

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
