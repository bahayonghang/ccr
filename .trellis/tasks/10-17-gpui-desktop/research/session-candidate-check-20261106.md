# 会话恢复 Candidate 差异检查

本文件接续 `session-delivery-check-20261106.md` 的 eager-listing finding，只检查后续 API/交互差异。不是原生验收或全门禁结论。

- Domain Candidate 的 home、manifest token 和原记录均不可由调用者构造/替换。列表只读 manifest，不打开任何 rollout；逐条及排序前后检查取消。
- Review 只准备选中项：重新核对原 Candidate、分块读取唯一 payload，再次核对同一 manifest；取消、缺失、变更及超限不取得确认 authority。Plan 保留原 home/manifest/payload 版本，恢复只复读选中 manifest，不重新扫描全库。
- 确认提交仍调用原锁/CAS owner；目标与索引不得覆盖并发更新。之前的 legacy 大文件/相对 home 行为不受 native 约束扩散影响。
- 新 read 对普通文件作检查，64 MiB payload budget 沿原会话 owner；每64 KiB读取边界可响应取消。UI取消/离页关闭 subscription，并推进 decision/generation；旧结果不能重开确认。准备中的操作不会登记成 retained writer。
- GUI仍每页六条，Review成功后才进入安全Cancel焦点确认；秘密正文不进入 retained completion。保存Markdown的预览与新文件CAS行为保持。
- 主会话独立源码检查未发现此差异中的新问题。最终owner domain11通过（包含普通文件检查与sort后取消）；desktop session_operations7通过。主会话对最新共享refresh回归重跑3通过，覆盖原字节/mtime恢复、SQLite member、absent deleted保护及正常增量快路径。集中GPUI门禁仍需复编最新源码。

这里的元数据列表仍按条目数量枚举和排序，不承诺整库O(1)扫描或固定总元数据内存。10k模拟元数据的分页测试只证明绘制有界，不充当真实磁盘成本或性能验收。
