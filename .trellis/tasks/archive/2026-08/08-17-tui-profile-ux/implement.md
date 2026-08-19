# 执行计划 — TUI Profile 界面设计优化

按 WS 顺序执行,每个 WS 完成后跑窄验证再进下一个;每 WS 一个 commit。

> 关键排序约束:WS1 改键位、WS5 改页脚标注,两者中间不能插入其它 commit,否则会留下「页脚宣传的行为与实际不符」的中间态。
> WS3 会删掉 `profile_meta_strings` 及其测试,而 `ui.rs:2463` 的页脚测试同时引用了它——WS5 只改该测试的页脚断言,把 `profile_meta_strings` 相关断言留给 WS3 一并删除。

## Step 1 — WS1 切换安全闭环(P0)

- [ ] `action.rs`:删除 `ApplyAndQuit` 变体(留着会因 never-constructed 触发 clippy dead_code)
- [ ] `app.rs:758`:`Enter` 改映射到 `Action::ApplySelected`,与 `Space`(`app.rs:759`)同义
- [ ] `app.rs:852`:删除 `Action::ApplyAndQuit` 的 dispatch 分支
- [ ] `ui.rs:1993` `last_apply_message`:输出改为「已切换到 {name}」/ "Switched to {name}",失败态保留错误详情
- [ ] `ui.rs:1951` `profile_summary_fields`:新增入参接收 `app.last_applied`,成功/失败时追加第三行反馈字段(tone 分别 Success / Error);`summary_height`(`ui.rs:639`)已自适应,不改布局常量
- [ ] `ui.rs:793` `profile_status_message`:去掉 `last_apply_message` 分支,strip 退化为纯 toast 通道
- [ ] 更新 `app.rs:1682` `map_key_o_is_profile_off`(原断言 `Enter => ApplyAndQuit`)
- [ ] 更新 `ui.rs:2782` `profile_status_strip_surfaces_last_apply_feedback`(反馈已移出 strip)
- [ ] 新增:`Enter` / `Space` 均映射 `ApplySelected` 的键位单测
- [ ] 新增:80×20、100×30、140×32 三档 Focus 反馈渲染断言;Wide 下反馈文案只出现一次
- 验证:`cargo test -p ccr-tui -- --test-threads=1`

## Step 2 — WS5 页脚标注(紧随 WS1,标注依赖新键位)

- [ ] `ui.rs:2211-2255` `footer_hints_for_width`:两条宽度分支都改为 `Enter/Space apply`、`o deactivate` /「o 解绑」;页脚不再出现任何 apply+quit 字样
- [ ] 紧凑分支(`width < 90`)保留 `PgUp/PgDn details` 与 `Ctrl+L language`(spec i18n 契约第 175-177 行)
- [ ] 更新 `ui.rs:2463` `footer_hint_mentions_reverse_tab_switching` 的 `"o off"` 断言(其中的 `profile_meta_strings` 断言留到 Step 4 一并删)
- [ ] 更新 `ui.rs:2482` `footer_omits_profile_off_on_auth_tabs` 的 `"o off"` 断言
- [ ] 更新 `ui.rs:3397` `wide_profile_draw_shows_shortcuts_only_in_global_footer`:`matches("Enter apply").count() == 1` 的判据要换成新页脚文案(否则会以 count==0 失败),同时更新其中的 "Applied successfully" 断言以匹配 Focus 新文案
- 验证:`cargo test -p ccr-tui -- --test-threads=1`

## Step 3 — WS2 banner + 图例 + 文案(P1)

- [ ] `ui.rs:132-199` banner 改常驻 current chip:数据读 `CodexRuntimeSummary` 的类型化字段(`mode` / `current_profile_name` / `login_state`),参照 `codex_runtime_mode_label`(`ui.rs:1690`)的写法用 `tui_text!` 组装
- [ ] model 从当前生效 profile 的 `ProfileConfig` 取(`current_profiles()` 里按 `is_current` 定位 → `profile_configs` 查 `config.model`);取不到就只显示名字,不留空占位
- [ ] **不得**给 `ui.rs:1714` `localize_codex_runtime_text` 的 replace 链新增条目
- [ ] `ui.rs:157` / `166` / `193`:「Active driver」「Control plane」人话化(双语)
- [ ] `ui.rs:182`:Auth 值由 `theme::success()` 改中性 subtext
- [ ] `ui.rs:1945` 图例补 `○ available`;`ui.rs:335` 删除 `✓` current_tag
- [ ] 新增:未绑定/绑定两态 banner 文案测试;`✓` 不出现的断言
- 验证:`cargo test -p ccr-tui -- --test-threads=1`

## Step 4 — WS3 删除 Selection 面板(P1,Wide 视口)

- [ ] 删除 `render_profile_meta_panel`(`ui.rs:567-588`)与 `profile_meta_strings`(`ui.rs:1921-1947`)
- [ ] 改写 `profile_list_rail_layout`(`ui.rs:227-234`)与 `render_profile_list_rail`(`ui.rs:477-482`);Wide 分支(`ui.rs:468`)把整个 list_area 给列表面板
- [ ] 图例移入列表面板 block 的 `title_bottom`(`ui.rs:534-541`)
- [ ] 确认 `render_empty_state`(`ui.rs:543`)走的空态/错误态不带出图例
- [ ] 删除 `ui.rs:2569`、`ui.rs:2722`、`ui.rs:2824` 三个 Selection 面板测试;`ui.rs:2475` 的 `profile_meta_strings` 断言一并删除
- [ ] 新增:列表 `title_bottom` 含 ●/▶/○ 的渲染测试;空态无图例的测试
- [ ] 三档 viewport 布局核对(重点看 Compact 80 列图例是否与顶部标题冲突或溢出)
- 验证:`cargo test -p ccr-tui -- --test-threads=1` + 手动宽/窄终端运行核对

## Step 5 — WS4 详情空值折叠 + 布尔中性化(P2)

- [ ] `DetailField` 增加显式 `unset` 标记,由 `opt_text`(`ui.rs:1814`)/ `bool_text`(`ui.rs:1822`)/ `tags_text`(`ui.rs:1830`)/ `optional_tone`(`ui.rs:932`)所在的 builder 侧设置;**渲染层不得比较字符串 `"-"`**
- [ ] 折叠实现在四个 builder 共用的组装路径上(codex `ui.rs:1133` / claude `ui.rs:1336` / grok / generic `ui.rs:940` 行为一致)
- [ ] 折叠态渲染「N 项未设置」/ "N unset" 一行 muted 摘要;展开状态存 `App`,新增 `x` 键切换(`x` 已确认未被占用)
- [ ] 展开键只在全局 footer 标注,详情面板内不写键位提示(spec 第 111 行)
- [ ] `ui.rs:1302-1305` `requires_openai` 的 `Some(false)` 由 `Warning` 改 `Muted`
- [ ] 新增:折叠/展开渲染测试,四个 builder 各覆盖一次
- [ ] 滚动无需改动(`ui.rs:730-732` 每帧按 `lines.len()` 重算 `max_scroll` 并回写)
- 验证:`cargo test -p ccr-tui -- --test-threads=1`

## Step 6 — WS6 文档与规范同步

- [ ] `docs/reference/commands/tui.md` 第 29 行键位表、第 52 行示例注释
- [ ] `docs/en/reference/commands/tui.md` 第 29、52 行同位置(locale parity 是硬门禁)
- [ ] `CHANGELOG.md` Unreleased 记录 Enter 键位行为变化
- [ ] `.trellis/spec/ccr-tui/backend/backend-guidelines.md`:第 308-310 行 Focus / Status strip 契约、第 405 行 Grok 场景的 Enter 描述、新增「键位不得依赖裸修饰键组合上报」条目
- 验证:`cd docs && bun run build && bun run audit`

## Step 7 — 收尾

- [ ] 全量验证:`just fmt-check`、`cargo test -p ccr-tui -- --test-threads=1`、`just lint-strict`
- [ ] 手动验证:宽(≥120 列)/窄(80 列)终端各跑一次 TUI,逐条过 PRD 验收标准,重点核对 Enter 后不退出且 Focus 有反馈、页脚标注、折叠行为
- [ ] review gate:`trellis-check` 全范围检查(最后迭代必须全 scope)

## Review Gates

- 每个 Step 后:`cargo test -p ccr-tui -- --test-threads=1` 绿才进下一个
- Step 7 的 `trellis-check` 全绿后才进 Phase 3.4 commit

## Rollback Points

- Step 1+2 commit:键位语义 + 页脚(必须相邻;回滚即恢复原 Enter 行为与原页脚)
- Step 3 commit:banner/文案/图例
- Step 4 commit:Selection 面板
- Step 5 commit:详情折叠
- Step 6 commit:文档与规范
