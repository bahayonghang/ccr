# 项目结构

让本小姐带你深入了解 CCR Desktop 的项目结构！(￣▽￣)／

## 总体目录结构

```
ccr-tauri/
├── src/                      # 🦀 Rust 后端代码
│   ├── main.rs               # 应用入口
│   ├── lib.rs                # 库入口
│   └── commands/             # Tauri Commands
│       └── mod.rs            # 命令定义和实现
│
├── src-ui/                   # 🎨 Vue 前端代码
│   ├── src/
│   │   ├── main.ts           # Vue 应用入口
│   │   ├── App.vue           # 根组件
│   │   ├── api/              # API 封装层
│   │   │   └── index.ts      # Tauri Commands 封装
│   │   ├── types/            # TypeScript 类型定义
│   │   │   └── index.ts      # 数据模型接口
│   │   └── style.css         # 全局样式
│   ├── index.html            # HTML 模板
│   ├── vite.config.ts        # Vite 配置
│   ├── tsconfig.json         # TypeScript 配置
│   └── package.json          # 前端依赖
│
├── docs/                     # 📚 VitePress 文档
│   ├── .vitepress/
│   │   └── config.ts         # 文档配置
│   ├── index.md              # 首页
│   ├── guide/                # 使用指南
│   ├── architecture/         # 架构文档
│   ├── api/                  # API 参考
│   └── development/          # 开发指南（本文档）
│
├── Cargo.toml                # Rust 依赖配置
├── tauri.conf.json           # Tauri 配置
├── build.rs                  # 构建脚本
├── capabilities/             # Tauri 2.0 权限配置
│   └── default.json          # 默认权限
└── README.md                 # 项目说明

```

---

## 后端结构 (Rust)

### src/main.rs

**职责：** Tauri 应用入口，初始化和配置

**关键内容：**
```rust
fn main() {
    // 1. 初始化日志
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    // 2. 构建 Tauri 应用
    tauri::Builder::default()
        // 3. 注册插件
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        // 4. 注册所有 Commands
        .invoke_handler(tauri::generate_handler![
            commands::list_configs,
            commands::switch_config,
            // ... 其他命令
        ])
        // 5. 运行应用
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败！");
}
```

::: tip 提示
所有新增的 Command 都必须在 `invoke_handler` 中注册！
:::

---

### src/lib.rs

**职责：** 库入口，导出公共模块

```rust
pub mod commands;

// 复用 CCR 核心库的服务层
pub use ccr::services::{
    ConfigService,
    SettingsService,
    HistoryService,
};
```

---

### src/commands/mod.rs

**职责：** 定义所有 Tauri Commands 和数据模型

**架构层次：**
```
┌─────────────────────────────┐
│   Tauri Commands (mod.rs)   │  ← 前端接口层
├─────────────────────────────┤
│   CCR Services               │  ← 业务逻辑层
│   • ConfigService            │
│   • SettingsService          │
│   • HistoryService           │
├─────────────────────────────┤
│   CCR Managers               │  ← 数据访问层
│   • ConfigManager            │
│   • SettingsManager          │
│   • HistoryManager           │
└─────────────────────────────┘
```

**模块划分：**

1. **数据模型** (Lines 14-63)
   ```rust
   pub struct ConfigInfo { ... }     // 配置信息
   pub struct ConfigList { ... }     // 配置列表
   pub struct HistoryEntry { ... }   // 历史记录
   pub struct BackupInfo { ... }     // 备份信息
   pub struct SystemInfo { ... }     // 系统信息
   ```

2. **配置管理 Commands** (Lines 69-307)
   - `list_configs` - 列出所有配置
   - `get_current_config` - 获取当前配置
   - `get_config` - 获取指定配置
   - `switch_config` - 切换配置
   - `create_config` - 创建新配置
   - `update_config` - 更新配置
   - `delete_config` - 删除配置
   - `import_config` - 导入配置
   - `export_config` - 导出配置
   - `validate_all` - 验证所有配置

3. **历史记录 Commands** (Lines 310-332)
   - `get_history` - 获取操作历史

4. **备份管理 Commands** (Lines 335-363)
   - `list_backups` - 列出所有备份
   - `restore_backup` - 恢复备份

5. **系统信息 Commands** (Lines 366-387)
   - `get_system_info` - 获取系统信息

---

## 前端结构 (Vue 3 + TypeScript)

### src-ui/src/main.ts

**职责：** Vue 应用初始化

```typescript
import { createApp } from 'vue'
import './style.css'
import App from './App.vue'

createApp(App).mount('#app')
```

简洁优雅，本小姐最喜欢这种风格！(^_^)b

---

### src-ui/src/App.vue

**职责：** 主应用组件，包含所有 UI 逻辑

**组件结构：**
```
App.vue (1355 行)
├── <script setup lang="ts">      # 逻辑层 (Lines 1-168)
│   ├── 导入依赖和类型
│   ├── 响应式状态定义
│   ├── 计算属性 (filteredConfigs, etc.)
│   ├── 生命周期钩子 (onMounted)
│   └── 事件处理函数
│
├── <template>                     # 视图层 (Lines 170-835)
│   ├── 顶部导航栏
│   ├── 主内容区
│   │   ├── 左侧边栏 (系统信息)
│   │   ├── 中间主区域 (配置列表/历史记录)
│   │   └── 右侧边栏 (导航)
│   └── 模态对话框 (添加/编辑配置)
│
└── <style>                        # 样式层 (Lines 837-1355)
    ├── CSS 变量定义
    ├── 布局样式
    ├── 组件样式
    └── 主题切换
```

**关键功能模块：**

1. **状态管理** (Lines 7-35)
   ```typescript
   const configs = ref<ConfigInfo[]>([])
   const currentConfig = ref<ConfigInfo | null>(null)
   const history = ref<HistoryEntry[]>([])
   const systemInfo = ref<SystemInfo | null>(null)
   const filterType = ref<FilterType>('all')
   const currentTab = ref<'configs' | 'history'>('configs')
   const theme = ref<'light' | 'dark'>('light')
   ```

2. **配置筛选** (Lines 57-73)
   ```typescript
   const filteredConfigs = computed(() => {
     if (filterType.value === 'all') return configs.value
     if (filterType.value === 'official_relay') {
       return configs.value.filter(c =>
         c.provider_type === 'OfficialRelay' ||
         c.provider_type === 'official_relay'
       )
     }
     // ...
   })
   ```

3. **数据加载** (Lines 81-103)
   ```typescript
   const loadData = async () => {
     try {
       const [configList, currentConfigData, historyData, sysInfo] =
         await Promise.all([
           listConfigs(),
           getCurrentConfig(),
           getHistory(50),
           getSystemInfo()
         ])
       // 更新状态...
     } catch (error) {
       console.error('加载数据失败:', error)
     }
   }
   ```

4. **配置切换** (Lines 105-114)
   ```typescript
   const handleSwitch = async (configName: string) => {
     try {
       const message = await switchConfig(configName)
       console.log(message)
       await loadData()
     } catch (error) {
       console.error('切换失败:', error)
     }
   }
   ```

---

### src-ui/src/api/index.ts

**职责：** 封装所有 Tauri Command 调用

**设计原则：**
- **单一职责**：每个函数只调用一个 Command
- **类型安全**：使用 TypeScript 类型约束
- **错误处理**：由调用方处理异常
- **参数转换**：camelCase → snake_case

**API 分组：**

1. **配置管理** (9 个函数)
   ```typescript
   export async function listConfigs(): Promise<ConfigList>
   export async function getCurrentConfig(): Promise<ConfigInfo | null>
   export async function getConfig(name: string): Promise<ConfigInfo>
   export async function switchConfig(name: string): Promise<string>
   export async function createConfig(request: CreateConfigRequest): Promise<string>
   export async function updateConfig(request: UpdateConfigRequest): Promise<string>
   export async function deleteConfig(name: string): Promise<string>
   export async function importConfig(content: string, merge: boolean, backup: boolean): Promise<string>
   export async function exportConfig(includeSecrets: boolean): Promise<string>
   ```

2. **历史记录** (1 个函数)
   ```typescript
   export async function getHistory(limit?: number): Promise<HistoryEntry[]>
   ```

3. **备份管理** (2 个函数)
   ```typescript
   export async function listBackups(): Promise<BackupInfo[]>
   export async function restoreBackup(backupPath: string): Promise<string>
   ```

4. **系统信息** (1 个函数)
   ```typescript
   export async function getSystemInfo(): Promise<SystemInfo>
   ```

::: warning 重要
API 层不应包含业务逻辑，只负责调用 Tauri Commands！
:::

---

### src-ui/src/types/index.ts

**职责：** 定义所有 TypeScript 类型和接口

**类型设计原则：**
- **与后端一致**：所有类型必须匹配 Rust 定义
- **可选字段**：使用 `| null` 而非 `?` 表示可选
- **命名规范**：snake_case 与后端保持一致

**类型定义：**

```typescript
// 配置信息接口
export interface ConfigInfo {
  name: string
  description: string
  base_url: string | null
  auth_token: string | null
  model: string | null
  small_fast_model: string | null
  is_current: boolean
  is_default: boolean
  provider: string | null
  provider_type: string | null      // "official_relay" | "third_party_model"
  account: string | null
  tags: string[] | null
}

// 配置列表接口
export interface ConfigList {
  current_config: string
  default_config: string
  configs: ConfigInfo[]
}

// 历史记录接口
export interface HistoryEntry {
  id: string
  timestamp: string
  operation: string
  from_config: string | null
  to_config: string | null
  actor: string
}

// 创建配置请求
export interface CreateConfigRequest {
  name: string
  description?: string
  base_url?: string
  auth_token?: string
  model?: string
  small_fast_model?: string
  provider?: string
  provider_type?: string
  account?: string
  tags?: string[]
}

// 更新配置请求
export interface UpdateConfigRequest extends CreateConfigRequest {
  old_name: string
  new_name: string
}
```

---

### src-ui/src/style.css

**职责：** 全局样式和主题定义

**样式架构：**

```css
/* 1. CSS 变量定义 (Lines 1-89) */
:root {
  /* 颜色系统 */
  --primary-color: #3b82f6;
  --bg-color: #f9fafb;
  --card-bg: #ffffff;
  /* ... */
}

[data-theme="dark"] {
  /* 深色主题变量 */
  --bg-color: #111827;
  --card-bg: #1f2937;
  /* ... */
}

/* 2. 基础样式 (Lines 91-200) */
* { box-sizing: border-box; }
body { font-family: sans-serif; }

/* 3. 布局样式 (Lines 202-400) */
.app-container { ... }
.main-content { ... }
.three-column-layout { ... }

/* 4. 组件样式 (Lines 402-1000) */
.config-card { ... }
.modal { ... }
.form-group { ... }

/* 5. 工具类 (Lines 1000-1100) */
.text-center { text-align: center; }
.mb-4 { margin-bottom: 1rem; }

/* 6. 响应式设计 (Lines 1100-1355) */
@media (max-width: 1024px) { ... }
```

---

## 配置文件

### Cargo.toml

**关键依赖：**

```toml
[dependencies]
# Tauri 核心
tauri = { version = "2", features = ["devtools"] }
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
tauri-plugin-shell = "2"

# 复用 CCR 核心库
ccr = { path = ".." }

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

### tauri.conf.json

**关键配置：**

```json
{
  "build": {
    "frontendDist": "../src-ui/dist",
    "devUrl": "http://localhost:1420"
  },
  "bundle": {
    "identifier": "com.ccr.desktop",
    "targets": ["app", "dmg", "deb", "nsis"]
  },
  "productName": "CCR Desktop",
  "version": "1.1.2"
}
```

---

### src-ui/vite.config.ts

**Vite 配置：**

```typescript
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    target: ['es2021', 'chrome100', 'safari13'],
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
})
```

---

## 构建产物

### 开发模式

```bash
cargo tauri dev
```

**产物位置：**
- Rust 编译产物：`target/debug/ccr-tauri`
- 前端开发服务器：`http://localhost:1420`

### 生产模式

```bash
cargo tauri build
```

**产物位置：**
- macOS: `target/release/bundle/macos/CCR Desktop.app`
- Linux: `target/release/bundle/deb/ccr-desktop_*.deb`
- Windows: `target/release/bundle/nsis/CCR Desktop_*.exe`

---

## 目录权限

Tauri 2.0 使用 **Capabilities** 系统管理权限：

**capabilities/default.json:**
```json
{
  "identifier": "default",
  "permissions": [
    "core:default",
    "dialog:default",
    "fs:default",
    "shell:default"
  ]
}
```

这些权限允许：
- 📂 访问文件系统
- 💬 显示对话框
- 🐚 执行 shell 命令

---

## 数据流向

```
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│   Vue UI    │─────▶│  API Layer  │─────▶│   Tauri     │
│  (App.vue)  │      │ (api/index) │      │  Commands   │
└─────────────┘      └─────────────┘      └─────────────┘
       ▲                                          │
       │                                          ▼
       │                                   ┌─────────────┐
       │                                   │  CCR Core   │
       │                                   │  Services   │
       │                                   └─────────────┘
       │                                          │
       └──────────────── Result ─────────────────┘
```

---

**Made with ❤️ by 哈雷酱**

哼，这份项目结构文档可是本小姐精心整理的呢！(￣▽￣)／
每个目录、每个文件的职责都写得清清楚楚～
现在你应该对整个项目结构了如指掌了吧，笨蛋！(^_^)b
