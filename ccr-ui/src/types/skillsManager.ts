/**
 * Skills Manager 页面类型定义
 */

import type { SkillRecord } from './skills'

/** 按名称聚合的 Skill 组 */
export interface SkillGroup {
  /** Skill 名称 (组标识) */
  name: string
  /** 描述 */
  description: string
  /** 关联的 Skill 记录列表 */
  items: SkillRecord[]
  /** 安装的平台列表 */
  platforms: string[]
  /** 来源标签 */
  origin: string
}

/** 按来源聚合的 Source 分组 (可折叠) */
export interface SkillSourceGroup {
  /** 来源标识 (sourceRef 或 sourceLabel) */
  source: string
  /** 来源类型 */
  sourceType: string
  /** 该来源下的 Skill 组 */
  skills: SkillGroup[]
}

/** 右侧面板状态 */
export type SkillPanelMode =
  | { type: 'empty' }
  | { type: 'detail'; skillId: string }
  | { type: 'create' }
  | { type: 'import' }
  | { type: 'import-github' }
