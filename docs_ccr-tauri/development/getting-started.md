# 开发指南

本小姐带你一步步开发 CCR Desktop！(￣▽￣)ゞ

## 环境准备

### 系统要求

| 组件 | 最低版本 | 推荐版本 |
|-----|---------|---------|
| Rust | 1.70.0+ | 最新稳定版 |
| Node.js | 18.0+ | 最新 LTS |
| Tauri CLI | 2.0+ | 最新稳定版 |
| 操作系统 | Windows 10+, macOS 10.15+, Ubuntu 20.04+ | 最新系统 |

### 平台特定依赖

**Windows:**
```powershell
# 安装 Visual Studio Build Tools 2019+
# 或 Visual Studio Community 2019+ (勾选 C++ 开发工具)
```

**macOS:**
```bash
# 安装 Xcode Command Line Tools
xcode-select --install

# 安装 Cocoa 依赖
brew install openssl
```

**Linux:**
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

# Fedora
sudo dnf install -y webkit2gtk4.0-devel \
    gcc \
    gcc-c++ \
    make \
    openssl-devel \
    gtk3-devel \
    libappindicator-gtk3 \
    librsvg2-devel
```

### 安装 Tauri CLI

```bash
# 使用 Cargo 安装
cargo install tauri-cli --version "^2.0.0"

# 或使用 npm (推荐)
npm install -g @tauri-apps/cli@latest

# 或使用 yarn
yarn global add @tauri-apps/cli@latest

# 或使用 pnpm
pnpm add -g @tauri-apps/cli@latest
```

## 项目结构

```
ccr-tauri/
├── src/                    # Rust 后端代码
│   ├── main.rs            # Tauri 应用入口
│   ├── lib.rs             # 库入口
│   └── commands/          # Tauri Commands
│       └── mod.rs         # 命令定义
├── src-ui/                 # Vue 前端代码
│   ├── src/
│   │   ├── main.ts        # 应用入口
│   │   ├── App.vue        # 根组件
│   │   ├── api/           # Tauri API 封装
│   │   │   └── index.ts   # API 函数
│   │   ├── types/         # TypeScript 类型定义
│   │   │   └── index.ts   # 数据模型接口
│   │   └── style.css      # 全局样式
│   ├── index.html         # HTML 模板
│   ├── vite.config.ts     # Vite 配置
│   └── package.json       # 前端依赖
├── Cargo.toml              # Rust 依赖
├── tauri.conf.json         # Tauri 配置
├── justfile                # 任务运行器
└── README.md               # 项目说明
```

## 开发流程

### 1. 克隆项目

```bash
git clone https://github.com/your-username/ccr-tauri.git
cd ccr-tauri
```

### 2. 安装依赖

```bash
# 安装前端依赖
cd src-ui
npm install
# 或
yarn
# 或
pnpm install

# 安装 Rust 依赖 (自动)
cargo build
```

### 3. 开发模式

::: code-group

```bash [桌面模式 (推荐)]
# 使用 Just (推荐)
just dev

# 或直接使用 Tauri CLI
cd src-ui
npm run tauri dev

# 或使用 Cargo
cargo tauri dev
```

```bash [WSL 优化模式]
# WSL 环境专用 (自动启用滚轮修复和图形优化)
just dev-wsl

# 或直接执行脚本
./dev-wsl.sh
```

```bash [Web 调试模式]
# 纯 Web 模式 (无桌面窗口，适合远程开发)
just dev-web

# 访问:
# 前端界面: http://localhost:5173
# 后端 API: http://localhost:3030

# 查看状态
just web-status

# 查看日志
just web-logs
just web-logs-follow  # 实时跟踪

# 停止服务
just stop-web
```

:::

开发模式会启动：
- 前端开发服务器 (热重载)
- Rust 后端进程 (自动重启)
- Tauri 应用窗口 (桌面模式)
- 或浏览器访问 (Web 模式)

::: tip Web 调试模式的优势
Web 调试模式特别适合：
- **WSL 环境**: 图形界面性能受限时的替代方案
- **远程开发**: SSH 到服务器时无需 X11 转发
- **前端调试**: 使用浏览器 DevTools 的完整功能
- **并行开发**: 前后端独立测试和调试

技术实现：
- 前端：Vite 开发服务器 (端口 5173)
- 后端：CCR Web API 服务器 (端口 3030)
- API 适配器：`src-ui/src/api/index.ts` 自动适配双模式
:::

### 4. 构建应用

```bash
# 调试构建
just build

# 发布构建
just release

# 或直接使用 Tauri CLI
cargo tauri build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录。

## 代码规范

### Rust 代码规范

```bash
# 格式化代码
cargo fmt

# 静态检查
cargo clippy

# 运行测试
cargo test
```

**命名规范：**
- 函数和变量：`snake_case`
- 类型（结构体、枚举）：`PascalCase`
- 常量：`SCREAMING_SNAKE_CASE`

**代码组织：**
```rust
// src/commands/mod.rs
use serde::{Deserialize, Serialize};
use tauri::State;

// 数据模型
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigInfo {
    pub name: String,
    pub path: String,
    pub is_current: bool,
}

// Tauri Commands
#[tauri::command]
pub async fn list_configs() -> Result<Vec<ConfigInfo>, String> {
    // 实现
    Ok(vec![])
}
```

### TypeScript 代码规范

```bash
# 格式化代码
npm run format

# 静态检查
npm run lint

# 类型检查
npm run type-check
```

**命名规范：**
- 变量和函数：`camelCase`
- 类和接口：`PascalCase`
- 常量：`UPPER_SNAKE_CASE`
- 文件名：`kebab-case`

**代码组织：**
```typescript
// src/types/index.ts
export interface ConfigInfo {
  name: string;
  path: string;
  isCurrent: boolean;
}

// src/api/index.ts
import { invoke } from '@tauri-apps/api/core';
import type { ConfigInfo } from '../types';

export async function listConfigs(): Promise<ConfigInfo[]> {
  return await invoke<ConfigInfo[]>('list_configs');
}
```

## 添加新功能

### 1. 添加新的 Tauri Command

**步骤：**
1. 在 `src/commands/mod.rs` 添加命令函数
2. 在 `src/main.rs` 注册命令
3. 在前端 `src-ui/src/api/index.ts` 封装 API
4. 在 UI 组件中调用

**示例：添加获取系统信息命令**

```rust
// src/commands/mod.rs
#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();
    
    Ok(SystemInfo {
        os,
        arch,
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub version: String,
}
```

```rust
// src/main.rs
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        commands::list_configs,
        commands::switch_config,
        commands::get_system_info,  // 注册新命令
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

```typescript
// src-ui/src/types/index.ts
export interface SystemInfo {
  os: string;
  arch: string;
  version: string;
}

// src-ui/src/api/index.ts
import { invoke } from '@tauri-apps/api/core';
import type { SystemInfo } from '../types';

export async function getSystemInfo(): Promise<SystemInfo> {
  return await invoke<SystemInfo>('get_system_info');
}
```

### 2. 添加新的前端组件

**步骤：**
1. 在 `src-ui/src/components/` 创建组件文件
2. 在 `App.vue` 中导入和使用
3. 添加必要的样式

**示例：添加系统信息组件**

```vue
<!-- src-ui/src/components/SystemInfo.vue -->
<template>
  <div class="system-info">
    <h3>系统信息</h3>
    <div class="info-item">
      <span class="label">操作系统:</span>
      <span class="value">{{ systemInfo.os }}</span>
    </div>
    <div class="info-item">
      <span class="label">架构:</span>
      <span class="value">{{ systemInfo.arch }}</span>
    </div>
    <div class="info-item">
      <span class="label">版本:</span>
      <span class="value">{{ systemInfo.version }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { getSystemInfo } from '../api';
import type { SystemInfo } from '../types';

const systemInfo = ref<SystemInfo>({
  os: '',
  arch: '',
  version: ''
});

onMounted(async () => {
  try {
    systemInfo.value = await getSystemInfo();
  } catch (error) {
    console.error('获取系统信息失败:', error);
  }
});
</script>

<style scoped>
.system-info {
  background: var(--surface-color);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 16px;
}

.info-item {
  display: flex;
  margin-bottom: 8px;
}

.label {
  font-weight: bold;
  width: 80px;
}

.value {
  color: var(--text-color-secondary);
}
</style>
```

```vue
<!-- src-ui/src/App.vue -->
<template>
  <div class="app">
    <header>
      <h1>CCR Desktop</h1>
    </header>
    
    <main>
      <SystemInfo />
      <!-- 其他组件 -->
    </main>
  </div>
</template>

<script setup lang="ts">
import SystemInfo from './components/SystemInfo.vue';
// 其他导入
</script>
```

## 调试技巧

### 1. 前端调试

- **浏览器开发者工具**: 在开发模式下，可以右键应用选择"检查元素"
- **Vue DevTools**: 安装浏览器扩展，可以查看 Vue 组件树和状态
- **Console 日志**: 使用 `console.log()` 输出调试信息

### 2. 后端调试

- **日志输出**: 使用 `println!` 或 `log` crate 输出日志
- **VS Code 调试**: 安装 `rust-analyzer` 和 `CodeLLDB` 扩展
- **错误处理**: 使用 `Result` 类型处理错误，并通过 `map_err` 转换错误信息

```rust
// 示例：添加详细日志
use log::{info, warn, error};

#[tauri::command]
pub async fn switch_config(name: String) -> Result<String, String> {
    info!("尝试切换到配置: {}", name);
    
    match ccr_core::switch_config(&name) {
        Ok(_) => {
            info!("成功切换到配置: {}", name);
            Ok(format!("✅ 成功切换到配置: {}", name))
        }
        Err(e) => {
            error!("切换配置失败: {}", e);
            Err(format!("切换配置失败: {}", e))
        }
    }
}
```

### 3. IPC 调试

- **网络面板**: 在浏览器开发者工具的网络面板查看 IPC 请求
- **Tauri DevTools**: 安装 Tauri DevTools 扩展，可以查看 IPC 通信

## 性能优化

### 1. 前端优化

- **按需加载**: 使用动态导入延迟加载大型依赖
- **虚拟列表**: 对于长列表使用虚拟滚动
- **防抖节流**: 对频繁操作使用防抖和节流

```typescript
// 示例：搜索防抖
import { ref, watch } from 'vue';
import { debounce } from 'lodash-es';

const searchQuery = ref('');
const debouncedQuery = ref('');

const debouncedSearch = debounce((query: string) => {
  debouncedQuery.value = query;
}, 300);

watch(searchQuery, (newQuery) => {
  debouncedSearch(newQuery);
});
```

### 2. 后端优化

- **异步处理**: 使用 `tokio` 异步处理耗时操作
- **缓存机制**: 缓存频繁访问的数据
- **并发控制**: 使用文件锁避免并发写入冲突

```rust
// 示例：异步处理
#[tauri::command]
pub async fn heavy_operation() -> Result<(), String> {
    tokio::task::spawn(async move {
        // 耗时操作
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }).await.map_err(|e| e.to_string())?;
    
    Ok(())
}
```

## 测试

### 1. Rust 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_name

# 显示测试输出
cargo test -- --nocapture
```

**单元测试示例：**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_configs() {
        let configs = list_configs().await.unwrap();
        assert!(!configs.is_empty());
    }
}
```

### 2. 前端测试

```bash
# 运行单元测试
npm run test

# 运行 E2E 测试
npm run test:e2e
```

**组件测试示例：**
```typescript
// src/components/__tests__/SystemInfo.spec.ts
import { mount } from '@vue/test-utils';
import SystemInfo from '../SystemInfo.vue';

describe('SystemInfo', () => {
  it('renders system information', async () => {
    const wrapper = mount(SystemInfo);
    
    // 等待异步数据加载
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.system-info').exists()).toBe(true);
    expect(wrapper.find('.label').text()).toBe('操作系统:');
  });
});
```

## 构建与发布

### 1. 构建配置

**Tauri 配置文件 (`tauri.conf.json`):**
```json
{
  "productName": "CCR Desktop",
  "version": "1.0.0",
  "identifier": "com.ccr.desktop",
  "build": {
    "beforeBuildCommand": "npm run build",
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "withGlobalTauri": false
  },
  "package": {
    "productName": "CCR Desktop",
    "version": "1.0.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "fs": {
        "all": false,
        "readFile": true,
        "writeFile": true,
        "scope": ["$HOME/.ccs_config.toml", "$HOME/.claude/**"]
      }
    }
  }
}
```

### 2. 代码签名

**Windows:**
```json
// tauri.conf.json
"tauri": {
  "bundle": {
    "windows": {
      "certificateThumbprint": "YOUR_CERTIFICATE_THUMBPRINT",
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.digicert.com"
    }
  }
}
```

**macOS:**
```json
// tauri.conf.json
"tauri": {
  "bundle": {
    "macOS": {
      "entitlements": null,
      "exceptionDomain": "",
      "frameworks": [],
      "providerShortName": null,
      "signingIdentity": "Developer ID Application: Your Name (TEAM_ID)"
    }
  }
}
```

### 3. 自动更新

```rust
// src/main.rs
use tauri_plugin_updater::UpdaterExt;

tauri::Builder::default()
    .plugin(tauri_plugin_updater::Builder::new().build())
    .setup(|app| {
        #[cfg(desktop)]
        {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Some(response) = handle.updater().check().await.ok().flatten() {
                    if let Some(body) = response.download().await.ok().flatten() {
                        let installed = body.install().await;
                        if installed.is_ok() {
                            handle.restart();
                        }
                    }
                }
            });
        }
        Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

## 常见问题

### 1. 构建失败

**问题**: `error: failed to run custom build command`

**解决方案**:
```bash
# 清理构建缓存
cargo clean

# 更新依赖
cargo update

# 重新构建
cargo build
```

### 2. 前端资源加载失败

**问题**: 应用窗口显示空白

**解决方案**:
1. 检查 `tauri.conf.json` 中的 `devUrl` 是否正确
2. 确保前端开发服务器正在运行
3. 检查控制台是否有错误信息

### 3. IPC 通信失败

**问题**: 调用 Tauri Command 返回错误

**解决方案**:
1. 确保命令已在 `main.rs` 中注册
2. 检查命令名称是否正确
3. 查看后端日志获取详细错误信息

### 4. 文件权限问题

**问题**: 无法访问配置文件

**解决方案**:
1. 检查 `tauri.conf.json` 中的文件权限配置
2. 确保文件路径正确
3. 检查文件是否存在

---

## 贡献指南

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

---

**Made with ❤️ by 哈雷酱**

哼，这份开发指南可是本小姐精心编写的呢！(￣▽￣)／
从环境准备到构建发布，从代码规范到调试技巧，全都讲得明明白白！
笨蛋你要是还不会开发...(,,><,,)