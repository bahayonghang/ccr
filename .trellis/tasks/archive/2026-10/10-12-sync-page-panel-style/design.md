# 同步页信息结构与面板

## Boundaries

只改同步页的渲染结构和样式。资产数据仍来自 `listSyncAssets()` 与 `getSyncStatus()`。分组顺序仍是 `useSyncPage` 的 `ccr`、`claude`、`codex`，空分组继续滤掉。

不改 Tauri `SYNC_ASSETS`、同步命令载荷、口令弹层或 WebDAV 保存。`asset.description` 留在 DTO 上，页面不再读取它。

## View Tree

```
PageShell.sync-page
  PageHeader
    h1 配置资产同步
    副标题（现有 subtitle）
    状态：连接芯片、数量、敏感值已遮罩
    动作：刷新、全部同步一次、按需强制全部同步
  门控横幅（仅 unconfigured / unreachable）
  .sync-console-grid
    main
      每个非空分组 article.sync-asset-group
        标题 + 排除说明 + 数量
        资产行（无嵌套卡片）
    aside
      WebDAV 面板
      操作输出（有结果才挂载）
      关于同步 details
  口令弹层（不变）
```

删除 `ScopeStrip` 和 `sync-console-intro`。

## Asset Row

`SyncAssetCard` 增加一个布尔 `showTitle`。父组件在渲染前算好：`showTitle = asset.name !== group.title`。不要在卡片里再翻译一遍分组标题。

`showTitle` 为假时不渲染 h 标题，也不留空位。为真时标题就是 `asset.name`。

始终不渲染 `asset.description`。`canonicalName` 仅在与 `asset.name` 不同时显示。

一行的内容顺序：可选标题、类型/敏感/加密标签、本地路径、远端路径、存在状态、动作。动作顺序保持 Push、Pull、Sync、按需 Force。

本地缺失沿用现有失败状态文案，并给行加 `sync-asset-card--missing` 的状态发丝（`--color-danger` 只作 1px 边或左侧 2px tick）。不要给整行铺危险色底。

`SyncAssetCard` 继续 `memo`。`showTitle` 是布尔。回调仍由 `SyncView` 用 `useCallback` 传入。

## WebDAV 与门控

页头连接芯片是唯一的连接状态。`SyncInfoSidebar` 删掉自己的芯片。

未配置：主栏横幅使用现有 `sync.gating.unconfigured*` 文案，主按钮打开已有的添加账号对话框。右栏只保留一句未配置说明。若保留入口，用 ghost 或 secondary，不能是 `bg-accent-primary` 的第二颗主按钮。

不可达：横幅使用现有 unreachable 文案，主按钮触发与现在相同的刷新重测。右栏展示已保存的服务器、用户名、路径。测试和断开保持次级。编辑从现在的实心琥珀改为 secondary。

已连接：右栏是账号明细加编辑、测试、断开，没有状态胶囊。

操作输出和关于同步的位置不动。输出卡的失败/部分成功去掉整卡状态色边，改到状态图标和状态文字上。

## Surface

删除 `.sync-page` 的背景声明，壳层 `--color-bg-base` 作为页面底。

分组、WebDAV、关于同步、操作输出、门控横幅：

- 填充 `var(--color-bg-surface)`
- 边框 `1px solid var(--color-border-default)`
- 圆角 `var(--radius-xl)`
- 内边距沿用约 1rem
- 休息态无阴影

分组内的资产行用 `border-top: 1px solid var(--color-border-subtle)` 分开。行本身不再有圆角外框和独立填充。

页头 `.sync-hero-button` 去掉 `border-radius: 999px`，让 `buttonClass` 的 8px 生效。连接芯片、敏感标签、加密标签保持 pill。

门控横幅填充用 surface，状态用 2px 左侧 tick：未配置用强调色，不可达用警告色。不使用整块色块填充。

中文标签不设置 `letter-spacing` 和 `uppercase`。分组眉题删除后，`.sync-eyebrow` 若无引用则删掉。

`64rem` 媒体查询保留单列。范围条的分隔线规则随 `ScopeStrip` 删除。

## Copy

两端同时删除不再引用的 key。保留 `sync.assets.total`、`sensitiveHint`、分组 `title` 与 `description`、门控文案、关于同步文案。

不把 Rust 英文 description 抄进词条。文件名已经是行标题，排除范围已经在分组描述里。

## Compatibility

渲染层变化不改变命令参数。禁用条件仍是 `syncGated`：未配置和不可达禁用同步，未测试不禁用。

回滚只需要还原 `ccr-ui/src/features/sync/**` 和两份 locale。没有迁移和持久化格式变化。

## Trade-offs

范围条删除后，文件名要进入分组才能看到。五份资产、分组描述里已经点名文件，单独的三列索引增加一次重复阅读。

右栏在窄宽度时落到清单下面。门控主按钮放在主栏横幅里，避免用户先滚过整份清单才看到「配置 WebDAV」。

每行 Sync 仍是琥珀主按钮。五颗小主按钮加上页头全量同步，是这份清单的命令密度。右栏编辑退出琥珀，避免账号维护和同步命令抢同一个强调。
