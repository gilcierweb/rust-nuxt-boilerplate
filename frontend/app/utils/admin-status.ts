type TranslateFn = (key: string) => string

type StatusMeta = {
  key:
    | 'active'
    | 'inactive'
    | 'pending'
    | 'archived'
    | 'inProgress'
    | 'completed'
    | 'cancelled'
    | 'paid'
    | 'approved'
    | 'rejected'
    | 'unknown'
  badgeClass: string
  fallback: string
}

type StatusContext = {
  resourceSlug?: string
  columnKey: string
}

const DEFAULT_STATUS_BY_CODE: Record<number, Omit<StatusMeta, 'key'> & { key: StatusMeta['key'] }> = {
  1: { key: 'active', badgeClass: 'badge-success', fallback: 'Active' },
  2: { key: 'inactive', badgeClass: 'badge-neutral', fallback: 'Inactive' },
}

const UNKNOWN_STATUS: StatusMeta = {
  key: 'unknown',
  badgeClass: 'badge-neutral',
  fallback: 'Unknown',
}

function parseStatusCode(value: unknown): number | null {
  if (typeof value === 'number' && Number.isFinite(value)) return value
  if (typeof value === 'string' && value.trim() !== '') {
    const parsed = Number(value)
    if (Number.isFinite(parsed)) return parsed
  }
  return null
}

function translateOrFallback(t: TranslateFn, key: string, fallback: string): string {
  const translated = t(key)
  if (!translated || translated === key) return fallback
  return translated
}

export function isSemanticStatusColumn(columnKey: string): boolean {
  return columnKey === 'status' || columnKey.endsWith('_status')
}

function resolveStatusMap(context?: StatusContext) {
  return DEFAULT_STATUS_BY_CODE
}

export function resolveSemanticStatusMeta(value: unknown, context?: StatusContext): StatusMeta {
  const code = parseStatusCode(value)
  if (code === null) return UNKNOWN_STATUS
  const statusMap = resolveStatusMap(context)
  return statusMap[code] || UNKNOWN_STATUS
}

export function formatSemanticStatusLabel(
  value: unknown,
  t: TranslateFn,
  context?: StatusContext,
): string {
  const code = parseStatusCode(value)
  const meta = resolveSemanticStatusMeta(value, context)
  const label = translateOrFallback(t, `admin.statusLabels.${meta.key}`, meta.fallback)

  if (meta.key === 'unknown' && code !== null) {
    return `${label} (${code})`
  }

  if (meta.key === 'unknown' && typeof value !== 'undefined' && value !== null && value !== '') {
    return `${label} (${String(value)})`
  }

  return label
}