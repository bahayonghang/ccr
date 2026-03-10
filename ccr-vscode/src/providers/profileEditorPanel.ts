/**
 * Profile Editor — WebviewPanel-based visual editor
 *
 * Section-card layout with grouped fields, platform badges, and auto-save.
 * Each field auto-saves on blur, with inline status indicators.
 */

import * as vscode from "vscode";
import type { ProfileInfo } from "../models/types";
import { writeProfileField, toggleProfileEnabled } from "../services/tomlReader";

/** Active panels keyed by "platform/profile" */
const activePanels = new Map<string, ProfileEditorPanel>();

export class ProfileEditorPanel {
  private readonly panel: vscode.WebviewPanel;
  private readonly panelKey: string;
  private profile: ProfileInfo;
  private readonly onDidSave: () => void;
  private disposed = false;

  /** Show an existing panel or create a new one */
  static createOrShow(
    extensionUri: vscode.Uri,
    profile: ProfileInfo,
    onDidSave: () => void,
  ): ProfileEditorPanel {
    const key = `${profile.platformName}/${profile.name}`;
    const existing = activePanels.get(key);
    if (existing && !existing.disposed) {
      existing.profile = profile;
      existing.panel.reveal();
      existing.sendProfileData();
      return existing;
    }

    return new ProfileEditorPanel(extensionUri, profile, onDidSave);
  }

  private constructor(
    extensionUri: vscode.Uri,
    profile: ProfileInfo,
    onDidSave: () => void,
  ) {
    this.profile = profile;
    this.onDidSave = onDidSave;
    this.panelKey = `${profile.platformName}/${profile.name}`;

    this.panel = vscode.window.createWebviewPanel(
      "ccrProfileEditor",
      `Edit: ${profile.name} (${profile.platformName})`,
      vscode.ViewColumn.One,
      {
        enableScripts: true,
        retainContextWhenHidden: true,
        localResourceRoots: [extensionUri],
      },
    );

    this.panel.iconPath = new vscode.ThemeIcon("notebook-edit");
    this.panel.webview.html = this.getHtml();

    this.panel.webview.onDidReceiveMessage((msg) => this.handleMessage(msg));
    this.panel.onDidDispose(() => {
      this.disposed = true;
      activePanels.delete(this.panelKey);
    });

    activePanels.set(this.panelKey, this);
  }

  private sendProfileData(): void {
    this.panel.webview.postMessage({ type: "profileData", profile: this.profile });
  }

  private handleMessage(msg: { type: string; field?: string; value?: unknown }): void {
    switch (msg.type) {
      case "ready":
        this.sendProfileData();
        break;

      case "saveField":
        this.saveField(msg.field!, msg.value as string);
        break;

      case "toggleEnabled":
        this.doToggleEnabled();
        break;
    }
  }

  private saveField(field: string, value: string): void {
    // Map camelCase field names back to snake_case TOML keys
    const fieldMap: Record<string, string> = {
      description: "description",
      baseUrl: "base_url",
      authToken: "auth_token",
      model: "model",
      smallFastModel: "small_fast_model",
      provider: "provider",
      providerType: "provider_type",
      account: "account",
      tags: "tags",
    };

    const tomlKey = fieldMap[field] ?? field;

    try {
      let writeValue: string | string[] | undefined;
      if (tomlKey === "tags") {
        writeValue = value
          ? value.split(",").map((t) => t.trim()).filter(Boolean)
          : undefined;
      } else {
        writeValue = value || undefined;
      }

      writeProfileField(this.profile.platformName, this.profile.name, tomlKey, writeValue);

      // Update local state
      (this.profile as unknown as Record<string, unknown>)[field] = tomlKey === "tags" ? writeValue : (value || undefined);

      this.panel.webview.postMessage({ type: "saveResult", field, success: true });
      this.onDidSave();
    } catch (err) {
      this.panel.webview.postMessage({
        type: "saveResult",
        field,
        success: false,
        error: String(err),
      });
    }
  }

  private doToggleEnabled(): void {
    try {
      const newState = toggleProfileEnabled(this.profile.platformName, this.profile.name);
      this.profile.enabled = newState;
      this.panel.webview.postMessage({ type: "saveResult", field: "enabled", success: true });
      this.sendProfileData();
      this.onDidSave();
    } catch (err) {
      this.panel.webview.postMessage({
        type: "saveResult",
        field: "enabled",
        success: false,
        error: String(err),
      });
    }
  }

  // ── HTML ──

  private getHtml(): string {
    const nonce = getNonce();

    return /* html */ `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta http-equiv="Content-Security-Policy"
        content="default-src 'none'; style-src 'nonce-${nonce}'; script-src 'nonce-${nonce}';" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Edit Profile</title>
  <style nonce="${nonce}">
    :root {
      --editor-max-width: 640px;
      --section-gap: 24px;
      --field-gap: 18px;
      --label-width: 150px;
      --input-radius: 5px;
      --card-bg: var(--vscode-editorWidget-background, var(--vscode-editor-background));
      --card-border: var(--vscode-widget-border, transparent);
      --subtle-border: var(--vscode-editorGroup-border, rgba(128,128,128,0.15));
      --success-color: var(--vscode-testing-iconPassed, #73c991);
      --error-color: var(--vscode-testing-iconFailed, #f48771);
      --focus-ring: var(--vscode-focusBorder);
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      font-family: var(--vscode-font-family);
      font-size: var(--vscode-font-size);
      color: var(--vscode-foreground);
      background: var(--vscode-editor-background);
      padding: 24px 32px;
      max-width: var(--editor-max-width);
    }

    /* ── Header Card ── */
    .header-card {
      background: var(--card-bg);
      border: 1px solid var(--card-border);
      border-radius: 8px;
      padding: 20px 24px;
      margin-bottom: var(--section-gap);
    }
    .header-top {
      display: flex;
      align-items: center;
      gap: 12px;
      margin-bottom: 16px;
    }
    .header-top h1 {
      font-size: 1.4em;
      font-weight: 600;
      color: var(--vscode-foreground);
    }
    .platform-badge {
      display: inline-flex;
      align-items: center;
      padding: 2px 10px;
      border-radius: 12px;
      font-size: 0.72em;
      font-weight: 600;
      letter-spacing: 0.03em;
      color: #fff;
      background: var(--badge-color, #888);
      white-space: nowrap;
    }
    .header-controls {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding-top: 14px;
      border-top: 1px solid var(--subtle-border);
    }
    .autosave-indicator {
      font-size: 0.8em;
      color: var(--vscode-descriptionForeground);
      transition: color 0.3s;
    }
    .autosave-indicator.saved { color: var(--success-color); }

    /* ── Toggle Switch ── */
    .toggle-wrap {
      display: flex;
      align-items: center;
      gap: 10px;
    }
    .toggle {
      position: relative;
      width: 44px;
      height: 22px;
      cursor: pointer;
    }
    .toggle input { display: none; }
    .toggle-track {
      position: absolute;
      inset: 0;
      background: var(--vscode-input-background);
      border: 1px solid var(--vscode-input-border, transparent);
      border-radius: 11px;
      transition: background 0.25s cubic-bezier(0.4, 0, 0.2, 1);
    }
    .toggle input:checked + .toggle-track {
      background: var(--vscode-button-background);
    }
    .toggle-thumb {
      position: absolute;
      top: 3px;
      left: 3px;
      width: 16px;
      height: 16px;
      background: var(--vscode-button-foreground, #fff);
      border-radius: 50%;
      box-shadow: 0 1px 3px rgba(0,0,0,0.2);
      transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
      pointer-events: none;
    }
    .toggle input:checked ~ .toggle-thumb {
      transform: translateX(22px);
    }
    .toggle-label {
      color: var(--vscode-foreground);
      font-size: 0.85em;
      font-weight: 500;
    }

    /* ── Section Card ── */
    .section-card {
      background: var(--card-bg);
      border: 1px solid var(--card-border);
      border-radius: 8px;
      margin-bottom: var(--section-gap);
      overflow: hidden;
    }
    .section-header {
      padding: 10px 20px;
      border-bottom: 1px solid var(--subtle-border);
      font-size: 0.72em;
      font-weight: 600;
      letter-spacing: 0.08em;
      text-transform: uppercase;
      color: var(--vscode-descriptionForeground);
      user-select: none;
    }
    .section-body {
      padding: 18px 20px;
    }

    /* ── Field Row ── */
    .field-row {
      display: grid;
      grid-template-columns: var(--label-width) 1fr;
      align-items: start;
      gap: 12px;
      margin-bottom: var(--field-gap);
    }
    .field-row:last-child { margin-bottom: 0; }
    .field-label-wrap { padding-top: 4px; }
    .field-label {
      font-weight: 500;
      color: var(--vscode-foreground);
      user-select: none;
    }
    .field-label .required {
      color: var(--error-color);
      font-weight: 700;
      margin-left: 2px;
    }
    .field-hint {
      display: block;
      font-weight: 400;
      font-size: 0.85em;
      color: var(--vscode-descriptionForeground);
      margin-top: 2px;
    }
    .field-input-wrap { position: relative; }
    input[type="text"],
    input[type="password"] {
      width: 100%;
      padding: 5px 32px 5px 8px;
      border: 1px solid var(--vscode-input-border, transparent);
      border-radius: var(--input-radius);
      background: var(--vscode-input-background);
      color: var(--vscode-input-foreground);
      font-family: var(--vscode-editor-font-family, monospace);
      font-size: var(--vscode-font-size);
      outline: none;
      transition: border-color 0.15s, box-shadow 0.15s;
    }
    input:hover {
      border-color: var(--vscode-input-border, rgba(128,128,128,0.4));
    }
    input:focus {
      border-color: var(--focus-ring);
      box-shadow: 0 0 0 1px var(--focus-ring);
    }
    .field-input-wrap.has-toggle input {
      padding-right: 72px;
    }

    /* ── Status Icon ── */
    .status-icon {
      position: absolute;
      right: 8px;
      top: 50%;
      transform: translateY(-50%) scale(0.8);
      font-size: 14px;
      opacity: 0;
      transition: opacity 0.3s, transform 0.3s;
    }
    .status-icon.visible {
      opacity: 1;
      transform: translateY(-50%) scale(1);
    }
    .status-icon.success { color: var(--success-color); }
    .status-icon.error { color: var(--error-color); }

    /* ── Password Toggle ── */
    .pwd-toggle {
      position: absolute;
      right: 28px;
      top: 50%;
      transform: translateY(-50%);
      background: none;
      border: none;
      color: var(--vscode-descriptionForeground);
      cursor: pointer;
      font-size: 0.78em;
      padding: 2px 6px;
      border-radius: 3px;
      transition: color 0.15s, background 0.15s;
      font-family: var(--vscode-font-family);
    }
    .pwd-toggle:hover {
      color: var(--vscode-foreground);
      background: var(--vscode-toolbar-hoverBackground, rgba(128,128,128,0.1));
    }
  </style>
</head>
<body>

  <!-- Header Card -->
  <div class="header-card">
    <div class="header-top">
      <h1 id="title">Edit Profile</h1>
      <span class="platform-badge" id="platform-badge"></span>
    </div>
    <div class="header-controls">
      <div class="toggle-wrap">
        <label class="toggle">
          <input type="checkbox" id="field-enabled" />
          <div class="toggle-track"></div>
          <div class="toggle-thumb"></div>
        </label>
        <span class="toggle-label" id="enabled-label">Enabled</span>
      </div>
      <span class="autosave-indicator" id="autosave-indicator">Auto-save</span>
    </div>
  </div>

  <!-- Section Cards (fields injected by JS) -->
  <div class="section-card">
    <div class="section-header">Connection</div>
    <div class="section-body" id="section-connection"></div>
  </div>
  <div class="section-card">
    <div class="section-header">Model</div>
    <div class="section-body" id="section-model"></div>
  </div>
  <div class="section-card">
    <div class="section-header">Identity</div>
    <div class="section-body" id="section-identity"></div>
  </div>
  <div class="section-card">
    <div class="section-header">Metadata</div>
    <div class="section-body" id="section-metadata"></div>
  </div>

  <script nonce="${nonce}">
    const vscode = acquireVsCodeApi();

    const PLATFORM_COLORS = {
      claude: '#FF6B35',
      codex: '#10B981',
      gemini: '#4285F4',
      qwen: '#00B5E2',
      iflow: '#FAAD14',
      droid: '#A78BFA',
    };

    const FIELD_GROUPS = {
      connection: [
        { key: 'baseUrl',   label: 'Base URL',   hint: 'API endpoint URL',  important: true },
        { key: 'authToken', label: 'Auth Token',  hint: 'API key or token',  important: true, secret: true },
      ],
      model: [
        { key: 'model',          label: 'Model',            hint: 'Default model name',               important: true },
        { key: 'smallFastModel', label: 'Small/Fast Model', hint: 'Lightweight model for quick tasks' },
      ],
      identity: [
        { key: 'provider',     label: 'Provider',      hint: 'Provider identifier' },
        { key: 'providerType', label: 'Provider Type', hint: 'Provider backend type' },
        { key: 'account',      label: 'Account',       hint: 'Account or organization name' },
      ],
      metadata: [
        { key: 'description', label: 'Description', hint: 'Human-readable profile description' },
        { key: 'tags',        label: 'Tags',        hint: 'Comma-separated tags' },
      ],
    };

    const fieldElements = {};
    const statusTimers = {};

    // Build field DOM for each section
    for (const [groupName, fields] of Object.entries(FIELD_GROUPS)) {
      const container = document.getElementById('section-' + groupName);
      if (!container) continue;

      for (const f of fields) {
        const row = document.createElement('div');
        row.className = 'field-row';

        const inputType = f.secret ? 'password' : 'text';
        const requiredMark = f.important ? '<span class="required">*</span>' : '';
        const wrapClass = f.secret ? 'field-input-wrap has-toggle' : 'field-input-wrap';
        let toggleBtn = '';
        if (f.secret) {
          toggleBtn = '<button class="pwd-toggle" data-toggle-for="' + f.key + '" title="Show/Hide">Show</button>';
        }

        row.innerHTML =
          '<div class="field-label-wrap">' +
            '<div class="field-label">' + f.label + requiredMark + '</div>' +
            '<span class="field-hint">' + f.hint + '</span>' +
          '</div>' +
          '<div class="' + wrapClass + '">' +
            '<input type="' + inputType + '" id="field-' + f.key + '" data-field="' + f.key + '" spellcheck="false" autocomplete="off" />' +
            toggleBtn +
            '<span class="status-icon" id="status-' + f.key + '"></span>' +
          '</div>';

        container.appendChild(row);
        fieldElements[f.key] = document.getElementById('field-' + f.key);
      }
    }

    // Auto-save on blur (only for input/textarea, not buttons)
    document.addEventListener('focusout', (e) => {
      const input = e.target;
      if (!input || !input.dataset || !input.dataset.field) return;
      if (input.tagName !== 'INPUT' && input.tagName !== 'TEXTAREA') return;
      const field = input.dataset.field;
      vscode.postMessage({ type: 'saveField', field, value: input.value });
    });

    // Password show/hide
    document.addEventListener('click', (e) => {
      if (!e.target.classList.contains('pwd-toggle')) return;
      const field = e.target.dataset.toggleFor;
      const input = document.getElementById('field-' + field);
      if (input.type === 'password') {
        input.type = 'text';
        e.target.textContent = 'Hide';
      } else {
        input.type = 'password';
        e.target.textContent = 'Show';
      }
    });

    // Enabled toggle
    const enabledCheckbox = document.getElementById('field-enabled');
    const enabledLabel = document.getElementById('enabled-label');
    enabledCheckbox.addEventListener('change', () => {
      vscode.postMessage({ type: 'toggleEnabled' });
    });

    // Auto-save flash indicator
    const autosaveEl = document.getElementById('autosave-indicator');
    let autosaveTimer = null;
    function flashAutosave() {
      autosaveEl.textContent = 'Saved \\u2713';
      autosaveEl.classList.add('saved');
      if (autosaveTimer) clearTimeout(autosaveTimer);
      autosaveTimer = setTimeout(() => {
        autosaveEl.textContent = 'Auto-save';
        autosaveEl.classList.remove('saved');
      }, 2000);
    }

    // Receive messages from extension
    window.addEventListener('message', (event) => {
      const msg = event.data;

      if (msg.type === 'profileData') {
        const p = msg.profile;
        document.getElementById('title').textContent = p.name;

        // Platform badge with color
        const badge = document.getElementById('platform-badge');
        badge.textContent = p.platformName;
        badge.style.setProperty('--badge-color', PLATFORM_COLORS[p.platformName] || '#888');

        enabledCheckbox.checked = p.enabled;
        enabledLabel.textContent = p.enabled ? 'Enabled' : 'Disabled';

        // Populate all fields across groups
        const allFields = Object.values(FIELD_GROUPS).flat();
        for (const f of allFields) {
          const val = p[f.key];
          const el = fieldElements[f.key];
          if (!el) continue;
          if (f.key === 'tags' && Array.isArray(val)) {
            el.value = val.join(', ');
          } else {
            el.value = val ?? '';
          }
        }
      }

      if (msg.type === 'saveResult') {
        if (msg.field === 'enabled') {
          enabledLabel.textContent = enabledCheckbox.checked ? 'Enabled' : 'Disabled';
          if (msg.success) flashAutosave();
          return;
        }

        const icon = document.getElementById('status-' + msg.field);
        if (icon) {
          icon.textContent = msg.success ? '\\u2713' : '\\u2717';
          icon.className = 'status-icon visible ' + (msg.success ? 'success' : 'error');

          if (statusTimers[msg.field]) clearTimeout(statusTimers[msg.field]);
          statusTimers[msg.field] = setTimeout(() => {
            icon.className = 'status-icon';
          }, 2000);
        }

        if (msg.success) flashAutosave();
      }
    });

    // Signal ready
    vscode.postMessage({ type: 'ready' });
  </script>
</body>
</html>`;
  }
}

function getNonce(): string {
  const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  let result = "";
  for (let i = 0; i < 32; i++) {
    result += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return result;
}
