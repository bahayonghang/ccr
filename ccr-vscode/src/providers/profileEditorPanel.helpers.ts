import type { ProfileEditorDraft, ProfileEditorMode, ProfileInfo } from "../models/types";

export function getPanelKey(mode: ProfileEditorMode, profile: ProfileInfo | ProfileEditorDraft): string {
  return mode === "create"
    ? `create/${(profile as ProfileEditorDraft).platformName}`
    : `${(profile as ProfileInfo).platformName}/${(profile as ProfileInfo).name}`;
}

export function getPanelTitle(mode: ProfileEditorMode, profile: ProfileInfo | ProfileEditorDraft): string {
  return mode === "create"
    ? `Add Profile: ${profile.platformName}`
    : `Edit: ${profile.name} (${profile.platformName})`;
}

export function normalizeFieldValue(tomlKey: string, value: string): string | string[] | undefined {
  if (tomlKey === "tags") {
    return value
      ? value.split(",").map((tag) => tag.trim()).filter(Boolean)
      : undefined;
  }

  return value || undefined;
}
