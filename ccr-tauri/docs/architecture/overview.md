# 总体架构

本小姐给你详细讲解 CCR Desktop 的整体架构设计！(￣▽￣)ゞ

## 架构概览

CCR Desktop 采用 **前后端分离** 的架构模式，基于 Tauri 2.0 框架构建：

```
┌─────────────────────────────────────────────────────────────┐
│                      CCR Desktop 应用                         │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌───────────────────┐          ┌────────────────────────┐  │
│  │   前端 (Vue 3)     │  Tauri   │    后端 (Rust)          │  │
│  │                   │◄────────►│                        │  │
│  │  • UI 组件        │  IPC     │  • Tauri Commands      │  │
│  │  • 状态管理       │          │  • 服务层逻辑          │  │
│  │  • API 封装       │          │  • CCR 核心库          │  │
│  └───────────────────┘          └────────────────────────┘  │
│                                          │                    │
│                                          ▼                    │
│                                  ┌──────────────────────┐    │
│                                  │   文件系统            │    │
│                                  │  • ~/.ccs_config.toml │    │
│                                  │  • ~/.claude/         │    │
│                                  └──────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## 核心组件

### 1. 前端层 (Vue 3 + TypeScript)

**职责：**
- 用户界面呈现
- 用户交互处理
- 状态管理
- API 调用封装

**技术栈：**
- **Vue 3**: 渐进式框架，Composition API
- **TypeScript**: 类型安全
- **Vite**: 快速构建和 HMR
- **CSS**: 自定义样式，支持深色/浅色主题

**目录结构：**
```
src-ui/
├── src/
│   ├── main.ts           # 应用入口
│   ├── App.vue           # 根组件
│   ├── api/              # Tauri API 封装
│   │   └── index.ts      # API 函数
│   ├── types/            # TypeScript 类型定义
│   │   └── index.ts
│   ├── components/       # Vue 组件 (可扩展)
│   ├── stores/           # Pinia 状态管理 (可选)
│   └── style.css         # 全局样式
├── index.html            # HTML 模板
├── vite.config.ts        # Vite 配置
└── package.json
```

---

### 2. 后端层 (Rust + Tauri)

**职责：**
- 处理前端命令请求
- 调用 CCR 核心库
- 文件系统操作
- 权限控制

**技术栈：**
- **Tauri 2.0**: 桌面应用框架
- **CCR Core**: 配置管理核心库
- **serde**: 序列化/反序列化
- **tokio**: 异步运行时 (可选)

**目录结构：**
```
ccr-tauri/
├── src/
│   ├── main.rs           # Tauri 应用入口
│   ├── lib.rs            # 库入口
│   └── commands/         # Tauri Commands
│       └── mod.rs        # 命令定义
├── Cargo.toml            # Rust 依赖
├── tauri.conf.json       # Tauri 配置
└── build.rs              # 构建脚本
```

---

### 3. 服务层 (CCR Core Library)

**职责：**
- 配置文件管理 (`~/.ccs_config.toml`)
- Settings 文件管理 (`~/.claude/settings.json`)
- 操作历史记录
- 备份管理
- 文件锁定机制

**架构层次：**
```
┌──────────────────────────────┐
│     Tauri Commands           │  ← 命令层
├──────────────────────────────┤
│     Service Layer            │  ← 业务逻辑层
│  • ConfigService             │
│  • SettingsService           │
│  • HistoryService            │
├──────────────────────────────┤
│     Manager Layer            │  ← 数据访问层
│  • ConfigManager             │
│  • SettingsManager           │
│  • HistoryManager            │
├──────────────────────────────┤
│     Core Layer               │  ← 基础设施层
│  • AtomicWriter              │
│  • LockManager               │
│  • ErrorHandling             │
├──────────────────────────────┤
│     Utils Layer              │  ← 工具层
│  • Validation                │
│  • Masking                   │
└──────────────────────────────┘
```

---

## 数据流

### 配置切换流程

```
┌────────┐      ┌────────┐      ┌────────┐      ┌──────────┐
│ 用户点  │─────►│ Vue    │─────►│ Tauri  │─────►│ CCR      │
│ 击切换  │      │ 组件   │      │ Command│      │ Service  │
└────────┘      └────────┘      └────────┘      └──────────┘
                    │                │                │
                    │                │                ▼
                    │                │          ┌──────────┐
                    │                │          │ 文件锁定  │
                    │                │          └──────────┘
                    │                │                │
                    │                │                ▼
                    │                │          ┌──────────┐
                    │                │          │ 读取配置  │
                    │                │          └──────────┘
                    │                │                │
                    │                │                ▼
                    │                │          ┌──────────┐
                    │                │          │ 更新     │
                    │                │          │ Settings │
                    │                │          └──────────┘
                    │                │                │
                    │                │                ▼
                    │                │          ┌──────────┐
                    │                │          │ 记录历史  │
                    │                │          └──────────┘
                    │                │                │
                    │                ▼                │
                    │          ┌──────────┐          │
                    │◄─────────│ 返回结果  │◄─────────┘
                    │          └──────────┘
                    ▼
              ┌──────────┐
              │ 更新 UI   │
              └──────────┘
```

---

## 通信机制

### Tauri IPC (进程间通信)

前端和后端通过 Tauri 的 IPC 机制通信：

**前端调用：**
```typescript
import { invoke } from '@tauri-apps/api/core'

// 调用 Tauri Command
const result = await invoke<string>('switch_config', {
  name: 'anthropic'
})
```

**后端处理：**
```rust
#[tauri::command]
pub async fn switch_config(name: String) -> Result<String, String> {
    // 处理逻辑
    Ok(format!("✅ 成功切换到配置: {}", name))
}
```

**消息序列化：**
- 使用 `serde_json` 进行 JSON 序列化
- 自动类型转换和验证
- 支持复杂数据结构

---

## 安全策略

### 1. 文件系统权限

Tauri 使用 **白名单** 机制限制文件访问：

```json
{
  "plugins": {
    "fs": {
      "scope": [
        "$HOME/.ccs_config.toml",
        "$HOME/.claude/**",
        "$HOME/.claude/backups/**"
      ]
    }
  }
}
```

只允许访问明确列出的路径！(￣▽￣)ゞ

### 2. Command 权限

所有 Command 必须在 `main.rs` 中显式注册：

```rust
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        commands::list_configs,
        commands::switch_config,
        // ... 其他命令
    ])
    .run(tauri::generate_context!())
```

### 3. CSP (Content Security Policy)

限制前端可以加载的资源：

```json
{
  "security": {
    "csp": "default-src 'self'; style-src 'self' 'unsafe-inline'"
  }
}
```

---

## 性能优化

### 1. 前端优化

**代码分割：**
```typescript
// 动态导入大型依赖
const heavy = await import('./heavy-module')
```

**虚拟列表：**
- 对于长列表使用虚拟滚动
- 只渲染可见区域

**节流防抖：**
- 搜索、筛选等频繁操作使用防抖
- 滚动、resize 使用节流

### 2. 后端优化

**异步处理：**
```rust
#[tauri::command]
pub async fn heavy_operation() -> Result<(), String> {
    // 使用 tokio 异步运行
    tokio::task::spawn(async move {
        // 耗时操作
    }).await
}
```

**并发控制：**
- 使用文件锁避免并发写入冲突
- 原子写入机制保证数据一致性

### 3. 缓存策略

```typescript
// 缓存系统信息
let cachedSystemInfo: SystemInfo | null = null
let cacheTime = 0

async function getSystemInfo() {
  const now = Date.now()
  if (cachedSystemInfo && now - cacheTime < 5000) {
    return cachedSystemInfo
  }

  cachedSystemInfo = await invoke('get_system_info')
  cacheTime = now
  return cachedSystemInfo
}
```

---

## 错误处理

### 分层错误处理

```
┌─────────────────────┐
│   用户界面           │  显示友好错误消息
├─────────────────────┤
│   API 层            │  捕获和转换错误
├─────────────────────┤
│   Tauri Command     │  Result<T, String>
├─────────────────────┤
│   Service 层        │  CcrError
├─────────────────────┤
│   底层操作           │  std::io::Error
└─────────────────────┘
```

**错误传播：**
```rust
// Service 层
fn load_config() -> Result<Config, CcrError> {
    let content = fs::read_to_string(path)
        .map_err(|e| CcrError::ConfigError(e.to_string()))?;
    Ok(parse_config(content)?)
}

// Command 层
#[tauri::command]
async fn load_config_command() -> Result<Config, String> {
    load_config().map_err(|e| e.to_string())
}
```

---

## 部署架构

### 跨平台支持

CCR Desktop 支持三大平台，使用统一代码库：

```
┌────────────────────────────────────────┐
│      统一代码库 (Vue 3 + Rust)          │
└─────────┬──────────┬────────────┬──────┘
          │          │            │
    ┌─────▼────┐ ┌──▼────┐ ┌────▼─────┐
    │  macOS   │ │ Linux │ │ Windows  │
    │          │ │       │ │          │
    │  .app    │ │ .deb  │ │  .msi    │
    │  .dmg    │ │.AppImage│ │  .exe  │
    └──────────┘ └───────┘ └──────────┘
```

---

## 扩展性设计

### 添加新 Command

1. 在 `src/commands/mod.rs` 添加函数
2. 在 `src/main.rs` 注册
3. 在前端 `api/index.ts` 封装
4. 在 UI 中调用

### 添加新功能模块

```typescript
// src/features/backup-scheduler/
├── BackupScheduler.vue
├── api.ts
├── types.ts
└── utils.ts
```

模块化设计，易于维护和测试！(^_^)b

---

**Made with ❤️ by 哈雷酱**

哼，这份架构文档可是本小姐精心绘制的呢！(￣▽￣)／
从整体架构到细节实现，从数据流到安全策略，全都讲得清清楚楚！
笨蛋你要是还看不懂这么详细的文档...(,,><,,)
