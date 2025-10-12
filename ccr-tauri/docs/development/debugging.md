# 调试技巧

哼，遇到 Bug 不要慌！让本小姐教你如何优雅地调试～ (￣▽￣)ゞ

## 调试工具概览

CCR Desktop 提供多层次的调试工具：

```
┌─────────────────────────────────────────┐
│   🎨 前端调试                             │
│   • Vue DevTools                        │
│   • Browser DevTools                    │
│   • Vite Dev Server                     │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│   🔗 IPC 调试                            │
│   • Tauri DevTools                      │
│   • Command Logging                     │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│   🦀 后端调试                             │
│   • env_logger                          │
│   • dbg! 宏                              │
│   • Rust Debugger (lldb/gdb)            │
└─────────────────────────────────────────┘
```

---

## 前端调试

### 1. 浏览器开发者工具

#### 打开方式

**macOS:**
```
Cmd + Option + I
```

**Windows/Linux:**
```
Ctrl + Shift + I
```

或者右键点击应用窗口，选择「检查元素」。

#### 常用功能

**控制台 (Console)**
```javascript
// 查看所有配置
console.log(configs.value)

// 查看当前配置
console.log(currentConfig.value)

// 检查系统信息
console.log(systemInfo.value)
```

**网络面板 (Network)**
- 虽然 Tauri 不走 HTTP，但可以看到资源加载情况
- 检查 CSS、JS 文件是否正确加载

**元素面板 (Elements)**
- 查看 DOM 结构
- 实时修改 CSS 样式
- 检查元素的事件监听器

**源代码面板 (Sources)**
- 设置断点
- 单步调试
- 查看变量值

---

### 2. Vue DevTools

#### 安装

**浏览器扩展：**
- [Chrome](https://chrome.google.com/webstore/detail/vuejs-devtools/)
- [Firefox](https://addons.mozilla.org/en-US/firefox/addon/vue-js-devtools/)
- [Edge](https://microsoftedge.microsoft.com/addons/detail/vuejs-devtools/)

**独立应用：**
```bash
npm install -g @vue/devtools
vue-devtools
```

#### 使用 Vue DevTools

**1. 查看组件树**

在 DevTools 的 **Vue** 标签中查看组件层级：

```
App (root)
├── data
│   ├── configs: Array[5]
│   ├── currentConfig: Object
│   ├── filterType: "all"
│   └── theme: "light"
└── computed
    └── filteredConfigs: Array[3]
```

**2. 检查响应式数据**

点击组件可以查看其所有响应式数据。

**3. 时间旅行调试**

查看数据的历史变化，回溯到之前的状态。

**4. 性能分析**

查看组件渲染性能，找出性能瓶颈。

---

### 3. Console.log 调试

#### 基础日志

```typescript
// 简单日志
console.log('当前配置:', currentConfig.value)

// 带标签的日志
console.log('[loadData] 开始加载数据...')

// 表格输出
console.table(configs.value)
```

#### 进阶日志

```typescript
// 分组日志
console.group('配置切换流程')
console.log('1. 验证配置存在')
console.log('2. 调用 switchConfig')
console.log('3. 重新加载数据')
console.groupEnd()

// 计时器
console.time('loadData')
await loadData()
console.timeEnd('loadData')  // loadData: 234ms

// 条件日志
console.assert(configs.value.length > 0, '配置列表为空！')
```

#### 样式化日志

```typescript
console.log('%c Success! ', 'background: #10b981; color: white; padding: 2px 5px; border-radius: 3px;')
console.log('%c Error! ', 'background: #ef4444; color: white; padding: 2px 5px; border-radius: 3px;')
```

---

### 4. 断点调试

#### 在代码中设置断点

```typescript
const handleSwitch = async (configName: string) => {
  debugger  // 👈 程序会在这里暂停

  try {
    const message = await switchConfig(configName)
    console.log(message)
  } catch (error) {
    console.error('切换失败:', error)
  }
}
```

#### 在 DevTools 中设置断点

1. 打开 **Sources** 面板
2. 找到 `App.vue` 文件
3. 点击行号设置断点
4. 触发相关操作
5. 程序会在断点处暂停

#### 条件断点

右键行号，选择「Add conditional breakpoint」：

```javascript
configName === 'anthropic'  // 只在切换到 anthropic 时暂停
```

---

### 5. 错误追踪

#### 全局错误处理

在 `main.ts` 中添加全局错误处理：

```typescript
const app = createApp(App)

app.config.errorHandler = (err, instance, info) => {
  console.error('全局错误:', err)
  console.error('组件:', instance)
  console.error('错误信息:', info)
}

app.mount('#app')
```

#### Promise 错误捕获

```typescript
window.addEventListener('unhandledrejection', (event) => {
  console.error('未处理的 Promise 拒绝:', event.reason)
})
```

---

## 后端调试

### 1. 日志输出

#### 基础日志

CCR Tauri 使用 `env_logger`，支持多个日志级别：

```rust
use log::{trace, debug, info, warn, error};

#[tauri::command]
pub async fn switch_config(name: String) -> Result<String, String> {
    info!("开始切换配置: {}", name);

    match perform_switch(&name) {
        Ok(_) => {
            info!("切换配置成功: {}", name);
            Ok(format!("✅ 成功切换到配置: {}", name))
        }
        Err(e) => {
            error!("切换配置失败: {} - {}", name, e);
            Err(e.to_string())
        }
    }
}
```

#### 设置日志级别

**环境变量方式：**

```bash
# 显示所有日志
RUST_LOG=trace cargo tauri dev

# 只显示 info 及以上级别
RUST_LOG=info cargo tauri dev

# 针对特定模块
RUST_LOG=ccr_tauri=debug,ccr=info cargo tauri dev
```

**代码方式（已在 main.rs 配置）：**

```rust
env_logger::Builder::from_env(
    env_logger::Env::default().default_filter_or("info")
).init();
```

#### 日志级别说明

| 级别    | 用途                 | 示例                              |
| ------- | -------------------- | --------------------------------- |
| `trace` | 最详细的调试信息     | 函数参数、返回值                  |
| `debug` | 调试信息             | 中间变量、执行流程                |
| `info`  | 一般信息             | 操作开始、完成                    |
| `warn`  | 警告信息             | 配置缺失、性能问题                |
| `error` | 错误信息             | 操作失败、异常情况                |

---

### 2. dbg! 宏调试

#### 基础用法

```rust
let config_name = "anthropic";
dbg!(config_name);  // [src/commands/mod.rs:150] config_name = "anthropic"

let configs = load_configs()?;
dbg!(&configs);     // 输出 configs 的 Debug 表示
```

#### 表达式调试

```rust
let result = dbg!(expensive_operation());  // 输出结果并返回
```

#### 多个变量

```rust
dbg!(config_name, &section, is_valid);
```

---

### 3. Rust Debugger

#### 使用 VS Code 调试

**1. 安装扩展：**
- [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb) (macOS/Linux)
- [C/C++](https://marketplace.visualstudio.com/items?itemName=ms-vscode.cpptools) (Windows)

**2. 配置 launch.json：**

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Tauri Debug",
      "cargo": {
        "args": [
          "build",
          "--manifest-path=./Cargo.toml",
          "--no-default-features"
        ]
      },
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

**3. 设置断点：**

在 Rust 代码中点击行号左侧设置断点。

**4. 启动调试：**

按 `F5` 或点击「开始调试」按钮。

---

### 4. 单元测试调试

#### 编写测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_switch_config() {
        let result = switch_config("anthropic".to_string());
        assert!(result.is_ok());

        let message = result.unwrap();
        assert!(message.contains("成功切换"));
    }
}
```

#### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_switch_config

# 显示输出
cargo test -- --nocapture

# 显示详细日志
RUST_LOG=debug cargo test -- --nocapture
```

---

## IPC 调试

### 1. Tauri DevTools

Tauri 2.0 内置了 DevTools 支持（仅在开发模式下）。

#### 启用 DevTools

在 `Cargo.toml` 中确认已启用：

```toml
[dependencies]
tauri = { version = "2", features = ["devtools"] }
```

#### 打开 DevTools

**方法 1：代码触发**
```rust
#[tauri::command]
async fn open_devtools(window: tauri::Window) {
    window.open_devtools();
}
```

**方法 2：快捷键**
- macOS: `Cmd + Option + I`
- Windows/Linux: `F12`

---

### 2. Command 日志

#### 在 Command 中添加日志

```rust
#[tauri::command]
pub async fn switch_config(name: String) -> Result<String, String> {
    log::info!("📥 收到切换配置请求: {}", name);

    let start = std::time::Instant::now();

    // 执行操作...

    let duration = start.elapsed();
    log::info!("⏱️  切换配置耗时: {:?}", duration);

    Ok(format!("✅ 成功切换到配置: {}", name))
}
```

---

### 3. 跟踪 IPC 消息

#### 前端日志

在 API 层添加日志包装：

```typescript
export async function switchConfig(name: string): Promise<string> {
  console.log(`📤 调用 switch_config: ${name}`)

  try {
    const result = await invoke('switch_config', { name })
    console.log(`📥 收到响应:`, result)
    return result
  } catch (error) {
    console.error(`❌ 调用失败:`, error)
    throw error
  }
}
```

---

## 常见问题排查

### 问题 1: 应用启动失败

**现象：**
```
Error: Failed to run tauri application
```

**排查步骤：**

1. **检查依赖**
   ```bash
   cargo check
   cd src-ui && npm install
   ```

2. **查看详细错误**
   ```bash
   cargo tauri dev --verbose
   ```

3. **清理缓存**
   ```bash
   cargo clean
   rm -rf target/
   cd src-ui && rm -rf node_modules/ dist/
   ```

4. **重新安装**
   ```bash
   cargo build
   cd src-ui && npm install
   ```

---

### 问题 2: 前端资源加载失败

**现象：**
```
Failed to load resource: net::ERR_FILE_NOT_FOUND
```

**排查步骤：**

1. **检查 Vite 配置**
   ```typescript
   // src-ui/vite.config.ts
   export default defineConfig({
     base: './',  // 👈 确保使用相对路径
   })
   ```

2. **检查 Tauri 配置**
   ```json
   {
     "build": {
       "frontendDist": "../src-ui/dist"  // 👈 确保路径正确
     }
   }
   ```

3. **重新构建前端**
   ```bash
   cd src-ui
   npm run build
   ```

---

### 问题 3: Command 调用失败

**现象：**
```javascript
Error: Command not found
```

**排查步骤：**

1. **检查 Command 是否注册**
   ```rust
   // src/main.rs
   .invoke_handler(tauri::generate_handler![
       commands::your_command,  // 👈 确保已添加
   ])
   ```

2. **检查函数签名**
   ```rust
   #[tauri::command]  // 👈 确保有这个属性
   pub async fn your_command() -> Result<String, String> {
       // ...
   }
   ```

3. **检查导出**
   ```rust
   // src/commands/mod.rs
   pub async fn your_command() -> Result<String, String> {
       // 👈 确保是 pub
   }
   ```

---

### 问题 4: 类型不匹配

**现象：**
```
TypeError: Cannot read property 'xxx' of null
```

**排查步骤：**

1. **检查后端返回类型**
   ```rust
   #[derive(Serialize)]
   pub struct ConfigInfo {
       pub name: String,
       pub base_url: Option<String>,  // 👈 注意 Option
   }
   ```

2. **检查前端类型定义**
   ```typescript
   interface ConfigInfo {
     name: string
     base_url: string | null  // 👈 匹配 Option<String>
   }
   ```

3. **添加空值检查**
   ```typescript
   if (config.base_url) {
     console.log(config.base_url)
   }
   ```

---

### 问题 5: 文件权限错误

**现象：**
```
Error: Permission denied
```

**排查步骤：**

1. **检查 capabilities 配置**
   ```json
   {
     "permissions": [
       "core:path:default",
       "core:path:allow-home-dir"
     ]
   }
   ```

2. **检查文件路径**
   ```rust
   let home = dirs::home_dir().ok_or("无法获取用户目录")?;
   let config_path = home.join(".ccs_config.toml");
   ```

3. **检查文件存在性**
   ```rust
   if !config_path.exists() {
       return Err("配置文件不存在".to_string());
   }
   ```

---

## 性能分析

### 1. 前端性能

#### 使用 Performance API

```typescript
const startTime = performance.now()

await loadData()

const endTime = performance.now()
console.log(`加载数据耗时: ${endTime - startTime}ms`)
```

#### Vue DevTools 性能分析

在 Vue DevTools 的 **Performance** 标签中：
- 开始录制
- 执行操作
- 停止录制
- 查看组件渲染时间

---

### 2. 后端性能

#### 使用 std::time

```rust
use std::time::Instant;

#[tauri::command]
pub async fn expensive_operation() -> Result<String, String> {
    let start = Instant::now();

    // 执行操作...

    let duration = start.elapsed();
    log::info!("操作耗时: {:?}", duration);

    Ok("完成".to_string())
}
```

---

## 调试技巧总结

### ✅ 最佳实践

1. **分层调试**：从前端到后端逐层排查
2. **添加日志**：在关键位置添加详细日志
3. **使用断点**：对于复杂逻辑使用断点调试
4. **单元测试**：编写测试用例验证功能
5. **版本控制**：使用 Git 追踪代码变更

### 🛠️ 调试工具箱

| 层级     | 工具                      | 用途                |
| -------- | ------------------------- | ------------------- |
| 前端     | Vue DevTools              | 组件和状态调试      |
| 前端     | Browser DevTools          | DOM 和网络调试      |
| IPC      | Tauri DevTools            | IPC 消息调试        |
| 后端     | env_logger                | 运行时日志          |
| 后端     | dbg! 宏                   | 快速变量输出        |
| 后端     | Rust Debugger             | 断点调试            |
| 性能     | Performance API           | 前端性能分析        |
| 性能     | Instant                   | 后端性能分析        |

---

**Made with ❤️ by 哈雷酱**

哼,这份调试指南可是本小姐的实战经验总结呢！(￣▽￣)／
从前端到后端，从日志到断点，所有调试技巧都教给你了～
遇到 Bug 不要慌，按照本小姐的方法逐步排查，肯定能解决！(^_^)b
