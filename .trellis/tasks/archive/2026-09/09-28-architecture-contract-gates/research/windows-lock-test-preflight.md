# Windows 锁测试前置核对

2026-09-28。尚未修改 lock.rs。

- T05 实现阶段记录 core 包 101 passed / 1 failed；单例复现 os error 33。
- `crates/ccr-core/src/core/lock.rs:314-323` 中，测试持有排他锁后，使用 `fs::read(path)` 从另一个句柄读取被锁字节范围。
- 测试目标是证明 FileLock 打开文件和竞争失败不会截断已有 native holder metadata。生产代码使用 `.truncate(false)`。
- T10 接手时应在不降低断言的条件下验证：锁持有期间经持锁文件句柄读取原字节，竞争失败后重复验证，释放后通过路径重新读取。平台相关行为需真实 Windows 运行。
- 这是一项待验证的测试修正方案，不能据此判定完整 core gate 通过。不得跳过或删除现有 metadata 断言。
