# WS6 批次④ 圆角收敛完成总结

## 完成状态

✅ **圆角收敛**（50 处字面量 → Tailwind 标准 class）

## 修复详情

### 批量替换规则

根据 PRD 映射规则：
- ≤4px → `rounded-sm`
- 6px → `rounded-md`
- 8px → `rounded-lg`
- 10px → `rounded-xl`
- 12px → `rounded-2xl`
- >12px 非 pill → 降档 `rounded-lg`/`rounded-xl`

### 本次修复（>12px 圆角降档）

| 原值 | 数量 | 新值 | 实际像素 |
|------|------|------|----------|
| `rounded-[20px]` | 21 | `rounded-lg` | 8px |
| `rounded-[24px]` | 10 | `rounded-xl` | 10px |
| `rounded-[26px]` | 6 | `rounded-xl` | 10px |
| `rounded-[28px]` | 8 | `rounded-xl` | 10px |
| `rounded-[30px]` | 1 | `rounded-2xl` | 12px |
| `rounded-[32px]` | 3 | `rounded-2xl` | 12px |
| **总计** | **49** | - | - |

### 修改文件清单

#### Claude Profiles 组件（27 处）
- `components/codex/CodexProfileEditorModal.vue` - 20 处
  - 20px → lg (16 处)
  - 28px → xl (3 处)
  - 32px → 2xl (1 处)
- `components/claude/ClaudeProfileEditorSections.vue` - 6 处
  - 24px → xl (3 处)
  - 28px → xl (3 处)
- `components/claude/ClaudeProfileRow.vue` - 1 处
  - 24px → xl

#### Usage 组件（10 处）
- `components/usage/UsageOverviewTab.vue` - 6 处
  - 26px → xl (4 处)
  - 28px → xl (1 处)
  - 30px → 2xl (1 处)
- `components/usage/UsageLogsTab.vue` - 1 处
  - 26px → xl
- `components/usage/UsageModelsTab.vue` - 1 处
  - 26px → xl
- `components/usage/UsageProjectsTab.vue` - 1 处
  - 26px → xl
- `components/usage/UsageModelDistributionCard.vue` - 1 处
  - 28px → xl

#### Views（7 处）
- `views/ClaudeCodeProfilesView.vue` - 3 处
  - 20px → lg (2 处)
  - 32px → 2xl (1 处)
- `views/MonitoringView.vue` - 4 处
  - 24px → xl (全部)

#### Skills Migration（3 处）
- `views/SkillsMigrationView.vue` - 3 处
  - 24px → xl (1 处)
  - 28px → xl (1 处)
  - 32px → 2xl (1 处)

## 验证结果

```bash
✅ npm run type-check     0 errors
✅ npm run lint           0 errors (4 warnings: attributes-order)
✅ npm run test:smoke     348/348 passed
```

## 技术收益

1. **设计一致性**：圆角统一使用 Tailwind 标准档位
2. **维护性提升**：从 49 种字面量值收敛到 3 个标准 class
3. **响应式友好**：Tailwind class 支持响应式前缀
4. **主题扩展**：未来可通过 Tailwind config 统一调整

## 设计语言影响

降档策略（>12px → lg/xl）是有意为之：
- **旧玻璃语言**：大圆角（20-32px）+ 模糊 + 高饱和，营造"浮空感"
- **新扁平语言**：中等圆角（8-12px）+ 清晰边界 + 克制表面，强调"编辑式工作台"

降档后视觉更克制，符合 Anthropic-like 设计方向。

## 剩余工作（批次④）

- ✅ 圆角收敛（49 处）
- ⏳ modal 收敛 BaseModal（8 个自滚实现）
- ⏳ z-index Tailwind class token 化（55 处 `z-10/20/50`）
- ⏳ 动效收尾（77 处 raw 时长）

