import type { PlatformInfo, ProfileInfo, TreeSectionInfo } from "../models/types";
import { isProfileCreationPlatform } from "../models/types";

export function supportsProfileMutation(platformName: string): boolean {
  return isProfileCreationPlatform(platformName);
}

export function getPlatformNodeContextValue(platform: PlatformInfo): string {
  if (!platform.enabled) {
    return "platform-disabled";
  }
  return supportsProfileMutation(platform.name)
    ? "platform-create-supported"
    : "platform";
}

export function getSectionNodeContextValue(section: TreeSectionInfo): string {
  return section.kind === "profiles" && supportsProfileMutation(section.platformName)
    ? "section-profiles-create-supported"
    : `section-${section.kind}`;
}

export function getProfileNodeContextValue(profile: ProfileInfo): string {
  if (supportsProfileMutation(profile.platformName)) {
    return profile.isCurrent ? "profile-current-supported" : "profile-supported";
  }

  return profile.isCurrent ? "readonly-profile-current" : "readonly-profile";
}
