# 统一 Profile 页面设计语言 — 技术设计

本文件定义跨子任务共享的边界与契约。子任务各自的实现细节写在各自的 `design.md`。

## 分层

```
configs/profiles.ts                 ProfilesConfig（保持现状，只服务旧路径与 apply/off 等无 payload 动作）
configs/profileDisplayRecord.ts     ProfileDisplayRecord 类型 + 供应商 canonical key（新增）
configs/profileCredentials.ts       明文凭据剥离（新增）
configs/profilePresentation.ts      ProfilePresentation（呈现元数据 + typed 投影，新增）
configs/profileEditorAdapter.ts     ProfileEditorAdapter 类型（新增，实现体在平台侧）
features/platform/profiles/
  useProfilesSurface.ts             呈现层状态：查询接入、筛选、视图、派生统计（新增）
  ProfilesSurface.tsx               页面装配（新增）
        ↓
components/profiles/                纯呈现组件，无平台分支
  ProfilesPageHeader / ProfilesStatStrip / ProfilesQuickRail / ProfilesToolbar
  ProfileCardGrid / ProfileTable / ProfilesEmptyState / ProfilesInspector*
  ProfileEditorModal / ProfileEditorFields / ProfilesRawEditorPanel
        ↑
features/{claude,codex,grok}/…      平台控制器：typed 读取、剥离、投影、写入、状态机、raw source
```

关键边界：**呈现层向下不认识任何平台 DTO，平台控制器向上不认识任何布局。** 两者通过 `ProfileDisplayRecord` 与 `ProfilePresentation` / `ProfileEditorAdapter` 三个契约相接。

`ProfilesConfig` 保持现状不动。它的 `ProfileRecord`（7 字段）与 `ProfileDraft`（4 字段）不足以承载三平台的读取与写入，因此**不作为统一契约使用**；`apply` / `remove` / `profileOff` / `exportAll` 这类无 payload 或纯名称参数的动作仍可经由它。

## 契约一：ProfileDisplayRecord

呈现层消费的唯一记录类型。由平台 `project()` 生成，生成时已完成凭据剥离。

```ts
export interface ProfileDisplayRecord {
  name: string;
  description: string;
  enabled: boolean;
  /** 是否为当前应用的 profile */
  current: boolean;
  tags: readonly string[];
  /** 与 presentation.fieldSlots 一一对应的四个展示值；空串由渲染层显示占位符 */
  slots: readonly [string, string, string, string];
  /** 搜索匹配用的合并小写文本，由 project 生成，不含凭据 */
  searchText: string;
  /** 供应商去重 key；null 表示不计入供应商统计 */
  vendorKey: string | null;
  /** 认证方式统计的分组 key（平台自定，如 subscription / openai_api_key / official） */
  authKey: string;
  /** 认证方式统计与徽章的 i18n label key */
  authLabelKey: string;
  /** 行内徽章。Grok 的 profile_kind 走这里 */
  badges: readonly {
    labelKey: string;
    tone: "neutral" | "accent" | "warning";
  }[];
  /** 排序维度（Filters 弹层的排序保留项） */
  sortKeys: { name: string; usageCount: number };
}
```

`slots` 是投影结果而非取值函数。原因：slot 取值需要平台 typed DTO 的字段，而呈现层拿不到 DTO。把投影一次做完，呈现层只读字符串。

## 契约二：ProfilePresentation

```ts
export interface ProfileFieldSlot {
  /** 列头与卡片字段的 label i18n key */
  labelKey: string;
  /** 表格列宽 */
  columnWidth: string;
  /** 表格中是否渲染为 chip */
  chip?: boolean;
}

export interface ProfilePresentation<TRecord = unknown> {
  key: string;
  /** 页头字形方块内的单字母 */
  glyph: string;
  nameKey: string;
  /** 配置文件名，显示在面包屑右侧徽标 */
  configFile: string;
  /** 配置路径说明 i18n key */
  configPathKey: string;
  fieldSlots: readonly [
    ProfileFieldSlot,
    ProfileFieldSlot,
    ProfileFieldSlot,
    ProfileFieldSlot,
  ];
  /** 平台 typed DTO → 展示投影。入参已经过凭据剥离 */
  project: (
    record: TRecord,
    ctx: { current: string | null },
  ) => ProfileDisplayRecord;
}
```

`TRecord` 分别是 `ClaudeProfile`、`CodexProfile`、`GrokProfileDto`。`project` 不做 IO，是纯函数，可被测试直接断言。

## 契约三：ProfileEditorAdapter

编辑器统一的是**外壳与字段渲染原语**，不是表单模型。原因：Codex 有五种 auth mode 且 base URL / secret / env key / model 按 mode 条件必填；Claude 只有 `subscription` / `api_key`；Grok 用 `preserve` / `replace_api_key` / `replace_env_key` / `clear` 四种 credential action 并区分 official 与 third-party。这三套状态机无法压进一组通用 `kind`。

```ts
export type ProfileEditorFieldKind =
  | "text"
  | "mono-text"
  | "choice"
  | "secret"
  | "multi-value"
  | "boolean"
  | "number";

export interface ProfileEditorFieldSpec {
  key: string;
  labelKey: string;
  kind: ProfileEditorFieldKind;
  options?: readonly string[]; // choice 的快捷候选，始终允许自由输入
  hintKey?: string;
  /** 由平台按当前表单值决定是否渲染 / 是否必填 */
  visible?: (form: unknown) => boolean;
  required?: (form: unknown) => boolean;
}

export interface ProfileEditorSection {
  id: string;
  titleKey?: string;
  /** grid 为两列区，row 为整行区，group 为带边框的分组（认证区） */
  layout: "grid" | "row" | "group";
  advanced?: boolean;
  fields: readonly ProfileEditorFieldSpec[];
}

export interface ProfileEditorIssue {
  /** 出错分段 id，用于汇总条跳转 */
  section: string;
  /** 具体字段；无法定位到单字段时省略 */
  field?: string;
  /** 已翻译的错误文案 */
  message: string;
}

export type ProfileWriteOutcome =
  | { status: "ok"; appliedName?: string }
  | { status: "recovery"; kind: string; message: string }
  | { status: "blocked"; message: string; forceAllowed: boolean }
  | { status: "error"; message: string };

export interface ProfileEditorAdapter<TForm = unknown, TRecord = unknown> {
  createEmpty(): TForm;
  /** 入参已剥离凭据；返回的表单密钥字段一律为空 */
  fromRecord(record: TRecord): TForm;
  sections: readonly ProfileEditorSection[];
  /** 返回校验问题列表，空数组表示通过 */
  validate(
    form: TForm,
    ctx: {
      isEditing: boolean;
      originalName: string | null;
      existingNames: readonly string[];
      /** 编辑既有 profile 时后端已存有 base URL，用于 Grok 的留空放行分支 */
      hasExistingBaseUrl: boolean;
    },
  ): readonly ProfileEditorIssue[];
  /** 平台内部自行组装 create / patch，含 dirty 字段与 credential action */
  submit(
    form: TForm,
    ctx: { isEditing: boolean; originalName: string | null; apply: boolean },
  ): Promise<ProfileWriteOutcome>;
}
```

三平台的 `submit` 各自复用现有构建器，不重写：

| 平台   | create                                         | update                                                                                      |
| ------ | ---------------------------------------------- | ------------------------------------------------------------------------------------------- |
| claude | `addClaudeProfile(request)`                    | `updateClaudeProfile(name, request)`，request 由 `claudeProfileEditor.ts` 组装              |
| codex  | `addCodexProfile(request)`                     | `updateCodexProfile(name, request)`，`buildCodexProfileRequest(form, resolvedModel)`        |
| grok   | `addGrokProfile(buildGrokCreateRequest(form))` | `updateGrokProfile(name, buildGrokPatch(form, dirtyFields))`，dirty 集合由 adapter 内部持有 |

Grok 的 dirty-patch 语义（absent 保留 / `null` 清除 / 有值替换）因此完整保留。`submit` 返回 `ProfileWriteOutcome`，`recovery` 与 `blocked` 由平台控制器接管后续 UI，编辑器只关闭或保持打开。

## 契约四：凭据剥离

```ts
// configs/profileCredentials.ts
export function stripCredentials<T>(
  record: T,
  secretKeys: readonly string[],
): T;
```

- 每个平台声明自己的 `secretKeys`：claude `['auth_token']`、codex `['auth_token']`、grok `[]`（`GrokProfileDto` 本身不含凭据）。
- 调用点是平台控制器 `useQuery` 的 `select`。剥离后的记录才进入 React state，`project()` 与 `fromRecord()` 都只见剥离后的记录。
- 后端仍返回明文（`profile_to_json` 的 `Secret::expose`）。剥离在前端边界完成，后端掩码化是另一任务，本任务不改。
- 编辑器密钥字段一律以空值初始化，留空表示不修改，由平台 `submit` 决定不序列化该字段。
- 提交失败时错误文案只透传后端消息，不拼接表单值。

## 契约五：raw-source capability

`ProfilesRawEditorPanel` 需要 `getRaw / saveRaw / onSaved / onClose`，`ProfilesConfig` 没有这些。由平台控制器提供：

```ts
export interface ProfileRawSourceCapability {
  getRaw(): Promise<RawFileGetResult>;
  saveRaw(
    content: string,
    token: string,
    force?: boolean,
  ): Promise<RawProfilesSaveResult>;
  /** 保存成功后由控制器执行的全量刷新 */
  refreshAll(): Promise<void>;
}
```

`ProfilesSurface` 持有 `sourceMode` 状态。进入前调用 `uiStore.requestConfirm` 的明文警告；`conflict` 只给重载/取消；`activation_conflict` 走显式危险确认后带 `force: true` 重试同一 content 与 token；保存成功后先清 dirty、退出 source mode，再执行 `refreshAll()`。这五项逐条对应 `raw-config-editor-contracts.md`。

capability 缺席时（Grok）页头不渲染该入口。

## 平台色 token 契约

每个平台四个颜色角色，命名沿用现有 `--color-platform-*` 前缀：

```
--color-platform-{key}          dot：状态点、侧栏标记
--color-platform-{key}-rgb      dot 的 rgb 分量
--color-platform-{key}-surface  bg：glyph 方块底、选中态底
--color-platform-{key}-border   border：glyph 方块边框、高亮卡边框
--color-platform-{key}-text     fg：glyph 字形、计数强调色
```

覆盖平台：claude / codex / grok / gemini / opencode / antigravity。antigravity 当前不存在任何平台色 token，dot 与 `-rgb` 也需新增。

新增名称总量按 `theme-token-contracts.md` 的治理流程处理：登记名称增量、更新冻结段叙述、判定归属（层一 `tokens.css` 明暗块，不进 `@theme` / `@theme inline`）、运行 `tests/theme-switch.smoke.test.tsx` 与 `tests/token-single-point.smoke.test.tsx`。

明色取值一律写为 hex 字面量，不用 `color-mix()`。原因：对比度测试需要直接从 CSS 文本解析并计算，`color-mix()` 无法在解析层求值。

「运行中」高亮不使用平台色，使用全局 accent。理由：运行中是跨平台一致的状态语义，若用平台色，同一状态在三个页面呈现三种颜色。

## 供应商 canonical key

`ProfileDisplayRecord.vendorKey` 的生成规则，供 `stats.vendorCount` 去重：

1. 输入为空、纯空白或非字符串 → `null`，不计入。
2. 无 scheme 时补 `https://` 后用 `new URL()` 解析；解析抛错 → `null`。
3. key = `hostname` 小写，去尾点。IPv6 保留方括号。
4. 端口：仅当端口存在且不是该 scheme 的默认端口时，追加 `:port`。
5. userinfo（`user:pass@`）丢弃，不进入 key。
6. path、query、fragment 不进入 key。

等价类测试（AC20）覆盖：大小写、默认端口与显式端口、userinfo、IPv6、尾点、无协议输入、空值、非法输入八类。

## 列表状态

数据加载归平台控制器：控制器用 `useQuery` 取 typed 列表（key 沿用 `['platform-profiles', config.cacheKey]`，不改缓存结构），在 `select` 中依次执行 `stripCredentials` 与 `presentation.project`，把 `ProfileDisplayRecord[]` 与 `current`、`canOff`、`rawSource` 一起交给 `ProfilesSurface`。

`useProfilesSurface` 只持有呈现层状态，不发起请求：`query`、`tagFilter`、`providerFilter`、`sortBy`、`viewMode`、`editorTarget`、`sourceMode`。

派生（`useMemo`）：`filtered`、`stats.total`、`stats.vendorCount`、`stats.tagCounts`、`stats.authCounts`。

- 搜索：对 `ProfileDisplayRecord.searchText` 做小写包含匹配。
- `stats` 对全量列表计算，不随筛选变化；统计条描述的是该平台整体。
- `viewMode` 按平台 key 持久化：沿用 `features/profiles/stores.ts` 的 Zustand + localStorage 模式，key 形如 `ccr:profiles:view:{platform}`，`try/catch` 包裹读写，storage 抛错时降级为纯内存 state 且当前会话内仍可切换。

`stats` 由 `useProfilesSurface` 从传入的 `ProfileDisplayRecord[]` 派生，不重新请求。

## 组件处置决策

决策 5：现行 `profiles-page-contracts.md` 规定的共享骨架能力全部保留并接入，不删除。骨架顺序按规格：`ProfilesPageHeader` → Off 横幅（`can_off === true` 时，位于 Header 与 StatStrip 之间，确认框 `type=warning`）→ `ProfilesStatStrip` → `ProfilesQuickRail` → `ProfilesToolbar` → 主列表 → `ProfilesInspector` 右栏。

| 组件                                         | 结论                                                                                                           |
| -------------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| `ProfilesStatStrip`                          | 改造。四固定槽换为设计稿的「总数 / 运行中 / 标签分布 / 认证方式」，接收派生 stats。                            |
| `ProfilesToolbar`                            | 改造。按设计稿重排为搜索 + 标签 pill + 视图切换；provider 与排序保留在 Filters 弹层内，焦点陷阱逻辑一并保留。  |
| `ProfilesQuickRail`                          | 接入。位置按规格置于 StatStrip 与 Toolbar 之间，`⌘/Ctrl+1..8` 绑定与持久化不变。                               |
| `ProfilesCommandPalette`                     | 接入。保留 `__off` 条目；Off 不进 Header 溢出菜单。                                                            |
| `ProfilesInspector` 系列 / `ProfileDiffRows` | 接入为右栏。卡片网格在 Inspector 展开时由三列降为两列，属设计稿偏差，记入 `research/design-source.md` 偏差表。 |
| `ProfilesRawEditorPanel`                     | 接入。由契约五的 capability 驱动。                                                                             |
| `ProfileListRow`                             | 由 `ProfileTable` 取代。设计稿表格是固定六列网格，现有行是自由 flex 布局，列无法对齐。                         |
| `GrokProfileCard` / `GrokProfilesPage`       | 由 `ProfileCardGrid` / `ProfilesSurface` 取代。                                                                |
| `GrokProfileEditorModal`                     | 由 `ProfileEditorModal` + `grokProfileEditorAdapter` 取代。                                                    |
| `useGrokProfilesPage`                        | **保留**为 Grok 平台控制器。delete 分支、rename recovery、Local-only fail-closed、activation 信封原样不动。    |
| `grokEditorValidation.ts`                    | 保留，作为 `grokProfileEditorAdapter.validate` 的实现体。                                                      |
| `utils/{claude,codex,grok}ProfileEditor.ts`  | 保留，作为三份 adapter 的 `fromRecord` / `submit` 实现体。                                                     |
| `utils/{claude,codex,grok}Profiles.ts`       | 保留。Inspector 接入后其 descriptor 构造函数重新有消费方。                                                     |

`can_off` 数据流：`ProfilesSnapshot` 当前不携带 `can_off`。Off 横幅的显示条件需要它，由平台控制器从各自的 typed 读取结果中取得并传给 `ProfilesSurface`，不改 `ProfilesConfig` 的既有字段。

## 可扩展性口径

新增平台的真实成本，按 `platform-surface-contracts.md` §7：

1. **层一**：`config/platformDescriptors.ts` 的 `platformSurfaceDescriptors` 增加一行，或给既有行的 `surfaces` 增加 `'profiles'`。这一层同时改变导航与路由清单。
2. **层二**：profiles surface 模块增加一份导出——一条 `ProfilesConfig` + 一条 `ProfilePresentation`（+ 需要写入时一条 `ProfileEditorAdapter`）。

不宣称「只改一处」。

antigravity 的现状需要说明：`platformSurfaceDescriptors` 中没有 `antigravity` 这个 id，`/antigravity` 是 descriptor id `gemini` 的 `rootPath`，且该行的 `surfaces` 不含 `profiles`。同一概念在设计稿（antigravity）与代码（gemini）中有两个名字，这是既有的命名不一致，本任务不改代码也不改设计稿，只在 rollout `notes.md` 记录并上报。

因此本任务只交付**层二**并对其做验证：新增 antigravity 的 `ProfilesConfig` 与 `ProfilePresentation` 注册项，测试从注册表（`profilesConfigs` / `profilePresentations` 映射）按 key 取出后渲染 `ProfilesSurface`，不手写 mock config。**不动 descriptor**——给 `surfaces` 加 `'profiles'` 会改变导航与 `flattenCatalog()` 的 75 条路径，`tests/platform-surface-unify.smoke.test.ts` 会失败。层一的改动与该平台是否上线是同一个决策，留给后续任务。

验收口径因此表述为：新增平台的 profiles 页面在层二只需两条注册项，层一的 descriptor 改动与上线决策绑定。不表述为「只需注册元数据即可得到完整页面」。

## 测试落位

全部新增契约测试位于 `ccr-ui/tests/*.smoke.test.ts(x)`。`vitest.smoke.config.ts` 的 `include` 是 `tests/**/*.smoke.test.{ts,tsx}`，`src/**/__tests__/` 下的文件不会被任何门禁执行。

跨子任务的测试文件清单：

| 文件                                            | 归属子任务      |
| ----------------------------------------------- | --------------- |
| `tests/profile-presentation.smoke.test.ts`      | registry-tokens |
| `tests/profile-credentials.smoke.test.ts`       | registry-tokens |
| `tests/platform-color-tokens.smoke.test.ts`     | registry-tokens |
| `tests/profiles-surface.smoke.test.tsx`         | list-surface    |
| `tests/profiles-vendor-key.smoke.test.ts`       | registry-tokens |
| `tests/profiles-view-mode.smoke.test.ts`        | list-surface    |
| `tests/profiles-raw-source.smoke.test.tsx`      | list-surface    |
| `tests/profile-editor-shell.smoke.test.tsx`     | editor          |
| `tests/profile-editor-adapters.smoke.test.ts`   | editor          |
| `tests/profiles-platform-wiring.smoke.test.tsx` | rollout         |

现有的 `tests/grok-profile-editor.smoke.test.ts`、`tests/grok-profiles-view.smoke.test.ts`、`tests/profiles-quick-switch.smoke.test.ts`、`tests/profiles-hotkeys.smoke.test.ts`、`tests/profiles-quick-rail.smoke.test.ts`、`tests/theme-switch.smoke.test.tsx`、`tests/token-single-point.smoke.test.tsx`、`tests/platform-surface-unify.smoke.test.ts` 必须继续通过，改动时在对应子任务的 change list 中列出。

## 视觉与响应式验收条件

所有手工走查与响应式断言使用同一组前置条件，否则同一构建在不同环境下结论不一致。

- viewport：宽 `1440×900`，窄 `900×800`（低于表格容器 `min-width: 1024px`，确保触发横向滚动）。
- zoom：100%。
- 主题矩阵：`data-theme` ∈ {light, dark} × `data-flavor` ∈ {neutral, clay}，`data-accent` 固定 `clay`，共四种组合。
- 数据夹具：每平台 7 条 profile，含 1 条当前应用、1 条 disabled、1 条 description 为 120 字符长文本、1 条 5 个标签、1 条 `baseUrl` 为空、2 条同 host 不同 path（验证供应商去重）。夹具落在 `ccr-ui/tests/fixtures/profiles.ts`，三平台共用。
- 滚动判据：表格容器 `scrollWidth > clientWidth` 且 `document.body.scrollWidth <= document.body.clientWidth`。
- 截图落位：`ccr-ui/tests/__screenshots__/profiles-{platform}-{theme}-{flavor}-{w}x{h}.png`，仅作走查记录，不进 CI 断言。

## 兼容性与回滚

- 路由不变，`ProfilesConfig` 的字段与导出不变，Tauri 命令签名不变。回滚粒度为单个子任务的提交。
- `--color-platform-codex` / `--color-platform-grok` 的取值变化会影响首页、侧栏、用量图表等已消费位置。这是决策 1 的已知溢出，`08-26-profile-registry-tokens` 需列出全部消费点并逐一确认视觉可接受。
- token 变更单独成一个提交，与契约结构变更分开。
