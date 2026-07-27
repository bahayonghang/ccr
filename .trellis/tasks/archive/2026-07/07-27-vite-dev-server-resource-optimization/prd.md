# Vite 开发服务器资源占用优化

## Goal

消除 `ccr-ui` Vite 开发服务器在 Windows 长时间运行时出现的句柄、内存和 CPU 持续增长，使前端开发、Rust 构建、IDE 与 smoke 测试可以同时运行而不导致整机卡顿。

## Background

- 2026-07-27 现场采样显示，Vite PID `68472` 已运行约 24.22 小时，工作集约 2.94 GB、私有内存约 3.93 GB、句柄约 204,000，并持续占用约 1.8 个逻辑核。
- 五秒只读采样中句柄从 203,734 增至 204,087，证明异常不是一次性启动峰值，而是长期资源增长。
- `ccr-ui/src-tauri/target` 当前约有 190,544 个文件；Vite 7.3.6 默认监视整个 `ccr-ui` 根目录，但当前 `vite.config.ts` 没有配置 `server.watch.ignored`。
- `ccr-ui/ref` 另有约 1,439 个镜像文件。`ccr-ui/logs` 会在开发期间持续变化。这些目录都不需要前端 HMR。
- Vite 原生 `server.warmup.clientFiles` 与 `dev-web-warm-start.mjs` 当前读取同一份 `clientFiles` 并重复请求；外层脚本还重复探测路由。
- `dev-web-warm-start.mjs` 在 Windows 只对直接子进程调用 `server.kill()`，而测量脚本已使用 `taskkill /T` 清理完整进程树。
- `dev-web-windows.ps1` 每次启动前删除 `node_modules/.vite`，迫使依赖预构建缓存反复重建。
- smoke 套件当前包含 103 个文件，`vitest.smoke.config.ts` 没有本地 worker 上限。
- 工作区已有无关改动，其中 `ccr-ui/package.json` 的版本从 6.5.3 改为 7.0.0；本任务必须保留该改动。

## Requirements

### R1. 收紧 Vite watcher 边界

- 在 Vite 配置中显式忽略 `src-tauri/target`、`ref` 和开发日志目录。
- 保留 `src/**`、Vite/Tailwind 配置、共享 catalog 数据和 `public/**` 的正常热更新行为。
- 不以关闭整个 watcher 或轮询模式作为默认方案。

### R2. 单一预热所有权

- Vite 原生 `server.warmup.clientFiles` 是模块预热的唯一所有者。
- 外层 warm-start 脚本只负责启动、一次有超时的健康检查和生命周期管理，不再逐个请求 `clientFiles`。
- 预热清单仅包含真实首屏依赖；设置页和其他懒加载路由保持按需转换。

### R3. 完整进程树生命周期

- Ctrl+C、SIGTERM、父脚本异常和测量脚本结束时都必须关闭完整 Vite 进程树。
- Windows 复用 `taskkill /PID <pid> /T /F`；其他平台使用有界等待与信号降级。
- 进程树终止逻辑必须由开发启动脚本和性能测量脚本共用，避免两个实现继续漂移。

### R4. 保留依赖预构建缓存

- 常规 `dev-web` 启动不得删除 `node_modules/.vite`。
- 仅在用户显式设置恢复开关时清理该缓存，并输出清晰提示。

### R5. 限制开发测试资源

- 本地 smoke 测试默认最多使用 2 个 worker；CI 根据可用并行度最多使用 4 个 worker。
- `CCR_TEST_WORKERS` 可显式覆盖 worker 数，但必须校验为正整数并受可用并行度约束。
- 新增开发服务器契约测试时优先使用纯 Node/jsdom-free 逻辑和受控假进程，不把真实 Vite 服务器作为常规 smoke 测试前置条件。

### R6. 建立资源回归门槛

- 提供可重复的 Windows 资源验证路径，采集 Vite PID、CPU、私有内存、工作集、句柄和退出后的监听状态。
- 验证包含冷启动、预热完成、`src-tauri/target` 受控文件变动、短时空闲 soak 和进程树退出。
- 资源验证产生的临时目录必须位于任务专用路径，并在 `finally` 中清理。

### R7. 范围与兼容性

- 不改变生产构建、Tauri 运行时、业务页面行为或公共 API。
- 不升级 Vite/Tailwind/Vitest，也不新增生产依赖；只有本地修复无效时才另开依赖升级任务。
- 不修改或回退本任务之外的 24 项现有工作区改动。

## Acceptance Criteria

- [x] AC1: 解析后的 Vite 开发配置明确忽略 `src-tauri/target/**`、`ref/**` 和 `logs/**`，同时未忽略 `src/**` 与所需共享 catalog 路径。
- [x] AC2: `dev-web-warm-start.mjs` 不再请求 `clientFiles`；每个模块只由 Vite 原生 warmup 预热一次，健康探测具备明确超时。
- [x] AC3: 预热清单中的每个路径都存在、无重复，并覆盖 `/` 首屏的 `main.ts`、`App.vue`、`MainLayout.vue` 和 `DashboardView.vue`，不包含 `AppSettingsView.vue`。
- [x] AC4: 常规 Windows `dev-web` 启动保留 `node_modules/.vite`；显式恢复开关仍可清理缓存。
- [x] AC5: 对启动父进程发送终止信号后 5 秒内，父进程、Vite 子进程和监听端口均消失；测量脚本成功、失败和超时路径同样无遗留进程。
- [x] AC6: Windows 资源验证中，预热后 60 秒空闲及对被忽略 `target` 目录的受控变动期间，句柄相对基线增长不超过 500，私有内存增长不超过 256 MB，且空闲 10 秒平均 CPU 不超过全机 5%。
- [x] AC7: smoke 配置默认本地 worker 数不超过 2，CI 不超过 `min(4, availableParallelism)`，显式合法覆盖生效，非法覆盖安全回退。
- [x] AC8: 针对 watcher 忽略、预热清单、worker 预算和进程树清理的聚焦测试通过；`bun run type-check`、`bun run lint`、`bun run test:smoke`、`just frontend-check-quick` 和 `git diff --check` 通过。
- [x] AC9: 最终差异不包含 `ccr-ui/package.json` 现有版本改动或其他无关工作区改动的回退/重写。

## Out Of Scope

- 重新设计业务页面、首屏视觉或 Tauri 后端。
- 升级 Vite、Tailwind、Vitest、Node/Bun 或引入新的 watcher 库。
- 为所有开发工具建立常驻资源守护进程或自动重启服务。
- 清理现有 `src-tauri/target` 构建产物。

## Technical Notes

- Vite 7.3.6 的默认忽略项包含 `.git`、`node_modules`、`test-results`、cacheDir 和 build outDir，但不包含 Rust `target`。
- 上游 Vite issue #22672 证明 Windows watcher 可进入 CPU/内存 runaway 状态，但其目录删除/同步盘触发条件与本项目不同，因此仅作为风险参考，不作为已确认根因。
- 本任务优先以本地 watcher 范围和资源斜率验证闭环；若仍复现，再单独最小化并提交上游问题。
