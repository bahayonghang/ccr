<!-- -->
<template>
  <div class="codex-profiles-view">
    <div class="codex-profiles-shell">
      <div class="codex-profiles-stack">
        <ModuleSubnav module="codex" />

        <main class="codex-profiles-main">
          <!-- Header Section -->
          <div class="codex-profiles-header">
            <div class="codex-profiles-header__intro">
              <div class="codex-profiles-header__icon">
                <SIcon
                  name="Settings"
                  size="w-6 h-6"
                  class="text-platform-codex"
                />
              </div>
              <div>
                <h1 class="text-2xl font-bold text-white">
                  {{ $t('codex.profiles.title') }}
                </h1>
                <p class="text-sm text-white/80 mt-1">
                  {{ $t('codex.profiles.subtitle') }}
                </p>
              </div>
            </div>

            <div class="codex-profiles-header__actions">
              <RouterLink
                to="/codex"
                class="btn btn-secondary"
              >
                <SIcon
                  name="ArrowLeft"
                  size="w-4 h-4"
                />
                <span>{{ $t('codex.profiles.backToCodex') }}</span>
              </RouterLink>

              <button
                class="btn btn-primary"
                @click="handleAdd"
              >
                <SIcon
                  name="Plus"
                  size="w-4 h-4"
                />
                {{ $t('codex.profiles.addProfile') }}
              </button>
            </div>
          </div>

          <!-- Status Cards -->
          <div class="codex-profiles-status-grid">
            <!-- Current Config -->
            <Card
              variant="glass"
              :gradient-border="true"
              glow-color="warning"
              class="group codex-profiles-status-card"
            >
              <div class="flex items-center gap-4">
                <div class="codex-profiles-status-icon codex-profiles-status-icon--warning">
                  <SIcon
                    name="Zap"
                    size="w-6 h-6"
                  />
                </div>
                <div>
                  <p class="text-xs font-medium text-white/50 uppercase tracking-wider mb-1">
                    {{ $t('codex.status.currentConfig') }}
                  </p>
                  <p class="text-xl font-bold text-white truncate">
                    {{ currentProfile || $t('codex.status.notSet') }}
                  </p>
                </div>
              </div>
            </Card>

            <!-- Total Profiles -->
            <Card
              variant="glass"
              :interactive="true"
              glow-color="primary"
              class="group codex-profiles-status-card"
            >
              <div class="flex items-center gap-4">
                <div class="codex-profiles-status-icon codex-profiles-status-icon--primary">
                  <SIcon
                    name="Layers"
                    size="w-6 h-6"
                  />
                </div>
                <div>
                  <p class="text-xs font-medium text-white/50 uppercase tracking-wider mb-1">
                    {{ $t('codex.status.totalProfiles') }}
                  </p>
                  <p class="text-xl font-bold text-white">
                    {{ profiles.length }}
                  </p>
                </div>
              </div>
            </Card>

            <!-- Config Mode -->
            <Card
              variant="glass"
              :interactive="true"
              :glow-color="currentConfigMode === 'official' ? 'success' : 'secondary'"
              class="group codex-profiles-status-card"
            >
              <div class="flex items-center gap-4">
                <div 
                  class="codex-profiles-status-icon"
                  :class="currentConfigMode === 'official' ? 'codex-profiles-status-icon--official' : 'codex-profiles-status-icon--relay'"
                >
                  <SIcon
                    :name="currentConfigMode === 'official' ? 'Globe' : 'Server'"
                    size="w-6 h-6"
                  />
                </div>
                <div>
                  <p class="text-xs font-medium text-white/50 uppercase tracking-wider mb-1">
                    {{ $t('codex.status.configMode') }}
                  </p>
                  <p class="text-xl font-bold text-white">
                    {{ currentConfigMode === 'official' ? $t('codex.profiles.officialConfig') : $t('codex.profiles.customRelay') }}
                  </p>
                </div>
              </div>
            </Card>
          </div>

          <!-- Quick Switch -->
          <Card
            v-if="profiles.length > 0"
            variant="glass"
            padding="lg"
          >
            <div class="flex items-center gap-2 mb-4">
              <SIcon
                name="Shuffle"
                size="w-5 h-5"
                class="text-platform-codex"
              />
              <h3 class="text-base font-semibold text-white">
                {{ $t('codex.profiles.quickSwitch') }}
              </h3>
            </div>
            <div class="codex-profiles-switches">
              <button
                v-for="profile in profiles"
                :key="profile.name"
                class="codex-profiles-switch"
                :class="[
                  profile.name === currentProfile ? 'codex-profiles-switch--active' : 'codex-profiles-switch--idle',
                  actionLoading ? 'codex-profiles-switch--busy' : '',
                ]"
                :disabled="actionLoading"
                @click="handleApply(profile.name)"
              >
                <SIcon
                  v-if="isOfficialConfig(profile)"
                  name="Star"
                  size="w-3.5 h-3.5"
                  :class="profile.name === currentProfile ? 'text-platform-codex' : 'text-yellow-500'"
                />
                <SIcon
                  v-if="busyProfileName === profile.name && busyAction === 'apply'"
                  name="RefreshCw"
                  size="w-3.5 h-3.5"
                  class="animate-spin"
                />
                <span>{{ profile.name }}</span>
                <div 
                  v-if="profile.name === currentProfile" 
                  class="codex-profiles-switch__active-indicator"
                >
                  <SIcon
                    name="Check"
                    size="w-2.5 h-2.5"
                  />
                </div>
              </button>
            </div>
          </Card>

          <!-- Profile List Title -->
          <div class="codex-profiles-section-heading">
            <h2 class="codex-profiles-section-heading__title">
              <SIcon
                name="ListFilter"
                size="w-5 h-5"
                class="text-platform-codex"
              />
              {{ $t('codex.profiles.listTitle') }}
            </h2>
          </div>
            
          <!-- Loading State -->
          <div
            v-if="loading"
            class="codex-profiles-loading"
          >
            <div class="codex-profiles-loading__spinner" />
          </div>

          <!-- Empty State -->
          <div
            v-else-if="profiles.length === 0"
            class="empty-state glass-effect rounded-2xl border border-white/5"
          >
            <div class="p-4 rounded-full glass-surface mb-4">
              <SIcon
                name="Boxes"
                size="w-8 h-8"
                class="text-white/50"
              />
            </div>
            <p class="text-white/80">
              {{ $t('codex.profiles.emptyState') }}
            </p>
          </div>

          <!-- Profile Grid -->
          <div
            v-else
            class="codex-profiles-grid"
          >
            <Card 
              v-for="profile in profiles" 
              :key="profile.name"
              variant="glass"
              class="group codex-profiles-card"
              :class="[currentProfile && profile.name === currentProfile ? 'config-card-active' : '']"
              :glow-color="currentProfile && profile.name === currentProfile ? 'warning' : 'primary'"
              padding="lg"
            >
              <!-- Active Indicator Background -->
              <div 
                v-if="currentProfile && profile.name === currentProfile"
                class="absolute top-0 right-0 w-32 h-32 bg-gradient-to-bl from-platform-codex/10 to-transparent -mr-8 -mt-8 rounded-bl-full pointer-events-none"
              />

              <div class="relative z-10">
                <div class="flex items-start justify-between gap-4 mb-4">
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 mb-2">
                      <h3 class="text-lg font-bold font-mono text-white truncate">
                        {{ profile.name }}
                      </h3>
                      <span 
                        v-if="currentProfile && profile.name === currentProfile"
                        class="badge badge-primary"
                      >
                        {{ $t('codex.profiles.currentBadge') }}
                      </span>
                      <span 
                        v-else-if="profile.enabled === false"
                        class="badge badge-danger"
                      >
                        {{ $t('codex.states.disabled') }}
                      </span>
                      <span 
                        v-else
                        class="badge badge-success"
                      >
                        {{ $t('codex.states.enabled') }}
                      </span>
                    </div>
                    <p
                      v-if="profile.description"
                      class="text-sm text-white/80 line-clamp-1"
                    >
                      {{ profile.description }}
                    </p>
                  </div>
                   
                  <!-- Actions -->
                  <div class="codex-profiles-card__actions">
                    <button 
                      class="codex-profiles-action-button codex-profiles-action-button--success"
                      :title="$t('codex.profiles.apply')"
                      :disabled="actionLoading"
                      @click.stop="handleApply(profile.name)"
                    >
                      <SIcon
                        :name="busyProfileName === profile.name && busyAction === 'apply' ? 'RefreshCw' : 'Check'"
                        size="w-4 h-4"
                        :class="{ 'animate-spin': busyProfileName === profile.name && busyAction === 'apply' }"
                      />
                    </button>
                    <button 
                      class="codex-profiles-action-button codex-profiles-action-button--primary"
                      :title="$t('codex.actions.edit')"
                      @click.stop="handleEdit(profile.name)"
                    >
                      <SIcon
                        name="Edit2"
                        size="w-4 h-4"
                      />
                    </button>
                    <button 
                      class="codex-profiles-action-button codex-profiles-action-button--danger"
                      :title="$t('codex.actions.delete')"
                      :disabled="actionLoading"
                      @click.stop="handleDelete(profile.name)"
                    >
                      <SIcon
                        :name="busyProfileName === profile.name && busyAction === 'delete' ? 'RefreshCw' : 'Trash2'"
                        size="w-4 h-4"
                        :class="{ 'animate-spin': busyProfileName === profile.name && busyAction === 'delete' }"
                      />
                    </button>
                  </div>
                </div>

                <!-- Info Grid -->
                <div class="codex-profiles-card__info-grid">
                  <div class="codex-profiles-card__info-item">
                    <span class="text-xs font-medium text-white/50 uppercase tracking-wider">
                      {{ $t('codex.profiles.fields.baseUrl') }}
                    </span>
                    <code class="codex-profiles-card__code">
                      {{ profileBaseUrl(profile) }}
                    </code>
                  </div>

                  <div class="codex-profiles-card__info-item">
                    <span class="text-xs font-medium text-white/50 uppercase tracking-wider">
                      {{ $t('codex.profiles.fields.model') }}
                    </span>
                    <div class="flex items-center gap-2">
                      <span class="codex-profiles-card__model-pill">
                        {{ profile.model }}
                      </span>
                    </div>
                  </div>

                  <div class="codex-profiles-card__info-item">
                    <span class="text-xs font-medium text-white/50 uppercase tracking-wider">
                      {{ $t('codex.profiles.fields.authMode') }}
                    </span>
                    <div class="flex items-center gap-2 flex-wrap">
                      <span class="codex-profiles-card__meta-pill">
                        {{ authModeLabel(profile.auth_mode) }}
                      </span>
                      <span
                        v-if="profile.openai_login_method"
                        class="px-2 py-0.5 rounded-md text-xs font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20"
                      >
                        {{ profile.openai_login_method }}
                      </span>
                    </div>
                  </div>

                  <div class="codex-profiles-card__info-item">
                    <span class="text-xs font-medium text-white/50 uppercase tracking-wider">
                      {{ $t('codex.profiles.fields.authSource') }}
                    </span>
                    <div class="flex items-center gap-2 flex-wrap">
                      <code class="codex-profiles-card__code">
                        {{ profile.auth_source || $t('codex.profiles.notAvailable') }}
                      </code>
                      <span
                        v-if="profile.credential_store"
                        class="px-2 py-0.5 rounded-md text-xs font-medium bg-sky-500/10 text-sky-300 border border-sky-500/20"
                      >
                        {{ profile.credential_store }}
                      </span>
                    </div>
                  </div>

                  <div
                    v-if="profile.env_key"
                    class="codex-profiles-card__info-item"
                  >
                    <span class="text-xs font-medium text-white/50 uppercase tracking-wider">
                      {{ $t('codex.profiles.fields.envKey') }}
                    </span>
                    <code class="codex-profiles-card__code">
                      {{ profile.env_key }}
                    </code>
                  </div>
                </div>

                <div
                  v-if="profile.shell_export_script"
                  class="mt-4 rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-3"
                >
                  <div class="flex items-start justify-between gap-3">
                    <div class="min-w-0">
                      <p class="text-xs font-medium uppercase tracking-wider text-emerald-300">
                        {{ $t('codex.profiles.envExportTitle') }}
                      </p>
                      <p class="mt-1 text-xs text-white/60">
                        {{ $t('codex.profiles.envExportHint') }}
                      </p>
                    </div>
                    <button
                      class="btn btn-secondary btn-sm"
                      @click.stop="copyProfileEnv(profile)"
                    >
                      {{ $t('codex.profiles.copyEnvExport') }}
                    </button>
                  </div>
                  <pre class="mt-3 overflow-x-auto rounded-lg glass-surface p-3 text-xs text-white/80"><code>{{ profile.shell_export_script }}</code></pre>
                </div>
                 
                <div
                  v-if="profile.tags?.length || profile.provider || (profile.extra && Object.keys(profile.extra).length > 0)"
                  class="mt-4 flex items-center justify-between border-t border-white/5 pt-3"
                >
                  <div class="flex flex-wrap gap-1.5">
                    <span 
                      v-if="profile.provider"
                      class="px-2 py-0.5 rounded-md text-xs font-medium glass-surface text-white/80"
                    >
                      {{ profile.provider }}
                    </span>
                    <span 
                      v-for="tag in profile.tags" 
                      :key="tag"
                      class="px-2 py-0.5 rounded-md text-xs font-medium glass-surface text-white/50"
                    >
                      #{{ tag }}
                    </span>
                  </div>
                   
                  <div
                    v-if="profile.extra && Object.keys(profile.extra).length > 0"
                    class="text-xs text-white/50 font-mono glass-surface px-2 py-1 rounded"
                  >
                    +{{ Object.keys(profile.extra).length }} extras
                  </div>
                </div>
              </div>
            </Card>
          </div>
            
          <!-- Add/Edit Modal -->
          <div
            v-if="showForm"
            class="codex-profiles-modal-overlay"
          >
            <Card
              variant="glass"
              class="codex-profiles-modal-card animate-in zoom-in-95 duration-200"
              :padding="'none'"
            >
              <!-- Modal Header -->
              <div class="codex-profiles-modal-header">
                <h2 class="text-xl font-bold text-white">
                  {{ editingName ? $t('codex.profiles.editProfile') : $t('codex.profiles.addProfile') }}
                </h2>
                <button
                  class="codex-profiles-modal-close"
                  @click="handleCloseForm"
                >
                  <SIcon
                    name="X"
                    size="w-5 h-5"
                  />
                </button>
              </div>

              <!-- Modal Content -->
              <div class="codex-profiles-modal-body">
                <!-- Use generic grid for form -->
                <div class="codex-profiles-form-grid">
                  <!-- Name & Desc -->
                  <div class="codex-profiles-form-grid--two-col">
                    <div class="codex-profiles-field">
                      <label class="codex-profiles-field__label">
                        {{ $t('codex.profiles.fields.name') }} <span class="text-red-500">*</span>
                      </label>
                      <input
                        v-model="form.name"
                        :disabled="!!editingName"
                        type="text"
                        class="codex-profiles-input"
                        :placeholder="$t('codex.profiles.placeholders.name')"
                      >
                    </div>
                    <div class="codex-profiles-field">
                      <label class="codex-profiles-field__label">
                        {{ $t('codex.profiles.fields.description') }}
                      </label>
                      <input
                        v-model="form.description"
                        type="text"
                        class="codex-profiles-input"
                        :placeholder="$t('codex.profiles.placeholders.description')"
                      >
                    </div>
                  </div>
                       
                  <div class="codex-profiles-form-grid--two-col">
                    <div class="codex-profiles-field">
                      <label class="codex-profiles-field__label">
                        {{ $t('codex.profiles.fields.authMode') }} <span class="text-red-500">*</span>
                      </label>
                      <select
                        v-model="form.auth_mode"
                        class="codex-profiles-input"
                      >
                        <option
                          v-for="authMode in availableAuthModeOptions"
                          :key="authMode"
                          :value="authMode"
                        >
                          {{ authModeLabel(authMode) }}
                        </option>
                      </select>
                      <p
                        v-if="isDeprecatedAuthMode(form.auth_mode)"
                        class="text-xs text-amber-300"
                      >
                        {{ $t('codex.profiles.deprecatedAuthModeHint', { mode: authModeLabel(form.auth_mode) }) }}
                      </p>
                    </div>
                    <div class="codex-profiles-field">
                      <label class="codex-profiles-field__label">
                        {{ $t('codex.profiles.fields.openAiLoginMethod') }}
                      </label>
                      <input
                        :value="displayOpenAiLoginMethod"
                        type="text"
                        class="codex-profiles-input codex-profiles-input--mono"
                        disabled
                      >
                    </div>
                  </div>

                  <!-- URL & Token -->
                  <div class="codex-profiles-field">
                    <label class="codex-profiles-field__label">
                      {{ $t('codex.profiles.fields.baseUrl') }}
                      <span
                        v-if="requiresBaseUrl"
                        class="text-red-500"
                      >*</span>
                    </label>
                    <input
                      v-model="form.base_url"
                      type="text"
                      class="codex-profiles-input codex-profiles-input--mono"
                      :placeholder="$t('codex.profiles.placeholders.baseUrl')"
                    >
                    <p class="text-xs text-white/50">
                      {{ requiresBaseUrl ? $t('codex.profiles.baseUrlRequiredHint') : $t('codex.profiles.baseUrlOptionalHint') }}
                    </p>
                  </div>

                  <div class="codex-profiles-field">
                    <label class="codex-profiles-field__label">
                      {{ $t('codex.profiles.fields.authToken') }}
                      <span
                        v-if="requiresSecret"
                        class="text-red-500"
                      >*</span>
                    </label>
                    <input
                      v-model="form.auth_token"
                      type="password"
                      class="codex-profiles-input codex-profiles-input--mono"
                      :placeholder="$t('codex.profiles.placeholders.authToken')"
                    >
                    <p class="text-xs text-white/50">
                      {{ authTokenHint }}
                    </p>
                  </div>

                  <div
                    v-if="requiresEnvKey"
                    class="codex-profiles-field"
                  >
                    <label class="codex-profiles-field__label">
                      {{ $t('codex.profiles.fields.envKey') }} <span class="text-red-500">*</span>
                    </label>
                    <input
                      v-model="form.env_key"
                      type="text"
                      class="codex-profiles-input codex-profiles-input--mono"
                      :placeholder="$t('codex.profiles.placeholders.envKey')"
                    >
                    <p class="text-xs text-white/50">
                      {{ $t('codex.profiles.envKeyHint') }}
                    </p>
                  </div>

                  <!-- Models -->
                  <div class="codex-profiles-form-grid--two-col">
                    <div class="codex-profiles-field">
                      <label class="codex-profiles-field__label">
                        {{ $t('codex.profiles.fields.model') }} <span class="text-red-500">*</span>
                      </label>
                      <select
                        v-model="selectedModelOption"
                        class="codex-profiles-input codex-profiles-input--mono"
                      >
                        <option
                          v-for="model in modelCatalog"
                          :key="model"
                          :value="model"
                        >
                          {{ model }}
                        </option>
                        <option :value="CUSTOM_MODEL_OPTION">
                          {{ $t('codex.profiles.customModelOption') }}
                        </option>
                      </select>
                      <input
                        v-if="selectedModelOption === CUSTOM_MODEL_OPTION"
                        v-model="customModelInput"
                        type="text"
                        class="codex-profiles-input codex-profiles-input--mono codex-profiles-input--spaced"
                        :placeholder="$t('codex.profiles.placeholders.customModel')"
                      >
                      <p class="text-xs text-white/50">
                        {{ selectedModelOption === CUSTOM_MODEL_OPTION ? $t('codex.profiles.customModelHint') : $t('codex.profiles.modelPresetHint') }}
                      </p>
                    </div>
                    <div class="codex-profiles-field">
                      <label class="codex-profiles-field__label">
                        {{ $t('codex.profiles.fields.smallFastModel') }}
                      </label>
                      <input
                        v-model="form.small_fast_model"
                        type="text"
                        class="codex-profiles-input codex-profiles-input--mono"
                        :placeholder="$t('codex.profiles.placeholders.smallFastModel')"
                      >
                    </div>
                  </div>

                  <div class="codex-profiles-form-grid--two-col">
                    <div class="codex-profiles-field">
                      <label class="codex-profiles-field__label">
                        {{ $t('codex.profiles.fields.wireApi') }}
                      </label>
                      <input
                        v-model="form.wire_api"
                        type="text"
                        class="codex-profiles-input codex-profiles-input--mono"
                        :placeholder="$t('codex.profiles.placeholders.wireApi')"
                      >
                    </div>
                    <div class="codex-profiles-checkbox-row">
                      <input
                        id="requiresOpenAiAuth"
                        :checked="requiresOpenAiAuth"
                        type="checkbox"
                        class="codex-profiles-checkbox"
                        disabled
                      >
                      <label
                        for="requiresOpenAiAuth"
                        class="codex-profiles-checkbox-label codex-profiles-checkbox-label--muted"
                      >
                        {{ $t('codex.profiles.fields.requiresOpenaiAuth') }}
                      </label>
                    </div>
                  </div>
                       
                  <!-- Metadata -->
                  <div class="codex-profiles-form-grid--three-col">
                    <div class="codex-profiles-field">
                      <label class="codex-profiles-field__label">
                        {{ $t('codex.profiles.fields.provider') }}
                      </label>
                      <input
                        v-model="form.provider"
                        type="text"
                        class="codex-profiles-input"
                        :placeholder="$t('codex.profiles.placeholders.provider')"
                      >
                    </div>
                    <div class="codex-profiles-field">
                      <label class="codex-profiles-field__label">
                        {{ $t('codex.profiles.fields.providerType') }}
                      </label>
                      <input
                        v-model="form.provider_type"
                        type="text"
                        class="codex-profiles-input"
                        :placeholder="$t('codex.profiles.placeholders.providerType')"
                      >
                    </div>
                    <div class="codex-profiles-field">
                      <label class="codex-profiles-field__label">
                        {{ $t('codex.profiles.fields.tags') }}
                      </label>
                      <input
                        v-model="tagsText"
                        type="text"
                        class="codex-profiles-input"
                        :placeholder="$t('codex.profiles.placeholders.tags')"
                      >
                    </div>
                  </div>
                       
                  <div class="codex-profiles-checkbox-row">
                    <input
                      id="profileEnabled"
                      v-model="form.enabled"
                      type="checkbox"
                      class="codex-profiles-checkbox"
                    >
                    <label
                      for="profileEnabled"
                      class="codex-profiles-checkbox-label"
                    >
                      {{ $t('codex.profiles.fields.enabled') }}
                    </label>
                  </div>
                       
                  <!-- Extra JSON -->
                  <div class="codex-profiles-field">
                    <label class="codex-profiles-field__label codex-profiles-field__label--between">
                      <span>{{ $t('codex.profiles.fields.extraJson') }}</span>
                      <span class="text-xs font-normal text-white/50">{{ $t('codex.profiles.extraHint') }}</span>
                    </label>
                    <textarea
                      v-model="extraText"
                      rows="6"
                      class="codex-profiles-input codex-profiles-input--mono codex-profiles-textarea"
                      :placeholder="$t('codex.profiles.placeholders.extraJson')"
                    />
                  </div>
                </div>
              </div>

              <!-- Footer -->
              <div class="codex-profiles-modal-footer">
                <button
                  class="btn btn-secondary"
                  @click="handleCloseForm"
                >
                  {{ $t('codex.actions.cancel') }}
                </button>
                <button 
                  class="btn btn-primary"
                  :disabled="saving"
                  @click="handleSave"
                >
                  <span
                    v-if="saving"
                    class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin mr-2"
                  />
                  {{ saving ? $t('codex.states.saving') : $t('codex.actions.save') }}
                </button>
              </div>
            </Card>
          </div>

          <ConfirmModal
            v-model:is-open="showConfirmModal"
            :type="confirmDialog.type"
            :title="confirmDialog.title"
            :message="confirmDialog.message"
            :confirm-text="confirmDialog.confirmText"
            :cancel-text="$t('common.cancel')"
            @confirm="executeConfirmedAction"
          />
        </main>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { computed, onActivated, onMounted, reactive, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import Card from '@/components/ui/Card.vue'
import ConfirmModal from '@/components/ConfirmModal.vue'
import { addCodexCustomModel, addCodexProfile, applyCodexProfile, deleteCodexProfile, getCodexProfile, listCodexModels, listCodexProfiles, updateCodexProfile } from '@/api'
import type {
  CodexAddCustomModelResponse,
  CodexModelsResponse,
  CodexProfile,
  CodexProfileAuthMode,
  CodexProfileRequest,
  CodexProfilesResponse,
  OpenAiLoginMethod,
} from '@/types'
import { logger } from '@/utils/logger'
import { useUIStore } from '@/stores/ui'

defineOptions({ name: 'CodexProfilesView' })

const { t } = useI18n()
const uiStore = useUIStore()

const loading = ref(false)
const saving = ref(false)
const actionLoading = ref(false)

const AVAILABLE_AUTH_MODES: CodexProfileAuthMode[] = ['openai_api_key', 'no_auth']
const DEPRECATED_AUTH_MODES: CodexProfileAuthMode[] = ['openai_chatgpt', 'provider_env_key']
const CUSTOM_MODEL_OPTION = '__custom__'

const profiles = ref<CodexProfile[]>([])
const currentProfile = ref<string | null>(null)
const codexBuiltinModels = ref<string[]>([])
const codexCustomModels = ref<string[]>([])
const selectedModelOption = ref<string>('')
const customModelInput = ref('')

const showForm = ref(false)
const editingName = ref<string | null>(null)
const busyProfileName = ref<string | null>(null)
const busyAction = ref<'apply' | 'delete' | null>(null)
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

const REFRESH_TTL_MS = 30_000

const tagsText = ref('')
const extraText = ref('{}')

const extractErrorMessage = (error: unknown) => {
  if (typeof error === 'string') {
    return error
  }
  if (error && typeof error === 'object') {
    const candidate = error as { message?: unknown, error?: unknown, cause?: unknown }
    for (const value of [candidate.message, candidate.error, candidate.cause]) {
      if (typeof value === 'string' && value.trim()) {
        return value
      }
    }
  }
  return null
}

const authModeToLoginMethod = (authMode: CodexProfileAuthMode): OpenAiLoginMethod | undefined => {
  switch (authMode) {
    case 'openai_chatgpt':
      return 'chatgpt'
    case 'openai_api_key':
      return 'api'
    default:
      return undefined
  }
}

const normalizeModelName = (value?: string | null) => value?.trim() || ''

const modelCatalog = computed(() => {
  const merged = [...codexBuiltinModels.value, ...codexCustomModels.value]
  return merged.filter((model, index) => merged.indexOf(model) === index)
})

const isDeprecatedAuthMode = (authMode?: CodexProfileAuthMode | null) => {
  return authMode ? DEPRECATED_AUTH_MODES.includes(authMode) : false
}

const availableAuthModeOptions = computed(() => {
  const options = [...AVAILABLE_AUTH_MODES]
  if (isDeprecatedAuthMode(form.auth_mode) && !options.includes(form.auth_mode)) {
    options.push(form.auth_mode)
  }
  return options
})

const usesOpenAiAuthMode = (authMode: CodexProfileAuthMode) => {
  return authMode === 'openai_chatgpt' || authMode === 'openai_api_key'
}

const isOfficialConfig = (profile: CodexProfile) => {
  return !profile.base_url?.trim()
}

const authModeLabel = (authMode?: CodexProfileAuthMode | null) => {
  return t(`codex.profiles.authModes.${authMode || 'no_auth'}`)
}

const profileBaseUrl = (profile: CodexProfile) => {
  return profile.base_url?.trim() || t('codex.profiles.officialBaseUrl')
}

const buildShellExportFallback = (profile: CodexProfile) => {
  const envExport = profile.env_export
  if (!envExport || Object.keys(envExport).length === 0) {
    return ''
  }
  return Object.entries(envExport)
    .map(([key, value]) => `export ${key}=${JSON.stringify(value)}`)
    .join('\n')
}

const copyProfileEnv = async (profile: CodexProfile) => {
  const script = profile.shell_export_script || buildShellExportFallback(profile)
  if (!script) {
    return
  }

  try {
    await navigator.clipboard.writeText(script)
    uiStore.showSuccess(t('codex.profiles.messages.envExportCopied'))
  } catch (error) {
    logger.error('Failed to copy profile env export:', error)
    uiStore.showError(t('codex.profiles.messages.envExportCopyFailed'))
  }
}

// 当前配置模式
const currentConfigMode = computed(() => {
  if (!currentProfile.value) return 'official'
  const profile = profiles.value.find(p => p.name === currentProfile.value)
  return profile && isOfficialConfig(profile) ? 'official' : 'custom'
})

const form = reactive<Required<Pick<CodexProfileRequest, 'name' | 'model' | 'auth_mode'>> & Partial<CodexProfileRequest>>({
  name: '',
  description: '',
  base_url: '',
  auth_token: '',
  model: '',
  small_fast_model: '',
  provider: '',
  provider_type: '',
  account: '',
  tags: [],
  enabled: true,
  wire_api: '',
  env_key: '',
  requires_openai_auth: false,
  auth_mode: 'no_auth',
  openai_login_method: undefined,
  extra: {},
})

const requiresBaseUrl = computed(() => !usesOpenAiAuthMode(form.auth_mode))
const requiresSecret = computed(() => form.auth_mode === 'openai_api_key')
const requiresEnvKey = computed(() => form.auth_mode === 'provider_env_key')
const requiresOpenAiAuth = computed(() => usesOpenAiAuthMode(form.auth_mode))
const displayOpenAiLoginMethod = computed(() => authModeToLoginMethod(form.auth_mode) || t('codex.profiles.notAvailable'))
const resolvedModelValue = computed(() => {
  return selectedModelOption.value === CUSTOM_MODEL_OPTION
    ? normalizeModelName(customModelInput.value)
    : normalizeModelName(selectedModelOption.value)
})
const authTokenHint = computed(() => {
  if (form.auth_mode === 'openai_chatgpt') {
    return t('codex.profiles.authTokenHints.openai_chatgpt')
  }
  if (form.auth_mode === 'openai_api_key') {
    return t('codex.profiles.authTokenHints.openai_api_key')
  }
  if (form.auth_mode === 'provider_env_key') {
    return t('codex.profiles.authTokenHints.provider_env_key')
  }
  return t('codex.profiles.authTokenHints.no_auth')
})

const loadModels = async () => {
  try {
    const data = await listCodexModels<CodexModelsResponse>()
    codexBuiltinModels.value = data.builtin_models || []
    codexCustomModels.value = data.custom_models || []
  } catch (error) {
    logger.error('Failed to load codex models:', error)
  }
}

const loadProfiles = async () => {
  try {
    loading.value = true
    const [profilesData] = await Promise.all([
      listCodexProfiles<CodexProfilesResponse>(),
      loadModels(),
    ])
    profiles.value = profilesData.profiles || []
    currentProfile.value = profilesData.current_profile ?? null
    lastLoadedAt.value = Date.now()
  } catch (error) {
    logger.error('Failed to load codex profiles:', error)
    uiStore.showError(t('codex.states.loadFailed'))
  } finally {
    loading.value = false
  }
}

const ensureLoaded = async (force = false) => {
  if (loading.value) return
  if (!force && lastLoadedAt.value && Date.now() - lastLoadedAt.value < REFRESH_TTL_MS) {
    return
  }
  await loadProfiles()
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

const resetForm = () => {
  Object.assign(form, {
    name: '',
    description: '',
    base_url: '',
    auth_token: '',
    model: '',
    small_fast_model: '',
    provider: '',
    provider_type: '',
    account: '',
    tags: [],
    enabled: true,
    wire_api: '',
    env_key: '',
    requires_openai_auth: false,
    auth_mode: 'no_auth',
    openai_login_method: undefined,
    extra: {},
  })
  selectedModelOption.value = modelCatalog.value[0] || CUSTOM_MODEL_OPTION
  customModelInput.value = ''
  tagsText.value = ''
  extraText.value = JSON.stringify({}, null, 2)
}

const applyProfileToForm = (profile: CodexProfile) => {
  Object.assign(form, {
    name: profile.name,
    description: profile.description || '',
    base_url: profile.base_url || '',
    auth_token: profile.auth_token || '',
    model: profile.model || '',
    small_fast_model: profile.small_fast_model || '',
    provider: profile.provider || '',
    provider_type: profile.provider_type || '',
    account: profile.account || '',
    tags: profile.tags || [],
    enabled: profile.enabled !== false,
    wire_api: profile.wire_api || '',
    env_key: profile.env_key || '',
    requires_openai_auth: profile.requires_openai_auth ?? usesOpenAiAuthMode(profile.auth_mode || 'no_auth'),
    auth_mode: profile.auth_mode || 'no_auth',
    openai_login_method: profile.openai_login_method || authModeToLoginMethod(profile.auth_mode || 'no_auth'),
    extra: profile.extra || {},
  })

  const normalizedModel = normalizeModelName(profile.model)
  if (normalizedModel && modelCatalog.value.includes(normalizedModel)) {
    selectedModelOption.value = normalizedModel
    customModelInput.value = ''
  } else {
    selectedModelOption.value = CUSTOM_MODEL_OPTION
    customModelInput.value = normalizedModel
  }

  tagsText.value = (form.tags || []).join(', ')
  extraText.value = JSON.stringify(form.extra || {}, null, 2)
}

const openFormModal = async (name?: string) => {
  editingName.value = name ?? null
  await loadModels()
  resetForm()
  showForm.value = true

  if (!name) {
    return
  }

  const profile = await getCodexProfile<CodexProfile>(name)
  applyProfileToForm(profile)
}

const resetBusyState = () => {
  busyProfileName.value = null
  busyAction.value = null
}

const handleProfileAction = async (
  name: string,
  action: 'apply' | 'delete',
  task: () => Promise<void>,
  successMessage: string,
  errorMessage: string,
) => {
  busyProfileName.value = name
  busyAction.value = action
  try {
    await task()
    await loadProfiles()
    uiStore.showSuccess(successMessage)
  } catch (error) {
    logger.error(`Failed to ${action} codex profile:`, error)
    uiStore.showError(extractErrorMessage(error) || errorMessage)
  } finally {
    resetBusyState()
  }
}

const handleAdd = async () => {
  await openFormModal()
}

const handleEdit = async (name: string) => {
  try {
    await openFormModal(name)
  } catch (error) {
    logger.error('Failed to load codex profile:', error)
    uiStore.showError(extractErrorMessage(error) || t('codex.states.loadFailed'))
    showForm.value = false
  }
}

const handleCloseForm = () => {
  showForm.value = false
  editingName.value = null
}

const parseTags = (raw: string): string[] | undefined => {
  const tags = raw
    .split(',')
    .map(s => s.trim())
    .filter(Boolean)
  return tags.length > 0 ? tags : undefined
}

const parseExtraJson = (raw: string): Record<string, unknown> | undefined => {
  const trimmed = raw.trim()
  if (!trimmed) return undefined
  const parsed = JSON.parse(trimmed)
  if (parsed === null || typeof parsed !== 'object' || Array.isArray(parsed)) {
    throw new Error('extra must be a JSON object')
  }
  return parsed
}

const syncDerivedAuthFields = () => {
  form.openai_login_method = authModeToLoginMethod(form.auth_mode)
  form.requires_openai_auth = usesOpenAiAuthMode(form.auth_mode)
  form.model = resolvedModelValue.value

  if (!requiresEnvKey.value) {
    form.env_key = ''
  }
}

const handleSave = async () => {
  syncDerivedAuthFields()

  if (!form.name.trim()) {
    uiStore.showError(t('codex.profiles.validation.nameRequired'))
    return
  }
  if (requiresBaseUrl.value && !form.base_url?.trim()) {
    uiStore.showError(t('codex.profiles.validation.baseUrlRequired'))
    return
  }
  if (requiresSecret.value && !form.auth_token?.trim()) {
    uiStore.showError(t('codex.profiles.validation.authTokenRequired'))
    return
  }
  if (requiresEnvKey.value && !form.env_key?.trim()) {
    uiStore.showError(t('codex.profiles.validation.envKeyRequired'))
    return
  }
  if (!resolvedModelValue.value) {
    uiStore.showError(t('codex.profiles.validation.modelRequired'))
    return
  }

  let extra: Record<string, unknown> | undefined
  try {
    extra = parseExtraJson(extraText.value) || undefined
  } catch {
    uiStore.showError(t('codex.profiles.validation.extraJsonInvalid'))
    return
  }

  const request: CodexProfileRequest = {
    name: form.name.trim(),
    description: form.description?.trim() ? form.description.trim() : undefined,
    base_url: form.base_url?.trim() ? form.base_url.trim() : undefined,
    auth_token: form.auth_token?.trim() ? form.auth_token.trim() : undefined,
    model: resolvedModelValue.value,
    small_fast_model: form.small_fast_model?.trim() ? form.small_fast_model.trim() : undefined,
    provider: form.provider?.trim() ? form.provider.trim() : undefined,
    provider_type: form.provider_type?.trim() ? form.provider_type.trim() : undefined,
    account: form.account?.trim() ? form.account.trim() : undefined,
    tags: parseTags(tagsText.value),
    enabled: form.enabled === true,
    wire_api: form.wire_api?.trim() ? form.wire_api.trim() : undefined,
    env_key: form.env_key?.trim() ? form.env_key.trim() : undefined,
    requires_openai_auth: requiresOpenAiAuth.value,
    auth_mode: form.auth_mode,
    openai_login_method: form.openai_login_method,
    extra,
  }

  try {
    saving.value = true
    const isEditing = Boolean(editingName.value)
    if (selectedModelOption.value === CUSTOM_MODEL_OPTION) {
      const response = await addCodexCustomModel<CodexAddCustomModelResponse>(resolvedModelValue.value)
      const models = response.models || []
      codexCustomModels.value = models.filter(model => !codexBuiltinModels.value.includes(model))
    }
    if (editingName.value) {
      await updateCodexProfile(editingName.value, request)
    } else {
      await addCodexProfile(request)
    }
    handleCloseForm()
    await loadProfiles()
    uiStore.showSuccess(
      isEditing ? t('codex.profiles.updateProfile') : t('codex.profiles.addProfile')
    )
  } catch (error) {
    logger.error('Failed to save codex profile:', error)
    uiStore.showError(extractErrorMessage(error) || t('codex.states.saveFailed'))
  } finally {
    saving.value = false
  }
}

const handleDelete = async (name: string) => {
  openConfirmDialog({
    title: t('codex.actions.delete'),
    message: t('codex.profiles.confirmDelete', { name }),
    confirmText: t('codex.actions.delete'),
    type: 'danger',
    action: async () => {
      await handleProfileAction(
        name,
        'delete',
        () => deleteCodexProfile(name),
        t('codex.actions.delete'),
        t('codex.states.deleteFailed'),
      )
    },
  })
}

const handleApply = async (name: string) => {
  openConfirmDialog({
    title: t('codex.profiles.apply'),
    message: t('codex.profiles.confirmApply', { name }),
    confirmText: t('codex.profiles.apply'),
    type: 'warning',
    action: async () => {
      await handleProfileAction(
        name,
        'apply',
        () => applyCodexProfile(name),
        t('codex.profiles.apply'),
        t('codex.states.saveFailed'),
      )
    },
  })
}

onMounted(async () => {
  await ensureLoaded(true)
})

onActivated(() => {
  void ensureLoaded(false)
})
</script>

<style scoped>
.codex-profiles-view {
  @apply min-h-full p-6;
}

.codex-profiles-shell {
  @apply mx-auto max-w-[1800px];
}

.codex-profiles-stack {
  @apply mt-6 space-y-6;
}

.codex-profiles-main {
  @apply flex w-full min-w-0 flex-col gap-6;
}

.codex-profiles-header {
  @apply flex items-center justify-between;
}

.codex-profiles-header__intro,
.codex-profiles-header__actions,
.codex-profiles-section-heading__title,
.codex-profiles-switches {
  @apply flex items-center gap-3;
}

.codex-profiles-header__icon {
  @apply rounded-xl bg-platform-codex/10 p-2;
}

.codex-profiles-status-grid {
  @apply grid grid-cols-1 gap-4 md:grid-cols-3;
}

.codex-profiles-status-card {
  @apply overflow-hidden;
}

.codex-profiles-status-icon {
  @apply rounded-xl p-3 transition-transform duration-300;
}

.group:hover .codex-profiles-status-icon {
  transform: scale(1.1);
}

.codex-profiles-status-icon--warning {
  @apply bg-yellow-500/10 text-yellow-500;
}

.codex-profiles-status-icon--primary {
  @apply bg-indigo-500/10 text-indigo-500;
}

.codex-profiles-status-icon--official {
  @apply bg-emerald-500/10 text-emerald-500;
}

.codex-profiles-status-icon--relay {
  @apply bg-pink-500/10 text-pink-500;
}

.codex-profiles-switch {
  @apply relative flex items-center gap-2.5 rounded-xl px-4 py-2.5 text-sm font-medium transition-all duration-300;
}

.codex-profiles-switch--active {
  @apply glass-effect-strong border border-platform-codex/50 text-platform-codex;

  box-shadow: 0 0 15px rgb(245 158 11 / 20%);
}

.codex-profiles-switch--idle {
  @apply glass-effect text-white/80 hover:border-platform-codex/30 hover:bg-white/10;
}

.codex-profiles-switch--busy {
  @apply cursor-not-allowed opacity-60;
}

.codex-profiles-switch__active-indicator {
  @apply flex h-4 w-4 items-center justify-center rounded-full bg-platform-codex text-[10px] text-white;
}

.codex-profiles-section-heading {
  @apply flex items-center justify-between;
}

.codex-profiles-section-heading__title {
  @apply text-xl font-bold text-white;
}

.codex-profiles-loading {
  @apply flex justify-center py-20;
}

.codex-profiles-loading__spinner {
  @apply h-12 w-12 animate-spin rounded-full border-4 border-transparent border-r-accent-secondary border-t-accent-primary;
}

.codex-profiles-grid {
  @apply grid grid-cols-1 gap-4 xl:grid-cols-2;
}

.codex-profiles-card {
  @apply relative overflow-hidden transition-[box-shadow,transform] duration-300 hover:-translate-y-1 hover:shadow-xl;
}

.codex-profiles-card__actions {
  @apply flex items-center gap-1 opacity-100 transition-opacity duration-200 xl:opacity-0;
}

.group:hover .codex-profiles-card__actions {
  opacity: 1;
}

.codex-profiles-action-button {
  @apply rounded-lg p-2 transition-colors hover:bg-white/10;
}

.codex-profiles-action-button--success {
  @apply text-accent-success;
}

.codex-profiles-action-button--primary {
  @apply text-accent-primary;
}

.codex-profiles-action-button--danger {
  @apply text-accent-danger;
}

.codex-profiles-card__info-grid {
  @apply grid grid-cols-1 gap-x-6 gap-y-3 text-sm sm:grid-cols-2;
}

.codex-profiles-card__info-item {
  @apply flex flex-col gap-1;
}

.codex-profiles-card__code {
  @apply truncate rounded px-2 py-1 font-mono text-white glass-surface;
}

.codex-profiles-card__model-pill {
  @apply rounded bg-accent-primary/5 px-2 py-0.5 font-mono text-accent-primary;
}

.codex-profiles-card__meta-pill {
  @apply rounded-md px-2 py-0.5 text-xs font-medium text-white/80 glass-surface;
}

.codex-profiles-modal-overlay {
  @apply fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4 backdrop-blur-md;
}

.codex-profiles-modal-card {
  @apply glass-modal max-h-[90vh] w-full max-w-3xl overflow-y-auto shadow-2xl;
}

.codex-profiles-modal-header {
  @apply sticky top-0 z-10 flex items-center justify-between border-b border-white/5 px-6 py-4 glass-effect-strong;
}

.codex-profiles-modal-close {
  @apply rounded-lg p-1 text-white/50 transition-colors hover:bg-white/10;
}

.codex-profiles-modal-body {
  @apply space-y-6 p-6;
}

.codex-profiles-form-grid {
  @apply grid grid-cols-1 gap-6;
}

.codex-profiles-form-grid--two-col {
  @apply grid grid-cols-1 gap-4 md:grid-cols-2;
}

.codex-profiles-form-grid--three-col {
  @apply grid grid-cols-1 gap-4 md:grid-cols-3;
}

.codex-profiles-field {
  @apply space-y-1.5;
}

.codex-profiles-field__label {
  @apply text-sm font-semibold text-white/80;
}

.codex-profiles-field__label--between {
  @apply flex justify-between;
}

.codex-profiles-input {
  @apply input;
}

.codex-profiles-input--mono {
  @apply font-mono text-sm;
}

.codex-profiles-input--spaced {
  @apply mt-2;
}

.codex-profiles-textarea {
  @apply text-xs leading-relaxed;
}

.codex-profiles-checkbox-row {
  @apply flex items-center gap-3 rounded-lg border border-white/5 p-3 glass-surface;
}

.codex-profiles-checkbox {
  @apply h-5 w-5 rounded border-white/10 text-accent-primary focus:ring-accent-primary/20;
}

.codex-profiles-checkbox-label {
  @apply cursor-pointer select-none text-sm font-medium text-white;
}

.codex-profiles-checkbox-label--muted {
  @apply cursor-default text-white/70;
}

.codex-profiles-modal-footer {
  @apply flex justify-end gap-3 border-t border-white/5 px-6 py-4 glass-surface;
}
</style>
