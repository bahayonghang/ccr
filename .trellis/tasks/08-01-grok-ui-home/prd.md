# Grok 平台首页与前端接线

> 父任务:`08-01-grok-ui-platform`。
> 依赖:`08-01-grok-tauri-commands` 完成(需要 `grok_get_dashboard_overview`、`getCliVersion(tool:'grok')` 与 generated client)。

## Goal

新建 Grok 平台首页(`GrokView.vue`,数据仪表盘型)并完成 Grok 在前端的全部公共接线:路由、侧边栏、子导航、i18n 命名空间、平台色 token、`src/types/grok.ts`、`src/api/domains/grok.ts` 骨架。本任务交付后,Profiles/Settings 两个子任务只需新增各自页面文件与 API 函数,不再动公共配置。

## Requirements

### 首页(父 design §6)

1. 头部:Grok Build 标识、版本 chip(ok/timeout/error/not_installed 四态文案)、当前 profile chip、auth_mode chip(inline_api_key/env_key/session);activation 为 drifted/unsafe_missing_entry_state 时显示警示 chip。
2. Readiness 三卡:安装状态 / Profiles(总数+当前,无 profile 时 warning 引导)/ Config(config.toml 存在性、activation 状态),tone 语义与 Codex 一致。
3. Next actions(最多 3 条,按优先级):未安装 → 安装指引(外链 docs.x.ai);无 profiles → 去 Profiles 页创建;有 profiles 无激活 → 去切换;drifted → 提示到 Profiles 页 off/修复;全就绪 → 打开 Settings。
4. 管理入口两行:Profiles / Settings(RouterLink)。
5. 常用命令 copy 列表:`ccr grok profile list` / `switch <name>` / `off` / `ccr grok profile init`。
6. 数据层 `useGrokDashboard` composable:TTL 缓存(overview 30s / version 60s)+ inflight 去重,`onMounted`/`onActivated` 刷新;skeleton 与 `EmptyState` 兜底,首载失败与刷新失败分开呈现。overview 响应无 version 字段,版本独立走 `getCliVersion({tool:'grok'})`。
7. **Local-only(父 design D9)**:`getCurrentEnvironment()` 非 local 时整页渲染 Local-only 提示横幅、不发起版本探测与 overview 之外的调用;overview 返回 `unsupported_environment` 信封时同样处理。
8. 不含用量面板(grok 无 usage 后端,父任务 out of scope)。

### 接线

8. 路由:`/grok`(depth 1, group 'grok', cache+cacheKey)、`/grok/profiles`、`/grok/settings`(depth 2,组件指向占位视图 `views/grok/GrokPlaceholderView.vue`;**占位文件的删除动作归父任务集成评审**,本任务与后续子任务均不删除该文件)。
9. 导航:mainLayoutShell 三 map、moduleSubnav `grok` 三项、`nav.grok`;侧边栏 modules 组新增 Grok 入口(与 Claude/Codex 并列,平台色图标)。
10. 平台色:tailwind `platform.grok` + 各主题 CSS var。
11. i18n:顶层 `grok: {}`(overview/dashboard/status/actions/states 等本页所需)+ `nav.grok`,中英同步。
12. `src/types/grok.ts`(profile/overview/config 类型,对齐 generated 绑定)+ barrel;`src/api/domains/grok.ts` 骨架(本页所需:overview、version 复用 runtime/system)+ `src/api/index.ts` 导出 `grokApi`。

## Acceptance Criteria

- [ ] 侧边栏出现 Grok 入口,首页可达,面包屑/顶栏标题正确;直达 `/grok/profiles`、`/grok/settings` 不 404(占位)
- [ ] 未安装 grok、已安装无 profile、有激活 profile、drifted 四种状态下,readiness 与 next actions 呈现正确(手工验证,可用 `GROK_HOME` 临时目录模拟;drift = apply 后手改 config.toml)
- [ ] 非 local 环境(或模拟 `unsupported_environment` 响应)显示 Local-only 横幅
- [ ] 页面切换 keep-alive 生效(返回首页不重复闪加载)
- [ ] `just frontend-check-quick` 全绿(含 i18n parity、api-facade-boundary 边界测试);router smoke 覆盖 grok 三条路由解析
- [ ] 桌面与窄视口、明暗主题截图走查通过
- [ ] 无对 `src/api/tauri.ts` 的改动;响应中无凭据明文(依赖后端 DTO,前端不额外持有)

## Out of scope

- Profiles / Settings 页面本体(仅路由占位)
- usage 面板、`platformUsageSpecs.ts` 扩展
