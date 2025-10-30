# CCR UI - 全栈Web应用程序

一个现代化的全栈Web应用程序，用于管理CCR（Claude代码配置切换器），采用Vue 3前端和Rust Axum后端。

## ⚡ 超快速开始

```bash
cd ccr-ui
just s    # 启动开发环境（就这么简单！）
```

**第一次使用？** 运行 `just quick-start` 自动完成所有设置。

**不知道用什么命令？** 直接运行 `just` 查看帮助！

---

## 功能特性

- **配置管理**：查看、切换和验证CCR配置
- **命令执行器**：通过可视化界面执行任何CCR命令
- **WebDAV同步**：将配置推送/拉取到云存储（坚果云、Nextcloud、ownCloud）
- **实时输出**：在终端风格的显示器中查看命令输出
- **多页面导航**：在不同功能之间轻松切换

## 架构

```
ccr-ui/
├── backend/           # Rust Axum服务器
│   ├── src/
│   │   ├── main.rs    # 服务器入口点
│   │   ├── executor/  # CLI子进程执行器
│   │   ├── handlers/  # API路由处理器
│   │   └── models/    # 请求/响应模型
│   └── Cargo.toml
├── frontend/          # Vue 3 + Vite应用程序（TypeScript）
│   ├── src/
│   │   ├── App.vue    # 根Vue组件
│   │   ├── main.ts    # 应用程序入口点
│   │   ├── components/# 可复用Vue组件
│   │   ├── views/     # 页面组件
│   │   ├── router/    # Vue Router配置
│   │   ├── store/     # Pinia状态管理
│   │   └── api/       # API客户端和工具
│   ├── package.json
│   └── vite.config.ts
└── README.md
```

## 前置条件

### 必需
- **Rust 1.70+**（包含Cargo）
- **Node.js 18+**（包含npm）
- **CCR** 已安装并在PATH中可用

### 可选但推荐
- **Just** - 命令运行器，简化工作流程
  - 安装：`cargo install just` 或 `brew install just`
  - 仓库：https://github.com/casey/just

## 安装

### 🚀 快速安装（使用Just - 推荐）

```bash
cd ccr-ui

# 检查前置条件
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

## 开发

### 🚀 快速开始（使用Just）

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

# 或使用自定义端口
cargo run -- --port 8081

# 或使用Just
just dev-backend
```

后端默认启动在 `http://127.0.0.1:8081`。

#### 启动前端开发服务器

```bash
cd frontend
npm run dev

# 或使用Just
just dev-frontend
```

前端将启动在 `http://localhost:5173`，支持Vite的快速热模块替换。

## 生产构建

### 🏗️ 使用Just（推荐）

```bash
cd ccr-ui

# 构建后端和前端
just build

# 完整CI工作流程（检查 + 测试 + 构建）
just ci

# 运行生产后端
just run-prod
```

### 📦 手动构建

#### 构建后端

```bash
cd backend
cargo build --release
```

二进制文件将位于 `target/release/ccr-ui-backend`。

#### 构建前端

```bash
cd frontend
npm run build
```

构建文件将位于 `frontend/dist/`。

## API端点

### 配置管理
- `GET /api/configs` - 列出所有配置
- `POST /api/switch` - 切换到指定配置
- `POST /api/validate` - 验证配置
- `POST /api/clean` - 清理旧备份
- `POST /api/export` - 导出配置
- `POST /api/import` - 导入配置
- `GET /api/history` - 获取操作历史
- `GET /api/system` - 获取系统信息

### 命令执行
- `POST /api/command/execute` - 执行CCR命令
- `GET /api/command/list` - 列出可用命令
- `GET /api/command/help/{command}` - 获取命令帮助

### WebDAV同步
- `GET /api/sync/status` - 获取同步配置状态
- `POST /api/sync/push` - 上传配置到云端（请求体：`{force: boolean}`）
- `POST /api/sync/pull` - 从云端下载配置（请求体：`{force: boolean}`）
- `GET /api/sync/info` - 获取同步功能信息
- `POST /api/sync/config` - 配置WebDAV同步（Web API不支持）

## 使用说明

### 配置管理页面

1. 查看所有可用配置
2. 查看当前激活的配置
3. 一键切换配置
4. 验证所有配置

### 命令执行器页面

1. 从列表中选择命令（13个可用命令）
2. 输入可选参数
3. 执行命令
4. 实时查看带语法高亮的输出
5. 复制输出或清除输出

## 可用命令

- `init` - 初始化配置文件
- `list` - 列出所有配置
- `current` - 显示当前配置
- `switch` - 切换到指定配置
- `validate` - 验证配置
- `optimize` - 优化配置文件
- `history` - 显示操作历史
- `clean` - 清理旧备份
- `export` - 导出配置
- `import` - 导入配置
- `update` - 更新CCR
- `version` - 显示版本信息

## 技术栈

### 后端
- **Axum 0.7** - Rust快速异步Web框架
- **Tokio 1.42** - 异步运行时
- **Serde** - 序列化
- **Chrono** - 日期/时间处理

### 前端
- **Vue 3.5** - 渐进式JavaScript框架
- **Vite 7.1** - 下一代前端工具
- **Vue Router 4.4** - Vue.js官方路由器
- **Pinia 2.2** - Vue状态管理
- **TypeScript 5.7** - 类型安全
- **Tailwind CSS 3.4** - 样式框架
- **Axios 1.7** - HTTP客户端
- **Lucide Vue Next** - 图标库
- **ANSI to HTML** - 终端输出渲染

## 配置

### 后端端口

默认：`8081`

通过命令行更改：
```bash
cargo run -- --port 3000
```

### 前端API配置

前端连接到后端API。如需配置，请在 `src/api/config.ts` 中修改：

```typescript
const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8081';
```

生产环境请设置 `VITE_API_URL` 环境变量。

## 安全注意事项

- 后端以子进程方式执行CCR命令
- 所有命令都会根据允许列表进行验证
- 无Shell注入漏洞（正确的参数传递）
- 为本地开发启用了CORS
- 建议仅在localhost上运行

## 故障排除

### 后端无法启动
- 确保CCR已安装：`ccr version`
- 检查端口8081是否可用
- 查看日志中的错误消息

### 前端无法连接到后端
- 确保后端在端口8081上运行
- 检查浏览器控制台的CORS错误
- 验证 `src/api/config.ts` 中的API配置

### 命令执行失败
- 确保CCR在您的PATH中
- 检查CCR版本是否兼容
- 查看命令输出中的具体错误

## 开发技巧

### 热重载
前端和后端在开发期间都支持热重载：
- 前端：Vite在文件更改时自动重载
- 后端：使用 `just watch-backend` 进行自动重启（需要 `cargo-watch`）

### 调试
- 前端：使用浏览器开发者工具
- 后端：设置 `RUST_LOG=debug` 获取详细日志

### 测试

**使用Just：**
```bash
just test          # 运行所有测试
just check         # 检查代码而不构建
just fmt           # 格式化代码
just clippy        # 运行Clippy代码检查
```

**手动：**
```bash
# 后端测试
cd backend
cargo test

# 前端代码检查
cd frontend
npm run lint
```

### 常用Just命令

**🌟 超简化版（推荐）：**
```bash
just        # 显示帮助
just s      # 启动开发（最常用！）
just i      # 安装依赖
just b      # 构建生产版本
just c      # 检查代码
just t      # 运行测试
just f      # 格式化代码
```

**📋 完整版：**
```bash
just help            # 显示帮助
just --list          # 显示所有命令（40+）
just info            # 显示项目信息
just check-prereqs   # 检查前置条件
just install         # 安装依赖
just dev             # 启动开发环境
just build           # 构建生产版本
just test            # 运行测试
just fmt             # 格式化代码
just clean           # 清理构建产物
just update          # 更新依赖
just quick-start     # 一键启动（检查+安装+开发）
```

**💡 速查表：**
| 简化 | 完整 | 说明 |
|-----|-----|------|
| `s` | `dev` | 启动开发 |
| `i` | `install` | 安装依赖 |
| `b` | `build` | 构建 |
| `c` | `check` | 检查 |
| `t` | `test` | 测试 |
| `f` | `fmt` | 格式化 |

查看所有命令：`just --list` 或查看 `SIMPLE_USAGE.md`

## 许可证

MIT许可证 - 与CCR项目相同

## 贡献

这是CCR项目的一部分。如需贡献，请参阅主CCR仓库。

## 相关项目

- **CCR** - 主CLI工具（Rust实现）
- **CCS** - Shell版本兼容性
- **CCR Tauri** - 桌面应用程序

---

**版本**：0.1.0  
**最后更新**：2025-01-13