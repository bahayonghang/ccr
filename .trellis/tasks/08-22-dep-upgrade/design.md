# 技术设计：依赖全量升级到最新兼容版

> 父任务：`08-22-react-migration`。选型与版本见父任务 `design.md` §1。本文件写升级路径、Tailwind v4 配置映射与核对方法。

## 1. 三段式升级与提交边界

| 段  | 内容                                                   | 单独 revert 的效果        |
| --- | ------------------------------------------------------ | ------------------------- |
| 1   | Vue → React 依赖替换 + `vite` 7.3.5 → 8.2.2 + 插件切换 | 回到 Vue 依赖树           |
| 2   | Tailwind 3.4.19 → 4.3.3 与配置模型切换                 | 回到 `tailwind.config.ts` |
| 3   | `ccr-ui/src-tauri` Rust 依赖升级                       | 回到旧 Cargo.lock         |

三段各自独立提交。段 1 与 `08-22-react-foundation` 的批次 1–2 交织：本任务先提交依赖变更，对方随后提交入口与构建配置。

`overrides` 复核（第 5 节）在段 1 与段 2 之后各做一次，因为两段都可能改变传递依赖树。

## 2. Tailwind v4 配置模型映射

`tailwind.config.ts` 现 201 行，逐项映射到 CSS-first 模型：

| 现状（JS 配置）                                        | v4 目标                                                                                                                                                       |
| ------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `content: [...]`                                       | v4 自动检测源文件。需核对 tray 独立入口与 `index.html` 是否都被检测到                                                                                         |
| `darkMode: ['class', '[data-theme="dark"]']`           | `@custom-variant dark (&:where([data-theme="dark"], [data-theme="dark"] *))`。三层主题模型（`data-theme` / `data-flavor` / `data-accent`）的语义不变          |
| `corePlugins: { preflight: false }`                    | v4 无 `corePlugins`。改为分别 `@import "tailwindcss/theme"` 与 `@import "tailwindcss/utilities"`，不引入 preflight。`src/styles/base.css` 自带 reset 继续生效 |
| `theme.extend.fontFamily`                              | `@theme` 内的 `--font-*`                                                                                                                                      |
| `theme.extend.fontWeight`（8 档位压缩到 400/500）      | `@theme` 内只定义两个 `--font-weight-*`。压缩语义靠「不定义其余档位」实现，多余档位的工具类因此不生成                                                         |
| `plugin(({ addComponents }) => ...)`（`:139`、`:180`） | `@utility` 指令，或改为 `src/styles/components/` 下的普通 CSS 类。二选一取决于该 plugin 产出的是工具类还是组件类，实施时读其内容判定                          |

448 个 CSS 变量迁到 `@theme inline` 的映射归 `08-22-design-system` R1。本任务只完成版本与配置模型切换，不动变量。

## 3. `@apply` 与 `@reference`

现状：648 处 `@apply` 集中在 25 个文件，另有 2 处在 `.css` 内。

v4 规则：组件级样式文件（非主入口）使用 `@apply` 需先 `@reference` 主样式表，否则 `@apply` 静默失效——生成的 CSS 里该规则为空，不报错。

处理方式：

1. 25 个文件逐个在顶部加 `@reference "../styles/main.css"`（相对路径按实际层级）。
2. 静默失效检测：段 2 完成后，对比升级前后的产物 CSS。检测方法为在每个 `@apply` 文件里取一条代表性规则，`rg` 其展开后的属性是否出现在产物 CSS 中。25 条逐条记录（AC5）。
3. 检测结果落盘为 `apply-verification.md`，25 行，无空缺。

单纯依赖 `bun run lint:style` 与视觉观察不足以发现静默失效，因此 AC5 要求逐文件记录。

## 4. `@uiw/react-codemirror` peer 依赖核对

现有 9 个 `@codemirror/*` 直接依赖：`commands`、`lang-json`、`lang-markdown`、`language`、`legacy-modes`、`lint`、`search`、`state`、`view`。

核对步骤：

1. `bun pm ls --all | rg '@codemirror/'`，确认每个包只出现一个版本。
2. `@codemirror/state` 若出现多个版本，加 `overrides` 收敛。CodeMirror 6 的插件系统在多个 `state` 实例下会抛运行时错误。
3. 构建产物核对：`rg -c 'FacetProvider|StateField' dist/assets/*.js` 或按 `@codemirror/state` 的特征字符串计数，确认只打进一份（AC4 的「无重复实例」由 `08-22-views-sync-tools` 在实现后再验一次）。

核对结论落盘为 `codemirror-peer-check.md`，供 `08-22-views-sync-tools` 使用（协同点 B）。

## 5. `overrides` 复核方法

9 项 pin：`fast-uri` 3.1.5、`flatted` 3.4.2、`js-yaml` 4.3.1、`nanoid` 3.3.18、`picomatch` 4.0.4、`postcss` `$postcss`、`rollup` 4.61.0、`esbuild` 0.28.1、`ws` 8.21.0。

逐项判定表的列：包名、pin 版本、pin 的原因、升级后是否仍有传递依赖引入低版本、判定（保留 / 移除）、依据。

判定依据的取得方式：

- `bun pm ls --all | rg <pkg>` 看谁引入、引入什么范围。
- 移除该 override 后重新 `bun install`，看解析结果是否落到可接受版本。
- `bun run audit:dependencies` 确认移除不引入告警。

`rollup` 4.61.0 与 `esbuild` 0.28.1 两项与 `vite` 8 的自带版本直接相关，段 1 后必须重新判定。`postcss` 的 `$postcss` 自引用与 Tailwind v4 的 PostCSS 插件形态相关，段 2 后必须重新判定。

落盘为 `overrides-review.md`，9 行，无空缺（AC9）。

## 6. `vite` 7 → 8 breaking change 核对

核对面（逐项确认现配置是否受影响）：

- `build.rollupOptions.output.manualChunks` 的接受形态。
- `optimizeDeps.noDiscovery` 语义。
- `server.fs.allow` 与 `server.warmup` 的配置键。
- `server.watch.ignored`。
- CSS 处理与 PostCSS 集成方式（与 Tailwind v4 叠加）。
- Vitest 4.1.10 对 vite 8 的支持。若不兼容，vitest 需同步升级，该项计入本任务范围。

核对结果落盘为 `vite8-migration-notes.md`。

## 7. `src-tauri` Rust 依赖与 ts-rs 协同

`ccr-ui/src-tauri/Cargo.toml` 的 `ts-rs = { version = "11", features = ["no-serde-warnings"] }`。

生成入口（已核实）：`cd ccr-ui && just bindings`，实际执行三条：

```
cargo test --manifest-path ../Cargo.toml -p ccr-cli   --features ts export_bindings
cargo test --manifest-path ../Cargo.toml -p ccr-usage --features ts export_bindings
cargo --config ../.cargo/tauri-ci.toml test --manifest-path src-tauri/Cargo.toml export_bindings
bun ./scripts/normalize-generated-bindings.mjs
```

生成前 `rm -rf src/types/generated`，生成后经 `normalize-generated-bindings.mjs` 规范化空白。漂移守卫为 `just tauri-bindings-check`（根）/ `just bindings-check`（`ccr-ui`）。

协同点 A 的分工：`08-22-workspace-cargo-upgrade` 升级 workspace 的 `ts-rs` 并执行生成；本任务升级 `src-tauri` 的 `ts-rs` 并对 204 个文件的 diff 逐条判定（R7、AC7）。两侧 `ts-rs` 版本必须一致，否则三条生成命令产出不同格式。

diff 判定的分类：格式变化（空白、引号、类型别名写法）、类型变化（字段可选性、联合成员、命名）。格式变化整批接受；类型变化逐条确认是否影响前端调用点，影响项登记。

落盘为 `ts-rs-diff-review.md`。

## 8. 依赖替换的路径影响

`vue-i18n` 的 dev / build 双入口 alias（`ccr-ui/vite.config.ts:22`）随 `vue-i18n` 移除一并删除。该 alias 存在的原因是桌面壳 CSP 与 runtime compiler 冲突；i18next 无 runtime compiler，`08-22-i18n-port` 需确认 CSP 下无等价问题。

`@intlify/eslint-plugin-vue-i18n` 移除后的静态检查能力缺口由 `08-22-i18n-port` R9 补齐。本任务只做移除，不设计替代方案。

`postcss-html` 判定：其作用是让 stylelint 解析 SFC 内 `<style>`。无 `.vue` 后不需要，移除。

## 9. 未决项

- 各框架无关依赖的「最新兼容版」具体版本号在实施时由 `bun outdated` 与 `cargo upgrade --dry-run` 给出，本文件不预设。
- Vitest 是否需随 vite 8 升级，见第 6 节最后一项。
- `plugin(({ addComponents }))` 迁为 `@utility` 还是普通 CSS 类，见第 2 节最后一行。
