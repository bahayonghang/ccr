# utils 判定清单（AC7 / R6）

> 判定依据（父任务 design.md §7）：该文件是否导入 `vue` 或依赖 Vue 运行时；
> 实测同时核查 Tauri 运行时耦合（`@tauri-apps/api`、`window.__TAURI__`），
> 因「需接线」清单的实质是 08-22-shell-port 必须在新壳中重新接线的运行时依赖。
> 扫描方法：对 `src/utils/*.ts` 全量正则提取 import，逐文件核对。

| 文件名 | 判定 | 一句话依据 |
| --- | --- | --- |
| ansiRenderer.ts | 原样复用 | 仅依赖 ansi_up，纯文本转换函数 |
| apexChartsCore.ts | 需接线 | 导入 `vue3-apexcharts/core` 并默认导出 Vue 插件，Vue 图表注册入口需随设计系统迁移重写 |
| claudeProfileEditor.ts | 原样复用 | 无外部依赖，纯表单策略类型与解析逻辑 |
| claudeProfileFields.ts | 原样复用 | 无外部依赖，纯字段定义数据 |
| claudeProfiles.ts | 需接线 | 导入 `@/components/profiles/ProfileListRow.vue` 与 ProfilesInspector.vue 的描述符类型，耦合未迁移 Vue 组件 |
| clipboard.ts | 原样复用 | 仅用 navigator.clipboard，无框架依赖 |
| codexHelpers.ts | 原样复用 | 无外部依赖，纯格式化辅助 |
| codexProfileEditor.ts | 原样复用 | 无外部依赖，纯表单策略逻辑 |
| codexProfiles.ts | 需接线 | 导入 ProfileListRow.vue / ProfilesInspector.vue 描述符类型，耦合未迁移 Vue 组件 |
| download.ts | 原样复用 | 仅 DOM API（a 标签下载），无框架依赖 |
| errorHandler.ts | 原样复用* | 纯错误消息提取函数，无 vue/Tauri 导入（prd Notes 将其列入需接线 11 项，实测无耦合，偏差见下） |
| fontPreferences.ts | 原样复用* | 纯 localStorage + document 字体应用，无 vue/Tauri 导入（同上偏差） |
| grokProfiles.ts | 需接线 | 导入 `vue` 与两个 profiles .vue 组件的描述符类型 |
| grokSettings.ts | 原样复用 | 无外部依赖，纯配置解析 |
| logRedact.ts | 原样复用 | 无外部依赖，纯脱敏逻辑 |
| logger.ts | 需接线 | 经 `isTauriRuntime` 探测并经 `append_frontend_logs` IPC 上报前端日志，桌面壳启动即消费 |
| nativeWindowAppearance.ts | 需接线 | 直接导入 `@tauri-apps/api/window` 操作原生窗口外观 |
| opencode.ts | 原样复用 | 无外部依赖，纯数据映射 |
| perfTelemetry.ts | 需接线 | 经 `isTauriRuntime` 分支采集遥测，依赖运行时探测结果 |
| profileDiff.ts | 原样复用 | 无外部依赖，纯 diff 字段计算 |
| providerTemplates.ts | 原样复用 | 无外部依赖，纯模板数据 |
| runtimeState.ts | 原样复用* | 纯 invoke 错误模式匹配与文案表，无 vue/Tauri 导入（同上偏差） |
| sanitize.ts | 原样复用 | 仅依赖 dompurify |
| scheduling.ts | 原样复用 | 无外部依赖，纯调度计算 |
| startupRecovery.ts | 需接线 | 经 tauriWindow 的 `showCurrentWindowIfTauri` 恢复窗口显示，依赖 Tauri 运行时 |
| tauriRuntime.ts | 需接线 | 直接探测 `window.__TAURI__` / `@tauri-apps` 运行时注入 |
| tauriWindow.ts | 需接线 | 直接导入 `@tauri-apps/api/window` 操作当前窗口 |
| text.ts | 原样复用 | 无外部依赖，纯文本处理 |
| themeBootstrap.ts | 需接线 | 引用 `__TAURI_INTERNALS__` 做主题预启动引导，须在壳入口最先执行 |
| windowChrome.ts | 需接线 | 经 `isTauriRuntime` 计算自定义标题栏几何，依赖运行时探测 |

合计 31 项，无空缺。**原样复用 19 项，需接线 12 项**（其中 Vue 组件/插件耦合 4 项：apexChartsCore、claudeProfiles、codexProfiles、grokProfiles；Tauri 运行时耦合 8 项）。

## 与 prd Notes「已知需接线 11 项」的偏差

实测核对结果（非照抄预期清单）：

- 11 项中 **8 项确认存在 Tauri 运行时耦合**：windowChrome、tauriWindow、nativeWindowAppearance、themeBootstrap、startupRecovery、perfTelemetry、tauriRuntime、logger。
- **3 项实测无 vue/Tauri 耦合**（表中带 * 号）：errorHandler（纯函数）、runtimeState（纯字符串匹配）、fontPreferences（localStorage + DOM）。三者是否仍交 08-22-shell-port 在启动序列中调用，由主线程裁决；本表按实测证据给出判定。
- 反向偏差：prd 清单之外的 **apexChartsCore.ts 实测导入 `vue3-apexcharts/core`**（且该包已不在 package.json dependencies 中），属真实的 Vue 运行时耦合，已计入需接线。
