<template>
  <div class="codex-auth-view">
    <div class="codex-auth-view__container">
      <div class="codex-auth-view__stack">
        <ModuleSubnav module="codex" />

        <main class="codex-auth-view__main">
          <div class="codex-auth-view__header">
            <div class="codex-auth-view__title-group">
              <div class="codex-auth-view__title-icon-shell">
                <SIcon
                  name="KeyRound"
                  size="w-6 h-6"
                  class="codex-auth-view__title-icon"
                />
              </div>
              <div>
                <h1 class="codex-auth-view__title">
                  {{ $t('codex.auth.title') }}
                </h1>
                <p class="codex-auth-view__subtitle">
                  {{
                    tf(
                      'codex.auth.managerSubtitle',
                      'Use one surface to add, import, switch, and review Codex accounts and model providers.'
                    )
                  }}
                </p>
              </div>
            </div>

            <div class="codex-auth-view__actions">
              <RouterLink
                to="/codex"
                class="inline-flex"
              >
                <Button
                  variant="secondary"
                  surface="status"
                  density="compact"
                  motion="subtle"
                >
                  <template #leading>
                    <SIcon
                      name="ArrowLeft"
                      size="w-4 h-4"
                    />
                  </template>
                  {{ $t('codex.auth.backToCodex') }}
                </Button>
              </RouterLink>

              <Button
                variant="secondary"
                surface="status"
                density="compact"
                motion="subtle"
                :disabled="loading"
                @click="handleRefresh"
              >
                <template #leading>
                  <SIcon
                    name="RefreshCw"
                    size="w-4 h-4"
                    :class="{ 'animate-spin': loading }"
                  />
                </template>
                {{ $t('codex.auth.refresh') }}
              </Button>

              <Button
                variant="secondary"
                surface="status"
                density="compact"
                motion="subtle"
                :disabled="!canSave"
                @click="handleSave"
              >
                <template #leading>
                  <SIcon
                    name="Save"
                    size="w-4 h-4"
                  />
                </template>
                {{ tf('codex.auth.actions.saveCurrent', 'Save current session') }}
              </Button>

              <Button
                variant="primary"
                surface="card"
                density="compact"
                motion="standard"
                @click="openAddAccountModal()"
              >
                <template #leading>
                  <SIcon
                    name="Plus"
                    size="w-4 h-4"
                  />
                </template>
                {{ tf('codex.auth.actions.addAccount', 'Add account') }}
              </Button>
            </div>
          </div>

          <div class="codex-auth-view__status-grid">
            <Card
              surface="status"
              :elevation="2"
              motion="subtle"
              :gradient-border="true"
              :glow-color="loginStateColor"
              class="codex-auth-view__status-card"
            >
              <div class="codex-auth-view__status-row">
                <div
                  class="codex-auth-view__status-icon-shell"
                  :class="loginStateIconClass"
                >
                  <SIcon
                    :name="loginStateIcon"
                    size="w-6 h-6"
                  />
                </div>
                <div>
                  <p class="codex-auth-view__status-label">
                    {{ $t('codex.auth.status.loginState') }}
                  </p>
                  <p class="codex-auth-view__status-value codex-auth-view__status-value--truncate">
                    {{ loginStateText }}
                  </p>
                </div>
              </div>
            </Card>

            <Card
              surface="status"
              :elevation="2"
              motion="subtle"
              class="codex-auth-view__status-card"
            >
              <div class="codex-auth-view__status-row">
                <div
                  class="codex-auth-view__status-icon-shell codex-auth-view__status-icon-shell--info"
                >
                  <SIcon
                    name="Users"
                    size="w-6 h-6"
                  />
                </div>
                <div>
                  <p class="codex-auth-view__status-label">
                    {{ $t('codex.auth.status.totalAccounts') }}
                  </p>
                  <p class="codex-auth-view__status-value">
                    {{ accounts.length }}
                  </p>
                </div>
              </div>
            </Card>

            <Card
              surface="status"
              :elevation="2"
              motion="subtle"
              class="codex-auth-view__status-card"
            >
              <div class="codex-auth-view__status-row">
                <div
                  class="codex-auth-view__status-icon-shell"
                  :class="
                    currentAccount
                      ? 'bg-emerald-500/10 text-emerald-500'
                      : 'bg-gray-500/10 text-text-muted'
                  "
                >
                  <SIcon
                    name="UserCheck"
                    size="w-6 h-6"
                  />
                </div>
                <div>
                  <p class="codex-auth-view__status-label">
                    {{ $t('codex.auth.status.currentAccount') }}
                  </p>
                  <p class="codex-auth-view__status-value codex-auth-view__status-value--truncate">
                    {{
                      currentAccount?.email ||
                        currentAccount?.name ||
                        $t('codex.auth.status.noAccount')
                    }}
                  </p>
                </div>
              </div>
            </Card>

            <Card
              surface="status"
              :elevation="2"
              motion="subtle"
              class="codex-auth-view__status-card"
            >
              <div class="codex-auth-view__status-row">
                <div
                  class="codex-auth-view__status-icon-shell codex-auth-view__status-icon-shell--neutral"
                >
                  <SIcon
                    name="Globe"
                    size="w-6 h-6"
                  />
                </div>
                <div>
                  <p class="codex-auth-view__status-label">
                    {{ tf('codex.auth.status.providerCount', 'Model providers') }}
                  </p>
                  <p class="codex-auth-view__status-value">
                    {{ providers.length }}
                  </p>
                </div>
              </div>
            </Card>
          </div>

          <Card
            surface="workspace"
            :elevation="2"
            motion="subtle"
            padding="lg"
            class="codex-auth-view__segment-card"
          >
            <div class="codex-auth-view__segment-row">
              <button
                type="button"
                class="codex-auth-view__segment"
                :class="{ 'codex-auth-view__segment--active': activeManagerTab === 'accounts' }"
                @click="activeManagerTab = 'accounts'"
              >
                <SIcon
                  name="LayoutGrid"
                  size="w-4 h-4"
                />
                <span>{{ $t('codex.auth.accountOverview') }}</span>
                <span class="codex-auth-view__segment-count">{{ accounts.length }}</span>
              </button>
              <button
                type="button"
                class="codex-auth-view__segment"
                :class="{ 'codex-auth-view__segment--active': activeManagerTab === 'providers' }"
                @click="activeManagerTab = 'providers'"
              >
                <SIcon
                  name="Blocks"
                  size="w-4 h-4"
                />
                <span>{{ tf('codex.auth.providers.title', 'Model providers') }}</span>
                <span class="codex-auth-view__segment-count">{{ providers.length }}</span>
              </button>
            </div>
          </Card>


          <CodexAuthAccountsTab
            v-if="activeManagerTab === 'accounts'"
            v-model:search-query="searchQuery"
            v-model:status-filter="statusFilter"
            v-model:plan-filter="planFilter"
            v-model:sort-by="sortBy"
            :loading="loading"
            :accounts="accounts"
            :current-info="currentInfo"
            :can-manage-auth-accounts="canManageAuthAccounts"
            :profile-guard-message="profileGuardMessage"
            :auth-action-error="authActionError"
            :status-options="statusOptions"
            :plan-options="planOptions"
            :sort-options="sortOptions"
            :filtered-accounts="filteredAccounts"
            :filters-results-count="filtersResultsCount"
            :has-active-filters="hasActiveFilters"
            :quota-map="quotaMap"
            :quota-loading="quotaLoading"
            :busy-name="busyName"
            :busy-action="busyAction"
            :action-loading="actionLoading"
            :format-auth-method="formatAuthMethod"
            @clear-filters="clearFilters"
            @open-add-account="openAddAccountModal()"
            @switch="handleSwitch"
            @delete="handleDelete"
            @refresh="handleRefreshSingle"
            @tag="handleTag"
            @export="handleExport"
            @rename="handleRename"
          />

          <CodexAuthProvidersTab
            v-else
            :provider-form="providerForm"
            :provider-error="providerError"
            :provider-saving="providerSaving"
            :provider-loading="providerLoading"
            :providers="providers"
            :selected-provider-template="selectedProviderTemplate"
            :selected-provider-endpoint="selectedProviderEndpoint"
            :codex-template-draft="codexTemplateDraft"
            :format-provider-updated-at="formatProviderUpdatedAt"
            @update:provider-form="(val) => Object.assign(providerForm, val)"
            @reset-form="resetProviderForm"
            @apply-template="applyCodexProviderTemplate"
            @use-manual-template="useManualProviderTemplate"
            @save-provider="handleSaveProvider"
            @load-providers="loadProviders"
            @use-in-api-form="applyProviderToApiForm"
            @edit-provider="editProvider"
            @delete-provider="requestDeleteProvider"
          />

          <SaveCodexSessionModal
            v-model="showSaveForm"
            :current-info="currentInfo"
            :format-auth-method="formatAuthMethod"
            @saved="handleRefresh"
          />

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
                class="px-6 py-4 border-b border-border-default/10 flex items-center justify-between sticky top-0 bg-bg-elevated/95 backdrop-blur z-10"
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
                            oauthBusy ||
                              (oauthPortBusy && !oauthPending) ||
                              !!preferredAccountNameError
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

                      <label
                        class="codex-auth-view__input-group codex-auth-view__input-group--full"
                      >
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

                      <label
                        class="codex-auth-view__input-group codex-auth-view__input-group--full"
                      >
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
                        :disabled="
                          importBusy || !importForm.content.trim() || !!preferredAccountNameError
                        "
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
                  <div
                    class="codex-auth-view__providers-grid codex-auth-view__providers-grid--modal"
                  >
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
                        <label
                          class="codex-auth-view__input-group codex-auth-view__input-group--full"
                        >
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
                            tf(
                              'codex.auth.api.switchAfter',
                              'Switch to the new API account immediately'
                            )
                          }}</span>
                        </label>
                      </div>

                      <div class="codex-auth-view__provider-actions">
                        <Button
                          variant="primary"
                          surface="card"
                          density="compact"
                          motion="standard"
                          :disabled="
                            apiKeyBusy || !apiKeyForm.apiKey.trim() || !!preferredAccountNameError
                          "
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
                        class="empty-state rounded-2xl border border-border-default/10 bg-bg-surface/40"
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
                          {{
                            tf('codex.auth.localImport.title', 'Import from local Codex runtime')
                          }}
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

                    <div
                      class="codex-auth-view__warning-panel codex-auth-view__warning-panel--neutral"
                    >
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

          <ConfirmModal
            v-model:is-open="showConfirmModal"
            :type="confirmDialog.type"
            :title="confirmDialog.title"
            :message="confirmDialog.message"
            :confirm-text="confirmDialog.confirmText"
            :cancel-text="$t('common.cancel')"
            @confirm="executeConfirmedAction"
          />

          <RenameCodexAccountModal
            v-model="showRenameDialog"
            :account-name="renameTarget"
            @renamed="handleRefresh"
          />
        </main>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { computed, onActivated, onBeforeUnmount, onMounted, reactive, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import BaseModal from '@/components/common/BaseModal.vue'
import Button from '@/components/ui/Button.vue'
import Card from '@/components/ui/Card.vue'
import CodexAuthAccountsTab from './codex/tabs/CodexAuthAccountsTab.vue'
import CodexAuthProvidersTab from './codex/tabs/CodexAuthProvidersTab.vue'
import RenameCodexAccountModal from './codex/components/RenameCodexAccountModal.vue'
import SaveCodexSessionModal from './codex/components/SaveCodexSessionModal.vue'
import ConfirmModal from '@/components/ConfirmModal.vue'
import ProviderTemplateSelector from '@/components/provider-templates/ProviderTemplateSelector.vue'
import { translateWithFallback } from '@/i18n/formatMessage'
import { useTf } from '@/composables/useTf'
import {
  listCodexProfiles,
  listCodexAuthAccounts,
  getCodexAuthCurrent,
  switchCodexAuth,
  deleteCodexAuth,
  getCodexAllQuotas,
  codexOAuthLoginCancel,
  codexImportAuthPayload,
  codexImportAuthFromLocal,
  codexAddAuthWithApiKey,
  codexListModelProviders,
  codexSaveModelProvider,
  codexDeleteModelProvider,
} from '@/api'
import {
  canCustomizeAccountName,
  detectImportPayloadNamingState,
  filterAndSortCodexAccounts,
  getAccountNameValidationMessage,
  getLoginStateIcon,
  getLoginStateIconClass,
  getLoginStateTone,
  normalizeAccountNameInput,
  usesOpenAiAuthMode,
  type AccountPlanFilter,
  type AccountSort,
  type AccountStatusFilter,
  type ImportPayloadNamingState,
} from './codex/codexAuthAccounts'
import type {
  CodexAccountQuota,
  CodexAddApiKeyAuthPayload,
  CodexAuthAccountItem,
  CodexAuthCurrentInfo,
  CodexAuthCurrentResponse,
  CodexAuthListResponse,
  CodexAuthMutationResponse,
  CodexImportAuthPayload,
  CodexModelProviderRecord,
  CodexModelProvidersResponse,
  CodexProfile,
  CodexProfilesResponse,
  LoginState,
} from '@/types'
import type { ProviderTemplateDraftContext, ProviderTemplateSelection } from '@/types/providerTemplates'
import { logger } from '@/utils/logger'
import { extractErrorMessage } from '@/utils/errorHandler'
import { useCodexOAuthFlow } from '@/composables/useCodexOAuthFlow'
import { useUIStore } from '@/stores/ui'
import {
  mapTemplateToCodexApiAccountPatch,
  mapTemplateToCodexProviderPatch,
} from '@/utils/providerTemplates'
import { REFRESH_TTL_MS } from '@/config/constants'

defineOptions({ name: 'CodexAuthView' })

const { t } = useI18n()
const uiStore = useUIStore()

type ManagerTab = 'accounts' | 'providers'
type AddMethod = 'oauth' | 'token' | 'api' | 'local'

const loading = ref(false)
const actionLoading = ref(false)
const quotaLoading = ref(false)
const providerLoading = ref(false)
const providerSaving = ref(false)
const importBusy = ref(false)
const apiKeyBusy = ref(false)
const localImportBusy = ref(false)

const accounts = ref<CodexAuthAccountItem[]>([])
const providers = ref<CodexModelProviderRecord[]>([])
const loginState = ref<LoginState>({ type: 'NotLoggedIn' })
const currentInfo = ref<CodexAuthCurrentInfo | null>(null)
const currentProfile = ref<CodexProfile | null>(null)
const authActionError = ref<string | null>(null)
const providerError = ref<string | null>(null)
const addAccountError = ref<string | null>(null)
const addAccountNotice = ref<string | null>(null)
const quotaMap = ref<Map<string, CodexAccountQuota>>(new Map())

const activeManagerTab = ref<ManagerTab>('accounts')
const activeAddMethod = ref<AddMethod>('oauth')
const showSaveForm = ref(false)
const showAddAccountModal = ref(false)
const busyName = ref<string | null>(null)
const busyAction = ref<'switch' | 'delete' | null>(null)
const showConfirmModal = ref(false)
const lastLoadedAt = ref(0)

const confirmDialog = reactive<{
  title: string
  message: string
  confirmText: string
  type: 'danger' | 'info' | 'warning'
}>({
  title: '',
  message: '',
  confirmText: '',
  type: 'warning',
})
let confirmedAction: (() => Promise<void>) | null = null

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

const providerForm = reactive({
  id: '',
  name: '',
  baseUrl: '',
  websiteUrl: '',
  apiKeyUrl: '',
  apiKeyName: 'API Key',
  apiKey: '',
})
const selectedProviderTemplate = ref<string | null>(null)
const selectedProviderEndpoint = ref('')
const selectedApiProviderTemplate = ref<string | null>(null)
const selectedApiProviderEndpoint = ref('')

const codexTemplateDraft = computed<ProviderTemplateDraftContext>(() => ({
  platform: 'codex',
  defaultName: providerForm.name || 'Codex provider',
  name: providerForm.name,
  websiteUrl: providerForm.websiteUrl,
  apiKeyUrl: providerForm.apiKeyUrl,
  category: 'third_party',
  baseUrls: providerForm.baseUrl.trim() ? [providerForm.baseUrl.trim()] : [],
  platformOverride: {
    baseUrl: providerForm.baseUrl,
    websiteUrl: providerForm.websiteUrl,
    apiKeyUrl: providerForm.apiKeyUrl,
  },
}))

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

const searchQuery = ref('')
const statusFilter = ref<AccountStatusFilter>('all')
const planFilter = ref<AccountPlanFilter>('all')
const sortBy = ref<AccountSort>('saved_desc')

const tf = useTf()

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

const currentAccount = computed(() => accounts.value.find((account) => account.is_current))
const canManageAuthAccounts = computed(() => usesOpenAiAuthMode(currentProfile.value?.auth_mode))
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

const profileGuardMessage = computed(() => {
  if (!currentProfile.value) {
    return t('codex.auth.profileGuard.noCurrentProfile')
  }
  if (!canManageAuthAccounts.value) {
    return tf(
      'codex.auth.profileGuard.unsupportedProfile',
      'Current profile "{name}" uses "{authMode}". Codex Auth account save/switch only works for OpenAI-auth current profiles.',
      {
        name: currentProfile.value.name,
        authMode: currentProfile.value.auth_mode || 'no_auth',
      }
    )
  }
  return tf(
    'codex.auth.profileGuard.supportedProfile',
    'Current profile "{name}" uses "{authMode}". Auth account save/switch is available.',
    {
      name: currentProfile.value.name,
      authMode: currentProfile.value.auth_mode || 'openai_chatgpt',
    }
  )
})

const canSave = computed(() => {
  return (
    canManageAuthAccounts.value &&
    (loginState.value.type === 'LoggedInUnsaved' || loginState.value.type === 'LoggedInSaved')
  )
})

const loginStateColor = computed(() => getLoginStateTone(loginState.value))

const loginStateIcon = computed(() => getLoginStateIcon(loginState.value))

const loginStateIconClass = computed(() => getLoginStateIconClass(loginState.value))

const loginStateText = computed(() => {
  switch (loginState.value.type) {
    case 'LoggedInSaved':
      return tf('codex.auth.loginState.loggedInSaved', 'Logged in ({name})', {
        name: loginState.value.account_name,
      })
    case 'LoggedInUnsaved':
      return t('codex.auth.loginState.loggedInUnsaved')
    case 'ApiKeyActive':
      return t('codex.auth.loginState.apiKeyActive')
    case 'ProviderKeyActive':
      return tf('codex.auth.loginState.providerKeyActive', 'Provider Key ({envKey})', {
        envKey: loginState.value.env_key,
      })
    case 'Unknown':
      return tf('codex.auth.loginState.unknown', 'Unknown state ({type})', {
        type: loginState.value.raw_type,
      })
    default:
      return t('codex.auth.loginState.notLoggedIn')
  }
})

const statusOptions = computed(() => [
  { value: 'all' as const, label: t('codex.auth.filters.statusOptions.all') },
  { value: 'current' as const, label: t('codex.auth.filters.statusOptions.current') },
  { value: 'virtual' as const, label: t('codex.auth.filters.statusOptions.virtual') },
  { value: 'attention' as const, label: t('codex.auth.filters.statusOptions.attention') },
])

const planOptions = computed(() => [
  { value: 'all' as const, label: t('codex.auth.filters.planOptions.all') },
  { value: 'plus' as const, label: t('codex.auth.filters.planOptions.plus') },
  { value: 'pro' as const, label: t('codex.auth.filters.planOptions.pro') },
  { value: 'team' as const, label: t('codex.auth.filters.planOptions.team') },
  { value: 'unknown' as const, label: t('codex.auth.filters.planOptions.unknown') },
])

const sortOptions = computed(() => [
  { value: 'saved_desc' as const, label: t('codex.auth.filters.sortOptions.savedDesc') },
  { value: 'used_desc' as const, label: t('codex.auth.filters.sortOptions.usedDesc') },
  { value: 'name_asc' as const, label: t('codex.auth.filters.sortOptions.nameAsc') },
])

const hasActiveFilters = computed(() => {
  return (
    Boolean(searchQuery.value.trim()) ||
    statusFilter.value !== 'all' ||
    planFilter.value !== 'all' ||
    sortBy.value !== 'saved_desc'
  )
})

const filteredAccounts = computed(() =>
  filterAndSortCodexAccounts({
    accounts: accounts.value,
    quotaMap: quotaMap.value,
    searchQuery: searchQuery.value,
    statusFilter: statusFilter.value,
    planFilter: planFilter.value,
    sortBy: sortBy.value,
  })
)

const filtersResultsCount = computed(() =>
  tf('codex.auth.filters.resultsCount', '{shown} / {total} accounts', {
    shown: filteredAccounts.value.length,
    total: accounts.value.length,
  })
)

const formatAuthMethod = (method?: string | null) => {
  switch (method) {
    case 'chatgpt':
      return tf('codex.auth.authMethods.chatgpt', 'ChatGPT OAuth')
    case 'api':
      return tf('codex.auth.authMethods.api', 'API Key')
    case 'provider':
      return tf('codex.auth.authMethods.provider', 'Provider key')
    default:
      return tf('codex.auth.authMethods.unknown', 'Unknown')
  }
}

const formatProviderUpdatedAt = (value?: string | null, detailed = false) => {
  if (!value) return t('common.notAvailable')
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return detailed
    ? date.toLocaleString()
    : new Intl.DateTimeFormat('zh-CN', {
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
      }).format(date)
}

const clearFilters = () => {
  searchQuery.value = ''
  statusFilter.value = 'all'
  planFilter.value = 'all'
  sortBy.value = 'saved_desc'
}

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

const loadCurrentProfile = async () => {
  try {
    const data = await listCodexProfiles<CodexProfilesResponse>()
    currentProfile.value =
      data.profiles.find((profile) => profile.name === data.current_profile) || null
  } catch (error) {
    logger.error('Failed to load current codex profile:', error)
    currentProfile.value = null
  }
}

const loadAccounts = async () => {
  try {
    loading.value = true
    authActionError.value = null
    const data = await listCodexAuthAccounts<CodexAuthListResponse>()
    accounts.value = data.accounts || []
    loginState.value = data.login_state
  } catch (error) {
    logger.error('Failed to load codex auth accounts:', error)
    uiStore.showError(extractErrorMessage(error) || t('codex.states.loadFailed'))
  } finally {
    loading.value = false
  }
}

const loadCurrentInfo = async () => {
  try {
    const data = await getCodexAuthCurrent<CodexAuthCurrentResponse>()
    currentInfo.value = data.logged_in && data.info ? data.info : null
  } catch (error) {
    logger.error('Failed to load current auth info:', error)
  }
}

const loadQuotas = async () => {
  try {
    quotaLoading.value = true
    const data = await getCodexAllQuotas<CodexAccountQuota[]>()
    const map = new Map<string, CodexAccountQuota>()
    for (const quota of data) {
      map.set(quota.account_name, quota)
    }
    quotaMap.value = map
  } catch (error) {
    logger.error('Failed to fetch codex quotas:', error)
  } finally {
    quotaLoading.value = false
  }
}

const loadProviders = async () => {
  try {
    providerLoading.value = true
    providerError.value = null
    const data = await codexListModelProviders<CodexModelProvidersResponse>()
    providers.value = data.providers || []
  } catch (error) {
    logger.error('Failed to load codex providers:', error)
    providerError.value =
      extractErrorMessage(error) ||
      tf('codex.auth.providers.loadFailed', 'Failed to load saved providers.')
  } finally {
    providerLoading.value = false
  }
}

const handleRefresh = async () => {
  await Promise.all([
    loadAccounts(),
    loadCurrentInfo(),
    loadCurrentProfile(),
    loadQuotas(),
    loadProviders(),
  ])
  lastLoadedAt.value = Date.now()
}

const ensureLoaded = async (force = false) => {
  if (loading.value) return
  if (!force && lastLoadedAt.value && Date.now() - lastLoadedAt.value < REFRESH_TTL_MS) {
    return
  }
  await handleRefresh()
}

const openConfirmDialog = (options: {
  title: string
  message: string
  confirmText: string
  type: 'danger' | 'info' | 'warning'
  action: () => Promise<void>
}) => {
  confirmDialog.title = options.title
  confirmDialog.message = options.message
  confirmDialog.confirmText = options.confirmText
  confirmDialog.type = options.type
  confirmedAction = options.action
  showConfirmModal.value = true
}

const executeConfirmedAction = async () => {
  if (!confirmedAction) return
  actionLoading.value = true
  try {
    await confirmedAction()
  } finally {
    actionLoading.value = false
    confirmedAction = null
  }
}

// 保存当前会话弹窗自身负责进程探测/表单/提交（SaveCodexSessionModal），
// 主视图仅在 canSave 允许时打开它（头部按钮已按 canSave 禁用）
const handleSave = () => {
  showSaveForm.value = true
}

const handleSwitch = async (name: string) => {
  authActionError.value = null
  openConfirmDialog({
    title: t('codex.auth.switch'),
    message: translateWithFallback(
      t,
      'codex.auth.confirmSwitch',
      '确定要切换到账户 "{name}" 吗？',
      { name }
    ),
    confirmText: t('codex.auth.switch'),
    type: 'warning',
    action: async () => {
      busyName.value = name
      busyAction.value = 'switch'
      try {
        await switchCodexAuth(name)
        await handleRefresh()
        uiStore.showSuccess(
          tf('codex.auth.feedback.switchSuccess', 'Switched account successfully.')
        )
      } catch (error) {
        logger.error('Failed to switch auth:', error)
        authActionError.value = extractErrorMessage(error) || t('codex.states.saveFailed')
        uiStore.showError(authActionError.value)
      } finally {
        busyName.value = null
        busyAction.value = null
      }
    },
  })
}

const handleDelete = async (name: string) => {
  authActionError.value = null
  openConfirmDialog({
    title: t('codex.actions.delete'),
    message: translateWithFallback(t, 'codex.auth.deleteConfirm', '确定要删除账户 "{name}" 吗？', {
      name,
    }),
    confirmText: t('codex.actions.delete'),
    type: 'danger',
    action: async () => {
      busyName.value = name
      busyAction.value = 'delete'
      try {
        await deleteCodexAuth(name)
        await handleRefresh()
        uiStore.showSuccess(
          tf('codex.auth.feedback.deleteSuccess', 'Account deleted successfully.')
        )
      } catch (error) {
        logger.error('Failed to delete auth:', error)
        authActionError.value = extractErrorMessage(error) || t('codex.states.deleteFailed')
        uiStore.showError(authActionError.value)
      } finally {
        busyName.value = null
        busyAction.value = null
      }
    },
  })
}

const handleRefreshSingle = async () => {
  await loadQuotas()
}

const handleTag = (_name: string) => {
  uiStore.showInfo(t('codex.auth.featureComingSoon'))
}

const handleExport = (_name: string) => {
  uiStore.showInfo(t('codex.auth.featureComingSoon'))
}

// ── 重命名账号 ──
// 弹窗自身负责表单/校验/提交（RenameCodexAccountModal），主视图仅做 profile 守卫 + 打开

const showRenameDialog = ref(false)
const renameTarget = ref('')

const handleRename = (name: string) => {
  if (!canManageAuthAccounts.value) {
    authActionError.value = profileGuardMessage.value
    return
  }
  renameTarget.value = name
  showRenameDialog.value = true
}

// OAuth 子流程（端口探测/启动/回调/完成/取消 + tauri 事件监听）抽至 useCodexOAuthFlow，
// 与添加账号弹窗共享命名草稿与成功收尾回调
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

const openAddAccountModal = async (method: AddMethod = 'oauth') => {
  showAddAccountModal.value = true
  activeAddMethod.value = method
  addAccountError.value = null
  addAccountNotice.value = null
  oauthTimeoutMessage.value = null
  resetAddAccountDraft()
  useManualApiProviderTemplate()
  await refreshOauthPortStatus()
}

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

// 函数声明（hoisted）以便上方 useCodexOAuthFlow 在初始化时按名引用
async function applyMutationSuccess(result: CodexAuthMutationResponse, successMessage: string) {
  await handleRefresh()
  uiStore.showSuccess(successMessage)
  addAccountNotice.value = result.account_name
    ? tf('codex.auth.feedback.savedAs', 'Saved as {name}.', { name: result.account_name })
    : successMessage
  resetOauthState()
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
      switchAfterImport: importForm.switchAfterImport && canManageAuthAccounts.value,
      preferredAccountName:
        importPayloadNamingState.value === 'single' ? effectivePreferredAccountName.value : null,
    }
    const result = await codexImportAuthPayload<CodexAuthMutationResponse>(payload)
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
    const result = await codexImportAuthFromLocal<CodexAuthMutationResponse>(
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
      apiBaseUrl: apiKeyForm.apiBaseUrl.trim() || null,
      providerName: apiKeyForm.providerName.trim() || null,
      saveProvider: apiKeyForm.saveProvider,
      switchAfterAdd: apiKeyForm.switchAfterAdd && canManageAuthAccounts.value,
      preferredAccountName: effectivePreferredAccountName.value,
    }
    const result = await codexAddAuthWithApiKey<CodexAuthMutationResponse>(payload)
    await applyMutationSuccess(
      result,
      tf('codex.auth.api.success', 'API key account added successfully.')
    )
    if (apiKeyForm.saveProvider) {
      await loadProviders()
    }
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

const resetProviderForm = () => {
  providerForm.id = ''
  providerForm.name = ''
  providerForm.baseUrl = ''
  providerForm.websiteUrl = ''
  providerForm.apiKeyUrl = ''
  providerForm.apiKeyName = 'API Key'
  providerForm.apiKey = ''
  providerError.value = null
  selectedProviderTemplate.value = null
  selectedProviderEndpoint.value = ''
}

const editProvider = (provider: CodexModelProviderRecord) => {
  providerForm.id = provider.id
  providerForm.name = provider.name
  providerForm.baseUrl = provider.base_url
  providerForm.websiteUrl = provider.website_url || ''
  providerForm.apiKeyUrl = provider.api_key_url || ''
  providerForm.apiKeyName = provider.api_keys[0]?.name || 'API Key'
  providerForm.apiKey = provider.api_keys[0]?.api_key || ''
  selectedProviderTemplate.value = null
  selectedProviderEndpoint.value = ''
  activeManagerTab.value = 'providers'
}

const useManualProviderTemplate = () => {
  selectedProviderTemplate.value = null
  selectedProviderEndpoint.value = ''
}

const applyCodexProviderTemplate = (selection: ProviderTemplateSelection) => {
  const patch = mapTemplateToCodexProviderPatch(selection.template, selection.endpoint)

  selectedProviderTemplate.value = selection.template.id
  selectedProviderEndpoint.value = selection.endpoint || ''
  providerForm.name = patch.name || selection.template.name
  providerForm.baseUrl = patch.baseUrl || ''
  providerForm.websiteUrl = patch.websiteUrl || ''
  providerForm.apiKeyUrl = patch.apiKeyUrl || ''
  providerError.value = null
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

const applyProviderToApiForm = (provider: CodexModelProviderRecord) => {
  apiKeyForm.providerName = provider.name
  apiKeyForm.apiBaseUrl = provider.base_url
  apiKeyForm.apiKey = provider.api_keys[0]?.api_key || apiKeyForm.apiKey
  apiKeyForm.saveProvider = false
  useManualApiProviderTemplate()
  showAddAccountModal.value = true
  activeAddMethod.value = 'api'
  addAccountNotice.value = tf(
    'codex.auth.api.presetApplied',
    'Loaded saved provider "{name}" into the API key form.',
    { name: provider.name }
  )
}

const handleSaveProvider = async () => {
  providerError.value = null
  if (!providerForm.name.trim()) {
    providerError.value = tf(
      'codex.auth.providers.validation.nameRequired',
      'Provider name is required.'
    )
    return
  }
  if (!providerForm.baseUrl.trim()) {
    providerError.value = tf(
      'codex.auth.providers.validation.baseUrlRequired',
      'Base URL is required.'
    )
    return
  }

  try {
    providerSaving.value = true
    const result = await codexSaveModelProvider<{ provider: CodexModelProviderRecord }>({
      id: providerForm.id || undefined,
      name: providerForm.name.trim(),
      baseUrl: providerForm.baseUrl.trim(),
      websiteUrl: providerForm.websiteUrl.trim() || null,
      apiKeyUrl: providerForm.apiKeyUrl.trim() || null,
      apiKeyName: providerForm.apiKeyName.trim() || null,
      apiKey: providerForm.apiKey.trim() || null,
    })
    await loadProviders()
    resetProviderForm()
    uiStore.showSuccess(
      tf('codex.auth.providers.saveSuccess', 'Saved provider saved successfully.')
    )
    if (showAddAccountModal.value) {
      addAccountNotice.value = tf(
        'codex.auth.providers.savedAndReady',
        'Saved provider "{name}" is now ready in the API key flow.',
        { name: result.provider.name }
      )
    }
  } catch (error) {
    providerError.value =
      extractErrorMessage(error) ||
      tf('codex.auth.providers.saveFailed', 'Failed to save the saved provider.')
  } finally {
    providerSaving.value = false
  }
}

const requestDeleteProvider = (provider: CodexModelProviderRecord) => {
  openConfirmDialog({
    title: tf('codex.auth.providers.deleteTitle', 'Delete saved provider'),
    message: tf(
      'codex.auth.providers.deleteMessage',
      'Delete saved provider "{name}"? Stored API keys under this saved provider will also be removed.',
      { name: provider.name }
    ),
    confirmText: t('codex.actions.delete'),
    type: 'danger',
    action: async () => {
      try {
        await codexDeleteModelProvider(provider.id)
        await loadProviders()
        uiStore.showSuccess(
          tf('codex.auth.providers.deleteSuccess', 'Saved provider deleted successfully.')
        )
      } catch (error) {
        providerError.value =
          extractErrorMessage(error) ||
          tf('codex.auth.providers.deleteFailed', 'Failed to delete the saved provider.')
      }
    },
  })
}

onMounted(async () => {
  await installOauthListeners()
  await ensureLoaded(true)
})

onActivated(() => {
  void ensureLoaded(false)
})

onBeforeUnmount(() => {
  void cleanupOauthListeners()
})
</script>

<style scoped>
/* codex-auth-view__* 结构样式已抽至全局共享层 src/styles/codex-auth-shared.css，
   供主视图、Accounts/Providers Tab 与各 Modal 共用，此处不再重复定义。 */

</style>
