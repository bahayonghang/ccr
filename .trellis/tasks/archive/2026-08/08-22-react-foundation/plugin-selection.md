# 插件选择测量：@vitejs/plugin-react vs @vitejs/plugin-react-swc

> 批次 2（implement.md「测量 … 二选一，数据落盘」）。测量日期：2026-08-23。
> 测量脚本：`measure-plugin.mjs`（本目录，bun 运行，可复现）。

## 测量方法

- 环境：Windows 11 Pro，Intel Core Ultra 9 275HX，bun 1.4.0，vite 8.2.2，React 19.2.8。
- 每次运行前删除 `node_modules/.vite`（冷缓存），启动 `vite --strictPort`（端口 15173）。
- `readyMs`：进程启动到 stdout 出现 `Local:` 行。
- `pageMs`：`GET /`（index.html）+ `GET /src/main.tsx`（入口模块，走完整转换管线）串行耗时。
- `hmrMs`：经 vite HMR websocket（协议 `vite-hmr`）观测，向 `src/shell/App.tsx` 追加注释到收到 `update` 消息的延迟，测后恢复原文件。
- 每个变体 3 次运行，两次运行之间完整关闭服务进程树。

## 原始数据

### @vitejs/plugin-react（babel，^6.1.0）

| run | readyMs | pageMs | hmrMs |
| --- | ------- | ------ | ----- |
| 1   | 6053.4  | 259.7  | 553.0 |
| 2   | 4526.0  | 145.2  | 529.7 |
| 3   | 1966.8  | 123.7  | 424.2 |

中位数：ready 4526.0 / page 145.2 / hmr 529.7。

### @vitejs/plugin-react-swc（^4.3.3，临时安装后测毕卸载）

| run | readyMs | pageMs | hmrMs |
| --- | ------- | ------ | ----- |
| 1   | 5763.2  | 143.2  | 491.4 |
| 2   | 4855.9  | 145.2  | 561.3 |
| 3   | 1620.5  | 141.9  | 409.2 |

中位数：ready 4855.9 / page 143.2 / hmr 491.4。

### 历史噪声样本（已并入，原 `measure-raw-plugin-react.txt` 删除）

早前两次采样（冷缓存口径不完全一致，仅作参考）：
run1 cold 2300ms / page 17.8ms / hmr 4.0ms；run2 cold 1072ms / page 360.6ms / hmr 26.9ms。
该批样本与本次口径不可比（pageMs 当时只取 `GET /`，hmrMs 采样方式不同），不参与决策。

## 结论

**选定 `@vitejs/plugin-react`。** `vite.config.ts` 维持现有 import，SWC 变体已从 package.json 与 lockfile 卸载。

理由：

1. 两个变体三项指标的中位数差异（ready 约 330ms、page 约 2ms、hmr 约 38ms）均小于各自 run 间波动（ready 波动超过 2.5s，hmr 波动约 150ms），SWC 的优势在本次测量中不可复现，属噪声范围。
2. `@vitejs/plugin-react` 已由 `08-22-dep-upgrade` 第一段提交锁定（^6.1.0），维持该选择不产生额外 diff。
3. SWC 变体额外引入一个原生二进制依赖，且安装时触发 bun 的 postinstall 信任拦截（`Blocked 1 postinstall`），在无实测收益的情况下不引入该成本。

## dev-warm-targets 生成方式

**结论：手写清单，仓库内不存在生成脚本。** 全仓检索 `dev-warm-targets` / `devWarmTargets` / `warm-targets`，命中的 4 处全部为消费方或校验方：

| 位置                                        | 角色                                       |
| ------------------------------------------- | ------------------------------------------ |
| `ccr-ui/vite.config.ts:5,55`                | 消费方：读 `clientFiles` 喂 `server.warmup` |
| `ccr-ui/scripts/dev-web-warm-start.mjs:35` | 消费方：读 `healthPath` 做健康检查          |
| `ccr-ui/tests/dev-tooling-resource.smoke.test.ts:54` | 校验方：断言 clientFiles 覆盖入口文件 |
| `scripts/quality/check_json_format.py:30`   | 校验方：JSON 格式检查白名单                 |

无任何脚本写入或再生成该 JSON。按 foundation design.md §10，原因已查明（手写），配置保留不删；目录重组后该文件的目标路径需随实现更新。
