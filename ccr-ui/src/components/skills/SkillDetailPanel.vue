<template>
  <div class="skill-detail-panel">
    <div
      v-if="!skill"
      class="skill-detail-panel__empty"
    >
      <SIcon
        name="BookOpen"
        size="w-8 h-8"
        class="text-text-muted/40"
      />
      <p>Select a skill to view details</p>
    </div>

    <template v-else>
      <!-- 头部 -->
      <div class="detail-header">
        <div class="detail-header__info">
          <div>
            <h2 class="detail-header__title">
              {{ skill.name }}
            </h2>
            <p class="detail-header__sub">
              {{ skill.origin }} · {{ skill.installCount }} installation(s)
            </p>
          </div>
        </div>
        <div class="detail-header__actions">
          <button
            type="button"
            class="detail-btn detail-btn--danger"
            @click="$emit('remove', skill.id)"
          >
            <SIcon
              name="Trash2"
              size="w-4 h-4"
            />
            <span>Remove</span>
          </button>
        </div>
      </div>

      <!-- 描述 -->
      <section
        v-if="skill.description"
        class="detail-section"
      >
        <p class="detail-desc">
          {{ skill.description }}
        </p>
      </section>

      <!-- 元数据 -->
      <section class="detail-section">
        <h3 class="detail-section__title">
          Metadata
        </h3>
        <div class="detail-grid">
          <div class="detail-field">
            <span class="detail-field__label">Origin</span>
            <span class="detail-field__value">{{ skill.origin }}</span>
          </div>
          <div class="detail-field">
            <span class="detail-field__label">Author</span>
            <span class="detail-field__value">{{ skill.author || 'Unknown' }}</span>
          </div>
          <div class="detail-field">
            <span class="detail-field__label">Version</span>
            <span class="detail-field__value">{{ skill.version || 'N/A' }}</span>
          </div>
          <div class="detail-field">
            <span class="detail-field__label">Category</span>
            <span class="detail-field__value">{{ skill.category || 'Uncategorized' }}</span>
          </div>
        </div>
        <div
          v-if="skill.tags.length > 0"
          class="detail-tags"
        >
          <span
            v-for="tag in skill.tags"
            :key="tag"
            class="detail-tag"
          >{{ tag }}</span>
        </div>
      </section>

      <!-- 安装列表 -->
      <section class="detail-section">
        <div class="detail-section__header">
          <h3 class="detail-section__title">
            Installations
          </h3>
          <button
            v-if="selectedPlatforms.length > 0"
            type="button"
            class="detail-btn"
            @click="$emit('sync', skill)"
          >
            <SIcon
              name="CopyPlus"
              size="w-4 h-4"
            />
            <span>Sync to selected</span>
          </button>
        </div>
        <div class="detail-install-list">
          <div
            v-for="inst in skill.installations"
            :key="inst.id"
            class="detail-install-row"
          >
            <AgentIcons
              :agents="[inst.platformId]"
              :compact="false"
            />
            <span class="detail-install-path">{{ shortenPath(inst.installPath) }}</span>
            <button
              type="button"
              class="detail-btn detail-btn--sm detail-btn--danger"
              @click="$emit('removeInstallation', skill.id, inst.id)"
            >
              <SIcon
                name="Trash2"
                size="w-3.5 h-3.5"
              />
            </button>
          </div>
          <p
            v-if="skill.installations.length === 0"
            class="detail-install-empty"
          >
            No installations
          </p>
        </div>
      </section>

      <!-- 内容预览 -->
      <section
        v-if="content"
        class="detail-section"
      >
        <div class="detail-section__header">
          <h3 class="detail-section__title">
            Content Preview
          </h3>
        </div>
        <pre class="detail-content-preview">{{ content }}</pre>
      </section>
    </template>
  </div>
</template>

<script setup lang="ts">
import { watch, ref } from 'vue'
import AgentIcons from '@/components/common/AgentIcons.vue'
import SIcon from '@/components/ui/SIcon.vue'
import type { SkillRecord, Platform } from '@/types/skills'

const props = defineProps<{
  skill: SkillRecord | null
  selectedPlatforms: Platform[]
  ensureContent: (skillId: string, installationId?: string | null, force?: boolean) => Promise<{ raw: string } | null>
}>()

defineEmits<{
  remove: [skillId: string]
  removeInstallation: [skillId: string, installationId: string]
  sync: [skill: SkillRecord]
}>()

const content = ref<string | null>(null)

watch(() => props.skill?.id, async (id) => {
  content.value = null
  if (!id || !props.skill?.installations.length) return
  try {
    const result = await props.ensureContent(id, props.skill.installations[0]?.id, false)
    if (result?.raw) {
      // 去掉 frontmatter 只显示正文
      const stripped = stripFrontmatter(result.raw)
      content.value = stripped.length > 500 ? stripped.slice(0, 500) + '\n...' : stripped
    }
  } catch { /* 静默处理 */ }
}, { immediate: true })

function stripFrontmatter(raw: string): string {
  const trimmed = raw.replace(/\r\n/g, '\n').trimStart()
  const lines = trimmed.split('\n')
  if (lines[0]?.trim() !== '---') return raw
  for (let i = 1; i < lines.length; i++) {
    if (lines[i].trim() === '---') return lines.slice(i + 1).join('\n').trim()
  }
  return raw
}

function shortenPath(path: string): string {
  const parts = path.replace(/\\/g, '/').split('/')
  return parts.length <= 4 ? path : '.../' + parts.slice(-4).join('/')
}
</script>

<style scoped>
.skill-detail-panel {
  height: 100%;
  overflow-y: auto;
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.skill-detail-panel__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
  height: 100%;
  font-size: 0.875rem;
  color: var(--color-text-muted);
}

.detail-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;

}

.detail-header__title {
  font-size: 1.125rem;
  font-weight: 700;
  color: var(--color-text-primary);
}

.detail-header__sub {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  margin-top: 0.125rem;
}

.detail-header__actions {
  display: flex;
  gap: 0.5rem;
  flex-shrink: 0;
}

.detail-desc {
  font-size: 0.875rem;
  line-height: 1.6;
  color: var(--color-text-secondary);

}

.detail-section {
  padding: 1rem;
  border-radius: 1rem;
  border: 1px solid var(--surface-card-border, rgb(var(--color-border-default-rgb) / 45%));
  background: var(--surface-card-bg);
  backdrop-filter: var(--surface-card-blur);
  box-shadow: var(--elevation-1);

}

.detail-section__title {
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.12em;
  color: var(--color-text-muted);
  margin-bottom: 0.75rem;
}

.detail-section__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  margin-bottom: 0.75rem;
}
.detail-section__header .detail-section__title { margin-bottom: 0; }

.detail-grid {
  display: grid;
  gap: 0.75rem;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));

}

.detail-field__label {
  display: block;
  font-size: 0.6875rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--color-text-muted);
  margin-bottom: 0.25rem;
}

.detail-field__value {
  font-size: 0.8125rem;
  color: var(--color-text-primary);
}

.detail-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
  margin-top: 0.75rem;

}

.detail-tag {
  padding: 0.125rem 0.5rem;
  border-radius: 9999px;
  font-size: 0.6875rem;
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 15%);
  color: rgb(var(--color-accent-primary-rgb));

}

.detail-install-list {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;

}

.detail-install-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.5rem 0.625rem;
  border-radius: 0.625rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 35%);
  background: rgb(var(--color-bg-base-rgb) / 42%);

}

.detail-install-path {
  flex: 1;
  font-family: var(--font-mono);
  font-size: 0.6875rem;
  color: var(--color-text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.detail-install-empty {
  font-size: 0.8125rem;
  color: var(--color-text-muted);
  text-align: center;
  padding: 0.75rem 0;
}

.detail-content-preview {
  padding: 0.75rem;
  border-radius: 0.625rem;
  background: rgb(var(--color-bg-base-rgb) / 55%);
  font-family: var(--font-mono);
  font-size: 0.6875rem;
  line-height: 1.6;
  color: var(--color-text-secondary);
  white-space: pre-wrap;
  overflow-wrap: break-word;
  max-height: 16rem;
  overflow-y: auto;

}

.detail-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.375rem 0.75rem;
  border-radius: 0.625rem;
  font-size: 0.8125rem;
  font-weight: 500;
  border: 1px solid var(--surface-status-border, rgb(var(--color-border-default-rgb) / 55%));
  background: var(--surface-status-bg);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all var(--motion-subtle-duration) var(--motion-subtle-ease);

}

.detail-btn:hover {
  color: var(--color-text-primary);
  background: var(--surface-card-bg);
}
.detail-btn--danger { color: rgb(239 68 68 / 85%); }

.detail-btn--danger:hover {
  color: rgb(239 68 68);
  background: rgb(239 68 68 / 8%);
}
.detail-btn--sm { padding: 0.25rem 0.5rem; }
</style>
