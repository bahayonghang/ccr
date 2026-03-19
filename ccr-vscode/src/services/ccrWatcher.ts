/**
 * File watcher service for CCR configuration files
 *
 * Monitors config.toml and platform profile files for changes,
 * triggering UI refresh with 300ms debounce.
 */

import * as vscode from "vscode";
import { getCcrRoot } from "./ccrPaths";

export class CcrWatcher implements vscode.Disposable {
  private watchers: vscode.FileSystemWatcher[] = [];
  private debounceTimer: ReturnType<typeof setTimeout> | undefined;
  private readonly debounceMs = 300;
  private onChangeCallbacks: (() => void)[] = [];

  constructor() {
    this.setupWatchers();
  }

  /** Register a callback to be called when config files change */
  onChange(callback: () => void): void {
    this.onChangeCallbacks.push(callback);
  }

  private setupWatchers(): void {
    const root = getCcrRoot();

    // Watch config.toml (registry)
    const registryPattern = new vscode.RelativePattern(
      vscode.Uri.file(root),
      "config.toml",
    );
    const registryWatcher = vscode.workspace.createFileSystemWatcher(registryPattern);
    registryWatcher.onDidChange(() => this.debouncedNotify());
    registryWatcher.onDidCreate(() => this.debouncedNotify());
    registryWatcher.onDidDelete(() => this.debouncedNotify());
    this.watchers.push(registryWatcher);

    // Watch all profiles.toml files
    const profilesPattern = new vscode.RelativePattern(
      vscode.Uri.file(root),
      "platforms/*/profiles.toml",
    );
    const profilesWatcher = vscode.workspace.createFileSystemWatcher(profilesPattern);
    profilesWatcher.onDidChange(() => this.debouncedNotify());
    profilesWatcher.onDidCreate(() => this.debouncedNotify());
    profilesWatcher.onDidDelete(() => this.debouncedNotify());
    this.watchers.push(profilesWatcher);

    // Watch Codex auth registry
    const codexAuthRegistryPattern = new vscode.RelativePattern(
      vscode.Uri.file(root),
      "platforms/codex/auth_registry.toml",
    );
    const codexAuthRegistryWatcher = vscode.workspace.createFileSystemWatcher(codexAuthRegistryPattern);
    codexAuthRegistryWatcher.onDidChange(() => this.debouncedNotify());
    codexAuthRegistryWatcher.onDidCreate(() => this.debouncedNotify());
    codexAuthRegistryWatcher.onDidDelete(() => this.debouncedNotify());
    this.watchers.push(codexAuthRegistryWatcher);
  }

  /** Debounce rapid file change events into a single refresh */
  private debouncedNotify(): void {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
    }
    this.debounceTimer = setTimeout(() => {
      this.debounceTimer = undefined;
      for (const cb of this.onChangeCallbacks) {
        cb();
      }
    }, this.debounceMs);
  }

  dispose(): void {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
    }
    for (const w of this.watchers) {
      w.dispose();
    }
    this.watchers = [];
    this.onChangeCallbacks = [];
  }
}
