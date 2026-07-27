export interface AuditException {
  id: string
  package: string
  owner: string
  rationale: string
  expires: string
  patchedVersions: string[]
}

export interface AuditPolicy {
  maxActiveExceptions: number
  exceptions: AuditException[]
}

export interface AuditAdvisory {
  id?: string
  url: string
  severity: string
  package?: string
  [key: string]: unknown
}

export type AuditReport = Record<string, AuditAdvisory[]>

export function validateAllowlist(policy: AuditPolicy, now?: Date): string[]
export function collectAdvisories(report: AuditReport): Array<AuditAdvisory & { id: string; package: string }>
export function validateAuditReport(report: AuditReport, policy: Pick<AuditPolicy, 'exceptions'>): string[]
