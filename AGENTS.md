<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

# Repository Guidelines

## 项目结构与模块组织
主仓库为 Rust 实现：`src/core` 提供锁管理与底层文件操作，`src/services` 封装业务流程，`src/managers` 聚合持久化逻辑，`src/commands` 暴露 CLI 子命令，`src/web` 提供内置 Web API。集成测试集中在 `tests/`，按 manager、service、并发及端到端场景分类。静态资源与文档分别位于 `web/` 与 `docs/`，React 管理界面在 `ccr-ui/frontend`，其 Axum 适配层位于 `ccr-ui/backend`。

## 构建、测试与开发命令
Rust 环境：使用 `just build`/`just release` 触发 `cargo build` 与发布优化，开发期推荐 `just dev`（等同 `cargo check` + `cargo test`）。运行 CLI 使用 `just run -- --help`，发布版可通过 `just run-release -- --version` 验证二进制。需要快速检查可执行 `cargo check`，生成文档用 `just doc`。前端在 `ccr-ui/frontend` 中运行 `pnpm install && pnpm dev`，后端桥接服务用 `just serve`（位于 `ccr-ui/justfile`）。

## 代码风格与命名规范
Rust 部分遵守 2024 edition 默认 4 空格缩进，模块命名使用蛇形（例如 `history_manager`），结构体与枚举使用帕斯卡命名。提交前必须运行 `just fmt` 与 `just clippy`，保持 `cargo fmt --check` 与 `clippy -D warnings` 无告警。前端遵循 ESLint + Prettier 组合（`pnpm lint`），组件命名使用帕斯卡式，Hooks 保持 `useXxx` 前缀。公共常量集中在 `src/utils/constants.rs`，请避免在业务代码内硬编码路径。

## 测试指引
核心测试框架为 Rust 标准测试；保持单元测试与集成测试覆盖率大于 95%，新增功能至少补充 happy path 与失败分支。运行全量测试使用 `just test`，包含忽略用例执行 `just test-all`。并发与端到端场景位于 `tests/concurrent_tests.rs` 与 `tests/end_to_end_tests.rs`，命名格式统一为 `should_<行为>`。调试单个用例时可运行 `cargo test --test manager_tests -- --nocapture`。

## 提交与 Pull Request 指南
Git 历史采用带 emoji 的 Conventional Commits，例如 `feat(ui): ✨ 添加版本管理功能`、`fix(docs): 📝 更新安装命令`，类型建议控制在 feat/fix/chore/docs/refactor/test。提交信息主题不超过 72 字符，正文用列表描述修改与影响范围。PR 需要：概述变更、列出测试结果（命令输出摘要即可）、关联 Issue，并在改动 UI 时附带截图或录屏。合并前确保 CI（`just ci`）通过，变更涉及配置文件时补充风险说明。

## 安全与配置提示
CLI 默认直接写入 `~/.claude/settings.json` 并生成时间戳备份，请勿绕过 `SettingsManager` 进行手工编辑；若需自定义备份路径，通过 `--config-dir` 或配置文件声明。Web 服务默认监听本地端口，外部暴露前需启用反向代理与鉴权。敏感密钥统一置于环境变量，避免提交到 `docs/examples`。当引入新依赖时，务必在 PR 描述内说明安全影响并更新 `Cargo.lock`。
