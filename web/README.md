# CCR Web Interface

CCR (Claude Code Configuration Switcher) 的 Web 界面提供了一个现代化的、用户友好的配置管理界面。

## 功能特点

### 🎨 现代化设计
- **双主题支持**: 深色主题（紫色风格）和明色主题（Apple 风格），默认明色主题
- **主题切换**: 一键切换主题，自动保存偏好设置到 localStorage
- **动态背景**: 渐变色动画背景效果
- **响应式设计**: 完美支持桌面和移动设备
- **流畅动画**: 丰富的过渡动画和交互效果

### ⚡ 核心功能
- **配置管理**: 添加、编辑、删除配置
- **配置切换**: 一键切换活跃配置
- **配置分类**: 多维度分类和筛选系统
  - **提供商类型**: 官方中转 / 第三方模型 / 未分类
  - **提供商名称**: 按具体提供商筛选（anyrouter, glm, moonshot 等）
  - **账号标识**: 区分同一提供商的不同账号
  - **标签系统**: 灵活的标签分类（free, stable, backup 等）
- **可视化增强**: 配置卡片展示提供商信息、账号、标签
- **智能过滤**: 配置列表和侧边栏导航同步过滤
- **历史记录**: 完整的操作历史追踪
- **实时验证**: 配置格式实时验证
- **自动备份**: 配置更改自动备份
- **导入导出**: 支持配置文件的导入和导出（可选包含敏感信息）
- **备份清理**: 定期清理旧备份文件，释放磁盘空间
- **错误重试**: 网络请求失败时自动提供重试选项

### 🔐 安全特性
- **敏感信息掩码**: API Token 自动掩码显示
- **文件锁机制**: 防止并发写入冲突
- **原子操作**: 确保配置更新的原子性

## 启动 Web 界面

### 基本用法

```bash
# 使用默认端口 8080
ccr web

# 指定自定义端口
ccr web --port 3000
ccr web -p 3000
```

### 自动打开浏览器

Web 界面启动后会自动在默认浏览器中打开。如果自动打开失败,可以手动访问:

```
http://localhost:8080
```

## 界面布局

### 1. 导航栏
- **Logo**: CCR 品牌标识
- **主题切换**: 切换深色/明色主题按钮
- **刷新按钮**: 重新加载所有配置数据
- **导入按钮**: 从文件导入配置
- **导出按钮**: 导出配置到文件
- **添加配置**: 打开新配置创建界面

### 2. 侧边栏

#### 当前配置显示
- **配置名称**: 显示当前激活的配置
- **状态指示**: 动态状态指示灯
- **扫描动画**: 科技感扫描线效果

#### 系统信息 (v1.0.2 新增)
实时显示主机系统信息，每 5 秒自动刷新：
- **主机名**: 当前主机的名称
- **操作系统**: 系统类型和版本
- **CPU**: 处理器型号和核心数
- **CPU 使用率**: 实时 CPU 使用率，带动态进度条
- **内存使用**: 已用/总内存（GB），带使用率进度条
- **运行时间**: 系统启动后的运行时间

#### 统计信息
- **总配置数**: 系统中的配置总数
- **历史记录数**: 操作历史记录数量

#### 快捷操作
- **验证配置**: 一键验证所有配置完整性
- **清理备份**: 清理过期的备份文件

### 3. 主内容区

#### 配置类型过滤 (v1.1.0 新增)
在配置列表上方提供快捷过滤按钮：
- **全部**: 显示所有配置
- **🔄 官方中转**: 仅显示官方 Claude 模型的中转服务
- **🤖 第三方模型**: 仅显示第三方模型服务（GLM、Kimi 等）
- **未分类**: 显示没有分类信息的配置

过滤器会同步更新配置列表和右侧目录导航。

#### 配置列表标签页
显示所有可用配置,每个配置卡片包含：
- **提供商类型徽章**: 🔄 官方中转 或 🤖 第三方模型
- **配置名称**: 配置的唯一标识
- **描述信息**: 带图标的配置用途说明
- **状态标识**: "当前" 或 "默认" 徽章
- **提供商信息**: 提供商名称和账号标识（紫色和绿色高亮）
- **标签列表**: 灵活的分类标签
- **配置详情**: Base URL, Auth Token, Model 等
- **操作按钮**: 切换、编辑、删除

#### 历史记录标签页
显示最近的操作历史：
- **操作类型**: 切换配置、备份、验证等
- **时间戳**: 操作发生时间
- **操作者**: 执行操作的用户
- **详细信息**: 配置切换详情等

## API 接口

CCR Web 服务器提供以下 RESTful API 接口：

### 配置管理

#### 获取配置列表
```http
GET /api/configs
```

响应:
```json
{
  "success": true,
  "data": {
    "current_config": "anthropic",
    "default_config": "anthropic",
    "configs": [...]
  }
}
```

#### 切换配置
```http
POST /api/switch
Content-Type: application/json

{
  "config_name": "anyrouter"
}
```

#### 添加配置
```http
POST /api/config
Content-Type: application/json

{
  "name": "new-config",
  "description": "描述",
  "base_url": "https://api.example.com",
  "auth_token": "your-token",
  "model": "model-name",
  "small_fast_model": "small-model",
  "provider": "example",
  "provider_type": "official_relay",
  "account": "main_account",
  "tags": ["production", "high-priority"]
}
```

**新增字段 (v1.1.0)**:
- `provider`: 提供商名称（如 "anyrouter", "glm"）
- `provider_type`: 提供商类型（"official_relay" 或 "third_party_model"）
- `account`: 账号标识（用于区分同一提供商的不同账号）
- `tags`: 标签数组（用于灵活分类）

#### 更新配置
```http
PUT /api/config/{name}
Content-Type: application/json

{
  "name": "updated-name",
  "description": "新描述",
  ...
}
```

#### 删除配置
```http
DELETE /api/config/{name}
```

### 历史记录

#### 获取历史记录
```http
GET /api/history
```

### 配置验证

#### 验证所有配置
```http
POST /api/validate
```

### 系统信息

#### 获取系统信息 (v1.0.2 新增)
```http
GET /api/system
```

响应:
```json
{
  "success": true,
  "data": {
    "hostname": "your-hostname",
    "os": "Linux",
    "os_version": "5.15.167.4",
    "kernel_version": "5.15.167.4-microsoft-standard-WSL2",
    "cpu_brand": "AMD Ryzen 7 5800X",
    "cpu_cores": 8,
    "cpu_usage": 45.2,
    "total_memory_gb": 16.0,
    "used_memory_gb": 8.2,
    "memory_usage_percent": 67.5,
    "total_swap_gb": 8.0,
    "used_swap_gb": 0.0,
    "uptime_seconds": 312480
  }
}
```

## 使用场景

### 场景 1: 添加新配置 (支持分类)

1. 点击导航栏的 "➕ 添加配置" 按钮
2. 填写配置信息：
   - **配置名称**: 唯一标识 (必填)
   - **描述**: 配置用途说明 (可选)
   - **Base URL**: API 端点 (必填)
   - **Auth Token**: API 密钥 (必填)
   - **Model**: 默认模型 (可选)
   - **Small Fast Model**: 快速模型 (可选)
   - **提供商类型**: 官方中转/第三方模型 (可选，v1.1.0 新增)
   - **提供商**: 提供商名称 (可选，v1.1.0 新增)
   - **账号**: 账号标识 (可选，v1.1.0 新增)
   - **标签**: 逗号分隔的标签列表 (可选，v1.1.0 新增)
3. 点击 "保存" 按钮
4. 配置将立即出现在配置列表中，并可通过分类筛选

### 场景 2: 切换配置

1. 在配置列表中找到目标配置
2. 点击配置卡片上的 "切换" 按钮
3. 确认切换操作
4. 配置立即生效,历史记录自动记录

### 场景 3: 编辑配置

1. 点击配置卡片上的 "编辑" 按钮
2. 修改所需字段
3. 点击 "保存" 按钮
4. 配置更新立即生效

### 场景 4: 删除配置

1. 点击配置卡片上的 "删除" 按钮
2. 确认删除操作
3. 配置从系统中移除

**注意**: 当前配置和默认配置不能删除

### 场景 5: 查看历史

1. 切换到 "历史记录" 标签页
2. 浏览最近的操作记录
3. 查看每个操作的详细信息

### 场景 6: 配置分类和筛选 (v1.1.0 新增)

1. **按提供商类型筛选**:
   - 点击 "🔄 官方中转" 按钮,查看所有官方 Claude 模型的中转服务
   - 点击 "🤖 第三方模型" 按钮,查看所有第三方模型服务（GLM、Kimi 等）
   - 点击 "未分类" 按钮,查看没有分类信息的配置
   - 点击 "全部" 按钮,恢复显示所有配置

2. **查看配置详情**:
   - 每个配置卡片顶部显示提供商类型徽章
   - 配置描述带图标高亮显示
   - 提供商信息以紫色卡片展示
   - 账号信息以绿色卡片展示
   - 标签以灰色小标签形式展示

3. **同步导航**:
   - 右侧配置目录会根据筛选器同步更新
   - 只显示当前筛选类别下的配置
   - 保持配置列表和导航菜单一致

## 技术架构

### 文件结构
Web 界面采用模块化文件结构，便于维护和开发：

```
web/
├── index.html    # HTML 结构（266 行）
├── style.css     # 所有 CSS 样式（~1000 行）
├── script.js     # 所有 JavaScript 代码（~900 行）
└── README.md     # 本文档
```

所有文件在编译时通过 `include_str!` 宏嵌入到 Rust 二进制文件中，实现单一可执行文件分发。

### 前端技术
- **纯 HTML/CSS/JavaScript**: 无外部依赖，所有资源嵌入二进制
- **模块化设计**: HTML/CSS/JS 分离，便于维护
- **响应式设计**: 支持各种屏幕尺寸
- **现代 CSS**:
  - CSS Custom Properties 实现主题切换
  - CSS Grid 和 Flexbox 布局
  - backdrop-filter 实现毛玻璃效果
- **Fetch API**: 异步数据获取
- **localStorage**: 主题偏好持久化

### 后端技术
- **Rust + tiny_http**: 轻量级 HTTP 服务器
- **RESTful API**: 标准化接口设计
- **静态资源服务**: 提供 HTML/CSS/JS 三个文件
- **文件锁机制**: 并发安全保证
- **原子操作**: 数据一致性保证

### 数据流

```
前端页面
  ├── index.html (HTML 结构)
  ├── style.css (样式定义)
  └── script.js (业务逻辑)
      ↓ Fetch API
Web 服务器 (src/web/handlers.rs)
  ├── serve_html()  → 提供 HTML
  ├── serve_css()   → 提供 CSS
  └── serve_js()    → 提供 JavaScript
      ↓
Service 层 (src/services/)
  ├── ConfigService
  ├── SettingsService
  ├── HistoryService
  └── BackupService
      ↓
配置文件
  ├── ~/.ccs_config.toml (配置源)
  ├── ~/.claude/settings.json (Claude 设置)
  └── ~/.claude/ccr_history.json (历史记录)
```

## 配置文件

CCR Web 界面直接读写以下文件：

- **`~/.ccs_config.toml`**: 配置源文件
- **`~/.claude/settings.json`**: Claude Code 设置文件
- **`~/.claude/ccr_history.json`**: 操作历史记录
- **`~/.claude/.locks/`**: 文件锁目录

## 安全注意事项

1. **本地访问**: 默认绑定到 `0.0.0.0`,可从局域网访问
2. **无认证**: 当前版本不提供身份认证
3. **敏感信息**: Auth Token 在界面上自动掩码
4. **建议**: 仅在可信网络环境中使用

## 故障排除

### Web 服务器无法启动

**问题**: 端口被占用

**解决方案**:
```bash
# 使用其他端口
ccr web --port 3000
```

### 配置无法加载

**问题**: 配置文件不存在或格式错误

**解决方案**:
```bash
# 验证配置文件
ccr validate

# 检查配置文件
cat ~/.ccs_config.toml
```

### 浏览器无法自动打开

**问题**: 系统无法检测到默认浏览器

**解决方案**:
手动访问控制台输出的 URL

### 配置切换失败

**问题**: 配置验证失败或文件权限问题

**解决方案**:
```bash
# 检查配置完整性
ccr validate

# 检查文件权限
ls -la ~/.ccs_config.toml
ls -la ~/.claude/settings.json
```

## 开发指南

### 修改前端界面

Web 界面采用模块化结构，方便独立修改各个部分：

#### 修改 HTML 结构
编辑 `web/index.html`:
- 只包含 HTML 结构和页面布局
- 通过 `<link>` 引用 `style.css`
- 通过 `<script>` 引用 `script.js`
- 文件会通过 `include_str!` 嵌入到二进制

#### 修改样式
编辑 `web/style.css`:
- 包含所有 CSS 样式定义
- 使用 CSS Custom Properties 实现主题切换
- 支持深色主题和明色主题
- 修改后需要重新编译

#### 修改 JavaScript 逻辑
编辑 `web/script.js`:
- 包含所有前端业务逻辑
- 主题管理、API 调用、UI 交互等
- 修改后需要重新编译

### 添加新的 API 端点

1. 在 `src/web/handlers.rs` 的 `handle_request` 方法中添加路由
2. 实现对应的处理方法
3. 在 `web/script.js` 中添加前端调用代码

### 测试 Web 功能

```bash
# 快速检查语法
cargo check

# 编译 debug 版本
cargo build

# 运行
./target/debug/ccr web

# 或者直接编译运行
cargo run -- web

# 在浏览器中测试各种功能
```

### 文件嵌入机制

Web 界面的三个文件在编译时通过 Rust 的 `include_str!` 宏嵌入：

```rust
// src/web/handlers.rs
fn serve_html(&self) -> Response<...> {
    let html = include_str!("../../web/index.html");
    // ...
}

fn serve_css(&self) -> Response<...> {
    let css = include_str!("../../web/style.css");
    // ...
}

fn serve_js(&self) -> Response<...> {
    let js = include_str!("../../web/script.js");
    // ...
}
```

这确保了最终的二进制文件是自包含的，无需外部依赖。

## 未来计划

- [x] 配置导入/导出
- [x] 备份清理功能
- [x] 双主题支持
- [x] 错误重试机制
- [x] 配置分类系统
- [x] 配置筛选功能
- [ ] 身份认证和授权
- [ ] HTTPS 支持
- [ ] 批量操作
- [ ] 配置模板
- [ ] 更多筛选维度（按标签、提供商筛选）
- [ ] 更详细的历史记录
- [ ] 性能监控

## 相关文档

- [CCR 主文档](./CLAUDE.md)
- [配置文件格式](../README.md)
- [CCS 兼容性说明](../CLAUDE.md)

---

**版本**: 1.1.0
**最后更新**: 2025-01-11

## 更新日志

### v1.1.0 (2025-01-11)
- 🏷️ 新增配置分类系统（提供商类型、提供商名称、账号、标签）
- 🔍 新增配置类型筛选按钮（全部/官方中转/第三方模型/未分类）
- 🎨 配置卡片增强显示（提供商徽章、描述、提供商信息、账号、标签）
- 🔄 右侧目录导航同步筛选器变化
- 🌈 不同元数据使用不同颜色和样式（描述、提供商紫色、账号绿色）
- 📡 API 响应新增分类字段
- ✅ 100% 向后兼容，现有配置无需修改

### v1.0.2 (2025-10-11)
- 📊 新增实时系统信息显示（CPU、内存、运行时间等）
- ⚡ 系统信息每 5 秒自动刷新
- 🎨 新增动态进度条动画效果
- 📡 新增 `/api/system` API 端点
- 📦 添加 `sysinfo` 依赖库

### v1.0.1 (2025-10-11)
- ✨ 文件结构重构：拆分为 HTML/CSS/JS 三个独立文件
- 🎨 新增双主题支持（深色紫色主题 + 明色 Apple 主题）
- 📥 新增配置导入导出功能
- 🗑️ 新增备份清理功能
- 🔄 新增请求失败重试机制
- 🎯 优化按钮加载状态显示
- 📝 完善文档说明
