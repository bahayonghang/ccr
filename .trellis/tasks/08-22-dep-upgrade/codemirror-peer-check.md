# @uiw/react-codemirror peer 依赖核对（段 1）

> 执行时间：2026-08-23。环境：bun 1.3.x，`ccr-ui/package.json` 段 1 提交后的依赖树。
> 结论供 `08-22-views-sync-tools` 使用（协同点 B）。

## 核对步骤与结果

### 1. `bun pm ls --all` 中每个 `@codemirror/*` 包的实例数

```
@codemirror/view@6.43.6            ×1
@codemirror/theme-one-dark@6.1.3   ×1
@codemirror/state@6.7.1            ×1
@codemirror/search@6.7.1           ×1
@codemirror/lint@6.9.7             ×1
@codemirror/legacy-modes@6.5.3     ×1
@codemirror/language@6.12.4        ×1
@codemirror/lang-markdown@6.5.1    ×1
@codemirror/lang-json@6.0.2        ×1
@codemirror/lang-javascript@6.2.5  ×1
@codemirror/lang-html@6.4.11       ×1
@codemirror/lang-css@6.3.1         ×1
@codemirror/commands@6.10.4        ×1
@codemirror/autocomplete@6.20.3    ×1
```

每个包只出现一个版本。lockfile（bun.lock）中 `@codemirror/state` 仅一条记录：`"@codemirror/state@6.7.1"`。

### 2. `@uiw/react-codemirror@4.25.11` 声明的版本范围

dependencies：

| 包 | @uiw 要求范围 | 现直接依赖 / 解析结果 | 相容 |
|---|---|---|---|
| `@codemirror/commands` | `^6.1.0` | `^6.10.4` → 6.10.4 | 是 |
| `@codemirror/state` | `^6.1.1` | `^6.7.1` → 6.7.1 | 是 |
| `@codemirror/theme-one-dark` | `^6.0.0` | （传递）→ 6.1.3 | 是 |
| `codemirror` | `^6.0.0` | （传递） | 是 |

peerDependencies：`@codemirror/state >=6.0.0`、`@codemirror/view >=6.0.0`、`react >=17.0.0`、`react-dom >=17.0.0`。React 19.2.8 满足；`>=6.0.0` 的宽松范围均被现有 9 个 pin 覆盖。bun install 无 peer 冲突告警。

## 结论

**不需要 `overrides` 收敛。** bun 将全部 `@codemirror/*` 需求解析为单一实例：9 个直接依赖的 `^6.x` 范围与 `@uiw/react-codemirror` 的 `^6.x` / `>=6.0.0` 范围在 CodeMirror 6 的 semver-compat 协议下收敛到同一版本，`@codemirror/state` 全树仅 6.7.1 一个副本，无多实例风险。

`08-22-views-sync-tools` 实现后按 dep-upgrade `design.md` §4 第 3 步在产物里复验一次即可（`rg -c 'FacetProvider|StateField' dist/assets/*.js` 特征计数）。
