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

          <template v-if="activeManagerTab === 'accounts'">
            <Card
              v-if="currentInfo"
              surface="workspace"
              :elevation="2"
              motion="subtle"
              padding="lg"
            >
              <div class="codex-auth-view__section-header">
                <SIcon
                  name="Info"
                  size="w-5 h-5"
                  class="codex-auth-view__section-icon"
                />
                <h3 class="codex-auth-view__section-title">
                  {{ $t('codex.auth.currentSession') }}
                </h3>
              </div>
              <div class="codex-auth-view__session-grid">
                <div class="codex-auth-view__session-field">
                  <span class="codex-auth-view__field-label">
                    {{ $t('codex.auth.fields.accountId') }}
                  </span>
                  <code class="codex-auth-view__field-code">
                    {{ currentInfo.account_id }}
                  </code>
                </div>
                <div class="codex-auth-view__session-field">
                  <span class="codex-auth-view__field-label">
                    {{ $t('codex.auth.fields.email') }}
                  </span>
                  <span class="codex-auth-view__field-value codex-auth-view__field-value--truncate">
                    {{ currentInfo.email || $t('codex.auth.status.notAvailable') }}
                  </span>
                </div>
                <div class="codex-auth-view__session-field">
                  <span class="codex-auth-view__field-label">
                    {{ tf('codex.auth.fields.authMethod', 'Auth method') }}
                  </span>
                  <span class="codex-auth-view__field-value codex-auth-view__field-value--muted">
                    {{ formatAuthMethod(currentInfo.auth_method) }}
                  </span>
                </div>
                <div class="codex-auth-view__session-field">
                  <span class="codex-auth-view__field-label">
                    {{ tf('codex.auth.fields.planType', 'Plan') }}
                  </span>
                  <span class="codex-auth-view__field-value codex-auth-view__field-value--muted">
                    {{ currentInfo.plan_type || $t('codex.auth.status.notAvailable') }}
                  </span>
                </div>
                <div class="codex-auth-view__session-field">
                  <span class="codex-auth-view__field-label">
                    {{ $t('codex.auth.fields.lastRefresh') }}
                  </span>
                  <span class="codex-auth-view__field-value codex-auth-view__field-value--muted">
                    {{ currentInfo.last_refresh || $t('codex.auth.status.notAvailable') }}
                  </span>
                </div>
              </div>
            </Card>

            <Card
              surface="workspace"
              :elevation="2"
              motion="subtle"
              padding="lg"
              :glow-color="canManageAuthAccounts ? 'success' : 'warning'"
            >
              <div class="codex-auth-view__guard">
                <div
                  class="codex-auth-view__guard-icon-shell"
                  :class="
                    canManageAuthAccounts
                      ? 'bg-emerald-500/10 text-emerald-400'
                      : 'bg-yellow-500/10 text-yellow-400'
                  "
                >
                  <SIcon
                    name="AlertTriangle"
                    size="w-5 h-5"
                  />
                </div>
                <div class="codex-auth-view__guard-body">
                  <p class="codex-auth-view__guard-title">
                    {{ $t('codex.auth.profileGuard.title') }}
                  </p>
                  <p class="codex-auth-view__guard-message">
                    {{ profileGuardMessage }}
                  </p>
                  <p
                    v-if="authActionError"
                    class="codex-auth-view__guard-error"
                  >
                    {{ authActionError }}
                  </p>
                </div>
              </div>
            </Card>

            <Card
              v-if="!loading && accounts.length > 0"
              surface="workspace"
              :elevation="2"
              motion="subtle"
              padding="lg"
              class="codex-auth-view__filters-card"
            >
              <div class="codex-auth-view__filters-grid">
                <label class="codex-auth-view__search-box">
                  <SIcon
                    name="Search"
                    size="w-4 h-4"
                  />
                  <input
                    v-model="searchQuery"
                    type="text"
                    :placeholder="$t('codex.auth.filters.searchPlaceholder')"
                  >
                </label>

                <div class="codex-auth-view__filter-group">
                  <p class="codex-auth-view__filter-label">
                    {{ $t('codex.auth.filters.statusLabel') }}
                  </p>
                  <div class="codex-auth-view__filter-row">
                    <button
                      v-for="option in statusOptions"
                      :key="option.value"
                      type="button"
                      class="codex-auth-view__filter-pill"
                      :class="{
                        'codex-auth-view__filter-pill--active': statusFilter === option.value,
                      }"
                      @click="statusFilter = option.value"
                    >
                      {{ option.label }}
                    </button>
                  </div>
                </div>


                <div class="codex-auth-view__filter-group">
                  <label
                    class="codex-auth-view__filter-label"
                    for="codex-auth-plan-filter"
                  >
                    {{ $t('codex.auth.filters.planLabel') }}
                  </label>
                  <select
                    id="codex-auth-plan-filter"
                    v-model="planFilter"
                    class="codex-auth-view__filter-select"
                  >
                    <option
                      v-for="option in planOptions"
                      :key="option.value"
                      :value="option.value"
                    >
                      {{ option.label }}
                    </option>
                  </select>
                </div>

                <div class="codex-auth-view__filter-group">
                  <label
                    class="codex-auth-view__filter-label"
                    for="codex-auth-sort"
                  >
                    {{ $t('codex.auth.filters.sortLabel') }}
                  </label>
                  <select
                    id="codex-auth-sort"
                    v-model="sortBy"
                    class="codex-auth-view__filter-select"
                  >
                    <option
                      v-for="option in sortOptions"
                      :key="option.value"
                      :value="option.value"
                    >
                      {{ option.label }}
                    </option>
                  </select>
                </div>
              </div>

              <div class="codex-auth-view__filters-footer">
                <p class="codex-auth-view__filters-summary">
                  {{ filtersResultsCount }}
                </p>
                <Button
                  v-if="hasActiveFilters"
                  variant="secondary"
                  surface="status"
                  density="compact"
                  motion="subtle"
                  @click="clearFilters"
                >
                  {{ $t('common.clearFilters') }}
                </Button>
              </div>
            </Card>

            <div
              v-if="loading"
              class="flex justify-center py-20"
            >
              <div
                class="w-12 h-12 rounded-full border-4 border-transparent border-t-accent-primary border-r-accent-secondary animate-spin"
              />
            </div>

            <div
              v-else-if="accounts.length === 0"
              class="empty-state glass-effect rounded-2xl border border-border-default/10"
            >
              <div class="p-4 rounded-full glass-surface mb-4">
                <SIcon
                  name="KeyRound"
                  size="w-8 h-8"
                  class="text-text-muted"
                />
              </div>
              <p class="text-text-primary">
                {{ $t('codex.auth.emptyState') }}
              </p>
              <p class="text-sm text-text-muted mt-2">
                {{
                  tf(
                    'codex.auth.emptyStateHintV2',
                    'Add a new account through OAuth, API key, token JSON, or import the local runtime snapshot.'
                  )
                }}
              </p>
              <Button
                variant="primary"
                surface="card"
                density="compact"
                motion="standard"
                class="mt-4"
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

            <div
              v-else-if="filteredAccounts.length === 0"
              class="empty-state glass-effect rounded-2xl border border-border-default/10"
            >
              <div class="p-4 rounded-full glass-surface mb-4">
                <SIcon
                  name="Search"
                  size="w-8 h-8"
                  class="text-text-muted"
                />
              </div>
              <p class="text-text-primary">
                {{ $t('codex.auth.filters.noResultsTitle') }}
              </p>
              <p class="text-sm text-text-muted mt-2">
                {{ $t('codex.auth.filters.noResultsHint') }}
              </p>
              <Button
                variant="secondary"
                surface="status"
                density="compact"
                motion="subtle"
                class="mt-4"
                @click="clearFilters"
              >
                {{ $t('common.clearFilters') }}
              </Button>
            </div>

            <div
              v-else
              class="grid grid-cols-1 xl:grid-cols-2 gap-4"
            >
              <CodexAccountCard
                v-for="account in filteredAccounts"
                :key="account.name"
                :account="account"
                :quota="quotaMap.get(account.name) ?? null"
                :quota-loading="quotaLoading"
                :is-current="account.is_current"
                :busy-action="busyName === account.name ? busyAction : null"
                :disabled="actionLoading"
                @switch="handleSwitch"
                @delete="handleDelete"
                @refresh="handleRefreshSingle"
                @tag="handleTag"
                @export="handleExport"
                @rename="handleRename"
              />
            </div>
          </template>

          <template v-else>
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
                    @click="resetProviderForm"
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
                  @select="applyCodexProviderTemplate"
                  @manual="useManualProviderTemplate"
                />

                <div class="codex-auth-view__provider-form">
                  <label class="codex-auth-view__input-group">
                    <span class="codex-auth-view__input-label">{{
                      tf('codex.auth.providers.fields.name', 'Provider name')
                    }}</span>
                    <input
                      v-model="providerForm.name"
                      type="text"
                      class="input"
                      :placeholder="
                        tf(
                          'codex.auth.providers.placeholders.name',
                          'e.g. OpenRouter / Azure OpenAI / Local gateway'
                        )
                      "
                    >
                  </label>
                  <label class="codex-auth-view__input-group">
                    <span class="codex-auth-view__input-label">{{
                      tf('codex.auth.providers.fields.baseUrl', 'Base URL')
                    }}</span>
                    <input
                      v-model="providerForm.baseUrl"
                      type="url"
                      class="input"
                      placeholder="https://..."
                    >
                  </label>
                  <label class="codex-auth-view__input-group">
                    <span class="codex-auth-view__input-label">{{
                      tf('codex.auth.providers.fields.websiteUrl', 'Website URL')
                    }}</span>
                    <input
                      v-model="providerForm.websiteUrl"
                      type="url"
                      class="input"
                      :placeholder="
                        tf(
                          'codex.auth.providers.placeholders.websiteUrl',
                          'Optional reference link'
                        )
                      "
                    >
                  </label>
                  <label class="codex-auth-view__input-group">
                    <span class="codex-auth-view__input-label">{{
                      tf('codex.auth.providers.fields.apiKeyUrl', 'API key docs URL')
                    }}</span>
                    <input
                      v-model="providerForm.apiKeyUrl"
                      type="url"
                      class="input"
                      :placeholder="
                        tf(
                          'codex.auth.providers.placeholders.apiKeyUrl',
                          'Optional onboarding link'
                        )
                      "
                    >
                  </label>
                  <label class="codex-auth-view__input-group">
                    <span class="codex-auth-view__input-label">{{
                      tf('codex.auth.providers.fields.apiKeyName', 'Stored key label')
                    }}</span>
                    <input
                      v-model="providerForm.apiKeyName"
                      type="text"
                      class="input"
                      :placeholder="
                        tf('codex.auth.providers.placeholders.apiKeyName', 'Default: API Key')
                      "
                    >
                  </label>
                  <label class="codex-auth-view__input-group">
                    <span class="codex-auth-view__input-label">{{
                      tf('codex.auth.providers.fields.apiKey', 'Stored API key')
                    }}</span>
                    <input
                      v-model="providerForm.apiKey"
                      type="password"
                      class="input"
                      :placeholder="
                        tf(
                          'codex.auth.providers.placeholders.apiKey',
                          'Optional. Leave empty to keep existing keys unchanged.'
                        )
                      "
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
                    @click="handleSaveProvider"
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
                    @click="loadProviders"
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
                          @click="applyProviderToApiForm(provider)"
                        >
                          {{ tf('codex.auth.providers.actions.useInApiForm', 'Use in API form') }}
                        </Button>
                        <Button
                          variant="secondary"
                          surface="status"
                          density="compact"
                          motion="subtle"
                          @click="editProvider(provider)"
                        >
                          {{ tf('common.edit', 'Edit') }}
                        </Button>
                        <Button
                          variant="danger"
                          surface="status"
                          density="compact"
                          motion="subtle"
                          @click="requestDeleteProvider(provider)"
                        >
                          {{ $t('codex.actions.delete') }}
                        </Button>
                      </div>
                    </div>
                  </article>
                </div>
              </Card>
            </div>
          </template>

          <BaseModal
            :model-value="showSaveForm"
            :title="tf('codex.auth.actions.saveCurrent', 'Save current session')"
            :description="$t('codex.auth.subtitle')"
            size="full"
            surface="glass"
            content-class="w-full max-w-[min(780px,calc(100vw-2rem))] max-h-[90vh] overflow-y-auto"
            @update:model-value="(value) => !value && handleCloseSaveForm()"
          >
            <template #header="{ titleId }">
              <div
                class="px-6 py-4 border-b border-border-default/10 flex items-center justify-between sticky top-0 bg-bg-elevated/95 backdrop-blur z-10"
              >
                <h2
                  :id="titleId"
                  class="text-xl font-bold text-text-primary"
                >
                  {{ tf('codex.auth.actions.saveCurrent', 'Save current session') }}
                </h2>
                <Button
                  variant="ghost"
                  surface="status"
                  density="compact"
                  motion="subtle"
                  @click="handleCloseSaveForm"
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

            <div class="codex-auth-view__save-shell">
              <div class="codex-auth-view__save-intro">
                <div class="codex-auth-view__save-kicker">
                  <span class="codex-auth-view__save-kicker-dot" />
                  {{ tf('codex.auth.saveModal.kicker', 'Capture the live runtime') }}
                </div>
                <p class="codex-auth-view__save-lede">
                  {{
                    tf(
                      'codex.auth.saveModal.lede',
                      'Store the current Codex login as a reusable CCR account entry with a clearer label, optional notes, and an expiration reminder.'
                    )
                  }}
                </p>
                <div class="codex-auth-view__save-meta">
                  <span class="codex-auth-view__meta-pill">
                    {{
                      currentInfo?.email ||
                        tf('codex.auth.saveModal.meta.runtimeOnly', 'Current runtime session')
                    }}
                  </span>
                  <span class="codex-auth-view__meta-pill codex-auth-view__meta-pill--muted">
                    {{ formatAuthMethod(currentInfo?.auth_method) }}
                  </span>
                </div>
              </div>

              <div
                v-if="processWarning"
                class="p-4 rounded-lg bg-yellow-500/10 border border-yellow-500/30 text-yellow-600 dark:text-yellow-400"
              >
                <div class="flex items-start gap-3">
                  <SIcon
                    name="AlertTriangle"
                    size="w-5 h-5"
                    class="flex-shrink-0 mt-0.5"
                  />
                  <div>
                    <p class="font-medium">
                      {{ $t('codex.auth.processWarning') }}
                    </p>
                    <p class="text-sm mt-1 opacity-80">
                      {{ processWarning }}
                    </p>
                  </div>
                </div>
              </div>

              <div class="codex-auth-view__save-grid">
                <div class="space-y-1.5">
                  <label class="text-sm font-semibold text-text-primary">
                    {{ $t('codex.auth.fields.accountName') }} <span class="text-red-500">*</span>
                  </label>
                  <input
                    v-model="saveForm.name"
                    type="text"
                    class="input"
                    :placeholder="$t('codex.auth.placeholders.accountName')"
                  >
                </div>
                <div class="space-y-1.5">
                  <label class="text-sm font-semibold text-text-primary">
                    {{ $t('codex.auth.fields.description') }}
                  </label>
                  <input
                    v-model="saveForm.description"
                    type="text"
                    class="input"
                    :placeholder="$t('codex.auth.placeholders.description')"
                  >
                </div>
                <div class="codex-auth-view__save-toggle">
                  <input
                    id="forceOverwrite"
                    v-model="saveForm.force"
                    type="checkbox"
                    class="w-5 h-5 rounded border-border-default/15 text-accent-primary focus:ring-accent-primary/20"
                  >
                  <label
                    for="forceOverwrite"
                    class="text-sm font-medium text-text-primary cursor-pointer select-none"
                  >
                    {{ $t('codex.auth.forceOverwrite') }}
                  </label>
                </div>
              </div>
            </div>

            <template #footer>
              <div
                class="px-6 py-4 border-t border-border-default/10 flex justify-end gap-3 bg-bg-surface/70"
              >
                <Button
                  variant="secondary"
                  surface="status"
                  density="compact"
                  motion="subtle"
                  @click="handleCloseSaveForm"
                >
                  {{ $t('codex.actions.cancel') }}
                </Button>
                <Button
                  variant="primary"
                  surface="card"
                  density="compact"
                  motion="standard"
                  :disabled="saving || !saveForm.name.trim()"
                  @click="handleConfirmSave"
                >
                  <template #leading>
                    <span
                      v-if="saving"
                      class="w-4 h-4 border-2 border-border-default/30 border-t-white rounded-full animate-spin"
                    />
                  </template>
                  {{ saving ? $t('codex.states.saving') : $t('codex.actions.save') }}
                </Button>
              </div>
            </template>
          </BaseModal>

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

          <BaseModal
            :model-value="showRenameDialog"
            :title="tf('codex.auth.rename.title', '重命名 Codex 账号')"
            size="md"
            surface="glass"
            content-class="w-full max-w-[min(440px,calc(100vw-2rem))]"
            @update:model-value="(value) => !value && handleCloseRenameDialog()"
          >
            <template #header="{ titleId }">
              <div
                class="px-5 py-3.5 border-b border-border-default/10 flex items-center justify-between"
              >
                <h2
                  :id="titleId"
                  class="text-base font-semibold text-text-primary"
                >
                  {{ tf('codex.auth.rename.title', '重命名 Codex 账号') }}
                </h2>
                <Button
                  variant="ghost"
                  surface="status"
                  density="compact"
                  motion="subtle"
                  @click="handleCloseRenameDialog"
                >
                  <template #leading>
                    <SIcon
                      name="X"
                      size="w-4 h-4"
                    />
                  </template>
                </Button>
              </div>
            </template>

            <div class="p-5 space-y-4">
              <div class="space-y-1.5">
                <label class="text-xs font-semibold uppercase tracking-wider text-text-muted">
                  {{ tf('codex.auth.rename.currentLabel', '当前名称') }}
                </label>
                <div class="px-3 py-2 rounded-lg bg-bg-surface/70 border border-border-default/15 font-mono text-sm text-text-secondary">
                  {{ renameForm.oldName || '—' }}
                </div>
              </div>

              <div class="space-y-1.5">
                <label
                  for="renameNewName"
                  class="text-xs font-semibold uppercase tracking-wider text-text-muted"
                >
                  {{ tf('codex.auth.rename.newLabel', '新名称') }}
                  <span class="text-red-500">*</span>
                </label>
                <input
                  id="renameNewName"
                  v-model="renameForm.newName"
                  type="text"
                  class="input"
                  :placeholder="tf('codex.auth.rename.placeholder', '输入新名称（字母/数字/_/-）')"
                  @keydown.enter.prevent="handleConfirmRename"
                >
                <p class="text-[11px] text-text-disabled">
                  {{ tf('codex.auth.rename.hint', '只能包含字母、数字、下划线和连字符，长度不超过 32 个字符。') }}
                </p>
              </div>

              <label class="flex items-center gap-2 text-sm text-text-secondary cursor-pointer select-none">
                <input
                  v-model="renameForm.force"
                  type="checkbox"
                  class="w-4 h-4 rounded border-border-default/15 text-accent-primary focus:ring-accent-primary/20"
                >
                {{ tf('codex.auth.rename.forceLabel', '覆盖同名账号 (force)') }}
              </label>

              <div
                v-if="renameError"
                class="px-3 py-2 rounded-lg bg-red-500/10 border border-red-500/20 text-xs text-red-400"
              >
                {{ renameError }}
              </div>
            </div>

            <template #footer>
              <div class="flex items-center justify-end gap-2 px-5 py-3 border-t border-border-default/10">
                <Button
                  variant="ghost"
                  surface="status"
                  density="compact"
                  :disabled="renameSubmitting"
                  @click="handleCloseRenameDialog"
                >
                  {{ $t('common.cancel') }}
                </Button>
                <Button
                  variant="primary"
                  density="compact"
                  :loading="renameSubmitting"
                  :disabled="!canSubmitRename"
                  @click="handleConfirmRename"
                >
                  {{ tf('codex.auth.rename.confirm', '重命名') }}
                </Button>
              </div>
            </template>
          </BaseModal>
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
import CodexAccountCard from '@/components/codex/CodexAccountCard.vue'
import ConfirmModal from '@/components/ConfirmModal.vue'
import ProviderTemplateSelector from '@/components/provider-templates/ProviderTemplateSelector.vue'
import { translateWithFallback } from '@/i18n/formatMessage'
import { useTf } from '@/composables/useTf'
import {
  listCodexProfiles,
  listCodexAuthAccounts,
  getCodexAuthCurrent,
  saveCodexAuth,
  switchCodexAuth,
  deleteCodexAuth,
  renameCodexAuth,
  detectCodexProcess,
  getCodexAllQuotas,
  codexOAuthLoginStart,
  codexOAuthLoginCompleted,
  codexOAuthLoginCancel,
  codexOAuthSubmitCallbackUrl,
  codexIsOAuthPortInUse,
  codexReleaseOAuthPort,
  codexOpenExternalUrl,
  codexImportAuthPayload,
  codexImportAuthFromLocal,
  codexAddAuthWithApiKey,
  codexListModelProviders,
  codexSaveModelProvider,
  codexDeleteModelProvider,
} from '@/api'
import {
  canCustomizeAccountName,
  canSubmitAccountRename,
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
  CodexAuthProcessResponse,
  CodexAuthSaveRequest,
  CodexImportAuthPayload,
  CodexModelProviderRecord,
  CodexModelProvidersResponse,
  CodexOAuthStartResponse,
  CodexProfile,
  CodexProfilesResponse,
  LoginState,
} from '@/types'
import type { ProviderTemplateDraftContext, ProviderTemplateSelection } from '@/types/providerTemplates'
import { logger } from '@/utils/logger'
import { isTauriRuntime } from '@/utils/tauriRuntime'
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
type UnlistenFn = () => void | Promise<void>

const loading = ref(false)
const saving = ref(false)
const actionLoading = ref(false)
const quotaLoading = ref(false)
const providerLoading = ref(false)
const providerSaving = ref(false)
const oauthBusy = ref(false)
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
const processWarning = ref<string | null>(null)
const busyName = ref<string | null>(null)
const busyAction = ref<'switch' | 'delete' | null>(null)
const showConfirmModal = ref(false)
const lastLoadedAt = ref(0)

const oauthLoginId = ref('')
const oauthAuthUrl = ref('')
const oauthCallbackUrl = ref('')
const oauthPending = ref(false)
const oauthPortBusy = ref(false)
const oauthTimeoutMessage = ref<string | null>(null)

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
let oauthUnlisteners: UnlistenFn[] = []

const saveForm = reactive({
  name: '',
  description: '',
  force: false,
})

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

const extractErrorMessage = (error: unknown) => {
  if (typeof error === 'string') {
    return error
  }
  if (error && typeof error === 'object') {
    const candidate = error as { message?: unknown; error?: unknown; cause?: unknown }
    for (const value of [candidate.message, candidate.error, candidate.cause]) {
      if (typeof value === 'string' && value.trim()) {
        return value
      }
    }
  }
  return null
}

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

const handleSave = async () => {
  authActionError.value = null
  if (!canManageAuthAccounts.value) {
    authActionError.value = profileGuardMessage.value
    return
  }

  try {
    const processInfo = await detectCodexProcess<CodexAuthProcessResponse>()
    processWarning.value = processInfo.has_running_process
      ? processInfo.warning ||
        t('codex.auth.processDetected', { pids: processInfo.pids.join(', ') })
      : null
  } catch {
    processWarning.value = null
  }

  saveForm.name = currentInfo.value?.email?.split('@')[0] || ''
  saveForm.description = ''
  saveForm.force = false
  showSaveForm.value = true
}

const handleCloseSaveForm = () => {
  showSaveForm.value = false
  processWarning.value = null
}

const handleConfirmSave = async () => {
  authActionError.value = null
  if (!saveForm.name.trim()) {
    uiStore.showError(t('codex.auth.validation.nameRequired'))
    return
  }

  try {
    saving.value = true
    const payload: CodexAuthSaveRequest = {
      name: saveForm.name.trim(),
      description: saveForm.description.trim() || undefined,
      force: saveForm.force,
    }
    await saveCodexAuth(payload)
    handleCloseSaveForm()
    await handleRefresh()
    uiStore.showSuccess(
      tf('codex.auth.feedback.saveCurrentSuccess', 'Current session saved as an account.')
    )
  } catch (error) {
    logger.error('Failed to save auth:', error)
    authActionError.value = extractErrorMessage(error) || t('codex.states.saveFailed')
    uiStore.showError(authActionError.value)
  } finally {
    saving.value = false
  }
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

const showRenameDialog = ref(false)
const renameSubmitting = ref(false)
const renameError = ref<string | null>(null)
const renameForm = reactive({
  oldName: '',
  newName: '',
  force: false,
})

const canSubmitRename = computed(() => {
  return canSubmitAccountRename(renameForm.oldName, renameForm.newName)
})

const handleRename = (name: string) => {
  if (!canManageAuthAccounts.value) {
    authActionError.value = profileGuardMessage.value
    return
  }
  renameForm.oldName = name
  renameForm.newName = name
  renameForm.force = false
  renameError.value = null
  showRenameDialog.value = true
}

const handleCloseRenameDialog = () => {
  if (renameSubmitting.value) return
  showRenameDialog.value = false
  renameError.value = null
  renameForm.oldName = ''
  renameForm.newName = ''
  renameForm.force = false
}

const handleConfirmRename = async () => {
  if (!canSubmitRename.value) {
    renameError.value = tf(
      'codex.auth.rename.invalidName',
      '新名称只能包含字母、数字、下划线与连字符，且不能与原名称相同。'
    )
    return
  }

  const oldName = renameForm.oldName
  const newName = renameForm.newName.trim()
  const force = renameForm.force

  renameError.value = null
  renameSubmitting.value = true
  busyName.value = oldName
  try {
    await renameCodexAuth(oldName, newName, force)
    showRenameDialog.value = false
    await handleRefresh()
    uiStore.showSuccess(
      tf('codex.auth.rename.success', '已将 {old} 重命名为 {new}', {
        old: oldName,
        new: newName,
      })
    )
    renameForm.oldName = ''
    renameForm.newName = ''
    renameForm.force = false
  } catch (error) {
    logger.error('Failed to rename auth:', error)
    const raw = extractErrorMessage(error) || t('codex.states.saveFailed')
    if (!force && raw.includes('已存在')) {
      renameError.value = tf(
        'codex.auth.rename.conflictHint',
        '{msg} · 勾选 "覆盖同名账号" 后再次确认可强制覆盖。',
        { msg: raw }
      )
    } else {
      renameError.value = raw
    }
  } finally {
    renameSubmitting.value = false
    busyName.value = null
  }
}

const resetOauthState = () => {
  oauthLoginId.value = ''
  oauthAuthUrl.value = ''
  oauthCallbackUrl.value = ''
  oauthPending.value = false
}

const refreshOauthPortStatus = async () => {
  if (!isTauriRuntime()) {
    oauthPortBusy.value = false
    return
  }
  try {
    oauthPortBusy.value = await codexIsOAuthPortInUse<boolean>()
  } catch (error) {
    logger.error('Failed to check oauth port:', error)
    oauthPortBusy.value = false
  }
}

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

const applyMutationSuccess = async (result: CodexAuthMutationResponse, successMessage: string) => {
  await handleRefresh()
  uiStore.showSuccess(successMessage)
  addAccountNotice.value = result.account_name
    ? tf('codex.auth.feedback.savedAs', 'Saved as {name}.', { name: result.account_name })
    : successMessage
  resetOauthState()
}

const handleReleaseOauthPort = async () => {
  try {
    oauthBusy.value = true
    const killed = await codexReleaseOAuthPort<number>()
    await refreshOauthPortStatus()
    uiStore.showSuccess(
      tf(
        'codex.auth.oauth.releasePortSuccess',
        'Released the callback port ({count} process(es)).',
        { count: killed }
      )
    )
  } catch (error) {
    addAccountError.value =
      extractErrorMessage(error) ||
      tf('codex.auth.oauth.releasePortFailed', 'Failed to release port 1455.')
  } finally {
    oauthBusy.value = false
  }
}

const handleStartOauth = async () => {
  addAccountError.value = null
  addAccountNotice.value = null
  oauthTimeoutMessage.value = null
  if (!ensurePreferredAccountNameIsValid()) {
    return
  }
  try {
    oauthBusy.value = true
    await refreshOauthPortStatus()
    if (oauthPortBusy.value && !oauthPending.value) {
      addAccountError.value = tf(
        'codex.auth.oauth.portBusyError',
        'Port 1455 is busy. Release it first, then retry the OAuth flow.'
      )
      return
    }

    const result = await codexOAuthLoginStart<CodexOAuthStartResponse>()
    oauthLoginId.value = result.loginId
    oauthAuthUrl.value = result.authUrl
    oauthPending.value = true
    await codexOpenExternalUrl(result.authUrl)
    addAccountNotice.value = tf(
      'codex.auth.oauth.started',
      'Browser authorization started. After the callback arrives, CCR will finish the login automatically.'
    )
  } catch (error) {
    addAccountError.value =
      extractErrorMessage(error) ||
      tf('codex.auth.oauth.startFailed', 'Failed to start OAuth authorization.')
  } finally {
    oauthBusy.value = false
  }
}

const handleSubmitOauthCallback = async () => {
  addAccountError.value = null
  if (!oauthLoginId.value || !oauthCallbackUrl.value.trim()) {
    addAccountError.value = tf(
      'codex.auth.oauth.callbackRequired',
      'Paste the callback URL before submitting it.'
    )
    return
  }

  try {
    oauthBusy.value = true
    await codexOAuthSubmitCallbackUrl(oauthLoginId.value, oauthCallbackUrl.value.trim())
    addAccountNotice.value = tf(
      'codex.auth.oauth.callbackSubmitted',
      'Callback received. Finalizing the OAuth account now...'
    )
  } catch (error) {
    addAccountError.value =
      extractErrorMessage(error) ||
      tf('codex.auth.oauth.callbackSubmitFailed', 'Failed to submit the callback URL.')
  } finally {
    oauthBusy.value = false
  }
}

const finalizeOauthLoginById = async (loginId: string) => {
  if (!ensurePreferredAccountNameIsValid()) {
    return
  }
  try {
    oauthBusy.value = true
    const result = await codexOAuthLoginCompleted<CodexAuthMutationResponse>(
      loginId,
      effectivePreferredAccountName.value
    )
    await applyMutationSuccess(
      result,
      tf('codex.auth.oauth.success', 'OAuth account added successfully.')
    )
    showAddAccountModal.value = false
  } catch (error) {
    addAccountError.value =
      extractErrorMessage(error) ||
      tf('codex.auth.oauth.completeFailed', 'Failed to complete the OAuth login.')
  } finally {
    oauthBusy.value = false
  }
}

const handleFinalizeOauth = async () => {
  if (!oauthLoginId.value) {
    addAccountError.value = tf(
      'codex.auth.oauth.notStarted',
      'Start the OAuth flow before finalizing it.'
    )
    return
  }
  await finalizeOauthLoginById(oauthLoginId.value)
}

const cancelOauthFlow = async () => {
  try {
    oauthBusy.value = true
    if (oauthLoginId.value) {
      await codexOAuthLoginCancel(oauthLoginId.value)
    }
    resetOauthState()
    await refreshOauthPortStatus()
  } catch (error) {
    addAccountError.value =
      extractErrorMessage(error) ||
      tf('codex.auth.oauth.cancelFailed', 'Failed to cancel the OAuth flow.')
  } finally {
    oauthBusy.value = false
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

const installOauthListeners = async () => {
  if (!isTauriRuntime()) return
  try {
    const { listen } = await import('@tauri-apps/api/event')
    const completed = await listen<{ loginId?: string }>(
      'codex-oauth-login-completed',
      async (event) => {
        const loginId = event.payload?.loginId
        if (!loginId || loginId !== oauthLoginId.value) return
        await finalizeOauthLoginById(loginId)
      }
    )
    const timeout = await listen<{ loginId?: string; timeoutSeconds?: number }>(
      'codex-oauth-login-timeout',
      async (event) => {
        const loginId = event.payload?.loginId
        if (!loginId || loginId !== oauthLoginId.value) return
        oauthTimeoutMessage.value = tf(
          'codex.auth.oauth.timeoutMessage',
          'No callback arrived within {seconds} seconds. You can restart the flow or paste the manual callback URL.',
          { seconds: event.payload?.timeoutSeconds ?? 300 }
        )
        resetOauthState()
        await refreshOauthPortStatus()
      }
    )
    oauthUnlisteners.push(completed, timeout)
  } catch (error) {
    logger.error('Failed to install oauth listeners:', error)
  }
}

const cleanupOauthListeners = async () => {
  const pending = [...oauthUnlisteners]
  oauthUnlisteners = []
  await Promise.allSettled(pending.map((unlisten) => Promise.resolve(unlisten())))
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
.codex-auth-view {
  min-height: 100%;
  padding: 1.5rem;
}

.codex-auth-view__container {
  max-width: 1800px;
  margin: 0 auto;
}

.codex-auth-view__stack {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  margin-top: 1.5rem;
}

.codex-auth-view__main {
  display: flex;
  width: 100%;
  min-width: 0;
  flex-direction: column;
  gap: 1.5rem;
}

.codex-auth-view__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.codex-auth-view__title-group,
.codex-auth-view__actions,
.codex-auth-view__section-header,
.codex-auth-view__status-row,
.codex-auth-view__field-inline,
.codex-auth-view__title-inline {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.codex-auth-view__section-header--spread {
  justify-content: space-between;
}

.codex-auth-view__title-icon-shell,
.codex-auth-view__guard-icon-shell {
  border-radius: 0.75rem;
  padding: 0.5rem;
}

.codex-auth-view__title-icon-shell {
  background: rgb(var(--platform-codex-rgb, 245 158 11) / 10%);
}

.codex-auth-view__title-icon,
.codex-auth-view__section-icon {
  color: var(--platform-codex, #f59e0b);
}

.codex-auth-view__title {
  color: var(--stage-text-primary);
  font-size: 1.5rem;
  line-height: 2rem;
  font-weight: 700;
}

.codex-auth-view__subtitle,
.codex-auth-view__section-copy {
  margin-top: 0.25rem;
  color: var(--stage-text-secondary);
  font-size: 0.875rem;
  line-height: 1.35rem;
}

.codex-auth-view__save-shell,
.codex-auth-view__composer-shell,
.codex-auth-view__composer-main {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.codex-auth-view__save-shell,
.codex-auth-view__composer-shell {
  padding: 1.5rem;
}

.codex-auth-view__save-intro,
.codex-auth-view__composer-card {
  border: 1px solid var(--stage-border-soft);
  border-radius: 1.25rem;
  background:
    linear-gradient(
      180deg,
      rgb(var(--color-bg-overlay-rgb) / 72%),
      rgb(var(--color-bg-overlay-rgb) / 42%)
    ),
    var(--stage-surface-soft);
  padding: 1.1rem 1.15rem;
}

.codex-auth-view__save-kicker,
.codex-auth-view__composer-eyebrow {
  display: inline-flex;
  align-items: center;
  gap: 0.45rem;
  color: var(--stage-text-secondary);
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.codex-auth-view__save-kicker-dot {
  width: 0.45rem;
  height: 0.45rem;
  border-radius: 9999px;
  background: var(--platform-codex, #f59e0b);
  box-shadow: 0 0 0 0.2rem rgb(var(--platform-codex-rgb, 245 158 11) / 10%);
}

.codex-auth-view__save-lede,
.codex-auth-view__composer-copy,
.codex-auth-view__composer-helper {
  margin-top: 0.6rem;
  color: var(--stage-text-secondary);
  font-size: 0.92rem;
  line-height: 1.5rem;
}

.codex-auth-view__composer-title {
  margin-top: 0.45rem;
  color: var(--stage-text-primary);
  font-size: 1.12rem;
  line-height: 1.55rem;
  font-weight: 650;
}

.codex-auth-view__save-meta,
.codex-auth-view__composer-meta,
.codex-auth-view__composer-rules {
  display: flex;
  flex-wrap: wrap;
  gap: 0.55rem;
  margin-top: 0.85rem;
}

.codex-auth-view__meta-pill {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  min-height: 2rem;
  border-radius: 9999px;
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 16%);
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  padding: 0.28rem 0.72rem;
  color: var(--color-accent-primary);
  font-size: 0.76rem;
  line-height: 1rem;
  font-weight: 600;
}

.codex-auth-view__meta-pill--muted,
.codex-auth-view__meta-pill--soft {
  border-color: var(--stage-border-soft);
  background: rgb(var(--color-bg-overlay-rgb) / 55%);
  color: var(--stage-text-secondary);
}

.codex-auth-view__save-grid {
  display: grid;
  gap: 1rem;
}

.codex-auth-view__save-toggle {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  min-height: 100%;
  padding: 0.95rem 1rem;
  border-radius: 1rem;
  border: 1px solid var(--stage-border-soft);
  background: var(--stage-surface-soft);
}

.codex-auth-view__composer-sidebar {
  min-width: 0;
}

.codex-auth-view__composer-helper--error {
  color: var(--color-danger);
}

.codex-auth-view__status-grid,
.codex-auth-view__session-grid,
.codex-auth-view__providers-grid,
.codex-auth-view__provider-form,
.codex-auth-view__filters-grid {
  display: grid;
  gap: 1rem;
}

.codex-auth-view__status-grid {
  grid-template-columns: repeat(1, minmax(0, 1fr));
}

.codex-auth-view__status-card {
  position: relative;
  overflow: hidden;
}

.codex-auth-view__status-icon-shell {
  border-radius: 0.75rem;
  padding: 0.75rem;
}

.codex-auth-view__status-icon-shell--info {
  background: rgb(99 102 241 / 10%);
  color: rgb(99 102 241 / 100%);
}

.codex-auth-view__status-icon-shell--neutral {
  background: rgb(148 163 184 / 12%);
  color: rgb(148 163 184 / 100%);
}

.codex-auth-view__status-label,
.codex-auth-view__field-label,
.codex-auth-view__input-label,
.codex-auth-view__filter-label,
.codex-auth-view__filters-summary {
  color: var(--stage-text-secondary);
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.codex-auth-view__status-value,
.codex-auth-view__section-title,
.codex-auth-view__provider-title {
  color: var(--stage-text-primary);
  font-size: 1.05rem;
  line-height: 1.5rem;
  font-weight: 650;
}

.codex-auth-view__status-value--truncate,
.codex-auth-view__field-value--truncate,
.codex-auth-view__field-code {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.codex-auth-view__segment-card {
  padding-top: 0.875rem;
  padding-bottom: 0.875rem;
}

.codex-auth-view__segment-row {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
}

.codex-auth-view__segment {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  border: 1px solid var(--stage-border-soft);
  border-radius: 9999px;
  background: var(--stage-surface-soft);
  color: var(--stage-text-secondary);
  padding: 0.625rem 0.95rem;
}

.codex-auth-view__segment--active {
  border-color: rgb(var(--color-accent-primary-rgb) / 20%);
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  color: var(--color-accent-primary);
}

.codex-auth-view__segment-count {
  border-radius: 9999px;
  background: rgb(var(--color-bg-overlay-rgb) / 65%);
  padding: 0.1rem 0.45rem;
  font-size: 0.75rem;
}

.codex-auth-view__session-grid {
  grid-template-columns: repeat(1, minmax(0, 1fr));
}

.codex-auth-view__session-field,
.codex-auth-view__filter-group,
.codex-auth-view__input-group {
  display: flex;
  flex-direction: column;
  gap: 0.45rem;
  min-width: 0;
}

.codex-auth-view__input-group--full {
  grid-column: 1 / -1;
}

.codex-auth-view__field-code,
.codex-auth-view__textarea {
  border: 1px solid var(--stage-border-soft);
  border-radius: 0.75rem;
  padding: 0.75rem 0.9rem;
  color: var(--stage-text-primary);
  background: var(--stage-surface-soft);
}

.codex-auth-view__textarea {
  resize: vertical;
  min-height: 6rem;
}

.codex-auth-view__textarea--mono,
.codex-auth-view__field-code {
  font-family: var(--font-mono);
}

.codex-auth-view__field-value--muted,
.codex-auth-view__provider-url,
.codex-auth-view__provider-copy,
.codex-auth-view__preset-url,
.codex-auth-view__checkbox-hint {
  color: var(--stage-text-secondary);
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.codex-auth-view__field-value--faint {
  color: var(--stage-text-muted);
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.codex-auth-view__field-value--strong {
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 500;
}

.codex-auth-view__expired-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  border: 1px solid rgb(239 68 68 / 20%);
  border-radius: 0.375rem;
  background: rgb(239 68 68 / 10%);
  padding: 0.125rem 0.5rem;
  color: rgb(239 68 68 / 100%);
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 500;
}

.codex-auth-view__guard {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
}

.codex-auth-view__guard-body {
  min-width: 0;
}

.codex-auth-view__guard-message {
  margin-top: 0.25rem;
  color: var(--stage-text-secondary);
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.codex-auth-view__guard-error,
.codex-auth-view__inline-error {
  margin-top: 0.75rem;
  border: 1px solid rgb(var(--color-danger-rgb) / 20%);
  border-radius: 0.75rem;
  background: rgb(var(--color-danger-rgb) / 10%);
  padding: 0.75rem 0.9rem;
  color: var(--color-danger);
  font-size: 0.875rem;
  line-height: 1.35rem;
}

.codex-auth-view__inline-note {
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 20%);
  border-radius: 0.75rem;
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  padding: 0.75rem 0.9rem;
  color: var(--color-accent-primary);
  font-size: 0.875rem;
  line-height: 1.35rem;
}

.codex-auth-view__search-box {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  border: 1px solid var(--stage-border-soft);
  border-radius: 9999px;
  background: var(--stage-surface-soft);
  padding: 0.75rem 1rem;
  color: var(--stage-text-secondary);
}

.codex-auth-view__search-box input,
.codex-auth-view__filter-select {
  width: 100%;
  min-width: 0;
  background: transparent;
  color: var(--stage-text-primary);
  outline: none;
}

.codex-auth-view__filter-row {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.codex-auth-view__filter-pill,
.codex-auth-view__filter-select {
  border: 1px solid var(--stage-border-soft);
  border-radius: 9999px;
  background: var(--stage-surface-soft);
  color: var(--stage-text-secondary);
  font-size: 0.75rem;
  line-height: 1rem;
}

.codex-auth-view__filter-pill {
  padding: 0.5rem 0.75rem;
}

.codex-auth-view__filter-pill--active {
  border-color: rgb(var(--color-accent-primary-rgb) / 20%);
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  color: var(--color-accent-primary);
}

.codex-auth-view__filter-select {
  padding: 0.75rem 1rem;
}

.codex-auth-view__filters-footer,
.codex-auth-view__provider-actions,
.codex-auth-view__oauth-actions,
.codex-auth-view__provider-actions-inline,
.codex-auth-view__provider-footer,
.codex-auth-view__checkbox-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  flex-wrap: wrap;
}

.codex-auth-view__providers-grid {
  grid-template-columns: minmax(0, 1fr);
}

.codex-auth-view__providers-grid--modal {
  grid-template-columns: minmax(0, 1.1fr) minmax(0, 0.9fr);
}

.codex-auth-view__provider-form {
  grid-template-columns: repeat(1, minmax(0, 1fr));
  margin-top: 1rem;
}

.codex-auth-view__template-selector {
  margin-top: 1rem;
}

.codex-auth-view__provider-list,
.codex-auth-view__preset-list {
  display: grid;
  gap: 0.9rem;
}

.codex-auth-view__provider-card,
.codex-auth-view__preset,
.codex-auth-view__warning-panel {
  border: 1px solid var(--stage-border-soft);
  border-radius: 1rem;
  background: var(--stage-surface-soft);
  padding: 1rem;
}

.codex-auth-view__warning-panel {
  margin-top: 1rem;
}

.codex-auth-view__warning-panel--neutral {
  background: rgb(var(--color-bg-overlay-rgb) / 35%);
}

.codex-auth-view__provider-head,
.codex-auth-view__provider-badges,
.codex-auth-view__provider-meta {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.75rem;
  flex-wrap: wrap;
}

.codex-auth-view__provider-badge {
  display: inline-flex;
  align-items: center;
  border-radius: 9999px;
  padding: 0.2rem 0.55rem;
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  color: var(--color-accent-primary);
  font-size: 0.75rem;
}

.codex-auth-view__provider-badge--muted {
  background: rgb(var(--color-bg-overlay-rgb) / 55%);
  color: var(--stage-text-secondary);
}

.codex-auth-view__provider-link {
  color: var(--color-accent-primary);
  font-size: 0.875rem;
}

.codex-auth-view__provider-footer {
  margin-top: 0.9rem;
  padding-top: 0.9rem;
  border-top: 1px solid var(--stage-border-soft);
}

.codex-auth-view__preset {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.25rem;
  text-align: left;
}

.codex-auth-view__preset-name {
  color: var(--stage-text-primary);
  font-weight: 600;
}

.codex-auth-view__preset-meta {
  color: var(--stage-text-muted);
  font-size: 0.75rem;
}

.codex-auth-view__checkbox-label {
  display: inline-flex;
  align-items: center;
  gap: 0.6rem;
  color: var(--stage-text-primary);
  font-size: 0.875rem;
}

.codex-auth-view__checkbox-label input {
  width: 1rem;
  height: 1rem;
}

.codex-auth-view__oauth-grid {
  display: grid;
  gap: 1rem;
  margin-top: 1rem;
}

@media (width >= 768px) {
  .codex-auth-view__status-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .codex-auth-view__save-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .codex-auth-view__session-grid,
  .codex-auth-view__provider-form {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (width >= 1280px) {
  .codex-auth-view__status-grid {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }

  .codex-auth-view__session-grid {
    grid-template-columns: repeat(6, minmax(0, 1fr));
  }

  .codex-auth-view__providers-grid {
    grid-template-columns: minmax(0, 0.95fr) minmax(0, 1.05fr);
  }

  .codex-auth-view__filters-grid {
    grid-template-columns: minmax(0, 1.5fr) minmax(0, 1.5fr) minmax(0, 0.9fr) minmax(
        0,
        0.9fr
      ) minmax(0, 0.9fr);
    align-items: end;
  }
}

@media (width <= 1100px) {
  .codex-auth-view__composer-shell,
  .codex-auth-view__providers-grid,
  .codex-auth-view__providers-grid--modal {
    grid-template-columns: minmax(0, 1fr);
  }
}

@media (width >= 1101px) {
  .codex-auth-view__composer-shell {
    display: grid;
    grid-template-columns: minmax(18rem, 23rem) minmax(0, 1fr);
    align-items: start;
    gap: 1.25rem;
  }

  .codex-auth-view__composer-sidebar {
    position: sticky;
    top: 1.5rem;
  }
}

@media (width <= 900px) {
  .codex-auth-view__header {
    flex-direction: column;
    align-items: flex-start;
  }

  .codex-auth-view__actions {
    flex-wrap: wrap;
  }
}
</style>
