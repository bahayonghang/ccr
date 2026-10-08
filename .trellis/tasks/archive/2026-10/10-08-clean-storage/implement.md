# ccr clean storage 实施顺序

实现时以 `prd.md` 的 R4 和 `design.md` 的候选表为准。不扩大删除集合。

## Checklist

1. 在 `CleanAction` 增加 `Storage(CleanStorageArgs)`，字段只有 `dry_run` 和 `force`。
2. 在 `dispatch.rs` 的 clean 分支调用 `clean_storage_command(dry_run, auto_yes || force)`。`action == None` 的菜单分支不动。
3. 更新 `help_config.rs` 的 `CLEAN_LONG_ABOUT` 与 `CLEAN_AFTER_LONG_HELP`，以及 `Commands::Clean` 的文档注释。保留 `help.rs` 已锁定的五句示例。
4. 新增 `commands/lifecycle/clean_storage.rs`：根目录解析、候选规划、字节统计、删除、结果聚合。`confirm_cleanup` 改为 `pub(super)`。
5. 从 `lifecycle/mod.rs` 和 `commands/mod.rs` 导出 `clean_storage_command`。
6. 在 `crates/ccr/tests/commands/clean.rs` 用现有 `run_clean` / `run_clean_with_input` 覆盖 AC2–AC6、AC8。符号链接用例加 `#[cfg(unix)]`。`LLMUSAGE_HOME` 用例在子进程环境里设置，测完由临时目录丢弃。
7. 在 `clean_storage.rs` 的单测覆盖 AC7：失败列表非空时返回错误，成功候选的计数仍在结果里。
8. 更新 `docs/reference/commands/clean.md` 和 `docs/en/reference/commands/clean.md`。
9. `definitions.rs` 补一条 `ccr clean storage --dry-run` 的解析测试。

## Validation

先跑窄测试：

```bash
cargo test -p ccr --test commands -- clean -- --test-threads=1
cargo test -p ccr-cli --lib commands::lifecycle::clean_storage -- --test-threads=1
```

帮助断言在 `commands` 集成测试的 `help` 过滤里：

```bash
cargo test -p ccr --test commands -- help_subcommand_supports_clean_path -- --test-threads=1
```

这两条通过后，再跑 Rust 门禁：

```bash
just fmt-check
just lint-strict
```

`just lint-strict` 或 `just fmt-check` 如果改动了文件，先看 diff，再决定是否把格式化结果留在本次改动里。不跑 `just ci`，除非窄测试和 lint 都过了且改动扩散到文档审计。

## Risky Files

- `crates/ccr-cli/src/cli/dispatch.rs`：只加 `Storage` 臂。不要改 `None` 臂。
- `crates/ccr-cli/src/cli/help_config.rs`：只加 storage 句子。不要改已锁定的示例。
- `crates/ccr-cli/src/commands/lifecycle/clean.rs`：只把 `confirm_cleanup` 的可见性改为 `pub(super)`。不改 planfiles / backups 的删除条件。

回退方式：这些改动都在命令层。恢复时删掉 `clean_storage.rs`、导出、子命令臂和文档小节即可。没有迁移，也没有配置字段。

## Before task.py start

- `prd.md`、`design.md`、`implement.md` 已写完。
- `implement.jsonl` 与 `check.jsonl` 各有至少一条真实 spec 或 research 记录。
- 用户已经明确同意这份计划。同意之前不运行 `task.py start`，也不改产品代码。
