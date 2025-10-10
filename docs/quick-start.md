# 快速开始

5 分钟快速上手 CCR！

## 🚀 安装

### 1. 安装 Rust（如果还没有）

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 2. 构建 CCR

```bash
cd ccs/ccr
cargo build --release
cargo install --path . --locked
```

### 3. 验证安装

```bash
ccr --version
# 输出: ccr 0.2.0
```

## ⚙️ 配置

### 创建配置文件

```bash
cat > ~/.ccs_config.toml << 'EOF'
default_config = "anthropic"
current_config = "anthropic"

[anthropic]
description = "Anthropic 官方 API"
base_url = "https://api.anthropic.com"
auth_token = "sk-ant-your-api-key-here"
model = "claude-sonnet-4-5-20250929"
small_fast_model = "claude-3-5-haiku-20241022"

[anyrouter]
description = "AnyRouter 代理服务"
base_url = "https://api.anyrouter.ai/v1"
auth_token = "your-anyrouter-token-here"
model = "claude-sonnet-4-5-20250929"
EOF
```

### 编辑配置

```bash
vim ~/.ccs_config.toml
# 替换 your-api-key-here 为真实的 API key
```

## 🎯 基本使用

### 列出所有配置

```bash
ccr list
```

### 查看当前状态

```bash
ccr current
```

### 切换配置

```bash
ccr switch anyrouter
# 或简写
ccr anyrouter
```

### 验证配置

```bash
ccr validate
```

### 查看历史

```bash
ccr history
```

### 启动 Web 界面

```bash
ccr web
```

## 🌐 Web 界面

1. 启动服务器：
```bash
ccr web
```

2. 浏览器自动打开 http://localhost:8080

3. 在 Web 界面中：
   - 查看所有配置
   - 切换配置
   - 添加/编辑配置
   - 查看历史记录

## 💡 常见使用场景

### 场景 1: 在不同 API 之间切换

```bash
# 使用官方 API
ccr anthropic

# 使用代理服务
ccr anyrouter

# 查看当前使用的配置
ccr current
```

### 场景 2: 添加新配置

```bash
# 方式 1: 手动编辑
vim ~/.ccs_config.toml

# 方式 2: Web 界面
ccr web
# 点击"添加配置"按钮
```

### 场景 3: 验证和排错

```bash
# 验证所有配置
ccr validate

# 查看详细信息
ccr current

# 查看操作历史
ccr history --limit 10
```

## 🎓 下一步

- 📖 [完整安装指南](/installation/)
- 🏗️ [了解架构设计](/architecture/)
- 📚 [学习所有命令](/commands/)
- 🌐 [Web API 参考](/api/web-api)
- 👨‍💻 [参与开发](/development/)

## 🔗 快速链接

- [GitHub 仓库](https://github.com/bahayonghang/ccs)
- [问题反馈](https://github.com/bahayonghang/ccs/issues)
- [更新日志](/changelog)
- [迁移指南](/migration)

