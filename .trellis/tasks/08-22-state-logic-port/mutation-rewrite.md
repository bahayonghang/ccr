# 就地修改改写清单（批次 1 排查 → 批次 5 填写改写）

> R5 / AC6。`rg` 形态：`.push(` / `.splice(` / `.sort(` / `.unshift(`，范围 `src/stores` + `src/composables`，共 46 处。
> 判定口径：仅「对响应式状态（ref.value / store state / reactive 对象）的就地修改」需改写为不可变写法；
> 函数内本地临时数组的累积（build 结果）不触发响应式，无需改写，判定列记「本地临时，无需改写」。

| 位置 | 原写法 | 判定 | 新写法 |
| --- | --- | --- | --- |
| `ccr-ui/src/stores/commands.ts:27` | `groups[category].push(cmd)` | 待批次 5 判定 | — |
| `ccr-ui/src/stores/commandsView.ts:49` | `this.expandedFolders.splice(index, 1)` | 待批次 5 判定 | — |
| `ccr-ui/src/stores/commandsView.ts:51` | `this.expandedFolders.push(folder)` | 待批次 5 判定 | — |
| `ccr-ui/src/stores/ui.ts:38` | `toasts.value.splice(index, 1)` | 待批次 5 判定 | — |
| `ccr-ui/src/stores/ui.ts:50` | `toasts.value.push(toast)` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useClaudeProfilesInsights.ts:46` | `if (requiresBaseUrl(profile) && isBlank(profile.base_url)) missing.push('base_url')` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useClaudeProfilesInsights.ts:47` | `if (!hasAnyModel(profile)) missing.push('model')` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useClaudeProfilesInsights.ts:48` | `if (isBlank(profile.account)) missing.push('account')` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useCodexDashboard.ts:244` | `tasks.push(` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useCodexDashboard.ts:260` | `tasks.push(` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useCodexDashboard.ts:276` | `tasks.push(` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useCodexDashboard.ts:427` | `actions.push({` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useCodexDashboard.ts:437` | `actions.push({` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useCodexDashboard.ts:445` | `actions.push({` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useCodexDashboard.ts:455` | `actions.push({` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useCodexDashboard.ts:465` | `actions.push({` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useCodexDashboard.ts:475` | `actions.push({` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useCodexDashboard.ts:485` | `actions.push({` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useCodexOAuthFlow.ts:248` | `oauthUnlisteners.push(completed, timeout)` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useCodexProfilesInsights.ts:50` | `if (requiresBaseUrl(profile) && isBlank(profile.base_url)) missing.push('base_url')` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useCodexProfilesInsights.ts:51` | `if (isBlank(profile.model)) missing.push('model')` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useGrokDashboard.ts:453` | `actions.push({` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useGrokDashboard.ts:465` | `actions.push({` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useGrokDashboard.ts:474` | `actions.push({` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useGrokDashboard.ts:483` | `actions.push({` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useGrokDashboard.ts:492` | `actions.push({` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useGrokDashboard.ts:503` | `actions.push({` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useMcpManager.ts:29` | `const sortedItems = [...items].sort((a, b) => {` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useMonitoringFeed.ts:248` | `nextEntries.splice(low, 0, entry)` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useMonitoringFeed.ts:342` | `unlisteners.push(unMonitoring)` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useMonitoringFeed.ts:347` | `unlisteners.push(unStats)` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useProfilesFilter.ts:100` | `return Array.from(set).sort()` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useProfilesFilter.ts:114` | `.sort((a, b) => a.label.localeCompare(b.label, undefined, { sensitivity: 'base' }))` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useProfilesFilter.ts:157` | `return sortFn ? [...list].sort(sortFn) : list` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useProfilesInsights.ts:128` | `.sort((a, b) => b.count - a.count || a.provider.localeCompare(b.provider))` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useProfilesInsights.ts:165` | `.sort((a, b) => b.count - a.count || a.tag.localeCompare(b.tag))` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useProfilesInsights.ts:177` | `.sort((a, b) => a.name.localeCompare(b.name))` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useProfilesInsights.ts:187` | `if (missing.length > 0) issues.push({ profile, missing })` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useProfilesInsights.ts:189` | `return issues.sort((a, b) => a.profile.name.localeCompare(b.profile.name))` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useProfilesInsights.ts:204` | `if (arr) arr.push(profile)` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useProfilesInsights.ts:210` | `result.push({` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useProfilesInsights.ts:212` | `profiles: [...group].sort((a, b) => a.name.localeCompare(b.name)),` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useProfilesInsights.ts:216` | `return result.sort(` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useStream.ts:122` | `queuedLines.push(...parts)` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useStream.ts:128` | `queuedLines.push(pendingBuffer)` | 待批次 5 判定 | — |
| `ccr-ui/src/composables/useStream.ts:258` | `lines.value.push(line)` | 待批次 5 判定 | — |
