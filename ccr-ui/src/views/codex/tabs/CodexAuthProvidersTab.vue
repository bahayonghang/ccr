<template>
  <div class="codex-auth-providers-tab">
    <div class="codex-auth-view__providers-grid">
      <Card
        surface="workspace"
        :elevation="2"
        motion="subtle"
        padding="lg"
      >
        <div
          class="codex-auth-view__section-header codex-auth-view__section-header--spread"
        >
          <div class="codex-auth-view__title-inline">
            <SIcon
              name="Blocks"
              size="w-5 h-5"
              class="codex-auth-view__section-icon"
            />
            <div>
              <h3 class="codex-auth-view__section-title">
                {{ tf('codex.auth.providers.formTitle', 'Saved provider') }}
              </h3>
              <p class="codex-auth-view__section-copy">
                {{
                  tf(
                    'codex.auth.providers.formHint',
                    'Save reusable base URLs and optional API keys. Provider templates only fill non-secret metadata.'
                  )
                }}
              </p>
            </div>
          </div>
          <Button
            v-if="
              providerForm.id ||
                providerForm.name ||
                providerForm.baseUrl ||
                providerForm.apiKey
            "
            variant="secondary"
            surface="status"
            density="compact"
            motion="subtle"
            @click="$emit('resetForm')"
          >
            {{ tf('codex.auth.providers.resetForm', 'Reset form') }}
          </Button>
        </div>

        <ProviderTemplateSelector
          class="codex-auth-view__template-selector"
          platform="codex"
          :selected-template-id="selectedProviderTemplate"
          :selected-endpoint="selectedProviderEndpoint"
          :draft-context="codexTemplateDraft"
          label="Provider template"
          helper="Search non-secret templates by name, host, tag, or model. API keys stay in the saved provider form."
          @select="$emit('applyTemplate', $event)"
          @manual="$emit('useManualTemplate')"
        />

        <div class="codex-auth-view__provider-form">
          <label class="codex-auth-view__input-group">
            <span class="codex-auth-view__input-label">{{
              tf('codex.auth.providers.fields.name', 'Provider name')
            }}</span>
            <input
              :value="providerForm.name"
              type="text"
              class="input"
              :placeholder="
                tf(
                  'codex.auth.providers.placeholders.name',
                  'e.g. OpenRouter / Azure OpenAI / Local gateway'
                )
              "
              @input="$emit('update:providerForm', { ...providerForm, name: ($event.target as HTMLInputElement).value })"
            >
          </label>
          <label class="codex-auth-view__input-group">
            <span class="codex-auth-view__input-label">{{
              tf('codex.auth.providers.fields.baseUrl', 'Base URL')
            }}</span>
            <input
              :value="providerForm.baseUrl"
              type="url"
              class="input"
              placeholder="https://..."
              @input="$emit('update:providerForm', { ...providerForm, baseUrl: ($event.target as HTMLInputElement).value })"
            >
          </label>
          <label class="codex-auth-view__input-group">
            <span class="codex-auth-view__input-label">{{
              tf('codex.auth.providers.fields.websiteUrl', 'Website URL')
            }}</span>
            <input
              :value="providerForm.websiteUrl"
              type="url"
              class="input"
              :placeholder="
                tf(
                  'codex.auth.providers.placeholders.websiteUrl',
                  'Optional reference link'
                )
              "
              @input="$emit('update:providerForm', { ...providerForm, websiteUrl: ($event.target as HTMLInputElement).value })"
            >
          </label>
          <label class="codex-auth-view__input-group">
            <span class="codex-auth-view__input-label">{{
              tf('codex.auth.providers.fields.apiKeyUrl', 'API key docs URL')
            }}</span>
            <input
              :value="providerForm.apiKeyUrl"
              type="url"
              class="input"
              :placeholder="
                tf(
                  'codex.auth.providers.placeholders.apiKeyUrl',
                  'Optional onboarding link'
                )
              "
              @input="$emit('update:providerForm', { ...providerForm, apiKeyUrl: ($event.target as HTMLInputElement).value })"
            >
          </label>
          <label class="codex-auth-view__input-group">
            <span class="codex-auth-view__input-label">{{
              tf('codex.auth.providers.fields.apiKeyName', 'Stored key label')
            }}</span>
            <input
              :value="providerForm.apiKeyName"
              type="text"
              class="input"
              :placeholder="
                tf('codex.auth.providers.placeholders.apiKeyName', 'Default: API Key')
              "
              @input="$emit('update:providerForm', { ...providerForm, apiKeyName: ($event.target as HTMLInputElement).value })"
            >
          </label>
          <label class="codex-auth-view__input-group">
            <span class="codex-auth-view__input-label">{{
              tf('codex.auth.providers.fields.apiKey', 'Stored API key')
            }}</span>
            <input
              :value="providerForm.apiKey"
              type="password"
              class="input"
              :placeholder="
                tf(
                  'codex.auth.providers.placeholders.apiKey',
                  'Optional. Leave empty to keep existing keys unchanged.'
                )
              "
              @input="$emit('update:providerForm', { ...providerForm, apiKey: ($event.target as HTMLInputElement).value })"
            >
          </label>
        </div>

        <div
          v-if="providerError"
          class="codex-auth-view__inline-error"
        >
          {{ providerError }}
        </div>

        <div class="codex-auth-view__provider-actions">
          <Button
            variant="primary"
            surface="card"
            density="compact"
            motion="standard"
            :disabled="
              providerSaving || !providerForm.name.trim() || !providerForm.baseUrl.trim()
            "
            @click="$emit('saveProvider')"
          >
            <template #leading>
              <span
                v-if="providerSaving"
                class="w-4 h-4 border-2 border-border-default/30 border-t-white rounded-full animate-spin"
              />
              <SIcon
                v-else
                :name="providerForm.id ? 'Save' : 'Plus'"
                size="w-4 h-4"
              />
            </template>
            {{
              providerForm.id
                ? tf('codex.auth.providers.actions.update', 'Update provider')
                : tf('codex.auth.providers.actions.create', 'Save provider')
            }}
          </Button>
        </div>
      </Card>

      <Card
        surface="workspace"
        :elevation="2"
        motion="subtle"
        padding="lg"
      >
        <div
          class="codex-auth-view__section-header codex-auth-view__section-header--spread"
        >
          <div class="codex-auth-view__title-inline">
            <SIcon
              name="Globe"
              size="w-5 h-5"
              class="codex-auth-view__section-icon"
            />
            <div>
              <h3 class="codex-auth-view__section-title">
                {{ tf('codex.auth.providers.listTitle', 'Saved providers') }}
              </h3>
              <p class="codex-auth-view__section-copy">
                {{
                  tf(
                    'codex.auth.providers.listHint',
                    'Saved providers can include API keys and can be injected directly into the API key account flow.'
                  )
                }}
              </p>
            </div>
          </div>
          <Button
            variant="secondary"
            surface="status"
            density="compact"
            motion="subtle"
            :disabled="providerLoading"
            @click="$emit('loadProviders')"
          >
            <template #leading>
              <SIcon
                name="RefreshCw"
                size="w-4 h-4"
                :class="{ 'animate-spin': providerLoading }"
              />
            </template>
            {{ $t('codex.auth.refresh') }}
          </Button>
        </div>

        <div
          v-if="providerLoading"
          class="space-y-3"
        >
          <div
            v-for="index in 3"
            :key="index"
            class="h-24 rounded-2xl bg-bg-surface/70 animate-pulse"
          />
        </div>

        <div
          v-else-if="providers.length === 0"
          class="empty-state rounded-2xl border border-border-default/10 bg-bg-surface/40"
        >
          <div class="p-4 rounded-full glass-surface mb-4">
            <SIcon
              name="Blocks"
              size="w-8 h-8"
              class="text-text-muted"
            />
          </div>
          <p class="text-text-primary">
            {{ tf('codex.auth.providers.emptyState', 'No saved providers yet') }}
          </p>
          <p class="text-sm text-text-muted mt-2">
            {{
              tf(
                'codex.auth.providers.emptyHint',
                'Create a saved provider if you often switch between OpenAI-compatible gateways.'
              )
            }}
          </p>
        </div>

        <div
          v-else
          class="codex-auth-view__provider-list"
        >
          <article
            v-for="provider in providers"
            :key="provider.id"
            class="codex-auth-view__provider-card"
          >
            <div class="codex-auth-view__provider-head">
              <div>
                <h4 class="codex-auth-view__provider-title">
                  {{ provider.name }}
                </h4>
                <p class="codex-auth-view__provider-url">
                  {{ provider.base_url }}
                </p>
              </div>
              <div class="codex-auth-view__provider-badges">
                <span class="codex-auth-view__provider-badge">
                  {{ provider.api_keys.length }}
                  {{ tf('codex.auth.providers.badges.keys', 'keys') }}
                </span>
                <span
                  class="codex-auth-view__provider-badge codex-auth-view__provider-badge--muted"
                >
                  {{ formatProviderUpdatedAt(provider.updated_at) }}
                </span>
              </div>
            </div>

            <div class="codex-auth-view__provider-meta">
              <a
                v-if="provider.website_url"
                :href="provider.website_url"
                target="_blank"
                rel="noreferrer"
                class="codex-auth-view__provider-link"
              >
                {{ tf('codex.auth.providers.links.website', 'Website') }}
              </a>
              <a
                v-if="provider.api_key_url"
                :href="provider.api_key_url"
                target="_blank"
                rel="noreferrer"
                class="codex-auth-view__provider-link"
              >
                {{ tf('codex.auth.providers.links.apiKeyDocs', 'API key docs') }}
              </a>
            </div>

            <div class="codex-auth-view__provider-footer">
              <div class="codex-auth-view__provider-copy">
                <span>{{ tf('codex.auth.providers.updatedAt', 'Updated') }}
                  {{ formatProviderUpdatedAt(provider.updated_at, true) }}</span>
              </div>
              <div class="codex-auth-view__provider-actions-inline">
                <Button
                  variant="secondary"
                  surface="status"
                  density="compact"
                  motion="subtle"
                  @click="$emit('useInApiForm', provider)"
                >
                  {{ tf('codex.auth.providers.actions.useInApiForm', 'Use in API form') }}
                </Button>
                <Button
                  variant="secondary"
                  surface="status"
                  density="compact"
                  motion="subtle"
                  @click="$emit('editProvider', provider)"
                >
                  {{ tf('common.edit', 'Edit') }}
                </Button>
                <Button
                  variant="danger"
                  surface="status"
                  density="compact"
                  motion="subtle"
                  @click="$emit('deleteProvider', provider)"
                >
                  {{ $t('codex.actions.delete') }}
                </Button>
              </div>
            </div>
          </article>
        </div>
      </Card>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { CodexModelProviderRecord } from '@/types'
import type { ProviderTemplateDraftContext, ProviderTemplateSelection } from '@/types/providerTemplates'
import SIcon from '@/components/ui/SIcon.vue'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import ProviderTemplateSelector from '@/components/provider-templates/ProviderTemplateSelector.vue'
import { useTf } from '@/composables/useTf'

defineOptions({ name: 'CodexAuthProvidersTab' })

interface ProviderForm {
  id: string
  name: string
  baseUrl: string
  websiteUrl: string
  apiKeyUrl: string
  apiKeyName: string
  apiKey: string
}

interface Props {
  providerForm: ProviderForm
  providerError: string | null
  providerSaving: boolean
  providerLoading: boolean
  providers: CodexModelProviderRecord[]
  selectedProviderTemplate: string | null
  selectedProviderEndpoint: string
  codexTemplateDraft: ProviderTemplateDraftContext
  formatProviderUpdatedAt: (timestamp: string, short?: boolean) => string
}

defineProps<Props>()

defineEmits<{
  'update:providerForm': [value: ProviderForm]
  resetForm: []
  applyTemplate: [selection: ProviderTemplateSelection]
  useManualTemplate: []
  saveProvider: []
  loadProviders: []
  useInApiForm: [provider: CodexModelProviderRecord]
  editProvider: [provider: CodexModelProviderRecord]
  deleteProvider: [provider: CodexModelProviderRecord]
}>()

const tf = useTf()
</script>

<style scoped>
/* 样式继承自 CodexAuthView.vue，无需重复定义 */
</style>
