/**
 * 跨层共享的通用类型别名。
 */

/** 任意键值对象。收口此前散落在多个文件中的同名局部别名。 */
export type UnknownRecord = Record<string, unknown>

export interface CommandResultLike {
  success?: boolean
  message?: string
  output?: string
  data?: {
    output?: string
  }
}

export interface VersionInfoResponse {
  current_version?: string
  build_time?: string
  git_commit?: string
  latest_version?: string
  has_update?: boolean
  release_url?: string
  release_notes?: string
  published_at?: string
}
