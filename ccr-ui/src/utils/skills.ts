import type { UnifiedSkill } from '@/types/skills'

export const toInstalledPackageSet = (
  skills: Pick<UnifiedSkill, 'name'>[],
): Set<string> => {
  return new Set(skills.map((skill) => skill.name))
}
