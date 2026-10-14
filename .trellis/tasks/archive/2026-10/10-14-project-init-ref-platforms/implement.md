# `ccr project init` 实施清单

按顺序实施。每一步保持可编译；进程测试在 fake Git 能回答 `user.name` 之后再跑。

## 清单

- [x] 在 `crates/ccr-cli/src/commands/project/init.rs` 增加五项平台枚举和 `trellis` 参数纯函数。顺序固定为 `--claude --codex --cursor --omp --grok`，`-y` 时在末尾加 `--yes`。空名称或空平台返回 `ValidationError`。
- [x] 交互模式使用模块顶部导入的 `dialoguer::MultiSelect`，五项默认勾选。空选择重新询问。交互失败返回 `UiError`。随后用 `prompt_required` 取得非空用户名。
- [x] `-y` 在 Git 阶段成功后用已解析的 Git 可执行文件运行 `git config user.name`。缺失或空白返回退出码 90，且不启动 Trellis。
- [x] Trellis 成功并满足最低文件后创建 `ref/`。已有目录保留内容；普通文件返回退出码 51，且不写 `.gitignore`。
- [x] `PROJECT_IGNORE_RULES` 追加 `ref/`。规则已完整时不重写文件，但仍确保 `ref/` 目录存在。更新成功摘要和跳过写入提示。
- [x] 更新 `project.rs` 文档注释、`help_config.rs`、中英文 `project-init.md` 和命令索引。
- [x] 扩展 `crates/ccr/tests/commands/project_init.rs` 的两套 fake Git，使到达 Trellis 的成功、失败和重试用例使用 `-y` 与 `FAKE_GIT_USER`。补上用户名缺失、`ref/` 保留、`ref` 文件冲突，以及忽略规则已完整时仍创建目录的用例。
- [x] 为参数函数和忽略规则合并补单元测试。更新 `crates/ccr/tests/commands/help.rs` 的项目初始化帮助断言。
- [x] 更新 `.trellis/spec/ccr-cli/backend/backend-guidelines.md` 的 Project Workflow Bootstrap，使规范与新的菜单、用户名和 `ref/` 契约一致。

## 验证

先跑窄测试：

```text
cargo test -p ccr-cli commands::project::init -- --test-threads=1
cargo test -p ccr --test commands project_init -- --test-threads=1
cargo test -p ccr --test commands help -- --test-threads=1
```

然后：

```text
just docs-check
just lint-strict
```

文档或帮助文案有改动时必须通过 `just docs-check`。Rust 改动必须通过 `just lint-strict` 和上面三组测试。

## 风险与回退

- 现有不带 `-y` 的进程测试会在复选框处失败。先改 fake Git 和用例，再把交互接到命令主路径。
- Windows 与 Unix fake Git 必须同时认识 `config user.name`，否则一边会把参数记错或提前退出。
- 回退就是恢复本清单中的源文件、测试、文档和规范。没有迁移，也不删除用户已经放进 `ref/` 的文件。

## 完成前检查

- [x] 交互参数没有 `--kimi` 和 `--yes`。
- [x] `-y` 参数以 `--yes` 结尾，并且用户名来自 Git 输出。
- [x] Git 或 Trellis 失败时目录里没有新建的 `ref/`，`.gitignore` 也没有被这次运行修改。
- [x] 中英文命令页描述同一套五项、用户名和 `ref/` 行为。
