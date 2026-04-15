# CLI 工作流

本页按任务组织 CCR 的核心 CLI 能力，而不是按命令字母表展开。

## 1. 初始化与平台切换

```bash
ccr init
ccr platform list
ccr platform switch codex
ccr platform current
```

适用场景：
- 首次初始化 Unified Mode
- 在 Claude / Codex / Gemini / Droid 之间切换当前工作平台
- 确认当前平台是否正确

## 2. Profile 生命周期

```bash
ccr add
ccr list
ccr current
ccr switch <name>
ccr enable <name>
ccr disable <name> --force
ccr delete <name>
```

建议顺序：
1. `ccr add`
2. `ccr list`
3. `ccr switch <name>`
4. `ccr current`
5. 需要时再做 `enable/disable/delete`

## 3. 校验、历史与清理

```bash
ccr validate
ccr history -l 50
ccr optimize
ccr clean --days 30 --dry-run
ccr clear --force
```

## 4. 导入、导出与整理

```bash
ccr export -o configs.toml --no-secrets
ccr import configs.toml --merge --backup
ccr clean --days 30 --dry-run
```

## 5. 临时覆盖与快速实验

```bash
ccr temp-token set sk-test-xxx --base-url https://api.example.com/v1 --model claude-opus-4
ccr temp-token show
ccr temp-token clear
ccr temp
```

## 6. 同步与多目录

```bash
ccr sync config
ccr sync folder add claude ~/.claude -r /ccr-sync/claude
ccr sync folder enable claude
ccr sync claude push
ccr sync all status
ccr sync push -i
```

## 7. Codex 多账号 auth

```bash
ccr codex auth save work
ccr codex auth list
ccr codex auth switch work
ccr codex auth current
```

适用场景：
- 一人维护多个 Codex / GitHub 登录身份
- 需要把当前 Codex 登录保存为命名账号
- 需要导出、导入或切换已保存账号

## 8. Codex -> OpenCode auth 迁移

```bash
# 先预览可迁移账号
ccr opencode auth import-codex --dry-run

# 再导入兼容账号
ccr opencode auth import-codex

# 需要脚本消费结果时输出 JSON
ccr opencode auth import-codex --json
```

这个流程适用于：
- 已经在 CCR 中保存了一批 Codex 账号
- 想让 OpenCode 也能直接切换这些账号
- 希望迁移是增量导入，而不是覆盖当前 OpenCode 设置

行为边界：
- 只导入已保存的 Codex 账号
- 只接受兼容的 ChatGPT OAuth 账号
- 不覆盖现有 OpenCode 账号
- 不切换当前 OpenCode 运行时登录
- 会报告跳过原因，方便后续清理或补录

如果你想直接进入 OpenCode Auth 页签做可视化查看：

```bash
ccr opencode
```

在 OpenCode Auth 页签里，按 `i` 可预览并确认导入兼容的已保存 Codex 账号。

## 9. 会话、Provider、技能与提示词

```bash
ccr sessions list
ccr sessions search "refactor"
ccr sessions resume <id>
ccr provider test --all
ccr provider verify <name>
ccr skills list
ccr prompts list
```

## 10. 成本与预算

```bash
ccr stats summary --range week --by-model --details
ccr budget status
ccr budget set --monthly 200 --warn-at 90 --enable
ccr pricing list --verbose
ccr pricing set my-model --input 3.0 --output 15.0
```

## 11. 什么时候进入图形界面

```bash
ccr ui -p 15173 --backend-port 38081
ccr
```

- `ccr ui`：推荐的图形界面入口
- `ccr`：默认构建下的终端交互界面

## 相关页面
- [`快速开始`](/guide/quick-start)
- [`入口选择`](/guide/entrypoints)
- [`UI 概览`](/guide/ui-overview)
- [`命令参考`](/reference/commands/)
