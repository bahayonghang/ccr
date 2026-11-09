# T01 精确版本 OMP 新生成验证

日期：2026-10-04（America/Chicago）。状态：**fresh-generation 契约 FAIL**。仅写父任务研究证据，真实仓库产品和现有 ignored extension 未改。

## 来源与生成

官方 [npm registry 精确版本](https://registry.npmjs.org/@mindfoldhq/trellis/0.7.0-beta.4)返回 HTTP 200。CLI tarball 905811 字节，下载 SHA-512 与 registry integrity 一致。来源原文、时间和 SHA 见[官方完整性](resume-2026-10-04-omp-matched-official-integrity.json)。

在系统临时目录的自有 `cli/` 安装 `@mindfoldhq/trellis@0.7.0-beta.4`，使用 `--ignore-scripts --no-audit --no-fund --save-exact`。实际 bin 由该临时包的 `bin/trellis.js` 执行；`--version` 返回 0.7.0-beta.4。安装锁的 CLI version/resolved/integrity 与官方元数据相符，core 版本也为 0.7.0-beta.4。CLI engines 为 Node >=18.17.0，本轮使用已有 Node v26.7.0。

`init --help` 已检查。随后在另一个自有临时 `project/` 执行：

```text
node <temporary-cli>/bin/trellis.js init --omp --yes --user ccr-review-fixture --no-monorepo --skip-existing
```

初始化退出 0，生成项目版本为 0.7.0-beta.4。没有对真实仓库执行 init/update，没有升级全局 CLI，没有修改用户 trust 或原 `.omp/`。安装、实际版本、help、初始化各有独立 JSON 及 stdout/stderr 原始日志回执，同名前缀为 `resume-2026-10-04-omp-matched-*`。

## 契约结果

仅复制当前工作区 `scripts/trellis/omp-context.test.ts` 的原始字节到临时项目，执行 `bun test scripts/trellis/omp-context.test.ts`：**退出 1，8 pass、6 fail、0 skip、50 assertions，14 项全部实际执行**。Bun 1.4.2。首次失败和完整输出见[测试回执](resume-2026-10-04-omp-matched-test.json)、[原始 stderr](resume-2026-10-04-omp-matched-test.stderr.log)。

失败测试为：

1. main and each role receive prd/design/implement markers
2. design and implementation edits refresh the existing session cache
3. explicit trusted roots permit artifact and manifest symlinks
4. artifact and file caps truncate on complete UTF-8 characters
5. total context cap emits a required-read notice without oversized content
6. binary and malformed UTF-8 artifacts stay omitted

这 6 条失败均在 design 内容或路径断言处发生。官方新生成扩展的缓存输入列表仅包含 prd/info 及 JSONL，材料化函数仅显式读取 prd/info；没有 design.md 或 implement.md 路径。生成源码已保存为[证据副本](resume-2026-10-04-omp-matched-generated-extension.ts:316)，材料化读取见同文件第 643 行。没有放宽断言、修改官方模板或补入现有定制扩展来获得通过。

生成扩展 SHA-256：`5fbd4cfe2461f5457c6ba93fb168ad0dbdfc0e76d6626fd0407eebc13baf8a35`。当前本机定制扩展 SHA-256：`a9ef13beb1bb7214e2ada5cb43fea596065ab296b68a8da171220a77840c5851`。当前测试 SHA-256：`61ba8ec1861c4dd234d0a994b01f19d7368340df960d4b8ca2676b3c562f738a`。详细来源和版本节点见[汇总](resume-2026-10-04-omp-matched-summary.json)。

## 验收边界

精确版本下载、临时安装与生成通过；正式要求的 14 项契约没有全部通过，后续 AC3 保持 **FAIL / 未完成**。此前本机定制扩展 14 pass、99 assertions 与无扩展夹具 `SKIPPED_UNVERIFIED` 分别保留，均不替代本次新生成 FAIL。

原全局 CLI 0.6.17 与项目 0.7.0-beta.4 不匹配的记录保留。临时精确 CLI 不改变全局版本。所有临时命令对真实测试源码、原 ignored extension、双语说明和扩展锁的 SHA 前后保持一致。临时生成流程只证明 CLI 资产生成，native OMP 客户端加载/trust 仍为 **UNVERIFIED**。

未再运行已成功本机门禁，未修改任何产品文件。主会话正式 CI 的源快照保持稳定。本次按授权保留生成失败，未进行产品或上游模板修复。
