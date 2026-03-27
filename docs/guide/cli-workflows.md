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

## 7. 会话、Provider、技能与提示词

```bash
ccr sessions list
ccr sessions search "refactor"
ccr sessions resume <id>
ccr provider test --all
ccr provider verify <name>
ccr skills list
ccr prompts list
```

## 8. 成本与预算

```bash
ccr stats summary --range week --by-model --details
ccr budget status
ccr budget set --monthly 200 --warn-at 90 --enable
ccr pricing list --verbose
ccr pricing set my-model --input 3.0 --output 15.0
```

## 9. 什么时候进入图形界面

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
