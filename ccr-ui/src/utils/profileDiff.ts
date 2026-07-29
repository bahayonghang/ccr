// Profiles 字段 diff 的平台无关壳：字段提取函数由平台注入，
// 输出供 ProfileDiffRows（Inspector 预览 / Apply 确认框）渲染。

/** 参与对比的单个字段定义（平台决定字段集合与取值方式） */
export interface ProfileDiffField<T> {
  /** 稳定 key，仅供 v-for / 测试断言使用，例如 'base_url' */
  key: string
  /** 展示标签（调用方先做 i18n） */
  label: string
  /** 从 profile 提取原始值；缺失返回 null/undefined */
  value: (profile: T) => string | null | undefined
}

/** diff 结果行：from = 当前值，to = 目标值（null 表示缺失，占位符由渲染层决定） */
export interface ProfileDiffRow {
  key: string
  label: string
  from: string | null
  to: string | null
  changed: boolean
}

const normalize = (value: string | null | undefined): string | null => {
  if (value == null) return null
  const trimmed = value.trim()
  return trimmed.length > 0 ? trimmed : null
}

/**
 * 计算「当前 → 目标」的逐字段 diff。
 * current 为 null（尚未激活任何 profile）时所有行 from = null 且恒 changed。
 * 比较基于 trim 后的字符串，避免尾部空白造成假差异。
 */
export function buildProfileDiff<T>(
  current: T | null,
  target: T,
  fields: readonly ProfileDiffField<T>[],
): ProfileDiffRow[] {
  return fields.map((field) => {
    const from = current ? normalize(field.value(current)) : null
    const to = normalize(field.value(target))
    return {
      key: field.key,
      label: field.label,
      from,
      to,
      changed: from !== to,
    }
  })
}
