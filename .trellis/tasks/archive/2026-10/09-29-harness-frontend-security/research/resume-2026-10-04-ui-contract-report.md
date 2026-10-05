# Research: T08 前端 braces 深度补丁与验收调整建议

- Query: 当前已交付 braces@3.0.3 补丁能否支持 T08 的正式验收；原 0/0 例外契约需要哪些调整。
- Scope: mixed；只读产品，写入本任务 research；不接受风险、不修改政策、不扩大版本。
- Date: 2026-10-04
- 派发归属：native hook 注入 T07；主会话随后显式确认本次归属 T08，仅写 `research/resume-2026-10-04-ui-*`。未更改共享任务指针，未读取 implement/check JSONL。

## Findings

### 结论与原契约冲突

当前局部检查通过。当前审计保留 1 条 high 公告，并由 1/1 例外放行。原 T08-AC1 与依赖治理要求 0/0，因此原 AC1 未满足，旧勾选属于历史验收。不能用本轮 audit 退出 0 宣称原 AC1 已完成。

- `.trellis/tasks/09-29-harness-frontend-security/prd.md:14`：保留 `maxActiveExceptions=0`。
- `.trellis/tasks/09-29-harness-frontend-security/design.md`：不得扩大 allowlist；原文件白名单只有 manifest、lock、依赖规范。
- `.trellis/spec/ccr/backend/dependency-governance.md:377`、`:413`：当前限额 0，audit JSON 应为空且例外 0/0。
- `ccr-ui/scripts/frontend-audit-allowlist.json:2`：限额 1；唯一 ID 为 `GHSA-vfj7-8cjw-p6xm`，package 为 braces，patchedVersions 为 3.0.3。
- 同文件 `:7`、`:9`：owner 为 `frontend-platform`，expires 为 `2027-01-03`。字段存在不能证明责任人已经接受风险。

### 当前补丁与官方完整性

`ccr-ui/patches/braces@3.0.3.patch:9` 定义 `MAX_BRACE_DEPTH=100`；`:18` 在 parse 遇到左花括号后检查深度，超过 100 抛 `SyntaxError`。manifest `:122` 与 lock 都登记该补丁。

已直接下载 npm 官方 3.0.3 tarball，SHA-512 与 registry 元数据及 bun.lock 一致：

`sha512-yQbXgO/OSZVD2IsiLlro+7Hf6Q18EJrKSEsdoMzKePKXct3gvD8oLcOQdIzGupr5Fj+EDe8gO/lxc1BzfMpxvA==`

安装包 10 文件中 9 文件与 tarball 逐字节相同；唯一变化 parse.js 可用 registry 源码加登记补丁逐字节重建。expected/installed SHA-256 均为 `561a9eba7a84551919cc77dd7cf0720199866f58bc6d6c8fee6f80ab6797c658`。原包 MAX_LENGTH 本身为 10000，未发现未登记的常量修改。

安装包并未更换版本号。审计数据库仍认为 3.0.3 在受影响范围内，补丁验证和公告例外必须分别记录。

### 当前运行证据

OS 为 Microsoft Windows 10.0.26300。实际 Bun 1.4.2、Node v26.7.0；仓库固定版本为 Bun 1.4.0、Node 24.20.0。本轮属于本地实际工具链证据，固定工具链兼容性 `UNVERIFIED`。

| 检查 | 结果 | 限制 |
| --- | --- | --- |
| `cd ccr-ui && bun install --frozen-lockfile` | exit 0，654 installs / 723 packages，无变更 | 使用已有 node_modules；官方逐文件核验补充安装身份 |
| 安装前后 lock SHA-256 | 均为 `2CC27709FCD10C18D1DFFF536B63AC234DB12644E742226AFA563D86A4EF547D` | 冻结安装未改锁 |
| `just frontend-audit` | exit 0；1 reported advisories，1/1 active exceptions | 原 0/0 契约仍失败 |
| `bun audit --json --audit-level=high` | exit 1；仅 braces GHSA-vfj7-8cjw-p6xm | 保存原始未放行报告 |
| 定向 policy smoke | 1 file / 3 tests PASS | 仅通用政策验证 |
| 真实安装包 parse/compile/expand：100 层 | Node/Bun 三 API 均成功 | compile 长度401，expand 101项 |
| 同三 API：101 层与2000层 | Node/Bun 均抛 `SyntaxError`，错误包含深度101/最大100 | 2000层长度8001，低于原10000字符限制 |
| 8组普通输入 compile 与 expand | 与内存去除登记补丁的原始 parse 结果全部相同 | 8组样例兼容；不代表完整产品兼容 |

2000 层输入下三 API 的测量区间为 Node 0.94–1.11 ms、Bun 5.00–5.40 ms。每一例均在独立子进程执行，5秒超时；子进程全部退出 0，因为验证脚本捕获预期错误。该退出码代表 probe 完成，成功/错误行为必须读取 payload。计时属于单次诊断，不是性能验收阈值。

8组普通输入覆盖列表、数字范围、嵌套列表、转义花括号、括号、美元前缀、补零范围、两个集合笛卡尔积。仅对 compile/expand 结果作前后比较。

首次 12000 层输入长度超过10000，先命中原长度限制。保留 v1 原记录；深度补丁有效性以 v2 的2000层/8001字符例为准。

### 依赖范围与未验证边界

直接开发依赖 `stylelint@17.14.1`、`eslint-plugin-boundaries@7.2.0` 通过 `micromatch@4.0.8` 使用 braces@3.0.3；stylelint 也通过 fast-glob@3.3.3 使用同一 micromatch。版本和路径来自当前 bun.lock，manifest 中两者均位于 devDependencies。没有发现产品源码直接调用 braces 的证据；未运行生产 bundle 可达性分析，产品攻击面为 `UNVERIFIED`。

- `ccr-ui/node_modules/braces/index.js:96`、`:120` 只在参数为字符串时进入 parse；调用者传入自建 AST 会绕过 parse 深度检查。源码表明当前补丁没有给任意 AST walker 加总深度限制，未执行任意 AST 安全验收。例外 rationale 应明确字符串入口范围，避免宣称所有 AST 输入均受控。
- parse 还生成 paren AST；现有 guard 仅计花括号深度。4999层括号/9999字符在当前 Node/Bun 的 parse/compile/expand 中成功；expand 单次约163–166ms（Node）、88ms（Bun）。该结果证明当前样例运行完成，不能证明所有 runtime 的递归安全上限。
- `ccr-ui/scripts/audit-dependencies.mjs:86` 的 validatePatches 只检查 manifest 是否有对应键。该函数未核验补丁内容或实际安装源码，也未验证补丁行为。policy smoke 不弥补这个缺口。
- 现有 policy smoke `:26` 起共3项，测试的是通用限额、expiry、package match、stale 和 GHSA提取。全目录定向搜索未找到 braces 深度补丁的持久回归测试。

### 推荐的具体验收文本（DRAFT，待批准）

建议由主会话先取得原 0/0 政策调整的明确批准，再更新 PRD/design/implement 与 dependency-governance。一般的继续实施授权未单独记录唯一例外的风险接受依据。本研究不代替批准。

> T08-AC1（建议）：原18条公告不再命中。当前审计仅允许 GHSA-vfj7-8cjw-p6xm / braces@3.0.3 一项例外，maxActiveExceptions=1，owner=frontend-platform，expiry=2027-01-03。任何其他公告、版本不匹配、过期、重复、stale 或缺少实际深度防护均失败。保留原始 audit JSON 的唯一公告及退出1记录。例外依据为已验证的 parse 字符串输入花括号深度100上限；不声称所有 AST/括号递归或产品利用面均已消除。例外期限内复核上游修复；上游兼容安全版本可用时移除例外和补丁。

> T08-AC3（建议补充）：在固定工具链下从冻结安装得到的真实 braces 模块执行持久回归：parse/compile/expand 对深度100成功、101与2000（字符数低于10000）统一抛预期 SyntaxError；普通列表/范围/转义/组合结果保持一致。原类型、lint、完整smoke、coverage与build门禁保持原阈值，不用本轮局部测试代替。

实施文件建议：新增 `ccr-ui/tests/quality/braces-depth-guard.smoke.test.ts`；通过 `createRequire` 从 `ccr-ui/package.json` 对应 resolver 加载真实安装的 braces，避免复制实现作为被测对象。测试应有案例级超时，并对错误类型和消息断言。普通输入使用明确预期值，研究的内存反向移除补丁方式不应成为产品测试依赖。更新 policy smoke 时保留现有失败规则；新增当前精确 ID/版本限制与真实补丁行为的断言。

定向验证命令：

```text
cd ccr-ui && bun install --frozen-lockfile
just frontend-audit
cd ccr-ui && bun run test:smoke -- tests/quality/frontend-dependency-audit.smoke.test.ts tests/quality/braces-depth-guard.smoke.test.ts
```

研究未创建该产品测试。批准后应由 implement 角色修改。根完整 frontend-check、coverage、just ci 由主会话统一执行。本轮均 `NOT_RUN`。

### 外部参考

- [GitHub Reviewed 公告](https://github.com/advisories/GHSA-vfj7-8cjw-p6xm)：影响 <=3.0.3，暂无 patched version；2026-09-18发布，2026-10-02更新。不能把今天的再次命中写成今天新发布。
- [官方公告 API](https://api.github.com/advisories/GHSA-vfj7-8cjw-p6xm)：原始版本范围和时间保存于 research JSON。
- [npm registry latest](https://registry.npmjs.org/braces/latest)：当前 latest 为3.0.3；官方 tarball完整性核验见 receipt。

### 已写证据索引

所有文件位于当前 `research/`，统一前缀 `resume-2026-10-04-ui-`：

- `frozen-install.log`、`frozen-install-receipt.json`：原始冻结安装和前后hash。
- `audit.log`、`audit-receipt.json`：仓库审计策略结果。
- `policy-smoke.log`、`policy-smoke-receipt.json`：定向3项测试。
- `raw-audit.json`、`raw-audit-stderr.log`、`raw-audit-receipt.json`：原始唯一公告与exit1。
- `braces-probe.cjs`、`braces-probe.log`、`braces-probe-receipt.json`：首次长度限制覆盖，保留。
- `braces-probe-v2.cjs`、`braces-probe-v2.log`、`braces-probe-v2-receipt.json`：28个Node/Bun子进程案例和8组兼容结果。
- `braces-registry-python.json`、`braces-advisory-python.json`：官方metadata/API数据。
- `braces-registry-3.0.3.tgz`、`braces-registry-parse.js`、`braces-registry-constants.js`：官方原始下载源。
- `braces-integrity-receipt.json`、`braces-patch-reconstruction-receipt.json`：SHA512及逐文件重建身份。

## Caveats / Not Found

- 本报告不修改产品、task metadata、spec，未执行任何git操作。
- 根CONTEXT.md不存在；未创建该文件。
- native注入归属差异按主会话明确确认处理，研究角色未加载implement/check JSONL。
- curl两次返回Schannel TLS失败；随后Python HTTPS读取官方registry/API与tarball成功，原失败事实保留于本报告。没有改变系统TLS或全局配置。
- 未完成固定版本工具链、完整coverage、完整frontend-check、root ci、hosted与原生客户端验收。
- 本轮研究发现的验收政策冲突未修复。唯一例外仍需明确批准，并需补充持久回归后独立复核。
