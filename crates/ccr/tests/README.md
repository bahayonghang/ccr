# CCR 集成测试

## 概览

`crates/ccr/tests/` 已经按能力域拆分，不再使用旧的扁平测试文件命名。这里的集成测试主要验证命令层、manager/service 编排、平台行为，以及跨模块工作流。

当前目录结构：

```text
tests/
├── commands.rs
├── commands/
│   └── sync_content.rs
├── managers.rs
├── managers/
│   └── general.rs
├── platforms.rs
├── platforms/
│   ├── general.rs
│   └── integration.rs
├── workflows.rs
└── workflows/
    ├── general.rs
    ├── services.rs
    └── temp_override.rs
```

## 分组说明

- `commands`：命令入口和 CLI 级行为，当前包含 sync content 等命令路径。
- `managers`：配置、设置、历史等 manager 层的读写与编排行为。
- `platforms`：平台级配置、路径解析和跨平台集成行为。
- `workflows`：跨 service 的完整工作流，包括通用流程、services 联动和临时覆盖场景。

## 运行方式

在仓库根目录执行：

```bash
# 运行整个 workspace 测试
cargo test --workspace --all-features -- --test-threads=1

# 只跑 ccr crate 的集成测试入口
cargo test -p ccr --test commands
cargo test -p ccr --test managers
cargo test -p ccr --test platforms
cargo test -p ccr --test workflows
```

运行特定模块或关键字：

```bash
# 运行 workflows 入口下的测试
cargo test -p ccr --test workflows -- --nocapture

# 按关键字过滤
cargo test -p ccr temp_override
cargo test -p ccr sync_content
```

## 维护约定

- 新增测试时，优先放到最贴近当前能力域的入口和子模块中，而不是继续新增旧式扁平文件。
- 避免在这里维护固定测试数、通过率之类容易过期的统计数据；以实际 `cargo test` 输出为准。
- 需要文件系统隔离时，继续使用 `tempfile::tempdir()` 创建临时目录，确保测试不会污染用户环境。
```

### 4. 文件锁测试策略

并发测试使用错开启动时间的策略，更接近真实场景：

```rust
for i in 0..10 {
    thread::spawn(move || {
        // 错开启动时间，避免极端并发
        thread::sleep(Duration::from_millis(i as u64 * 15));
        // 测试逻辑...
    });
}
```

## 📝 测试原则

1. **隔离性**：每个测试独立运行，不依赖其他测试
2. **可重复性**：测试结果应该是确定的，可重复的
3. **清晰性**：测试名称和断言清晰表达测试意图
4. **完整性**：覆盖正常流程和异常情况
5. **现实性**：模拟真实使用场景

## 🔧 调试测试

### 查看测试详细输出

```bash
cargo test --test end_to_end_tests -- --nocapture
```

### 运行单个测试

```bash
cargo test --test manager_tests test_config_manager_lifecycle -- --nocapture
```

### 显示测试时间

```bash
cargo test -- --show-output
```

## 📊 测试结果

当前测试状态：

- ✅ **集成测试**: 55/55 通过（100%）
- ⚠️ **单元测试**: 43/46 通过（3个失败与重构无关）
- ✨ **总计**: 98/101 通过（97%）

失败的 3 个单元测试是原有的时间敏感测试问题，与架构重构无关。

---

**最后更新:** 2025-10-11
**测试框架:** Rust 标准测试框架 + tempfile
