import { randomBytes } from "crypto";
import * as vscode from "vscode";
import { EDITABLE_FIELDS, PROFILE_EDITABLE_FIELDS_BY_PLATFORM, getEditableProfileFields } from "../models/types";
import type { ProfileCreationPlatform, ProfileEditorDraft, ProfileEditorMode, ProfileInfo } from "../models/types";
import { getPanelKey, getPanelTitle, normalizeFieldValue } from "./profileEditorPanel.helpers";
import {
  execPlatformProfileDisable,
  execPlatformProfileEnable,
  execPlatformProfileSetField,
} from "../services/ccrCli";

const TOML_KEY_BY_EDITABLE_FIELD = Object.fromEntries(
  EDITABLE_FIELDS.map((item) => [item.key, item.tomlKey]),
) as Record<string, string>;

type EditorMessage =
  | { type: "ready" | "toggleEnabled" | "cancelCreate" }
  | { type: "saveField" | "copyField"; field: string; value: unknown }
  | { type: "createProfile"; draft: ProfileEditorDraft };

export class ProfileEditorPanel {
  private static readonly activePanels = new Map<string, ProfileEditorPanel>();

  private readonly panel: vscode.WebviewPanel;
  private readonly panelKey: string;
  private readonly mode: ProfileEditorMode;
  private profile: ProfileInfo | ProfileEditorDraft;
  private readonly onDidSave?: () => void;
  private readonly onDidCreate?: (draft: ProfileEditorDraft) => Promise<void>;
  private disposed = false;

  static createOrShow(
    extensionUri: vscode.Uri,
    profile: ProfileInfo,
    onDidSave: () => void,
  ): ProfileEditorPanel {
    const key = getPanelKey("edit", profile);
    const existing = ProfileEditorPanel.activePanels.get(key);
    if (existing && !existing.disposed) {
      existing.profile = profile;
      existing.panel.reveal();
      existing.sendProfileData();
      return existing;
    }

    return new ProfileEditorPanel(extensionUri, "edit", profile, onDidSave);
  }

  static createForNewProfile(
    extensionUri: vscode.Uri,
    platformName: ProfileCreationPlatform,
    onDidCreate: (draft: ProfileEditorDraft) => Promise<void>,
  ): ProfileEditorPanel {
    const key = getPanelKey("create", { name: "", platformName, enabled: true });
    const existing = ProfileEditorPanel.activePanels.get(key);
    if (existing && !existing.disposed) {
      existing.panel.reveal();
      existing.sendProfileData();
      return existing;
    }

    const draft: ProfileEditorDraft = {
      name: "",
      platformName,
      enabled: true,
    };

    return new ProfileEditorPanel(extensionUri, "create", draft, undefined, onDidCreate);
  }

  static disposeAll(): void {
    for (const panel of ProfileEditorPanel.activePanels.values()) {
      panel.panel.dispose();
    }
    ProfileEditorPanel.activePanels.clear();
  }

  private constructor(
    extensionUri: vscode.Uri,
    mode: ProfileEditorMode,
    profile: ProfileInfo | ProfileEditorDraft,
    onDidSave?: () => void,
    onDidCreate?: (draft: ProfileEditorDraft) => Promise<void>,
  ) {
    this.mode = mode;
    this.profile = profile;
    this.onDidSave = onDidSave;
    this.onDidCreate = onDidCreate;
    this.panelKey = getPanelKey(mode, profile);

    this.panel = vscode.window.createWebviewPanel(
      "ccrProfileEditor",
      getPanelTitle(mode, profile),
      vscode.ViewColumn.One,
      {
        enableScripts: true,
        localResourceRoots: [extensionUri],
      },
    );

    this.panel.iconPath = new vscode.ThemeIcon(mode === "create" ? "add" : "notebook-edit");
    this.panel.webview.html = this.getHtml();

    this.panel.webview.onDidReceiveMessage((msg) => this.handleMessage(msg as EditorMessage));
    this.panel.onDidDispose(() => {
      this.disposed = true;
      ProfileEditorPanel.activePanels.delete(this.panelKey);
    });

    ProfileEditorPanel.activePanels.set(this.panelKey, this);
  }

  private sendProfileData(): void {
    this.panel.webview.postMessage({
      type: "profileData",
      mode: this.mode,
      profile: { ...this.profile },
    });
  }

  private handleMessage(msg: EditorMessage): void {
    switch (msg.type) {
      case "ready":
        this.sendProfileData();
        break;
      case "saveField":
        if (this.mode === "edit" && typeof msg.field === "string") {
          void this.saveField(msg.field, typeof msg.value === "string" ? msg.value : "");
        }
        break;
      case "toggleEnabled":
        if (this.mode === "edit") {
          void this.doToggleEnabled();
        }
        break;
      case "copyField":
        if (typeof msg.field === "string" && typeof msg.value === "string") {
          void this.copyField(msg.field, msg.value);
        }
        break;
      case "createProfile":
        if (this.mode === "create") {
          void this.doCreateProfile(msg.draft);
        }
        break;
      case "cancelCreate":
        if (this.mode === "create") {
          this.panel.dispose();
        }
        break;
    }
  }

  private async copyField(field: string, value: string): Promise<void> {
    try {
      await vscode.env.clipboard.writeText(value);
      this.panel.webview.postMessage({ type: "copyResult", field, success: true });
    } catch (err) {
      this.panel.webview.postMessage({
        type: "copyResult",
        field,
        success: false,
        error: String(err),
      });
    }
  }

  private async saveField(field: string, value: string): Promise<void> {
    const profile = this.profile as ProfileInfo;
    const allowedFields = new Set(getEditableProfileFields(profile.platformName));
    const tomlKey = TOML_KEY_BY_EDITABLE_FIELD[field] ?? field;

    if (!allowedFields.has(field)) {
      this.panel.webview.postMessage({
        type: "saveResult",
        field,
        success: false,
        error: `Field '${field}' is not editable for ${profile.platformName}.`,
      });
      return;
    }

    try {
      const writeValue = normalizeFieldValue(tomlKey, value);

      const result = await execPlatformProfileSetField(
        profile.platformName,
        profile.name,
        tomlKey,
        writeValue,
      );
      if (!result.success) {
        throw new Error(result.stderr || "Unknown error");
      }
      (profile as unknown as Record<string, unknown>)[field] = tomlKey === "tags"
        ? writeValue
        : (value || undefined);

      this.panel.webview.postMessage({ type: "saveResult", field, success: true });
      this.onDidSave?.();
    } catch (err) {
      this.panel.webview.postMessage({
        type: "saveResult",
        field,
        success: false,
        error: String(err),
      });
    }
  }

  private async doToggleEnabled(): Promise<void> {
    const profile = this.profile as ProfileInfo;

    try {
      const result = profile.enabled
        ? await execPlatformProfileDisable(profile.platformName, profile.name, true)
        : await execPlatformProfileEnable(profile.platformName, profile.name);
      if (!result.success) {
        throw new Error(result.stderr || "Unknown error");
      }
      const newState = result.data?.enabled ?? !profile.enabled;
      profile.enabled = newState;
      this.panel.webview.postMessage({ type: "saveResult", field: "enabled", success: true });
      this.sendProfileData();
      this.onDidSave?.();
    } catch (err) {
      this.panel.webview.postMessage({
        type: "saveResult",
        field: "enabled",
        success: false,
        error: String(err),
      });
    }
  }

  private async doCreateProfile(draft: ProfileEditorDraft): Promise<void> {
    try {
      await this.onDidCreate?.(draft);
      this.panel.webview.postMessage({ type: "createResult", success: true });
      this.panel.dispose();
    } catch (err) {
      this.panel.webview.postMessage({
        type: "createResult",
        success: false,
        error: String(err),
      });
    }
  }

  private getHtml(): string {
    const nonce = getNonce();
    const serializedAllowedEditableFields = JSON.stringify(PROFILE_EDITABLE_FIELDS_BY_PLATFORM);

    return /* html */ `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta
    http-equiv="Content-Security-Policy"
    content="default-src 'none'; style-src 'nonce-${nonce}'; script-src 'nonce-${nonce}';"
  />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Profile Editor</title>
  <style nonce="${nonce}">
    :root {
      --editor-max-width: 820px;
      --page-padding: 24px;
      --section-gap: 18px;
      --field-gap: 16px;
      --label-width: 180px;
      --panel-radius: 12px;
      --control-radius: 8px;
      --platform-color: var(--vscode-textLink-foreground, #8b5cf6);
      --platform-soft: color-mix(in srgb, var(--platform-color) 8%, transparent);
      --platform-border: color-mix(in srgb, var(--platform-color) 20%, var(--vscode-widget-border, transparent));
      --panel-bg: var(--vscode-editorWidget-background, var(--vscode-editor-background));
      --panel-muted-bg: color-mix(in srgb, var(--panel-bg) 92%, var(--vscode-sideBar-background, transparent));
      --panel-border: var(--vscode-widget-border, rgba(128, 128, 128, 0.24));
      --panel-border-soft: color-mix(in srgb, var(--panel-border) 72%, transparent);
      --text-strong: var(--vscode-foreground);
      --text-muted: var(--vscode-descriptionForeground);
      --input-bg: var(--vscode-input-background);
      --input-fg: var(--vscode-input-foreground);
      --input-border: var(--vscode-input-border, rgba(128, 128, 128, 0.28));
      --focus-ring: var(--vscode-focusBorder, var(--platform-color));
      --success-color: var(--vscode-testing-iconPassed, #73c991);
      --error-color: var(--vscode-errorForeground, var(--vscode-testing-iconFailed, #f48771));
      --button-fg: var(--vscode-button-foreground, #ffffff);
      --button-bg: var(--vscode-button-background, #0e639c);
      --button-bg-hover: var(--vscode-button-hoverBackground, #1177bb);
      --button-secondary-bg: color-mix(in srgb, var(--platform-color) 8%, var(--panel-bg));
      --button-secondary-border: color-mix(in srgb, var(--platform-color) 18%, var(--panel-border));
    }

    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      min-height: 100vh;
      font-family: var(--vscode-font-family);
      font-size: var(--vscode-font-size);
      color: var(--text-strong);
      background: var(--vscode-editor-background);
      padding: var(--page-padding);
      line-height: 1.5;
    }
    .page { max-width: var(--editor-max-width); margin: 0 auto; }
    #loading {
      display: grid; place-items: center; min-height: 220px; border-radius: var(--panel-radius);
      border: 1px solid var(--panel-border-soft);
      background: var(--panel-muted-bg);
      color: var(--text-muted);
    }
    #loading.hidden { display: none; }
    #editor-content { display: none; }
    .header-card {
      padding: 20px; border-radius: var(--panel-radius);
      border: 1px solid var(--platform-border);
      background: var(--panel-bg);
      margin-bottom: var(--section-gap);
    }
    .header-top { display: flex; justify-content: space-between; gap: 20px; margin-bottom: 18px; }
    .eyebrow {
      display: inline-flex; align-items: center; gap: 8px; padding: 4px 8px; border-radius: 999px;
      background: var(--platform-soft);
      color: color-mix(in srgb, var(--platform-color) 72%, var(--text-strong));
      font-size: 0.72em; font-weight: 700; letter-spacing: 0.08em; text-transform: uppercase; margin-bottom: 10px;
    }
    .title-row { display: flex; align-items: center; flex-wrap: wrap; gap: 10px; }
    .title-row h1 {
      font-size: 1.7em; font-weight: 700; line-height: 1.1; color: var(--text-strong); letter-spacing: -0.01em;
    }
    .platform-badge {
      display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px; border-radius: 999px;
      background: var(--platform-soft);
      border: 1px solid var(--platform-border);
      color: var(--text-strong); font-size: 0.8em; font-weight: 600; letter-spacing: 0.04em; text-transform: uppercase;
    }
    .subtitle { margin-top: 8px; color: var(--text-muted); max-width: 620px; }
    .header-meta { min-width: 220px; display: flex; flex-direction: column; align-items: flex-end; gap: 8px; }
    .autosave-indicator {
      display: inline-flex; align-items: center; justify-content: center; min-width: 120px; padding: 6px 10px;
      border-radius: 999px; border: 1px solid var(--panel-border-soft); background: var(--panel-muted-bg);
      color: var(--text-muted); font-size: 0.78em; font-weight: 600; letter-spacing: 0.04em; text-transform: uppercase;
      transition: border-color 0.2s ease, color 0.2s ease, background 0.2s ease;
    }
    .autosave-indicator.saved {
      background: color-mix(in srgb, var(--success-color) 14%, var(--panel-bg));
      border-color: color-mix(in srgb, var(--success-color) 30%, var(--panel-border));
      color: var(--success-color);
    }
    .header-controls {
      display: flex; justify-content: space-between; gap: 12px; align-items: center;
      padding-top: 14px; border-top: 1px solid var(--panel-border-soft);
    }
    .header-note { color: var(--text-muted); font-size: 0.88em; }
    .toggle-wrap { display: inline-flex; align-items: center; gap: 12px; }
    .toggle { position: relative; display: inline-flex; align-items: center; width: 52px; height: 28px; }
    .sr-only {
      position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden;
      clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0;
    }
    .toggle-input:focus-visible + .toggle-track {
      outline: 2px solid var(--focus-ring);
      outline-offset: 2px;
    }
    .toggle-track {
      position: absolute; inset: 0; border-radius: 999px;
      border: 1px solid var(--input-border);
      background: color-mix(in srgb, var(--platform-soft) 70%, var(--input-bg));
      transition: background 0.2s ease, border-color 0.2s ease;
    }
    .toggle-thumb {
      position: absolute; top: 3px; left: 3px; width: 20px; height: 20px; border-radius: 50%; background: var(--button-fg);
      transition: transform 0.2s ease; pointer-events: none;
    }
    .toggle-input:checked + .toggle-track { background: color-mix(in srgb, var(--platform-color) 18%, var(--input-bg)); border-color: var(--platform-border); }
    .toggle-input:checked + .toggle-track + .toggle-thumb { transform: translateX(24px); }
    .toggle-label { font-weight: 600; color: var(--text-strong); }
    .section-card {
      display: none; margin-bottom: var(--section-gap); border-radius: var(--panel-radius); border: 1px solid var(--panel-border-soft);
      background: var(--panel-bg);
      overflow: hidden;
    }
    .section-header {
      padding: 14px 18px 12px; border-bottom: 1px solid var(--panel-border-soft);
      background: var(--panel-muted-bg);
    }
    .section-title { color: var(--text-strong); font-size: 0.88em; font-weight: 700; letter-spacing: 0.05em; text-transform: uppercase; }
    .section-subtitle { margin-top: 4px; color: var(--text-muted); font-size: 0.88em; line-height: 1.5; }
    .section-body { padding: 18px; }
    .field-row {
      display: grid; grid-template-columns: minmax(160px, var(--label-width)) minmax(0, 1fr);
      gap: 14px; align-items: start; margin-bottom: var(--field-gap);
    }
    .field-row:last-child { margin-bottom: 0; }
    .field-label-wrap { padding-top: 8px; }
    .field-label { display: inline-flex; align-items: center; gap: 4px; font-weight: 700; color: var(--text-strong); letter-spacing: 0.01em; }
    .required { color: color-mix(in srgb, var(--error-color) 82%, #ffffff); margin-left: 0; }
    .field-hint { display: block; margin-top: 6px; color: var(--text-muted); font-size: 0.88em; line-height: 1.45; }
    .field-input-wrap { display: grid; gap: 8px; min-width: 0; }
    .input-shell {
      display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: 10px; align-items: center; padding: 10px;
      border-radius: var(--control-radius); border: 1px solid var(--input-border);
      background: var(--input-bg);
      transition: border-color 0.15s ease, box-shadow 0.15s ease;
    }
    .input-shell:focus-within {
      border-color: var(--focus-ring);
      box-shadow: 0 0 0 1px var(--focus-ring);
    }
    .field-input {
      width: 100%; min-width: 0; border: none; background: transparent; color: var(--input-fg);
      font-family: var(--vscode-editor-font-family, monospace); font-size: var(--vscode-font-size); line-height: 1.45; outline: none;
    }
    .field-input::placeholder { color: color-mix(in srgb, var(--text-muted) 88%, transparent); }
    .field-actions { display: inline-flex; align-items: center; gap: 8px; flex-wrap: wrap; justify-content: flex-end; }
    .control-btn {
      min-width: 58px; border: 1px solid var(--button-secondary-border); border-radius: 999px; background: var(--button-secondary-bg);
      color: var(--text-strong); padding: 6px 10px; font-size: 0.78em;
      font-weight: 700; letter-spacing: 0.03em; cursor: pointer; transition: background 0.15s ease, border-color 0.15s ease, color 0.15s ease;
      font-family: var(--vscode-font-family);
    }
    .control-btn:hover:not(:disabled) {
      background: color-mix(in srgb, var(--platform-color) 12%, var(--panel-bg));
      border-color: color-mix(in srgb, var(--platform-color) 26%, var(--panel-border));
    }
    .control-btn:focus-visible { outline: 2px solid var(--focus-ring); outline-offset: 2px; }
    .control-btn:disabled { cursor: not-allowed; opacity: 0.45; }
    .submit-actions { display: none; gap: 12px; justify-content: flex-end; margin-top: 22px; }
    .submit-actions.visible { display: flex; }
    .submit-btn {
      border: 1px solid transparent; border-radius: 999px; padding: 10px 18px; font-weight: 700; cursor: pointer;
      transition: background 0.15s ease, border-color 0.15s ease, opacity 0.15s ease;
      font-family: var(--vscode-font-family);
    }
    .submit-btn.primary { color: var(--button-fg); background: var(--button-bg); }
    .submit-btn.primary:hover:not(:disabled) { background: var(--button-bg-hover); }
    .submit-btn.secondary { color: var(--text-strong); background: var(--button-secondary-bg); border-color: var(--button-secondary-border); }
    .submit-btn:focus-visible { outline: 2px solid var(--focus-ring); outline-offset: 2px; }
    .submit-btn:disabled { opacity: 0.55; cursor: not-allowed; }
    .field-feedback {
      display: inline-flex; align-items: center; justify-content: center; min-width: 68px; padding: 6px 10px; border-radius: 999px;
      font-size: 0.76em; font-weight: 700; letter-spacing: 0.04em; text-transform: uppercase; opacity: 0; transform: translateY(4px);
      transition: opacity 0.2s ease, transform 0.2s ease; pointer-events: none;
    }
    .field-feedback.visible { opacity: 1; transform: translateY(0); }
    .field-feedback.success {
      color: var(--success-color); background: color-mix(in srgb, var(--success-color) 12%, var(--panel-bg));
      border: 1px solid color-mix(in srgb, var(--success-color) 24%, var(--panel-border));
    }
    .field-feedback.error {
      color: var(--error-color); background: color-mix(in srgb, var(--error-color) 10%, var(--panel-bg));
      border: 1px solid color-mix(in srgb, var(--error-color) 22%, var(--panel-border));
    }
    .live-region {
      position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden;
      clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0;
    }
    @media (max-width: 900px) {
      body { padding: 20px; }
      .header-top { flex-direction: column; }
      .header-meta { align-items: flex-start; }
      .field-row { grid-template-columns: 1fr; gap: 10px; }
      .field-label-wrap { padding-top: 0; }
    }
    @media (max-width: 640px) {
      body { padding: 16px; }
      .header-card, .section-body { padding-left: 16px; padding-right: 16px; }
      .section-header { padding-left: 16px; padding-right: 16px; }
      .input-shell { grid-template-columns: 1fr; }
      .field-actions { justify-content: flex-start; }
      .submit-actions { flex-direction: column-reverse; }
    }
  </style>
</head>
<body>
  <div class="page">
    <div id="loading">Loading profile editor...</div>
    <div id="editor-content">
      <div id="editor-status" class="live-region" role="status" aria-live="polite" aria-atomic="true"></div>
      <section class="header-card">
        <div class="header-top">
          <div>
            <div class="eyebrow">Configuration editor</div>
            <div class="title-row">
              <h1 id="title">Profile</h1>
              <span class="platform-badge" id="platform-badge"></span>
            </div>
            <p class="subtitle" id="subtitle">Review and update connection, model, and identity settings for this profile.</p>
          </div>
          <div class="header-meta">
            <span class="autosave-indicator" id="autosave-indicator">Auto-save</span>
            <span class="header-note" id="mode-note">Changes save when you leave a field.</span>
          </div>
        </div>
        <div class="header-controls">
          <div class="toggle-wrap">
            <label class="toggle" for="field-enabled">
              <input type="checkbox" class="toggle-input sr-only" id="field-enabled" />
              <span class="toggle-track" aria-hidden="true"></span>
              <span class="toggle-thumb" aria-hidden="true"></span>
            </label>
            <span class="toggle-label" id="enabled-label">Enabled</span>
          </div>
          <span class="header-note" id="header-note">Edits apply only to this saved profile.</span>
        </div>
      </section>
      <section class="section-card" data-editor-section>
        <div class="section-header">
          <div class="section-title">Connection</div>
          <div class="section-subtitle">Core access settings used when CCR applies this profile.</div>
        </div>
        <div class="section-body" id="section-connection"></div>
      </section>
      <section class="section-card" data-editor-section>
        <div class="section-header">
          <div class="section-title">Optional models</div>
          <div class="section-subtitle">Use explicit overrides only when this profile should bypass inherited defaults.</div>
        </div>
        <div class="section-body" id="section-model"></div>
      </section>
      <section class="section-card" data-editor-section>
        <div class="section-header">
          <div class="section-title">Identity</div>
          <div class="section-subtitle">Provider and account metadata that help distinguish this endpoint.</div>
        </div>
        <div class="section-body" id="section-identity"></div>
      </section>
      <section class="section-card" data-editor-section>
        <div class="section-header">
          <div class="section-title">Metadata</div>
          <div class="section-subtitle">Descriptions and tags that make the profile easier to recognize later.</div>
        </div>
        <div class="section-body" id="section-metadata"></div>
      </section>
      <div class="submit-actions" id="submit-actions">
        <button type="button" class="submit-btn secondary" id="cancel-create">Cancel</button>
        <button type="button" class="submit-btn primary" id="submit-create">Create Profile</button>
      </div>
    </div>
  </div>
  <script nonce="${nonce}">
    const vscode = acquireVsCodeApi();
    const PLATFORM_ACCENTS = {
      claude: { color: '#ff8a3d', soft: 'rgba(255, 138, 61, 0.16)', glow: 'rgba(255, 138, 61, 0.3)', icon: '🤖' },
      codex: { color: '#22c55e', soft: 'rgba(34, 197, 94, 0.16)', glow: 'rgba(34, 197, 94, 0.28)', icon: '💻' },
      default: { color: '#8b5cf6', soft: 'rgba(139, 92, 246, 0.16)', glow: 'rgba(139, 92, 246, 0.3)', icon: '🧩' },
    };
    const FIELD_GROUPS = {
      connection: [
        { key: 'baseUrl', label: 'Base URL', hint: 'API endpoint URL', required: true, placeholder: 'https://api.example.com/v1', actions: ['copy'] },
        { key: 'authToken', label: 'Auth Token', hint: 'API key or token', required: true, secret: true, placeholder: 'Paste the full credential here', actions: ['toggle', 'copy'] },
      ],
      model: [
        { key: 'model', label: 'Model', hint: 'Optional default model override. Leave blank to use the platform default.', placeholder: 'Optional' },
        { key: 'smallFastModel', label: 'Small/Fast Model', hint: 'Optional lightweight model for quick tasks.', placeholder: 'Optional' },
      ],
      identity: [
        { key: 'provider', label: 'Provider', hint: 'Provider identifier' },
        { key: 'providerType', label: 'Provider Type', hint: 'Provider backend type' },
        { key: 'account', label: 'Account', hint: 'Account or organization name' },
      ],
      metadata: [
        { key: 'name', label: 'Profile Name', hint: 'Unique profile identifier', required: true, placeholder: 'new-profile', createOnly: true },
        { key: 'description', label: 'Description', hint: 'Human-readable profile description' },
        { key: 'tags', label: 'Tags', hint: 'Comma-separated tags', placeholder: 'free, backup, relay' },
      ],
    };
    const fieldLabels = Object.values(FIELD_GROUPS).flat().reduce((map, field) => {
      map[field.key] = field.label;
      return map;
    }, {});
    const SECRET_FIELD_KEYS = Object.values(FIELD_GROUPS)
      .flat()
      .filter((field) => field.secret)
      .map((field) => field.key);

    function sanitizeProfile(profile) {
      if (!profile) return profile;
      const sanitized = { ...profile };
      SECRET_FIELD_KEYS.forEach((key) => {
        if (key in sanitized) sanitized[key] = undefined;
      });
      return sanitized;
    }
    const allowedEditableFields = ${serializedAllowedEditableFields};
    const fieldElements = {};
    const fieldState = {};
    const copyButtons = {};
    const toggleButtons = {};
    const statusTimers = {};
    const copyTimers = {};
    let currentProfile = null;
    let currentMode = 'edit';

    function cloneProfile(profile) { return JSON.parse(JSON.stringify(profile)); }
    function getAccent(platformName) { return PLATFORM_ACCENTS[platformName] || PLATFORM_ACCENTS.default; }
    function setStatusMessage(message) {
      const statusEl = document.getElementById('editor-status');
      if (statusEl) {
        statusEl.textContent = message;
      }
    }
    function setPlatformAccent(platformName) {
      const accent = getAccent(platformName);
      const root = document.documentElement.style;
      root.setProperty('--platform-color', accent.color);
      root.setProperty('--platform-soft', accent.soft);
      root.setProperty('--platform-border', 'color-mix(in srgb, ' + accent.color + ' 20%, var(--vscode-widget-border, transparent))');
      return accent;
    }
    function isFieldVisible(platformName, fieldKey) {
      if (fieldKey === 'name') {
        return currentMode === 'create';
      }
      const allowed = allowedEditableFields[platformName] || Object.values(FIELD_GROUPS).flat().map((field) => field.key);
      return allowed.includes(fieldKey);
    }
    function getDisplayValue(field, value) {
      if (field === 'tags' && Array.isArray(value)) {
        return value.join(', ');
      }
      return value ?? '';
    }
    function getNormalizedFieldValue(field) {
      const input = fieldElements[field];
      if (!input) return undefined;
      if (field === 'tags') {
        return input.value ? input.value.split(',').map((tag) => tag.trim()).filter(Boolean) : undefined;
      }
      return input.value || undefined;
    }
    function syncFieldOriginal(field, value) {
      if (!fieldState[field]) {
        fieldState[field] = { original: '', dirty: false };
      }
      fieldState[field].original = value;
      fieldState[field].dirty = false;
    }
    function updateCopyButtonState(field) {
      const button = copyButtons[field];
      const input = fieldElements[field];
      if (!button || !input) return;
      button.disabled = input.value.trim().length === 0;
    }
    function setFieldFeedback(field, success) {
      const feedback = document.getElementById('status-' + field);
      if (!feedback) return;
      const message = success ? 'Saved' : 'Error';
      feedback.textContent = message;
      feedback.className = 'field-feedback visible ' + (success ? 'success' : 'error');
      setStatusMessage((fieldLabels[field] || field) + ' ' + (success ? 'saved.' : 'failed to save.'));
      if (statusTimers[field]) clearTimeout(statusTimers[field]);
      statusTimers[field] = setTimeout(() => { feedback.className = 'field-feedback'; }, success ? 1800 : 2600);
    }
    function setCreateFeedback(success, error) {
      const autosaveEl = document.getElementById('autosave-indicator');
      autosaveEl.textContent = success ? 'Created ✓' : 'Create failed';
      autosaveEl.classList.toggle('saved', !!success);
      if (!success && error) {
        document.getElementById('header-note').textContent = error;
      }
      setStatusMessage(success ? 'Profile created successfully.' : ('Profile creation failed. ' + (error || '')));
      document.getElementById('submit-create').disabled = false;
      document.getElementById('cancel-create').disabled = false;
    }
    function setCopyFeedback(field, success) {
      const button = copyButtons[field];
      if (!button) return;
      const originalLabel = button.dataset.defaultLabel || 'Copy';
      button.textContent = success ? 'Copied' : 'Failed';
      button.disabled = true;
      setStatusMessage((fieldLabels[field] || field) + ' ' + (success ? 'copied to clipboard.' : 'copy failed.'));
      if (copyTimers[field]) clearTimeout(copyTimers[field]);
      copyTimers[field] = setTimeout(() => {
        button.textContent = originalLabel;
        updateCopyButtonState(field);
      }, success ? 1400 : 1800);
    }
    function flashAutosave() {
      const autosaveEl = document.getElementById('autosave-indicator');
      autosaveEl.textContent = 'Saved ✓';
      autosaveEl.classList.add('saved');
      setStatusMessage('Profile changes saved.');
      window.clearTimeout(flashAutosave.timer);
      flashAutosave.timer = window.setTimeout(() => {
        autosaveEl.textContent = currentMode === 'create' ? 'Create mode' : 'Auto-save';
        autosaveEl.classList.remove('saved');
      }, 1800);
    }
    flashAutosave.timer = 0;
    function persistState() { vscode.setState({ profile: sanitizeProfile(currentProfile), mode: currentMode }); }
    function saveFieldIfDirty(field) {
      if (currentMode !== 'edit') return;
      const input = fieldElements[field];
      const state = fieldState[field];
      if (!input || !state || !state.dirty) return;
      vscode.postMessage({ type: 'saveField', field, value: input.value });
    }
    function registerField(groupName, field) {
      const container = document.getElementById('section-' + groupName);
      if (!container) return;
      const row = document.createElement('div');
      row.className = 'field-row';
      const requiredMark = field.required ? '<span class="required" aria-hidden="true">*</span>' : '';
      const toggleButton = field.actions && field.actions.includes('toggle')
        ? '<button type="button" class="control-btn" data-action="toggle" data-field="' + field.key + '">Show</button>'
        : '';
      const copyButton = field.actions && field.actions.includes('copy')
        ? '<button type="button" class="control-btn" data-action="copy" data-field="' + field.key + '" data-default-label="Copy">Copy</button>'
        : '';
      row.innerHTML =
        '<div class="field-label-wrap">' +
          '<label class="field-label" for="field-' + field.key + '">' + field.label + requiredMark + '</label>' +
          '<span class="field-hint" id="hint-' + field.key + '">' + field.hint + '</span>' +
        '</div>' +
        '<div class="field-input-wrap">' +
          '<div class="input-shell">' +
            '<input class="field-input" type="' + (field.secret ? 'password' : 'text') + '" id="field-' + field.key + '" data-field="' + field.key + '" autocomplete="off" spellcheck="false" placeholder="' + (field.placeholder || '') + '" aria-describedby="hint-' + field.key + '"' + (field.required ? ' aria-required="true"' : '') + ' />' +
            '<div class="field-actions">' + toggleButton + copyButton + '<span class="field-feedback" id="status-' + field.key + '" aria-hidden="true"></span></div>' +
          '</div>' +
        '</div>';
      row.dataset.fieldKey = field.key;
      row.dataset.createOnly = field.createOnly ? 'true' : 'false';
      container.appendChild(row);
      const input = document.getElementById('field-' + field.key);
      fieldElements[field.key] = input;
      fieldState[field.key] = { original: '', dirty: false };
      input.addEventListener('input', () => {
        fieldState[field.key].dirty = input.value !== fieldState[field.key].original;
        updateCopyButtonState(field.key);
      });
      input.addEventListener('blur', () => { saveFieldIfDirty(field.key); });
      input.addEventListener('keydown', (event) => {
        if (event.key === 'Enter' && currentMode === 'edit') {
          event.preventDefault();
          input.blur();
        }
      });
      const copyButtonElement = row.querySelector('[data-action="copy"]');
      if (copyButtonElement) {
        copyButtons[field.key] = copyButtonElement;
        copyButtonElement.addEventListener('click', () => {
          if (!input.value.trim()) return;
          vscode.postMessage({ type: 'copyField', field: field.key, value: input.value });
        });
      }
      const toggleButtonElement = row.querySelector('[data-action="toggle"]');
      if (toggleButtonElement) {
        toggleButtons[field.key] = toggleButtonElement;
        toggleButtonElement.addEventListener('click', () => {
          const isPassword = input.type === 'password';
          input.type = isPassword ? 'text' : 'password';
          toggleButtonElement.textContent = isPassword ? 'Hide' : 'Show';
        });
      }
      updateCopyButtonState(field.key);
    }
    Object.entries(FIELD_GROUPS).forEach(([groupName, fields]) => {
      fields.forEach((field) => registerField(groupName, field));
    });
    const enabledCheckbox = document.getElementById('field-enabled');
    const enabledLabel = document.getElementById('enabled-label');
    const submitActions = document.getElementById('submit-actions');
    const submitCreate = document.getElementById('submit-create');
    const cancelCreate = document.getElementById('cancel-create');
    enabledCheckbox.addEventListener('change', () => {
      if (currentMode === 'create') {
        enabledLabel.textContent = enabledCheckbox.checked ? 'Enabled' : 'Disabled';
        if (currentProfile) {
          currentProfile.enabled = enabledCheckbox.checked;
          persistState();
        }
        return;
      }
      vscode.postMessage({ type: 'toggleEnabled' });
    });
    submitCreate.addEventListener('click', () => {
      const draft = {
        name: fieldElements.name.value.trim(),
        platformName: currentProfile.platformName,
        description: getNormalizedFieldValue('description'),
        baseUrl: getNormalizedFieldValue('baseUrl'),
        authToken: getNormalizedFieldValue('authToken'),
        model: getNormalizedFieldValue('model'),
        smallFastModel: getNormalizedFieldValue('smallFastModel'),
        provider: getNormalizedFieldValue('provider'),
        providerType: getNormalizedFieldValue('providerType'),
        account: getNormalizedFieldValue('account'),
        tags: getNormalizedFieldValue('tags'),
        enabled: enabledCheckbox.checked,
      };
      submitCreate.disabled = true;
      cancelCreate.disabled = true;
      vscode.postMessage({ type: 'createProfile', draft });
    });
    cancelCreate.addEventListener('click', () => vscode.postMessage({ type: 'cancelCreate' }));
    function populateProfile(profile, mode) {
      currentMode = mode;
      currentProfile = cloneProfile(profile);
      const accent = setPlatformAccent(profile.platformName);
      document.getElementById('title').textContent = mode === 'create' ? 'New Profile' : profile.name;
      document.getElementById('subtitle').textContent = mode === 'create'
        ? 'Fill in all profile details here, then create it in one step.'
        : (profile.description || 'Tune routing, credentials and identity details for this profile.');
      document.getElementById('mode-note').textContent = mode === 'create'
        ? 'Review all fields, then submit once.'
        : 'Changes save when you leave a field.';
      document.getElementById('header-note').textContent = mode === 'create'
        ? 'Nothing is written until you click Create Profile.'
        : 'Pinned edits stay local to this profile.';
      const autosave = document.getElementById('autosave-indicator');
      autosave.textContent = mode === 'create' ? 'Create mode' : 'Auto-save';
      autosave.classList.remove('saved');
      const badge = document.getElementById('platform-badge');
      badge.textContent = accent.icon + ' ' + profile.platformName;
      enabledCheckbox.checked = !!profile.enabled;
      enabledLabel.textContent = profile.enabled ? 'Enabled' : 'Disabled';
      submitActions.classList.toggle('visible', mode === 'create');
      document.querySelectorAll('[data-field-key]').forEach((row) => {
        const fieldKey = row.getAttribute('data-field-key');
        row.style.display = isFieldVisible(profile.platformName, fieldKey) ? '' : 'none';
      });
      Object.values(FIELD_GROUPS).flat().forEach((field) => {
        const input = fieldElements[field.key];
        if (!input) return;
        const displayValue = getDisplayValue(field.key, profile[field.key]);
        input.value = displayValue;
        syncFieldOriginal(field.key, displayValue);
        updateCopyButtonState(field.key);
        if (field.secret) input.type = 'password';
        if (toggleButtons[field.key]) toggleButtons[field.key].textContent = 'Show';
        input.readOnly = false;
      });
      document.getElementById('loading').classList.add('hidden');
      document.getElementById('editor-content').style.display = 'block';
      document.querySelectorAll('[data-editor-section]').forEach((section) => { section.style.display = 'block'; });
      persistState();
    }
    const previousState = vscode.getState();
    if (previousState && previousState.profile && previousState.mode) {
      populateProfile(previousState.profile, previousState.mode);
    }
    window.addEventListener('message', (event) => {
      const msg = event.data;
      if (msg.type === 'profileData') {
        populateProfile(msg.profile, msg.mode);
        return;
      }
      if (msg.type === 'copyResult') {
        setCopyFeedback(msg.field, msg.success);
        return;
      }
      if (msg.type === 'createResult') {
        setCreateFeedback(msg.success, msg.error);
        return;
      }
      if (msg.type === 'saveResult') {
        if (msg.field === 'enabled') {
          if (msg.success) {
            if (currentProfile) {
              currentProfile.enabled = enabledCheckbox.checked;
              persistState();
            }
            flashAutosave();
            enabledLabel.textContent = enabledCheckbox.checked ? 'Enabled' : 'Disabled';
          } else if (currentProfile) {
            enabledCheckbox.checked = !!currentProfile.enabled;
            enabledLabel.textContent = currentProfile.enabled ? 'Enabled' : 'Disabled';
          }
          return;
        }
        if (msg.success) {
          const input = fieldElements[msg.field];
          if (input) {
            syncFieldOriginal(msg.field, input.value);
            if (currentProfile) {
              currentProfile[msg.field] = getNormalizedFieldValue(msg.field);
              persistState();
            }
          }
          flashAutosave();
        }
        setFieldFeedback(msg.field, msg.success);
      }
    });
    vscode.postMessage({ type: 'ready' });
  </script>
</body>
</html>`;
  }
}

function getNonce(): string {
  return randomBytes(16).toString("hex");
}
