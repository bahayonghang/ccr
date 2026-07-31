<template>
  <BaseModal
    :model-value="showAddAccountModal"
    :title="tf('codex.auth.actions.addAccount', 'Add account')"
    :description="
      tf(
        'codex.auth.addAccountDescription',
        'Add a Codex account through OAuth, token JSON, API key, or local import.'
      )
    "
    size="full"
    surface="glass"
    content-class="w-full max-w-[min(1120px,calc(100vw-2rem))] max-h-[92vh] overflow-y-auto"
    @update:model-value="(value) => !value && closeAddAccountModal()"
  >
    <template #header="{ titleId }">
      <div
        class="px-6 py-4 border-b border-border-default/10 flex items-center justify-between sticky top-0 bg-bg-elevated z-10"
      >
        <div>
          <h2
            :id="titleId"
            class="text-xl font-bold text-text-primary"
          >
            {{ tf('codex.auth.actions.addAccount', 'Add account') }}
          </h2>
          <p class="text-sm text-text-muted mt-1">
            {{
              tf(
                'codex.auth.addAccountDescription',
                'Store one or more Codex credentials and switch them from CCR.'
              )
            }}
          </p>
        </div>
        <Button
          variant="ghost"
          surface="status"
          density="compact"
          motion="subtle"
          @click="closeAddAccountModal"
        >
          <template #leading>
            <SIcon
              name="X"
              size="w-5 h-5"
            />
          </template>
        </Button>
      </div>
    </template>

    <div class="codex-auth-view__composer-shell">
      <aside class="codex-auth-view__composer-sidebar">
        <div class="codex-auth-view__composer-card">
          <p class="codex-auth-view__composer-eyebrow">
            {{ tf('codex.auth.naming.eyebrow', 'Account blueprint') }}
          </p>
          <h3 class="codex-auth-view__composer-title">
            {{ tf('codex.auth.naming.title', 'Decide how this account should be saved') }}
          </h3>
          <p class="codex-auth-view__composer-copy">
            {{
              tf(
                'codex.auth.naming.copy',
                'Choose the ingest method, give the account a clearer name if needed, then let CCR save or switch it in one flow.'
              )
            }}
          </p>

          <div class="codex-auth-view__composer-meta">
            <span class="codex-auth-view__meta-pill">
              {{ activeAddTabLabel }}
            </span>
            <span class="codex-auth-view__meta-pill codex-auth-view__meta-pill--muted">
              {{ preferredAccountNameBadge }}
            </span>
          </div>

          <label class="codex-auth-view__input-group codex-auth-view__input-group--full">
            <span class="codex-auth-view__input-label">
              {{ tf('codex.auth.naming.fieldLabel', 'Custom saved name') }}
            </span>
            <input
              v-model="addAccountDraft.preferredAccountName"
              data-testid="codex-add-account-name-input"
              type="text"
              class="input"
              :disabled="!canCustomizePreferredAccountName"
              :placeholder="
                tf(
                  'codex.auth.naming.placeholder',
                  'Optional. Leave empty to auto-generate from email, provider, or payload.'
                )
              "
            >
          </label>
          <p
            data-testid="codex-add-account-name-helper"
            class="codex-auth-view__composer-helper"
            :class="{
              'codex-auth-view__composer-helper--error': !!preferredAccountNameError,
            }"
          >
            {{ preferredAccountNameError || preferredAccountNameHelper }}
          </p>

          <div class="codex-auth-view__composer-rules">
            <span class="codex-auth-view__meta-pill codex-auth-view__meta-pill--soft">
              {{ tf('codex.auth.naming.rules.charset', 'Letters, numbers, _ and - only') }}
            </span>
            <span class="codex-auth-view__meta-pill codex-auth-view__meta-pill--soft">
              {{ tf('codex.auth.naming.rules.length', 'Max 32 characters') }}
            </span>
          </div>
        </div>
      </aside>

      <div class="codex-auth-view__composer-main">
        <div class="codex-auth-view__segment-row codex-auth-view__segment-row--modal">
          <button
            v-for="tab in addAccountTabs"
            :key="tab.value"
            type="button"
            class="codex-auth-view__segment codex-auth-view__segment--modal"
            :class="{ 'codex-auth-view__segment--active': activeAddMethod === tab.value }"
            @click="switchAddMethod(tab.value)"
          >
            <SIcon
              :name="tab.icon"
              size="w-4 h-4"
            />
            <span>{{ tab.label }}</span>
          </button>
        </div>

        <div
          v-if="addAccountNotice"
          class="codex-auth-view__inline-note"
        >
          {{ addAccountNotice }}
        </div>
        <div
          v-if="addAccountError"
          class="codex-auth-view__inline-error"
        >
          {{ addAccountError }}
        </div>

        <template v-if="activeAddMethod === 'oauth'">
          <Card
            surface="workspace"
            :elevation="1"
            motion="subtle"
            padding="lg"
          >
            <div class="codex-auth-view__title-inline">
              <SIcon
                name="Globe"
                size="w-5 h-5"
                class="codex-auth-view__section-icon"
              />
              <div>
                <h3 class="codex-auth-view__section-title">
                  {{ tf('codex.auth.oauth.title', 'OpenAI OAuth authorization') }}
                </h3>
                <p class="codex-auth-view__section-copy">
                  {{
                    tf(
                      'codex.auth.oauth.hint',
                      'CCR listens on http://localhost:1455/auth/callback. After the browser flow completes, the account will be imported and switched automatically.'
                    )
                  }}
                </p>
              </div>
            </div>

            <div
              v-if="oauthPortBusy && !oauthPending"
              class="codex-auth-view__warning-panel"
            >
              <div>
                <p class="font-medium text-text-primary">
                  {{ tf('codex.auth.oauth.portBusyTitle', 'Port 1455 is occupied') }}
                </p>
                <p class="text-sm text-text-muted mt-1">
                  {{
                    tf(
                      'codex.auth.oauth.portBusyHint',
                      'Release the callback port before starting OAuth, otherwise the browser redirect cannot be captured.'
                    )
                  }}
                </p>
              </div>
              <Button
                variant="secondary"
                surface="status"
                density="compact"
                motion="subtle"
                :disabled="oauthBusy"
                @click="handleReleaseOauthPort"
              >
                {{ tf('codex.auth.oauth.releasePort', 'Release port') }}
              </Button>
            </div>

            <div
              v-if="oauthTimeoutMessage"
              class="codex-auth-view__warning-panel"
            >
              <div>
                <p class="font-medium text-text-primary">
                  {{ tf('codex.auth.oauth.timeoutTitle', 'Authorization timed out') }}
                </p>
                <p class="text-sm text-text-muted mt-1">
                  {{ oauthTimeoutMessage }}
                </p>
              </div>
            </div>

            <div class="codex-auth-view__oauth-grid">
              <div class="codex-auth-view__oauth-actions">
                <Button
                  variant="primary"
                  surface="card"
                  density="compact"
                  motion="standard"
                  :disabled="
                    oauthBusy || (oauthPortBusy && !oauthPending) || !!preferredAccountNameError
                  "
                  @click="handleStartOauth"
                >
                  <template #leading>
                    <span
                      v-if="oauthBusy"
                      class="w-4 h-4 border-2 border-border-default/30 border-t-white rounded-full animate-spin"
                    />
                    <SIcon
                      v-else
                      :name="oauthPending ? 'ExternalLink' : 'PlayCircle'"
                      size="w-4 h-4"
                    />
                  </template>
                  {{
                    oauthPending
                      ? tf('codex.auth.oauth.openBrowser', 'Open browser again')
                      : tf('codex.auth.oauth.start', 'Start OAuth authorization')
                  }}
                </Button>

                <Button
                  v-if="oauthPending"
                  variant="secondary"
                  surface="status"
                  density="compact"
                  motion="subtle"
                  :disabled="oauthBusy"
                  @click="handleFinalizeOauth"
                >
                  {{ tf('codex.auth.oauth.finish', 'Finish login') }}
                </Button>

                <Button
                  v-if="oauthPending"
                  variant="secondary"
                  surface="status"
                  density="compact"
                  motion="subtle"
                  :disabled="oauthBusy"
                  @click="cancelOauthFlow"
                >
                  {{ tf('codex.auth.oauth.cancel', 'Cancel OAuth') }}
                </Button>
              </div>

              <label class="codex-auth-view__input-group codex-auth-view__input-group--full">
                <span class="codex-auth-view__input-label">{{
                  tf('codex.auth.oauth.authUrl', 'Authorization URL')
                }}</span>
                <textarea
                  :value="oauthAuthUrl"
                  rows="3"
                  class="codex-auth-view__textarea"
                  readonly
                />
              </label>

              <label class="codex-auth-view__input-group codex-auth-view__input-group--full">
                <span class="codex-auth-view__input-label">{{
                  tf('codex.auth.oauth.callbackUrl', 'Manual callback URL')
                }}</span>
                <textarea
                  v-model="oauthCallbackUrl"
                  rows="4"
                  class="codex-auth-view__textarea"
                  :placeholder="
                    tf(
                      'codex.auth.oauth.callbackPlaceholder',
                      'If the browser could not return to CCR, paste the final localhost callback URL here.'
                    )
                  "
                />
              </label>

              <div class="codex-auth-view__oauth-actions">
                <Button
                  variant="secondary"
                  surface="status"
                  density="compact"
                  motion="subtle"
                  :disabled="!oauthPending || oauthBusy || !oauthCallbackUrl.trim()"
                  @click="handleSubmitOauthCallback"
                >
                  {{ tf('codex.auth.oauth.submitCallback', 'Submit callback URL') }}
                </Button>
              </div>
            </div>
          </Card>
        </template>

        <template v-else-if="activeAddMethod === 'token'">
          <Card
            surface="workspace"
            :elevation="1"
            motion="subtle"
            padding="lg"
          >
            <div class="codex-auth-view__title-inline">
              <SIcon
                name="FileJson"
                size="w-5 h-5"
                class="codex-auth-view__section-icon"
              />
              <div>
                <h3 class="codex-auth-view__section-title">
                  {{ tf('codex.auth.import.title', 'Import token / auth JSON') }}
                </h3>
                <p class="codex-auth-view__section-copy">
                  {{
                    tf(
                      'codex.auth.import.hint',
                      'Paste a single auth.json payload or a Cockpit Tools-style export bundle. CCR will normalize and save each account entry.'
                    )
                  }}
                </p>
              </div>
            </div>

            <label class="codex-auth-view__input-group codex-auth-view__input-group--full">
              <span class="codex-auth-view__input-label">{{
                tf('codex.auth.import.payload', 'JSON payload')
              }}</span>
              <textarea
                v-model="importForm.content"
                rows="14"
                class="codex-auth-view__textarea codex-auth-view__textarea--mono"
                :placeholder="
                  tf(
                    'codex.auth.import.placeholder',
                    'Paste auth.json, export JSON, or a serialized Codex account payload here...'
                  )
                "
              />
            </label>

            <div class="codex-auth-view__checkbox-row">
              <label class="codex-auth-view__checkbox-label">
                <input
                  v-model="importForm.switchAfterImport"
                  type="checkbox"
                  :disabled="!canManageAuthAccounts"
                >
                <span>{{
                  tf(
                    'codex.auth.import.switchAfter',
                    'Switch to the first imported account immediately'
                  )
                }}</span>
              </label>
              <span
                v-if="!canManageAuthAccounts"
                class="codex-auth-view__checkbox-hint"
              >
                {{
                  tf(
                    'codex.auth.import.switchDisabledHint',
                    'Switch after import is unavailable until the current profile uses OpenAI auth.'
                  )
                }}
              </span>
            </div>

            <div class="codex-auth-view__provider-actions">
              <Button
                variant="primary"
                surface="card"
                density="compact"
                motion="standard"
                :disabled="importBusy || !importForm.content.trim() || !!preferredAccountNameError"
                @click="handleImportPayload"
              >
                <template #leading>
                  <span
                    v-if="importBusy"
                    class="w-4 h-4 border-2 border-border-default/30 border-t-white rounded-full animate-spin"
                  />
                  <SIcon
                    v-else
                    name="Download"
                    size="w-4 h-4"
                  />
                </template>
                {{ tf('codex.auth.import.action', 'Import payload') }}
              </Button>
            </div>
          </Card>
        </template>

        <template v-else-if="activeAddMethod === 'api'">
          <div class="codex-auth-view__providers-grid codex-auth-view__providers-grid--modal">
            <Card
              surface="workspace"
              :elevation="1"
              motion="subtle"
              padding="lg"
            >
              <div class="codex-auth-view__title-inline">
                <SIcon
                  name="KeyRound"
                  size="w-5 h-5"
                  class="codex-auth-view__section-icon"
                />
                <div>
                  <h3 class="codex-auth-view__section-title">
                    {{ tf('codex.auth.api.title', 'Create API key account') }}
                  </h3>
                  <p class="codex-auth-view__section-copy">
                    {{
                      tf(
                        'codex.auth.api.hint',
                        'Store one API key as a named Codex account, optionally attaching it to a reusable saved provider.'
                      )
                    }}
                  </p>
                </div>
              </div>

              <ProviderTemplateSelector
                class="mb-4"
                platform="codex"
                :selected-template-id="selectedApiProviderTemplate"
                :selected-endpoint="selectedApiProviderEndpoint"
                :draft-context="codexApiTemplateDraft"
                :label="tf('codex.auth.api.templateLabel', 'Provider template')"
                :helper="
                  tf(
                    'codex.auth.api.templateHelper',
                    'Fill the non-secret provider name and base URL from a reusable Codex template.'
                  )
                "
                @select="applyCodexApiProviderTemplate"
                @manual="useManualApiProviderTemplate"
              />

              <div class="codex-auth-view__provider-form">
                <label class="codex-auth-view__input-group">
                  <span class="codex-auth-view__input-label">{{
                    tf('codex.auth.api.fields.providerName', 'Provider name')
                  }}</span>
                  <input
                    v-model="apiKeyForm.providerName"
                    type="text"
                    class="input"
                    :placeholder="
                      tf(
                        'codex.auth.api.placeholders.providerName',
                        'Optional. Used as the saved account label when possible.'
                      )
                    "
                  >
                </label>
                <label class="codex-auth-view__input-group">
                  <span class="codex-auth-view__input-label">{{
                    tf('codex.auth.api.fields.baseUrl', 'Base URL')
                  }}</span>
                  <input
                    v-model="apiKeyForm.apiBaseUrl"
                    type="url"
                    class="input"
                    :placeholder="
                      tf(
                        'codex.auth.api.placeholders.baseUrl',
                        'Leave empty for the OpenAI default endpoint.'
                      )
                    "
                  >
                </label>
                <label class="codex-auth-view__input-group codex-auth-view__input-group--full">
                  <span class="codex-auth-view__input-label">{{
                    tf('codex.auth.api.fields.apiKey', 'API key')
                  }}</span>
                  <input
                    v-model="apiKeyForm.apiKey"
                    type="password"
                    class="input"
                    placeholder="sk-..."
                  >
                </label>
              </div>

              <div class="codex-auth-view__checkbox-row">
                <label class="codex-auth-view__checkbox-label">
                  <input
                    v-model="apiKeyForm.saveProvider"
                    type="checkbox"
                  >
                  <span>{{
                    tf('codex.auth.api.saveProvider', 'Also save/update saved provider')
                  }}</span>
                </label>
                <label class="codex-auth-view__checkbox-label">
                  <input
                    v-model="apiKeyForm.switchAfterAdd"
                    type="checkbox"
                    :disabled="!canManageAuthAccounts"
                  >
                  <span>{{
                    tf('codex.auth.api.switchAfter', 'Switch to the new API account immediately')
                  }}</span>
                </label>
              </div>

              <div class="codex-auth-view__provider-actions">
                <Button
                  variant="primary"
                  surface="card"
                  density="compact"
                  motion="standard"
                  :disabled="apiKeyBusy || !apiKeyForm.apiKey.trim() || !!preferredAccountNameError"
                  @click="handleAddApiKeyAccount"
                >
                  <template #leading>
                    <span
                      v-if="apiKeyBusy"
                      class="w-4 h-4 border-2 border-border-default/30 border-t-white rounded-full animate-spin"
                    />
                    <SIcon
                      v-else
                      name="Plus"
                      size="w-4 h-4"
                    />
                  </template>
                  {{ tf('codex.auth.api.action', 'Save API account') }}
                </Button>
              </div>
            </Card>

            <Card
              surface="workspace"
              :elevation="1"
              motion="subtle"
              padding="lg"
            >
              <div class="codex-auth-view__title-inline">
                <SIcon
                  name="Blocks"
                  size="w-5 h-5"
                  class="codex-auth-view__section-icon"
                />
                <div>
                  <h3 class="codex-auth-view__section-title">
                    {{ tf('codex.auth.api.presetsTitle', 'Saved providers') }}
                  </h3>
                  <p class="codex-auth-view__section-copy">
                    {{
                      tf(
                        'codex.auth.api.presetsHint',
                        'Click one saved provider to fill the API key form with its stored base URL and the latest saved key.'
                      )
                    }}
                  </p>
                </div>
              </div>

              <div
                v-if="providers.length === 0"
                class="empty-state rounded-2xl border border-border-default/10 bg-bg-elevated"
              >
                <p class="text-text-primary">
                  {{ tf('codex.auth.api.noPresets', 'No saved providers yet') }}
                </p>
                <p class="text-sm text-text-muted mt-2">
                  {{
                    tf(
                      'codex.auth.api.noPresetsHint',
                      'Create saved providers in the Model providers tab if you want reusable third-party endpoints.'
                    )
                  }}
                </p>
              </div>

              <div
                v-else
                class="codex-auth-view__preset-list"
              >
                <button
                  v-for="provider in providers"
                  :key="provider.id"
                  type="button"
                  class="codex-auth-view__preset"
                  @click="applyProviderToApiForm(provider)"
                >
                  <span class="codex-auth-view__preset-name">{{ provider.name }}</span>
                  <span class="codex-auth-view__preset-url">{{ provider.base_url }}</span>
                  <span class="codex-auth-view__preset-meta">{{ provider.api_keys.length }}
                    {{ tf('codex.auth.providers.badges.keys', 'keys') }}</span>
                </button>
              </div>
            </Card>
          </div>
        </template>

        <template v-else>
          <Card
            surface="workspace"
            :elevation="1"
            motion="subtle"
            padding="lg"
          >
            <div class="codex-auth-view__title-inline">
              <SIcon
                name="FolderDown"
                size="w-5 h-5"
                class="codex-auth-view__section-icon"
              />
              <div>
                <h3 class="codex-auth-view__section-title">
                  {{ tf('codex.auth.localImport.title', 'Import from local Codex runtime') }}
                </h3>
                <p class="codex-auth-view__section-copy">
                  {{
                    tf(
                      'codex.auth.localImport.hint',
                      'Capture the current ~/.codex auth snapshot into CCR without editing JSON manually.'
                    )
                  }}
                </p>
              </div>
            </div>

            <div class="codex-auth-view__warning-panel codex-auth-view__warning-panel--neutral">
              <div>
                <p class="font-medium text-text-primary">
                  {{
                    tf(
                      'codex.auth.localImport.summary',
                      'This reads the active local auth.json and turns it into a managed CCR account entry.'
                    )
                  }}
                </p>
                <p class="text-sm text-text-muted mt-1">
                  {{
                    tf(
                      'codex.auth.localImport.note',
                      'Use this when the Codex CLI is already authenticated on the machine and you want CCR to adopt that state.'
                    )
                  }}
                </p>
              </div>
            </div>

            <div class="codex-auth-view__provider-actions">
              <Button
                variant="primary"
                surface="card"
                density="compact"
                motion="standard"
                :disabled="localImportBusy || !!preferredAccountNameError"
                @click="handleImportFromLocal"
              >
                <template #leading>
                  <span
                    v-if="localImportBusy"
                    class="w-4 h-4 border-2 border-border-default/30 border-t-white rounded-full animate-spin"
                  />
                  <SIcon
                    v-else
                    name="FolderDown"
                    size="w-4 h-4"
                  />
                </template>
                {{ tf('codex.auth.localImport.action', 'Import local runtime account') }}
              </Button>
            </div>
          </Card>
        </template>
      </div>
    </div>
  </BaseModal>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import BaseModal from '@/components/common/BaseModal.vue'
import Button from '@/components/ui/Button.vue'
import Card from '@/components/ui/Card.vue'
import SIcon from '@/components/ui/SIcon.vue'
import ProviderTemplateSelector from '@/components/provider-templates/ProviderTemplateSelector.vue'
import { useTf } from '@/composables/useTf'
import { useUIStore } from '@/stores/ui'
import { useCodexOAuthFlow } from '@/composables/useCodexOAuthFlow'
import {
  codexAddAuthWithApiKey,
  codexImportAuthFromLocal,
  codexImportAuthPayload,
  codexOAuthLoginCancel,
} from '@/api'
import {
  canCustomizeAccountName,
  detectImportPayloadNamingState,
  getAccountNameValidationMessage,
  normalizeAccountNameInput,
  type ImportPayloadNamingState,
} from '../codexAuthAccounts'
import type {
  CodexAddApiKeyAuthPayload,
  CodexAuthMutationResponse,
  CodexImportAuthPayload,
  CodexModelProviderRecord,
} from '@/types'
import type {
  ProviderTemplateDraftContext,
  ProviderTemplateSelection,
} from '@/types/providerTemplates'
import { mapTemplateToCodexApiAccountPatch } from '@/utils/providerTemplates'
import { extractErrorMessage } from '@/utils/errorHandler'
import { logger } from '@/utils/logger'

type AddMethod = 'oauth' | 'token' | 'api' | 'local'

defineOptions({ name: 'AddCodexAccountModal' })

const props = withDefaults(
  defineProps<{
    modelValue: boolean
    providers: CodexModelProviderRecord[]
    canManageAuthAccounts: boolean
    initialMethod?: AddMethod
    presetProvider?: CodexModelProviderRecord | null
    // 账号增删改成功后由主视图刷新列表；以可 await 的回调注入，确保「刷新完成→再关闭」的时序
    refreshOnMutation?: () => Promise<void> | void
  }>(),
  {
    initialMethod: 'oauth',
    presetProvider: null,
    refreshOnMutation: undefined,
  }
)

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const tf = useTf()
const uiStore = useUIStore()

// 开关态由父组件 v-model 控制；可写计算属性既供模板绑定，也供 oauth composable 关闭弹窗
const showAddAccountModal = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
})

const activeAddMethod = ref<AddMethod>('oauth')
const addAccountError = ref<string | null>(null)
const addAccountNotice = ref<string | null>(null)
const importBusy = ref(false)
const apiKeyBusy = ref(false)
const localImportBusy = ref(false)

const importForm = reactive({
  content: '',
  switchAfterImport: true,
})

const addAccountDraft = reactive({
  preferredAccountName: '',
})

const apiKeyForm = reactive({
  apiKey: '',
  apiBaseUrl: '',
  providerName: '',
  saveProvider: false,
  switchAfterAdd: true,
})

const selectedApiProviderTemplate = ref<string | null>(null)
const selectedApiProviderEndpoint = ref('')

const codexApiTemplateDraft = computed<ProviderTemplateDraftContext>(() => ({
  platform: 'codex',
  defaultName: apiKeyForm.providerName || 'Codex API provider',
  name: apiKeyForm.providerName,
  category: 'third_party',
  baseUrls: apiKeyForm.apiBaseUrl.trim() ? [apiKeyForm.apiBaseUrl.trim()] : [],
  platformOverride: {
    baseUrl: apiKeyForm.apiBaseUrl,
  },
}))

const validateAccountNameInput = (value: string | null) => {
  const validationMessage = getAccountNameValidationMessage(value)
  switch (validationMessage) {
    case 'reserved':
      return tf(
        'codex.auth.naming.validation.reserved',
        '"default" is reserved. Please choose another account name.'
      )
    case 'length':
      return tf(
        'codex.auth.naming.validation.length',
        'Account names must stay within 32 characters.'
      )
    case 'charset':
      return tf(
        'codex.auth.naming.validation.charset',
        'Use letters, numbers, underscores, and hyphens only.'
      )
    case null:
    default:
      return null
  }
}

const addAccountTabs = computed(() => [
  { value: 'oauth' as const, label: tf('codex.auth.methods.oauth', 'OAuth'), icon: 'Globe' },
  {
    value: 'token' as const,
    label: tf('codex.auth.methods.token', 'Token / JSON'),
    icon: 'FileJson',
  },
  { value: 'api' as const, label: tf('codex.auth.methods.api', 'API Key'), icon: 'KeyRound' },
  {
    value: 'local' as const,
    label: tf('codex.auth.methods.local', 'Local import'),
    icon: 'FolderDown',
  },
])

const activeAddTabLabel = computed(() => {
  return (
    addAccountTabs.value.find((tab) => tab.value === activeAddMethod.value)?.label ||
    tf('codex.auth.naming.meta.unknownMethod', 'Account flow')
  )
})

const importPayloadNamingState = computed<ImportPayloadNamingState>(() =>
  detectImportPayloadNamingState(importForm.content)
)

const canCustomizePreferredAccountName = computed(() =>
  canCustomizeAccountName(activeAddMethod.value, importPayloadNamingState.value)
)

const normalizedPreferredAccountName = computed(() => {
  return normalizeAccountNameInput(addAccountDraft.preferredAccountName)
})

const preferredAccountNameError = computed(() => {
  if (!canCustomizePreferredAccountName.value) return null
  return validateAccountNameInput(normalizedPreferredAccountName.value)
})

const effectivePreferredAccountName = computed(() => {
  if (!canCustomizePreferredAccountName.value || preferredAccountNameError.value) {
    return null
  }
  return normalizedPreferredAccountName.value
})

const preferredAccountNameBadge = computed(() => {
  if (!canCustomizePreferredAccountName.value) {
    return tf('codex.auth.naming.meta.lockedToPayload', 'Locked to payload naming')
  }
  if (effectivePreferredAccountName.value) {
    return tf('codex.auth.naming.meta.customName', 'Custom name ready')
  }
  return tf('codex.auth.naming.meta.autoName', 'Auto-name from runtime data')
})

const preferredAccountNameHelper = computed(() => {
  if (activeAddMethod.value === 'token') {
    switch (importPayloadNamingState.value) {
      case 'bundle':
        return tf(
          'codex.auth.naming.helper.bundleLocked',
          'Export bundles keep their embedded account names. Custom renaming is disabled for this import mode.'
        )
      case 'multiple':
        return tf(
          'codex.auth.naming.helper.multiLocked',
          'Bulk JSON imports may create multiple accounts, so custom renaming is disabled here.'
        )
      case 'invalid':
        return tf(
          'codex.auth.naming.helper.invalidJson',
          'Once the payload resolves to a single valid account, this custom name will become available again.'
        )
      case 'single':
        return effectivePreferredAccountName.value
          ? tf(
              'codex.auth.naming.helper.singleCustom',
              'This name will override the payload-derived account label for the imported account.'
            )
          : tf(
              'codex.auth.naming.helper.singleAuto',
              'Leave this empty to auto-name the imported account from the payload email, provider, or account id.'
            )
      case 'empty':
      default:
        return tf(
          'codex.auth.naming.helper.empty',
          'Optional. Leave it blank until you know whether you want a custom label.'
        )
    }
  }

  if (effectivePreferredAccountName.value) {
    return tf(
      'codex.auth.naming.helper.custom',
      'CCR will save the next account with this exact name instead of generating one automatically.'
    )
  }

  return tf(
    'codex.auth.naming.helper.auto',
    'Leave this empty to let CCR derive the account name from email, provider, or runtime metadata.'
  )
})

const resetAddAccountDraft = () => {
  addAccountDraft.preferredAccountName = ''
}

const ensurePreferredAccountNameIsValid = () => {
  if (preferredAccountNameError.value) {
    addAccountError.value = preferredAccountNameError.value
    return false
  }
  return true
}

const useManualApiProviderTemplate = () => {
  selectedApiProviderTemplate.value = null
  selectedApiProviderEndpoint.value = ''
}

const applyCodexApiProviderTemplate = (selection: ProviderTemplateSelection) => {
  const patch = mapTemplateToCodexApiAccountPatch(selection.template, selection.endpoint)

  selectedApiProviderTemplate.value = selection.template.id
  selectedApiProviderEndpoint.value = selection.endpoint || ''
  apiKeyForm.providerName = patch.providerName || selection.template.name
  apiKeyForm.apiBaseUrl = patch.apiBaseUrl || ''
  addAccountError.value = null
}

// 模板预设回填 API 表单：既供弹窗内"已保存提供商"列表，也供打开时携带的 presetProvider
const applyProviderToApiForm = (provider: CodexModelProviderRecord) => {
  apiKeyForm.providerName = provider.name
  apiKeyForm.apiBaseUrl = provider.base_url
  apiKeyForm.apiKey = provider.api_keys[0]?.api_key || apiKeyForm.apiKey
  apiKeyForm.saveProvider = false
  useManualApiProviderTemplate()
  activeAddMethod.value = 'api'
  addAccountNotice.value = tf(
    'codex.auth.api.presetApplied',
    'Loaded saved provider "{name}" into the API key form.',
    { name: provider.name }
  )
}

// 成功添加账号后的统一收尾（函数声明以便下方 composable 在初始化时按名引用）；
// 先 await 主视图刷新，再提示/重置——保证关闭弹窗时列表已是最新，避免闪烁旧数据
async function applyMutationSuccess(result: CodexAuthMutationResponse, successMessage: string) {
  await props.refreshOnMutation?.()
  uiStore.showSuccess(successMessage)
  addAccountNotice.value = result.account_name
    ? tf('codex.auth.feedback.savedAs', 'Saved as {name}.', { name: result.account_name })
    : successMessage
  resetOauthState()
}

const {
  oauthLoginId,
  oauthAuthUrl,
  oauthCallbackUrl,
  oauthPending,
  oauthPortBusy,
  oauthBusy,
  oauthTimeoutMessage,
  resetOauthState,
  refreshOauthPortStatus,
  handleReleaseOauthPort,
  handleStartOauth,
  handleSubmitOauthCallback,
  handleFinalizeOauth,
  cancelOauthFlow,
  installOauthListeners,
  cleanupOauthListeners,
} = useCodexOAuthFlow({
  effectivePreferredAccountName,
  ensurePreferredAccountNameIsValid,
  applyMutationSuccess,
  addAccountError,
  addAccountNotice,
  showAddAccountModal,
})

const closeAddAccountModal = async () => {
  showAddAccountModal.value = false
  addAccountError.value = null
  addAccountNotice.value = null
  oauthTimeoutMessage.value = null
  if (oauthPending.value && oauthLoginId.value) {
    try {
      await codexOAuthLoginCancel(oauthLoginId.value)
    } catch (error) {
      logger.warn('Failed to cancel oauth flow while closing modal:', error)
    }
  }
  resetOauthState()
  resetAddAccountDraft()
}

const switchAddMethod = async (method: AddMethod) => {
  activeAddMethod.value = method
  addAccountError.value = null
  addAccountNotice.value = null
  if (method !== 'api') {
    useManualApiProviderTemplate()
  }
  if (method === 'oauth') {
    await refreshOauthPortStatus()
  }
}

const handleImportPayload = async () => {
  addAccountError.value = null
  addAccountNotice.value = null
  if (!importForm.content.trim()) {
    addAccountError.value = tf(
      'codex.auth.import.validation.contentRequired',
      'Paste a JSON payload before importing it.'
    )
    return
  }
  if (!ensurePreferredAccountNameIsValid()) {
    return
  }

  try {
    importBusy.value = true
    const payload: CodexImportAuthPayload = {
      content: importForm.content,
      switchAfterImport: importForm.switchAfterImport && props.canManageAuthAccounts,
      preferredAccountName:
        importPayloadNamingState.value === 'single'
          ? effectivePreferredAccountName.value ?? undefined
          : undefined,
    }
    const result = await codexImportAuthPayload(payload)
    await applyMutationSuccess(
      result,
      tf('codex.auth.import.success', 'Imported account payload successfully.')
    )
    importForm.content = ''
    showAddAccountModal.value = false
  } catch (error) {
    addAccountError.value =
      extractErrorMessage(error) ||
      tf('codex.auth.import.failed', 'Failed to import the JSON payload.')
  } finally {
    importBusy.value = false
  }
}

const handleImportFromLocal = async () => {
  addAccountError.value = null
  addAccountNotice.value = null
  if (!ensurePreferredAccountNameIsValid()) {
    return
  }
  try {
    localImportBusy.value = true
    const result = await codexImportAuthFromLocal(
      effectivePreferredAccountName.value
    )
    await applyMutationSuccess(
      result,
      tf('codex.auth.localImport.success', 'Imported the local runtime account successfully.')
    )
    showAddAccountModal.value = false
  } catch (error) {
    addAccountError.value =
      extractErrorMessage(error) ||
      tf('codex.auth.localImport.failed', 'Failed to import the local runtime account.')
  } finally {
    localImportBusy.value = false
  }
}

const handleAddApiKeyAccount = async () => {
  addAccountError.value = null
  addAccountNotice.value = null
  if (!apiKeyForm.apiKey.trim()) {
    addAccountError.value = tf(
      'codex.auth.api.validation.apiKeyRequired',
      'Enter an API key before saving the account.'
    )
    return
  }
  if (!ensurePreferredAccountNameIsValid()) {
    return
  }

  try {
    apiKeyBusy.value = true
    const payload: CodexAddApiKeyAuthPayload = {
      apiKey: apiKeyForm.apiKey.trim(),
      apiBaseUrl: apiKeyForm.apiBaseUrl.trim() || undefined,
      providerName: apiKeyForm.providerName.trim() || undefined,
      saveProvider: apiKeyForm.saveProvider,
      switchAfterAdd: apiKeyForm.switchAfterAdd && props.canManageAuthAccounts,
      preferredAccountName: effectivePreferredAccountName.value ?? undefined,
    }
    const result = await codexAddAuthWithApiKey(payload)
    await applyMutationSuccess(
      result,
      tf('codex.auth.api.success', 'API key account added successfully.')
    )
    apiKeyForm.apiKey = ''
    showAddAccountModal.value = false
  } catch (error) {
    addAccountError.value =
      extractErrorMessage(error) ||
      tf('codex.auth.api.failed', 'Failed to save the API key account.')
  } finally {
    apiKeyBusy.value = false
  }
}

// 弹窗打开时重置草稿/方法，并按 presetProvider 决定是否回填 API 表单
watch(
  () => props.modelValue,
  async (open) => {
    if (!open) return
    activeAddMethod.value = props.initialMethod
    addAccountError.value = null
    addAccountNotice.value = null
    oauthTimeoutMessage.value = null
    resetAddAccountDraft()
    if (props.presetProvider) {
      applyProviderToApiForm(props.presetProvider)
    } else {
      useManualApiProviderTemplate()
    }
    await refreshOauthPortStatus()
  }
)

onMounted(async () => {
  await installOauthListeners()
})

onBeforeUnmount(() => {
  void cleanupOauthListeners()
})
</script>
