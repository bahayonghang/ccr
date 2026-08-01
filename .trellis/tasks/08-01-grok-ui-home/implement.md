# 执行计划:Grok 平台首页与前端接线

修订记录:2026-08-01 依据 Codex 审阅修订(i18n 脚本路径、Local-only、bindings 校验、smoke/截图矩阵、占位所有权)。

## 前置

- [ ] 依赖确认:`08-01-grok-tauri-commands` 已交付且**契约冻结**(`grok_get_dashboard_overview` 可调、`getCliVersion(tool:'grok')` 生效、`src/api/generated/` 与 `src/types/generated/grok/` 产物就绪)
- [ ] 读父任务 design D6/D7/D8/D9 与本任务 design.md;读 `.trellis/spec/ccr-ui/frontend/api-facade-boundary.md`
- [ ] 参照物:`CodexView.vue`、`useCodexDashboard.ts`、`GeminiCliView.vue`、`mainLayoutShell.ts`、`moduleSubnav.ts`

## 步骤(按序)

1. [ ] 类型与 API:`src/types/grok.ts`(re-export 生成物)+ barrel;`src/api/domains/grok.ts`(overview + 分区注释)+ `src/api/index.ts` 导出 `grokApi`
2. [ ] i18n:zh-CN/en-US 同步加 `nav.grok` + `grok.{overview,dashboard,states,placeholder}`;`cd ccr-ui && node scripts/check-i18n.mjs` 即时校验(脚本在 ccr-ui/ 下,勿在仓库根执行)
3. [ ] 平台色:tailwind `platform.grok` + 主题 CSS var
4. [ ] 接线:router 三条(占位视图 `views/grok/GrokPlaceholderView.vue`,**本任务不含删除计划**)、mainLayoutShell 三 map、moduleSubnav、platformCapabilities
5. [ ] `useGrokDashboard.ts`(环境门 → localOnly 态;TTL/inflight;activation 派生)
6. [ ] `GrokView.vue`:Local-only 横幅 + hero/chips(含 drift 警示)+ readiness + actions + 管理入口 + 命令列表;keep-alive `defineOptions({name:'GrokView'})`

## 验证

- [ ] `just frontend-check-quick`(typecheck + lint + smoke:i18n parity、api-facade-boundary);若有 router smoke 测试基建则补 grok 三条路由用例,无则在手工走查覆盖
- [ ] `just tauri-bindings-check`(确认未手改生成物)
- [ ] `bun run tauri dev` 手工走查:侧边栏入口 → 首页四态(未安装/无 profile/激活/drifted,临时 `GROK_HOME` 模拟)→ next action 跳转 → 子页占位 → 返回缓存生效;WSL 环境(若可用)或模拟信封验证 Local-only 横幅
- [ ] 桌面 + 窄视口 × 明暗主题截图走查(平台色对比度)

## 评审门

- 步骤 4 后:导航信息架构(入口位置/分组/命名)截图确认一次
- 完成后:交 `trellis-check`;视觉偏差走 `frontend-quality-reviewer` 复查

## 回滚点

步骤 1-4(接线)与 5-6(页面)分 commit;占位视图保证 profiles/settings 未完成时主干可发布。
