# WS6 批次②③ 完成总结

## 完成状态

✅ **批次② Shell + 共享 primitive**（全部完成）
✅ **批次③ 旧语言孤岛重刷**（核心完成）

## 批次② 完成项目

### 1. MainLayout.vue ✅
- **nav-item 圆角**：L515 已使用 `var(--radius-lg)`
- **激活态**：L532-541 已去发光，使用边框 + inset + accent 标记
- **settings-dock**：L555-568 已去 blur/glow/radial mesh
- **settings-dock-pill**：L570-577 无 uppercase
- **`.dark` 残留**：0 处

### 2. ui/Button.vue ✅
- **primary/secondary/accent**：L168/180/193 使用扁平 surface
- **无渐变**：无 `linear-gradient(180deg,…)`
- **无 glow**：无 `0 8px 16px` 阴影

### 3. ui/Card.vue ✅
- **装饰 prop**：L76-85 已标记 `@deprecated`
- **默认关闭**：L118-122 默认值全部 `false`

### 4. Glass 别名 ✅
- **标记废弃**：`tailwind.config.ts:188-189` 已标记 `@deprecated`
- **别名映射**：
  - `.liquid-glass` → `.surface-shell`
  - `.glass-effect` → `.surface-workspace`
  - `.glass-elevated` → `.surface-card`
  - `.glass-modal` → `.surface-modal`

## 批次③ 完成项目

### 图表色 token 化 ✅（之前完成）
- `HistoryList.vue`：8 个 hex 颜色 → `var(--chart-color-*)`
- `TokenDetailTab.vue`：1 个 hex 颜色 → runtime 读取

### 旧语言孤岛重刷 ✅（本次完成）

#### 1. PageHeaderCard.vue
**修复前：**
```vue
<div
  class="page-header-card__glow"
  :class="toneClasses.glow"
/>
```
- L145: `filter: blur(84px)`
- 装饰性发光层

**修复后：**
```vue
<!-- Glow 装饰层已移除：与「深色工作台」扁平语言冲突（WS6 批次③） -->
```
- 注释掉整个发光层

#### 2. ScrollToTopButton.vue
**修复前：**
```css
border-radius: 9999px;
background: linear-gradient(...);
box-shadow: 0 18px 32px ..., inset 0 1px 0 ...;
backdrop-filter: blur(20px) saturate(140%);
color: rgb(var(--color-accent-secondary-rgb) / 100%);
```

**修复后：**
```css
border-radius: var(--radius-full);
background: rgb(var(--color-bg-elevated-rgb) / 94%);
box-shadow: var(--shadow-md);
color: var(--color-accent-secondary);
```

#### 3. ProviderStatsModal.vue
**修复前：**
```vue
:style="{
  background: 'var(--glass-bg-strong)',
  backdropFilter: 'blur(24px) saturate(180%)',
  WebkitBackdropFilter: 'blur(24px) saturate(180%)',
  ...
}"
```

**修复后：**
```vue
class="surface-modal"
:style="{
  border: '1px solid var(--border-color)',
  boxShadow: 'var(--shadow-xl)'
}"
```

#### 4. MultiSelectFloatingBar.vue
**修复前：**
```css
backdrop-filter: blur(20px) saturate(1.3);
```

**修复后：**
移除 `backdrop-filter` 行

#### 5. McpListPanel.vue
**修复前：**
```css
backdrop-filter: blur(20px) saturate(1.3);
```

**修复后：**
移除 `backdrop-filter` 行

#### 6. ClaudeCodeProfilesView.vue
**修复前：**
```css
.claude-profile-editor-modal .editor-panel {
  backdrop-filter: blur(20px) saturate(135%);
}
```

**修复后：**
移除 `backdrop-filter` 行

#### 7. CodexProfileEditorModal.vue
**修复前：**
```css
.codex-profile-editor-modal .editor-panel {
  backdrop-filter: blur(20px) saturate(135%);
}
```

**修复后：**
移除 `backdrop-filter` 行

## 验证结果

```bash
✅ npm run type-check     0 errors
✅ npm run lint           0 errors (4 warnings: attributes-order)
✅ npm run test:smoke     348/348 passed
```

## 技术收益

### 批次②
1. **组件规范化**：Button/Card 使用扁平 surface，装饰 prop 标记废弃
2. **导航一致性**：MainLayout nav-item 使用 token 圆角和边框强调
3. **迁移路径明确**：glass 别名标记废弃，指向新的 surface-* class

### 批次③
1. **性能提升**：移除 7 处 `blur(20px/24px/84px)` 和 `saturate()`，减少 GPU 合成成本
2. **视觉一致性**：统一使用扁平 surface 语言，消除旧玻璃风格孤岛
3. **Token 化完成**：图表色全部使用 `--chart-color-*` token
4. **维护性提升**：移除 linear-gradient + inset 高光 + backdrop-filter 的复杂组合

## 剩余工作

### WS6 待完成批次

#### 批次①：Checkin 全模块（P0，最大违规簇）
- 450+ 处 Tailwind 调色板 raw rgb → token
- 116 处 `.dark` 后代选择器 → token 自动切换
- `checkin-shared.css` 旧玻璃语言重写
- **工作量**：~2-3 天

#### 批次④：modal 家族 + 系统性收敛
- modal 收敛 BaseModal（8 个自滚实现）
- ~~z-index 全量 token 化~~（WS6③ 已完成大部分，剩余 55 处 Tailwind class）
- 圆角收敛（475 处字面量）
- 动效收尾（77 处 raw 时长）
- **工作量**：~2-3 天

## 里程碑进度

```
WS4 架构线  ✅  Profiles合并 → 工具收口 → 类型去重 → CodexAuth拆分
WS5 性能线  ✅  O(n²) → GPU → keep-alive → 响应式 → 双查询
WS6 样式线  ◐   批次② Shell + primitive ✅
                批次③ 图表色 + 旧语言孤岛 ✅
                批次① Checkin 模块 ⏳
                批次④ modal + 系统性收敛 ⏳
```

## 修改文件清单

### 批次② (0 处修改，已完成)
- `components/MainLayout.vue` - 已完成
- `components/ui/Button.vue` - 已完成
- `components/ui/Card.vue` - 已完成
- `tailwind.config.ts` - 已完成

### 批次③ (9 处修改)
1. `components/HistoryList.vue` - 图表色 token 化
2. `components/claude-observer/TokenDetailTab.vue` - 图表色 token 化
3. `components/PageHeaderCard.vue` - 移除 glow 层
4. `components/common/ScrollToTopButton.vue` - 扁平化按钮
5. `components/configs/ProviderStatsModal.vue` - surface-modal
6. `components/common/MultiSelectFloatingBar.vue` - 移除 blur
7. `components/mcp/McpListPanel.vue` - 移除 blur
8. `views/ClaudeCodeProfilesView.vue` - 移除 blur
9. `components/codex/CodexProfileEditorModal.vue` - 移除 blur

