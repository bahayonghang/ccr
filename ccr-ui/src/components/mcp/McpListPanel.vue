<template>
  <div class="mcp-list-panel">
    <ListSearchHeader
      :search-value="searchQuery"
      :placeholder="$t('mcp.searchServers')"
      :label="$t('mcp.searchServers')"
      @update:search-value="$emit('update:searchQuery', $event)"
    >
      <!-- 多选切换 -->
      <button
        type="button"
        class="list-action-btn"
        :class="{ 'list-action-btn--active': isMultiSelectMode }"
        :aria-label="isMultiSelectMode ? 'Done selecting' : 'Multi-select'"
        :title="isMultiSelectMode ? 'Done selecting' : 'Multi-select'"
        @click="$emit('toggleMultiSelect')"
      >
        <SIcon
          :name="isMultiSelectMode ? 'CheckCircle2' : 'LayoutGrid'"
          size="w-4 h-4"
        />
      </button>

      <!-- 添加下拉 -->
      <div class="relative">
        <button
          type="button"
          class="list-action-btn"
          aria-label="Add MCP server"
          @click="showAddMenu = !showAddMenu"
        >
          <SIcon
            name="Plus"
            size="w-4 h-4"
          />
        </button>
        <Transition
          enter-active-class="transition-all duration-150 ease-out"
          enter-from-class="opacity-0 scale-95"
          enter-to-class="opacity-100 scale-100"
          leave-active-class="transition-all duration-100 ease-in"
          leave-from-class="opacity-100 scale-100"
          leave-to-class="opacity-0 scale-95"
        >
          <div
            v-if="showAddMenu"
            class="add-menu"
            @click="showAddMenu = false"
          >
            <button
              type="button"
              class="add-menu__item"
              @click="$emit('create')"
            >
              Manual creation
            </button>
            <button
              type="button"
              class="add-menu__item"
              @click="$emit('import')"
            >
              Import from JSON
            </button>
          </div>
        </Transition>
      </div>

      <!-- 刷新 -->
      <button
        type="button"
        class="list-action-btn"
        aria-label="Refresh"
        @click="$emit('refresh')"
      >
        <SIcon
          name="RefreshCw"
          size="w-4 h-4"
          :class="{ 'animate-spin': loading }"
        />
      </button>
    </ListSearchHeader>

    <!-- 列表区域 -->
    <div class="mcp-list-panel__scroll">
      <div
        v-if="groups.length === 0 && !loading"
        class="mcp-list-panel__empty"
      >
        {{ searchQuery ? 'No servers match your search' : 'No MCP servers configured' }}
      </div>

      <button
        v-for="group in groups"
        :key="group.name"
        type="button"
        class="mcp-list-item"
        :class="{
          'mcp-list-item--selected': selectedKeys.has(group.name),
        }"
        @click="$emit('select', group.name)"
      >
        <div class="mcp-list-item__icon">
          <SIcon
            :name="group.transportType === 'stdio' ? 'Terminal' : 'Globe'"
            size="w-4 h-4"
          />
        </div>
        <div class="mcp-list-item__content">
          <span class="mcp-list-item__name">
            {{ group.name }}
            <span
              v-if="group.hiddenCount"
              class="mcp-list-item__hidden-count"
            >
              +{{ group.hiddenCount }} hidden
            </span>
          </span>
          <span class="mcp-list-item__transport">{{ shortenLabel(group.transportLabel) }}</span>
          <span class="mcp-list-item__scopes">
            {{ (group.scopes ?? []).join(' / ') }}
          </span>
        </div>
        <AgentIcons
          :agents="group.platforms"
          :compact="true"
        />
      </button>
    </div>

    <!-- 多选浮动条 -->
    <MultiSelectFloatingBar
      :selected-count="selectedKeys.size"
      :total-count="groups.length"
      @delete="$emit('bulkDelete')"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import ListSearchHeader from '@/components/common/ListSearchHeader.vue'
import MultiSelectFloatingBar from '@/components/common/MultiSelectFloatingBar.vue'
import AgentIcons from '@/components/common/AgentIcons.vue'
import SIcon from '@/components/ui/SIcon.vue'
import type { McpGroup } from '@/types/mcpManager'

defineProps<{
  /** 过滤后的 MCP 分组列表 */
  groups: McpGroup[]
  /** 搜索关键词 */
  searchQuery: string
  /** 选中的 group key 集合 */
  selectedKeys: Set<string>
  /** 是否处于多选模式 */
  isMultiSelectMode: boolean
  /** 加载状态 */
  loading: boolean
}>()

defineEmits<{
  'update:searchQuery': [value: string]
  'select': [name: string]
  'create': []
  'import': []
  'refresh': []
  'toggleMultiSelect': []
  'bulkDelete': []
}>()

const showAddMenu = ref(false)

/** 截断过长的 transport label */
function shortenLabel(label: string, max = 40): string {
  if (label.length <= max) return label
  return label.slice(0, max).trimEnd() + '...'
}
</script>

<style scoped>
.mcp-list-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.mcp-list-panel__scroll {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.mcp-list-panel__empty {
  padding: 1.5rem 0.75rem;
  text-align: center;
  font-size: 0.8125rem;
  color: var(--color-text-muted);
}

/* 列表项 */
.mcp-list-item {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  width: 100%;
  padding: 0.625rem 0.75rem;
  border-radius: 0.75rem;
  border: 1px solid transparent;
  background: transparent;
  text-align: left;
  cursor: pointer;
  transition:
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.mcp-list-item:hover {
  background: var(--surface-status-bg, rgb(var(--color-bg-elevated-rgb) / 72%));
}

.mcp-list-item--selected {
  background: var(--surface-card-bg, rgb(var(--color-bg-elevated-rgb) / 85%));
  border-color: rgb(var(--color-accent-primary-rgb) / 15%);
}

.mcp-list-item__icon {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 1.5rem;
  height: 1.5rem;
  color: var(--color-text-muted);
}

.mcp-list-item__content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.mcp-list-item__name {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.mcp-list-item__hidden-count {
  flex-shrink: 0;
  border-radius: 999px;
  padding: 0.06rem 0.35rem;
  background: rgb(var(--color-warning-rgb, 245 158 11) / 10%);
  color: rgb(var(--color-warning-rgb, 245 158 11) / 92%);
  font-size: 0.58rem;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.mcp-list-item__transport {
  font-size: 0.6875rem;
  color: var(--color-text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.mcp-list-item__scopes {
  color: rgb(var(--color-accent-primary-rgb) / 82%);
  font-size: 0.62rem;
  font-weight: 650;
  letter-spacing: 0.08em;
  overflow: hidden;
  text-overflow: ellipsis;
  text-transform: uppercase;
  white-space: nowrap;
}

/* 操作按钮 */
.list-action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  flex-shrink: 0;
  border-radius: 0.5rem;
  color: var(--color-text-muted);
  cursor: pointer;
  transition:
    color var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.list-action-btn:hover {
  color: var(--color-text-primary);
  background: var(--surface-status-bg);
}

.list-action-btn--active {
  color: rgb(var(--color-accent-primary-rgb));
  background: rgb(var(--color-accent-primary-rgb) / 10%);
}

/* 添加菜单 */
.add-menu {
  position: absolute;
  top: 100%;
  right: 0;
  z-index: 20;
  margin-top: 0.25rem;
  min-width: 11rem;
  padding: 0.25rem;
  border-radius: 0.75rem;
  border: 1px solid var(--surface-card-border, rgb(var(--color-border-default-rgb) / 45%));
  background: var(--surface-card-bg, rgb(var(--color-bg-elevated-rgb) / 95%));
  backdrop-filter: blur(20px) saturate(1.3);
  box-shadow: var(--elevation-2);
}

.add-menu__item {
  display: block;
  width: 100%;
  padding: 0.5rem 0.75rem;
  border-radius: 0.5rem;
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
  text-align: left;
  cursor: pointer;
  transition:
    color var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.add-menu__item:hover {
  color: var(--color-text-primary);
  background: var(--surface-status-bg);
}
</style>
