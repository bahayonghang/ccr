# 快速开始

本指南将帮助你快速搭建和运行 CCR UI 项目。

## 系统要求

在开始之前，请确保你的系统满足以下要求：

### 必需组件

- **Rust 1.70+** (包含 Cargo)
- **Node.js 18+** (包含 npm)
- **CCR** 已安装并在 PATH 中可用

### 推荐工具

- **Just** - 命令运行器，简化工作流程
  - 安装：`cargo install just` 或 `brew install just`
  - 仓库：https://github.com/casey/just

## 安装步骤

### 🚀 快速安装（使用 Just - 推荐）

```bash
cd ccr-ui

# 检查系统要求
just check-prereqs

# 安装所有依赖
just install
```

### 📦 手动安装

#### 1. 安装后端依赖

```bash
cd backend
cargo build --release
```

#### 2. 安装前端依赖

```bash
cd frontend
npm install
```

## 开发环境启动

### 🚀 快速启动（使用 Just）

```bash
cd ccr-ui

# 🌟 超简单：只需一个字母！
just s               # 启动开发环境（s = start）

# 或者一键全自动（检查 + 安装 + 开发）
just quick-start

# 查看帮助（不知道用什么命令时）
just                 # 显示常用命令帮助
just --list          # 显示所有40+命令
```

**简化命令速查：**
- `just s` = 启动开发
- `just i` = 安装依赖  
- `just b` = 构建生产版本
- `just c` = 检查代码
- `just t` = 运行测试
- `just f` = 格式化代码

### 📝 手动启动

#### 启动后端服务器

```bash
cd backend
cargo run

# 或者指定自定义端口
cargo run -- --port 8081

# 或者使用 Just
just dev-backend
```

后端服务器默认在 `http://127.0.0.1:8081` 启动。

#### 启动前端开发服务器

```bash
cd frontend
npm run dev

# 或者使用 Just
just dev-frontend
```

前端开发服务器在 `http://localhost:5173` 启动，支持热重载。

## 验证安装

### 1. 检查后端服务

访问 `http://127.0.0.1:8081/api/system/info` 应该返回系统信息：

```json
{
  "status": "ok",
  "version": "0.1.0",
  "system": {
    "os": "linux",
    "arch": "x86_64",
    "cpu_count": 8,
    "username": "user"
  }
}
```

### 2. 检查前端应用

访问 `http://localhost:5173` 应该看到 CCR UI 的主界面。

### 3. 测试 CCR 集成

在应用中尝试执行 `ccr list` 命令，应该能看到当前的配置列表。

## 常见问题

### 端口冲突

如果遇到端口冲突，可以修改端口：

```bash
# 后端
cargo run -- --port 8082

# 前端（修改 vite.config.ts）
export default defineConfig({
  server: {
    port: 5174
  }
})
```

### CCR 命令不可用

确保 CCR 已正确安装：

```bash
# 检查 CCR 是否在 PATH 中
which ccr

# 测试 CCR 命令
ccr --version
```

### 依赖安装失败

如果遇到依赖安装问题：

```bash
# 清理缓存
cargo clean
npm cache clean --force

# 重新安装
just install
```

## 下一步

现在你已经成功运行了 CCR UI，可以：

1. 查看 [项目结构](/guide/project-structure) 了解代码组织
2. 阅读 [前端开发指南](/frontend/development) 学习前端开发
3. 查看 [后端架构](/backend/architecture) 了解后端设计
4. 参考 [API 文档](/backend/api) 了解接口详情

## 获取帮助

如果在安装或使用过程中遇到问题：

- 查看 [FAQ](/faq) 获取常见问题解答
- 在 [GitHub Issues](https://github.com/your-username/ccr/issues) 提交问题
- 查看项目的 [贡献指南](/contributing) 了解如何参与开发