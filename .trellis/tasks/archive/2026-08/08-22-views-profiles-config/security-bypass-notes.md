# 安全行为与绕过路径（批次 5）

## AC6 备份

配置切换走现有 `switchConfig`（`src/api` wrapper → `switch_config`）。前端不新增直写路径，不跳过备份。备份与恢复由 `crates/ccr-config` / Tauri 实现。

## AC7 原子写入

配置新增 / 更新 / 删除走 `addConfig` / `updateConfig` / `deleteConfig`。无 `fs`、无直写 `invoke`、无新增 wrapper。中断写入后原文件完整的验证在 Rust 侧（tempfile + rename）。前端验证：调用路径仅现有 wrapper。

## AC5 掩码

- Auth token 输入默认 `type="password"`。
- 模板应用不写入 `auth_token`。
- 日志只报 `getErrorMessage`，不序列化表单。
- smoke：`maskSensitive` 对 token 不保留明文片段。

## 前端绕过路径

未发现本批次新增的直写或绕过路径。`src/api` / `src/config` / `src/configs` 未改。

## profileDiff（R9）

Converter 结果以只读 textarea 展示转换文本，不与 `profileDiff` 共用渲染组件。Profile 差异仍由批次 1 `ProfileDiffRows` 负责。

## 原始编辑器（AC8 分责）

本批次 Configs / Edit 弹层未嵌入 CodeMirror。桥接仍属 `08-22-views-sync-tools`。

## 已知缺陷（不在本任务修）

1. `src/views/OpenCodeProvidersView.vue` 仍 import 已删除的 `ProviderTemplateSelector.vue`。由 `08-22-views-secondary-platforms` 改接到 `features/configs` 的 React 选择器，或经 platform 再导出。
2. AppSettings 写偏好走 utils/api，并用动态 `import('@/shell/stores/shellPreferences')` 同步壳层 store，避免 feature → shell 静态依赖。
