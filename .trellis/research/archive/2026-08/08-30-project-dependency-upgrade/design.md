# 全项目依赖审计与分批升级 — Design

## 1. 设计目标

把依赖升级变成一组可独立验证、可停止、可回滚的状态转换，而不是一次性重写全部 manifest/lock。每一批只包含一个风险层级和一组强相关依赖；聚焦门禁与完整 `just ci` 都通过后，才允许进入下一批。

## 2. 权威边界

| 依赖面 | manifest authority | lock authority | 聚焦门禁 |
|---|---|---|---|
| 根 Rust | 根 workspace + member `Cargo.toml` | `Cargo.lock` | `just lint-strict`、`just test`、`cargo audit` |
| Tauri Rust | `ccr-ui/src-tauri/Cargo.toml` + command macro | `ccr-ui/src-tauri/Cargo.lock` | `just tauri-ci`、Tauri lock audit |
| React UI | `ccr-ui/package.json` | `ccr-ui/bun.lock` | `just frontend-check-quick/test/build`、custom Bun audit |
| VS Code | `ccr-vscode/package.json` | `ccr-vscode/package-lock.json` | `just vscode-ci`、`npm audit --package-lock-only` |
| Docs | `docs/package.json` | `docs/bun.lock` | `just docs-check`、`bun audit` |
| Actions/toolchain | workflows、`rust-toolchain.toml`、`packageManager` | exact SHA/version | governance checks + affected hosted-equivalent local gates |

`docs/package-lock.json` 不应继续作为第二权威源。实施时先把根 `just docs` 从 npm 对齐到 Bun，再删除该冗余锁；这个治理变更单独成批，避免与版本更新混淆。

## 3. 风险分类

### Low

- 同一兼容范围内的 lock-only refresh。
- patch/minor 且上游无 breaking note、仓库未依赖内部/未稳定 API。
- 同 major GitHub Action tag 的 pinned SHA refresh。

### Medium

- manifest exact pin 变更，即使仍在同 major。
- Tauri/VS Code packaging/build-chain 更新。
- pre-1.0 minor、跨多个相互约束组件、改变解析图或测试运行时的升级。
- Rust/Node/Bun compiler/runtime 变更。

### High / Breaking

- SemVer major 或 0.x 兼容边界变化。
- 上游明确删除/重命名 API、改变默认值、配置解析、许可或运行行为。
- 密码学/持久化格式、CLI 输出、插件最低运行时、图表 DOM/CSS/视觉合同变化。

漏洞严重度与升级实现风险是两个维度。安全发现必须独立标记，不能因“升级风险高”而从报告消失。用户已决定安全修复优先：先清空已确认安全修复队列，再进入普通升级；安全队列内部仍按实施风险从低到高排序，因此 breaking 安全修复晚于兼容安全修复，但早于普通 Low 更新。

## 4. 执行优先级与任务拓扑

执行队列固定为：

1. Security/Low：仅锁文件或兼容范围内即可消除漏洞。
2. Security/Medium：需要协调框架、打包链或多组件版本，但没有已确认 major break。
3. Security/High：修复漏洞必须跨 Breaking 边界。
4. Ordinary/Low -> Ordinary/Medium -> Ordinary/High：安全队列清空或形成用户明确批准的有期限例外后才开始。

继续使用一个顺序执行的 Trellis 主任务，不建立可并行子任务。根/Tauri 双 Cargo 图、三个 JavaScript lock authority、版本漂移脚本与最终 `just ci` 是共享状态；拆成并行子任务会破坏批次间的稳定性前置条件。实施计划内部的每个 batch 就是唯一推进单元。

## 5. 批次状态机

```text
选择当前最高优先级未完成队列
  -> 刷新动态证据
  -> 只更新本批 manifest/lock/source
  -> 检查 diff 与解析路径
  -> 运行本批聚焦门禁
  -> 运行本批安全复扫
  -> 运行完整 just ci
  -> 审计 git diff/status 与未验证边界
  -> 通过：进入下一批 / 失败：留在本批定位修复
```

完整 `just ci` 不能替代以下缺口：

- Tauri 批次额外运行 `just tauri-ci` 与 Tauri `Cargo.lock` audit。
- VS Code 批次额外运行 `npm audit --package-lock-only`（`vscode-ci` 本身不做 audit）。
- docs 批次额外保存 `bun audit` 零漏洞证据。
- UI 使用仓库 `bun run audit:dependencies`，不以裸 Bun 输出替代例外治理。

## 6. 失败处理与回滚

1. 任一聚焦检查、安全复扫或 `just ci` 失败，立即停止推进，记录失败命令与首个根因。
2. 修复只限当前批次引入的兼容问题；若需要新增依赖、公共 API/数据格式变更或无关重构，回到 planning 并取得新授权。
3. 修复后从最窄失败门禁开始重跑，最终必须重新跑本批完整门禁和 `just ci`。
4. 若目标版本不可稳定兼容，撤销当前批次的精确文件变更，不回退已通过批次；禁止使用 `git reset --hard` 或覆盖用户工作。
5. 每批结束记录 current -> resolved target、锁图变化、修复内容、Passed/Failed/Skipped/UNVERIFIED。
6. 安全批次若没有可用的兼容修复，必须停在本批：可选择 Breaking 修复、暂缓或有期限例外，但后两者需要用户再次明确批准；不得静默越过并开始普通更新。

## 7. 兼容与迁移合同

- Crypto：`aes-gcm/argon2` 升级必须保持现有密文 envelope/KDF 参数可解密；新增 fixture 只证明离线兼容，不证明生产密钥安全。
- CLI tables：comfy-table 8 的 `load_preset` 删除需迁移到新 style API，并对关键表格做 snapshot/文本宽度回归。
- Tauri：framework/build/CLI/API/plugin 版本按兼容矩阵协调，不只升级其中一项；必须验证 Windows build 与命令/TS binding 漂移。
- VS Code：`@types/vscode` 不得继续无界漂移高于 `engines.vscode`；类型版本与最低支持版本必须同批裁决。
- TypeScript 7：在 typescript-eslint/programmatic API 兼容方案成立前标记 deferred，不用“能安装”代替工具链兼容证据。
- ApexCharts：单独验证模块子路径、wrapper peer、CSS/DOM、视觉、性能与许可证；没有这些证据不得升级到 7。
- Toolchain：Rust compiler pin 与 MSRV 分离；Node 24 LTS 与 Node 26 Current 分离；Bun 1.4 作为 runtime/package-manager 迁移单独验证。

## 8. 动态证据与停止条件

每批开始前刷新 crates.io/npm/Bun/RustSec/GitHub tag 与官方迁移说明。以下任一情况阻止进入下一批：

- 漏洞仍解析在目标锁图中且没有已批准的例外；
- compiler/linter 出现未处理 deprecation；
- 完整 `just ci` 非零、超时或中断；
- 工作树含本批之外的新改动；
- 关键上游 breaking/许可条款尚未完成审查；
- 目标版本要求提高最低运行时/MSRV，而规划未授权。
