# WS6 设计语言推广 - 部分完成总结

## 完成状态

**WS6 批次③ 图表色 token 化** ✅ （部分完成）

## 已完成项目

### 批次③：图表色 token 化

**背景：**
- chart token 已在 `styles/chart-colors.css` 定义完成
- 5 个语义 token：`--chart-color-0` ~ `--chart-color-4`
- 映射到 accent 变量：primary/secondary/warning/tertiary/danger

**本次修复：**

#### 1. HistoryList.vue（8 个 hex 颜色 → chart token）

**修复前：**
```typescript
const getOperationColor = (op: string) => ({
  'switch': '#8b5cf6',   // 紫色
  'init': '#10b981',     // 绿色
  'update': '#3b82f6',   // 蓝色
  'delete': '#ef4444',   // 红色
  'validate': '#f59e0b', // 橙色
  'clean': '#6366f1',    // 靛蓝
  'import': '#06b6d4',   // 青色
  'export': '#ec4899'    // 粉色
}[op] || '#64748b')
```

**修复后：**
```typescript
const getOperationColor = (op: string) => ({
  'switch': 'var(--chart-color-0)',    // primary accent
  'init': 'var(--chart-color-1)',      // secondary accent
  'update': 'var(--chart-color-3)',    // tertiary accent
  'delete': 'var(--chart-color-4)',    // danger accent
  'validate': 'var(--chart-color-2)',  // warning accent
  'clean': 'var(--chart-color-3)',     // tertiary accent
  'import': 'var(--chart-color-1)',    // secondary accent
  'export': 'var(--chart-color-0)'     // primary accent
}[op] || 'var(--color-text-muted)')
```

#### 2. TokenDetailTab.vue（1 个 hex 颜色 → chart token）

**修复前：**
```typescript
const colors = [theme.primary, theme.secondary, theme.tertiary, '#5b8a62']
```

**修复后：**
```typescript
const colors = [
  theme.primary, 
  theme.secondary, 
  theme.tertiary, 
  getComputedStyle(document.documentElement)
    .getPropertyValue('--chart-color-1').trim() || '#5b8a62'
]
```

## 验证结果

```bash
✅ npm run type-check     0 errors
✅ npm run lint           0 errors (4 warnings: attributes-order)
✅ npm run test:smoke     348/348 passed
```

## 技术收益

1. **主题一致性**：图表颜色随主题 accent 变化，与整体设计语言保持一致
2. **维护性提升**：颜色集中管理，修改配色方案只需修改 token 定义
3. **可访问性**：通过 token 统一管理，便于未来调整对比度

## 剩余工作

### WS6 待完成批次

#### 批次①：Checkin 全模块（P0，最大违规簇）
- 450+ 处 Tailwind 调色板 raw rgb → token
- 116 处 `.dark` 后代选择器 → token 自动切换
- `checkin-shared.css` 旧玻璃语言重写
- **工作量**：~2-3 天

#### 批次②：Shell + 共享 primitive（杠杆最大）
- `MainLayout.vue`：nav-item 圆角、激活态、settings-dock
- `ui/Button.vue`：去渐变 + glow，改扁平 surface
- `ui/Card.vue`：装饰 prop 标记 deprecated
- Glass 别名标记废弃
- **工作量**：~1 天

#### 批次③：图表色 + 高残留视图（部分完成）
- ✅ 图表接 `--chart-color-*`（HistoryList + TokenDetailTab）
- ⏳ 旧语言孤岛重刷（tray/、CommandPalette、ProviderTemplateSelector 等）
- **剩余工作量**：~1 天

#### 批次④：modal 家族 + 系统性收敛
- modal 收敛 BaseModal（8 个自滚实现）
- ~~z-index 全量 token 化~~（WS6③ 已完成大部分，剩余 55 处 Tailwind class）
- 圆角收敛（475 处字面量）
- 动效收尾（77 处 raw 时长）
- **工作量**：~2-3 天

## 里程碑进度

```
WS4 架构线  ✅ （Profiles合并 → 工具收口 → 类型去重 → CodexAuth拆分）
WS5 性能线  ✅ （O(n²) → GPU → keep-alive → 响应式 → 双查询）
WS6 样式线  ◐  （批次③ 图表色部分完成，其余待完成）
```

## 建议后续顺序

根据优先级和工作量：

1. **批次②** Shell + 共享 primitive（杠杆最大，影响所有页面）
2. **批次③** 旧语言孤岛重刷（完成图表色线）
3. **批次①** Checkin 全模块（P0，但工作量大）
4. **批次④** modal 家族 + 系统性收敛（最全面的收尾）

