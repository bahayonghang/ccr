# 2026-10-04 验收调整候选

状态：T01 生成边界调整已获用户批准；T08 例外调整仍为 DRAFT、待批准。原验收和历史记录保持有效的时间边界。

## T01 当前交付边界

2026-10-03 的 17592add 将 `.omp/` 和 `.codex/` 保持为本地生成目录，共享 skill 源迁入 `.github/skills/`。当前 OMP 测试在没有生成扩展时跳过。原 T01-AC3 要求仅版本库内容运行契约测试；当前版本不满足这个前提。

本机生成扩展：14 pass、99 assertions。仅当前 HEAD 的测试源码夹具：0 pass、14 skip、0 fail。两组结果不能互相替代。证据为 `resume-2026-10-04-omp-fresh-fixture.json` 和 T08 `research/evidence/resume-2026-10-04-omp-generated.json`。

候选验收：保留当前本地生成目录边界；仓库交付检查单独验证共享源。OMP 契约验收要求存在匹配版本生成扩展且全部 14 项实际执行。缺少扩展时保留 SKIPPED_UNVERIFIED，退出 0 不关闭契约验收。新客户端原生加载、trust 和新检出生成流程另有独立证据。

用户已明确批准 T01-AC3 调整，见 approval.json。后续匹配 0.7.0-beta.4 的官方生成夹具实际执行结果为 8 pass、6 fail；本机定制扩展 14 pass 不关闭该失败。最小版本补丁候选见 `resume-2026-10-04-omp-matched-candidate-proposal.md`，其实施范围仍待批准。历史归档文件不改写成当前版本已通过。

## T08 当前审计边界

2026-10-03 的 c50716b5 更新 DOMPurify，增加 braces 3.0.3 本地补丁和一个例外。原 T08-AC1 要求 `maxActiveExceptions=0`；当前策略为 1，不能沿用原勾选宣称满足该条。

现有例外为 GHSA-vfj7-8cjw-p6xm，owner 为 frontend-platform，expires 为 2027-01-03，patchedVersions 为 3.0.3。补丁在 parse 阶段拒绝超过 100 层的花括号嵌套。局部补丁与策略证据由 T08 research 的本轮新报告提供。

候选验收：仅允许这一个精确公告、包、补丁版本及限期例外；其他公告、过期或失配例外均失败。保留原审计输出，不将例外通过写成 0 公告。补齐深度 100/101、极深输入和普通展开的持久回归测试，运行冻结安装、策略 smoke、完整前端、覆盖率及父任务聚合门槛。

该候选改变原 0/0 策略，需要用户明确批准；未获批准前不修改 PRD 或规范使之自动符合当前配置。
