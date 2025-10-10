# CCR Web Interface

CCR (Claude Code Configuration Switcher) 的 Web 界面提供了一个现代化的、用户友好的配置管理界面。

## 功能特点

### 🎨 现代化设计
- **深色主题**: 优雅的深色界面，降低眼睛疲劳
- **动态背景**: 渐变色动画背景效果
- **响应式设计**: 完美支持桌面和移动设备
- **流畅动画**: 丰富的过渡动画和交互效果

### ⚡ 核心功能
- **配置管理**: 添加、编辑、删除配置
- **配置切换**: 一键切换活跃配置
- **历史记录**: 完整的操作历史追踪
- **实时验证**: 配置格式实时验证
- **自动备份**: 配置更改自动备份

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

Web 界面启动后会自动在默认浏览器中打开。如果自动打开失败，可以手动访问:

```
http://localhost:8080
```

## 界面布局

### 1. 导航栏
- **Logo**: CCR 品牌标识
- **刷新按钮**: 重新加载所有配置数据
- **添加配置**: 打开新配置创建界面

### 2. 侧边栏

#### 当前配置显示
- **配置名称**: 显示当前激活的配置
- **状态指示**: 动态状态指示灯
- **扫描动画**: 科技感扫描线效果

#### 统计信息
- **总配置数**: 系统中的配置总数
- **历史记录数**: 操作历史记录数量

#### 快捷操作
- **验证配置**: 一键验证所有配置完整性

### 3. 主内容区

#### 配置列表标签页
显示所有可用配置，每个配置卡片包含：
- **配置名称**: 配置的唯一标识
- **描述信息**: 配置用途说明
- **状态标识**: "当前" 或 "默认" 徽章
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
  "small_fast_model": "small-model"
}
```

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

## 使用场景

### 场景 1: 添加新配置

1. 点击导航栏的 "➕ 添加配置" 按钮
2. 填写配置信息：
   - **配置名称**: 唯一标识 (必填)
   - **描述**: 配置用途说明 (可选)
   - **Base URL**: API 端点 (必填)
   - **Auth Token**: API 密钥 (必填)
   - **Model**: 默认模型 (可选)
   - **Small Fast Model**: 快速模型 (可选)
3. 点击 "保存" 按钮
4. 配置将立即出现在配置列表中

### 场景 2: 切换配置

1. 在配置列表中找到目标配置
2. 点击配置卡片上的 "切换" 按钮
3. 确认切换操作
4. 配置立即生效，历史记录自动记录

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

## 技术架构

### 前端技术
- **纯 HTML/CSS/JavaScript**: 无外部依赖
- **响应式设计**: 支持各种屏幕尺寸
- **现代 CSS**: 使用 CSS Grid 和 Flexbox
- **Fetch API**: 异步数据获取

### 后端技术
- **Rust + tiny_http**: 轻量级 HTTP 服务器
- **RESTful API**: 标准化接口设计
- **文件锁机制**: 并发安全保证
- **原子操作**: 数据一致性保证

### 数据流

```
前端页面 (index.html)
    ↓ Fetch API
Web 服务器 (web.rs)
    ↓
配置管理器 (config.rs)
    ↓
配置文件 (~/.ccs_config.toml)
```

## 配置文件

CCR Web 界面直接读写以下文件：

- **`~/.ccs_config.toml`**: 配置源文件
- **`~/.claude/settings.json`**: Claude Code 设置文件
- **`~/.claude/ccr_history.json`**: 操作历史记录
- **`~/.claude/.locks/`**: 文件锁目录

## 安全注意事项

1. **本地访问**: 默认绑定到 `0.0.0.0`，可从局域网访问
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

编辑文件 `ccr/web/index.html`:
- 所有 HTML、CSS 和 JavaScript 都在此文件中
- 使用 `include_str!` 宏嵌入到编译后的二进制文件

### 添加新的 API 端点

1. 在 `src/web.rs` 的 `handle_request` 方法中添加路由
2. 实现对应的处理方法
3. 更新前端 JavaScript 调用新接口

### 测试 Web 功能

```bash
# 编译
cargo build

# 运行
./target/debug/ccr web

# 在浏览器中测试各种功能
```

## 未来计划

- [ ] 身份认证和授权
- [ ] HTTPS 支持
- [ ] 配置导入/导出
- [ ] 批量操作
- [ ] 配置模板
- [ ] 搜索和筛选
- [ ] 更详细的历史记录
- [ ] 性能监控

## 相关文档

- [CCR 主文档](./CLAUDE.md)
- [配置文件格式](../README.md)
- [CCS 兼容性说明](../CLAUDE.md)

---

**版本**: 0.1.0
**最后更新**: 2025-10-10
