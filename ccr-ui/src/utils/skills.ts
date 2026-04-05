import type { MarketplaceItem, SkillRecord, UnifiedSkill } from '@/types/skills'

function normalizeComparable(value?: string | null): string {
  return (value ?? '')
    .trim()
    .toLowerCase()
}

function getMarketplaceSourceRefs(item: MarketplaceItem): Set<string> {
  const refs = new Set<string>()
  const packageId = normalizeComparable(item.package)
  const basePackage = normalizeComparable(item.package.split('@')[0] ?? item.package)
  const repoRef = normalizeComparable(`${item.owner}/${item.repo}`)
  const githubUrl = normalizeComparable(`https://github.com/${item.owner}/${item.repo}`)

  if (packageId) refs.add(packageId)
  if (basePackage) refs.add(basePackage)
  if (repoRef) refs.add(repoRef)
  if (githubUrl) refs.add(githubUrl)

  return refs
}

function getMarketplaceSkillNames(item: MarketplaceItem): Set<string> {
  const names = new Set<string>()
  const explicitSkill = normalizeComparable(item.skill)
  const repoName = normalizeComparable(item.repo)
  const packageSkill = normalizeComparable(item.package.split('@')[1] ?? '')

  if (explicitSkill) names.add(explicitSkill)
  if (packageSkill) names.add(packageSkill)
  if (repoName) names.add(repoName)

  return names
}

export const toInstalledPackageSet = (
  skills: Pick<UnifiedSkill, 'name'>[],
): Set<string> => {
  return new Set(skills.map((skill) => skill.name))
}

export const isMarketplaceItemInstalled = (
  item: MarketplaceItem,
  skills: Pick<SkillRecord, 'name' | 'origin' | 'sourceRef' | 'sourceLabel'>[],
): boolean => {
  const sourceRefs = getMarketplaceSourceRefs(item)
  const itemNames = getMarketplaceSkillNames(item)

  return skills.some((skill) => {
    const sourceCandidates = [
      normalizeComparable(skill.sourceRef),
      normalizeComparable(skill.sourceLabel),
    ].filter(Boolean)

    const sourceMatched = sourceCandidates.some((candidate) => sourceRefs.has(candidate))
    if (!sourceMatched) {
      return false
    }

    if (itemNames.size === 0) {
      return true
    }

    return itemNames.has(normalizeComparable(skill.name))
  })
}
