<template>
  <div class="skill-list-panel">
    <ListSearchHeader
      :search-value="searchQuery"
      placeholder="Search skills..."
      label="Search skills"
      @update:search-value="$emit('update:searchQuery', $event)"
    >
      <button
        type="button"
        class="list-action-btn"
        :class="{ 'list-action-btn--active': isMultiSelectMode }"
        :title="isMultiSelectMode ? 'Done selecting' : 'Multi-select'"
        @click="$emit('toggleMultiSelect')"
      >
        <SIcon
          :name="isMultiSelectMode ? 'CheckCircle2' : 'LayoutGrid'"
          size="w-4 h-4"
        />
      </button>

      <div class="relative">
        <button
          type="button"
          class="list-action-btn"
          aria-label="Add skill"
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
              Create custom skill
            </button>
            <button
              type="button"
              class="add-menu__item"
              @click="$emit('import')"
            >
              Import from file
            </button>
            <button
              type="button"
              class="add-menu__item"
              @click="$emit('importGithub')"
            >
              Import from GitHub
            </button>
          </div>
        </Transition>
      </div>

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

    <!-- 统计条 -->
    <div class="skill-list-stats">
      <span>{{ stats.logicalSkills }} skills</span>
      <span class="skill-list-stats__dot" />
      <span>{{ stats.installations }} installs</span>
      <span class="skill-list-stats__dot" />
      <span>{{ stats.sources }} sources</span>
    </div>

    <!-- 列表 -->
    <div class="skill-list-panel__scroll">
      <div
        v-if="groups.length === 0 && !loading"
        class="skill-list-panel__empty"
      >
        {{ searchQuery ? 'No skills match your search' : 'No skills installed' }}
      </div>

      <button
        v-for="group in groups"
        :key="group.items[0]?.id ?? group.name"
        type="button"
        class="skill-list-item"
        :class="{ 'skill-list-item--selected': selectedKeys.has(group.items[0]?.id ?? '') }"
        @click="$emit('select', group.items[0]?.id ?? '')"
      >
        <div class="skill-list-item__icon">
          <SIcon
            name="BookOpen"
            size="w-4 h-4"
          />
        </div>
        <div class="skill-list-item__content">
          <span class="skill-list-item__name">{{ group.name }}</span>
          <span class="skill-list-item__desc">{{ shortenDesc(group.description) }}</span>
        </div>
        <div class="skill-list-item__meta">
          <span class="skill-list-item__origin">{{ group.origin }}</span>
          <AgentIcons
            :agents="group.platforms"
            :compact="true"
          />
        </div>
      </button>
    </div>

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
import type { SkillGroup } from '@/types/skillsManager'

defineProps<{
  groups: SkillGroup[]
  searchQuery: string
  selectedKeys: Set<string>
  isMultiSelectMode: boolean
  loading: boolean
  stats: {
    logicalSkills: number;
    installations: number;
    sources: number;
  }
}>()

defineEmits<{
  'update:searchQuery': [value: string]
  'select': [skillId: string]
  'create': []
  'import': []
  'importGithub': []
  'refresh': []
  'toggleMultiSelect': []
  'bulkDelete': []
}>()

const showAddMenu = ref(false)

function shortenDesc(desc: string, max = 60): string {
  if (!desc) return 'No description'
  if (desc.length <= max) return desc
  return desc.slice(0, max).trimEnd() + '...'
}
</script>

<style scoped>
.skill-list-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.skill-list-stats {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  font-size: 0.6875rem;
  color: var(--color-text-muted);
  border-bottom: 1px solid rgb(var(--color-border-default-rgb) / 30%);

}

.skill-list-stats__dot {
  width: 3px;
  height: 3px;
  border-radius: 50%;
  background: var(--color-text-muted);
  opacity: 0.5;
}

.skill-list-panel__scroll {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;

}

.skill-list-panel__empty {
  padding: 1.5rem 0.75rem;
  text-align: center;
  font-size: 0.8125rem;
  color: var(--color-text-muted);
}

.skill-list-item {
  display: flex;
  align-items: flex-start;
  gap: 0.625rem;
  width: 100%;
  padding: 0.625rem 0.75rem;
  border-radius: 0.75rem;
  border: 1px solid transparent;
  background: transparent;
  text-align: left;
  cursor: pointer;
  transition: background-color var(--motion-subtle-duration) var(--motion-subtle-ease), border-color var(--motion-subtle-duration) var(--motion-subtle-ease);

}
.skill-list-item:hover { background: var(--surface-status-bg, rgb(var(--color-bg-elevated-rgb) / 72%)); }

.skill-list-item--selected {
  background: var(--surface-card-bg, rgb(var(--color-bg-elevated-rgb) / 85%));
  border-color: rgb(var(--color-accent-primary-rgb) / 15%);

}

.skill-list-item__icon {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 1.5rem;
  height: 1.5rem;
  margin-top: 0.125rem;
  color: var(--color-text-muted);

}

.skill-list-item__content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.skill-list-item__name {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.skill-list-item__desc {
  font-size: 0.6875rem;
  color: var(--color-text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.skill-list-item__meta {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 0.25rem;
  flex-shrink: 0;
}

.skill-list-item__origin {
  font-size: 0.5625rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--color-text-muted);
  padding: 0.0625rem 0.375rem;
  border-radius: 0.25rem;
  background: rgb(var(--color-bg-base-rgb) / 55%);

}

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
  transition: color var(--motion-subtle-duration) var(--motion-subtle-ease), background-color var(--motion-subtle-duration) var(--motion-subtle-ease);

}

.list-action-btn:hover {
  color: var(--color-text-primary);
  background: var(--surface-status-bg);
}

.list-action-btn--active {
  color: rgb(var(--color-accent-primary-rgb));
  background: rgb(var(--color-accent-primary-rgb) / 10%);
}

.add-menu {
  position: absolute;
  top: 100%;
  right: 0;
  z-index: 20;
  margin-top: 0.25rem;
  min-width: 12rem;
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
  transition: color var(--motion-subtle-duration) var(--motion-subtle-ease), background-color var(--motion-subtle-duration) var(--motion-subtle-ease);

}

.add-menu__item:hover {
  color: var(--color-text-primary);
  background: var(--surface-status-bg);
}
</style>
