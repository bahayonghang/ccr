# 技术设计:Grok 平台首页与前端接线

前置阅读:父任务 design.md(D6/D7/D8/D9)、`research/frontend-platform-patterns.md`。参照物:`CodexView.vue` + `useCodexDashboard.ts`(仪表盘骨架)、`GeminiCliView.vue`(轻量板块)。
修订记录:2026-08-01 依据 Codex 审阅修订(overview 无 version 字段、Local-only 横幅、占位文件所有权、i18n 脚本路径)。

## 1. 页面结构(GrokView.vue)

```
<GrokView>                                 // defineOptions({ name: 'GrokView' })
├── Local-only 横幅(仅非 local 环境或 unsupported_environment 信封时渲染,替代下方数据区)
├── 头部 hero:SIcon 平台标识 + 标题/副标题 + meta chips
│     chips: 版本(CliVersionEntry.status 四态) | 当前 profile | auth_mode | drift/unsafe 警示(条件)
├── readiness grid(3 × Card variant="glass")
│     安装状态 / Profiles / Config —— { tone, title, detail, action? }
├── action console:primaryAction 大按钮 + nextActions 列表(≤3)
├── 管理入口 compact list:Profiles、Settings(RouterLink + 图标 + 描述)
└── 常用命令 copy 列表(copyText util,复用既有样式)
```

加载态:首载 skeleton;失败 `EmptyState` + 重试按钮;刷新失败保留旧数据 + toast(与 useCodexDashboard 的 loadError/refreshError 二分一致)。

## 2. useGrokDashboard composable

仿 `useCodexDashboard` 简化(无 usage、无 auth 详情):

- 模块级共享 state + TTL:overview 30s、version 60s;inflight Promise 去重;`refresh(force)`。
- 数据源:`grokApi.getGrokDashboardOverview({force})`(响应含 `activation` 四态,**无 version 字段**)、`getCliVersion({tool:'grok', timeoutMs:1500, force})`(复用 `api/runtime/system.ts`,不新增;**仅 local 环境调用**)。
- 环境门:先 `getCurrentEnvironment()`;非 local → `localOnly` 状态,页面渲染横幅,不发起后续调用。
- 派生:`readinessItems`(3 项)、`nextActions`(优先级:not_installed > profiles_total===0 > activation!==active(含 drifted 引导) > 默认)、`primaryAction`、`versionChip` 文案 key、`activationChip`(drifted/unsafe 警示)。

## 3. 接线清单(唯一动公共配置的子任务)

| 文件                                                    | 改动                                                                                                                                                                                                                                                                                                          |
| ------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src/router/index.ts`                                   | `grok`(depth 1, group 'grok', `cache:true, cacheKey:'GrokView'`)、`grok-profiles`、`grok-settings`(depth 2, group 'grok');**占位策略**:profiles/settings 路由组件先指向轻量占位视图 `views/grok/GrokPlaceholderView.vue`(一张 Card + "开发中" i18n 文案 + 返回首页),后续子任务替换 import 即可,路由结构不再变 |
| `src/config/mainLayoutShell.ts`                         | navSections modules 组加 `{to:'/grok', labelKey:'nav.grok', icon:'Zap'(实现时从 SIcon 集内选), iconClass:'text-platform-grok/90'}`;RouteTitleMap 加 `grok`/`grok-profiles`/`grok-settings`;GroupTitleMap 加 `grok`                                                                                            |
| `src/config/moduleSubnav.ts`                            | `grok: [首页, Profiles, Settings]`(labelKey 用 `nav.*` 通用 key)                                                                                                                                                                                                                                              |
| `tailwind.config.ts` + `src/styles/`(主题 CSS var 文件) | `platform.grok`;色值取 x.ai 品牌系(近黑主色,深浅主题各配一档;具体 RGB 实现时与既有 `--color-platform-*` 对齐亮度)                                                                                                                                                                                             |
| `src/config/platformCapabilities.ts`                    | grok 条目(`supportsProfiles: true` 等按现有字段填)                                                                                                                                                                                                                                                            |
| `src/types/grok.ts` + `src/types/index.ts`              | 以 `src/types/generated/grok/` ts-rs 产物为准做 re-export 或薄别名(`GrokProfileDto`/`GrokProfilesResponse`(含 activation)/`GrokDashboardOverview`/settings 响应与 patch 类型)                                                                                                                                 |
| `src/api/domains/grok.ts` + `src/api/index.ts`          | domain 骨架:`getGrokDashboardOverview`;预留分区注释(profiles 区/settings 区由后续子任务填充);运行时校验 helper `isGrokProfilesResponse` 式样                                                                                                                                                                  |
| i18n zh/en                                              | `nav.grok`;`grok.overview.*`、`grok.dashboard.{header,readiness,actions,management,statusLabels,empty,error}`、`grok.states.*`;占位页 `grok.placeholder.*`                                                                                                                                                    |

## 4. 关键取舍

- 版本探测走通用 `getCliVersion`(后端子任务已把 grok 加入 `CLI_VERSION_TOOLS`);overview 响应已按父 design D6 移除 version 字段,首页版本状态只有这一个来源。
- 占位视图单独一个文件而不是内联 redirect:保证 subnav/面包屑在子页可先行验证。**占位文件由父任务集成评审删除**(profiles/settings 子任务只替换各自路由 import,不删文件)。
- 平台色不确定品牌观感时,先用中性灰紫过渡——色值调整是一行 CSS var,不阻塞。
- i18n 校验脚本位于 `ccr-ui/scripts/check-i18n.mjs`,须在 `ccr-ui/` 目录下执行。

## 5. 回滚

移除路由三条 + 三个 map 条目 + subnav 条目 + i18n 键 + 新增文件;不触碰其他平台代码。
