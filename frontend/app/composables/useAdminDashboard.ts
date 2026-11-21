import { computed } from 'vue'

import {
  extractPaginationMeta,
  normalizeResourceResponse,
  toAuditDashboardPayload,
} from '~/utils/admin-resources'
import { formatUtcDateShort } from '~/utils/format-date'

type AuditLogItem = Record<string, any>

const AUDIT_ACTION_LIMIT = 6
const AUDIT_ACTIVITY_WINDOW_DAYS = 14
const RECENT_ACTIVITY_PER_SOURCE = 3
const RECENT_ACTIVITY_LIMIT = 8

function countBy<T>(items: readonly T[], getKey: (item: T) => string | null): Map<string, number> {
  const counts = new Map<string, number>()

  for (const item of items) {
    const key = getKey(item)
    if (key == null) continue
    counts.set(key, (counts.get(key) ?? 0) + 1)
  }

  return counts
}

function toUtcDayKey(value: unknown): string | null {
  if (typeof value !== 'string' || value.length === 0) return null

  const timestamp = Date.parse(value)
  if (Number.isNaN(timestamp)) return null

  return new Date(timestamp).toISOString().slice(0, 10)
}

function buildUtcDayWindow(dayCount: number, endDate: Date = new Date()): string[] {
  const endUtcDay = Date.UTC(endDate.getUTCFullYear(), endDate.getUTCMonth(), endDate.getUTCDate())

  return Array.from({ length: dayCount }, (_, index) => {
    const offset = dayCount - 1 - index
    return new Date(endUtcDay - offset * 86_400_000).toISOString().slice(0, 10)
  })
}

function parseTimestamp(value?: string): number {
  if (!value) return 0
  const timestamp = Date.parse(value)
  return Number.isNaN(timestamp) ? 0 : timestamp
}

function singleSeriesCategory(key: string, name: string) {
  return { [key]: { name } }
}

/**
 * Dashboard data and aggregations for the admin overview page.
 * Fetches users/roles/audit logs and derives the stats, charts and
 * recent-activity feed consumed by the dashboard template.
 */
export async function useAdminDashboard() {
  const { t, locale } = useI18n()

  // Nuxt composables must run while the Nuxt instance is on the call stack,
  // so create all fetches before the first `await` and await them together.
  const usersFetch = useApiFetch<any>('/admin/users', {
    key: 'admin-dashboard-users',
    server: true,
    default: () => [],
  })
  const rolesFetch = useApiFetch<any>('/admin/roles', {
    key: 'admin-dashboard-roles',
    server: true,
    default: () => [],
  })
  const auditLogsFetch = useApiFetch<any>('/admin/audit-logs', {
    key: 'admin-dashboard-audit-logs',
    server: true,
    default: () => [],
    query: { per_page: 100 },
    transform: toAuditDashboardPayload,
  })
  await Promise.all([usersFetch, rolesFetch, auditLogsFetch])

  const users = computed<any[]>(() => normalizeResourceResponse(usersFetch.data.value))
  const roles = computed<any[]>(() => normalizeResourceResponse(rolesFetch.data.value))
  const auditLogs = computed<AuditLogItem[]>(() => normalizeResourceResponse(auditLogsFetch.data.value))

  const pendingAny = computed(() =>
    usersFetch.pending.value || rolesFetch.pending.value || auditLogsFetch.pending.value,
  )

  const stats = computed(() => ({
    // Totals come from pagination metadata, not the loaded page (max per_page rows).
    totalUsers: extractPaginationMeta(usersFetch.data.value).total,
    totalRoles: extractPaginationMeta(rolesFetch.data.value).total,
    totalAuditLogs: extractPaginationMeta(auditLogsFetch.data.value).total,
  }))

  const auditActionCounts = computed(() =>
    [...countBy(auditLogs.value, (item) => String(item?.action || 'unknown')).entries()]
      .sort((left, right) => right[1] - left[1])
      .slice(0, AUDIT_ACTION_LIMIT)
      .map(([action, count]) => ({ action, count })),
  )

  const auditByDay = computed(() => {
    const countsByDay = countBy(auditLogs.value, (item) => toUtcDayKey(item?.created_at))

    return buildUtcDayWindow(AUDIT_ACTIVITY_WINDOW_DAYS).map((date) => ({
      date,
      count: countsByDay.get(date) ?? 0,
    }))
  })

  const auditCumulative = computed(() => {
    let runningTotal = 0

    return auditByDay.value.map(({ date, count }) => ({
      date,
      cumulative: (runningTotal += count),
    }))
  })

  const resourceDistribution = computed(() => ([
    { resource: t('admin.dashboard.totalUsers'), count: stats.value.totalUsers },
    { resource: t('admin.dashboard.totalRoles'), count: stats.value.totalRoles },
    { resource: t('admin.dashboard.totalAuditLogs'), count: stats.value.totalAuditLogs },
  ]))

  const resourceCategories = computed(() => singleSeriesCategory('count', t('admin.common.total')))
  const auditCountCategories = computed(() => singleSeriesCategory('count', t('admin.dashboard.auditLogCount')))
  const auditCumulativeCategories = computed(() => singleSeriesCategory('cumulative', t('admin.dashboard.totalAuditLogs')))

  const recentActivity = computed(() => {
    const userActivity = users.value.slice(0, RECENT_ACTIVITY_PER_SOURCE).map((item) => ({
      key: `user-${item.id}`,
      date: item.updated_at || item.created_at,
      title: t('admin.dashboard.activityUser'),
      description: `${item.first_name || ''} ${item.last_name || ''}`.trim() || item.email || item.id,
    }))

    const roleActivity = roles.value.slice(0, RECENT_ACTIVITY_PER_SOURCE).map((item) => ({
      key: `role-${item.id}`,
      date: item.updated_at || item.created_at,
      title: t('admin.dashboard.activityRole'),
      description: item.name || item.id,
    }))

    const auditActivity = auditLogs.value.slice(0, RECENT_ACTIVITY_PER_SOURCE).map((item) => ({
      key: `audit-${item.id}`,
      date: item.created_at,
      title: t('admin.dashboard.activityAudit'),
      description: `${item.action} - ${item.resource_type || ''}`.trim(),
    }))

    return [...userActivity, ...roleActivity, ...auditActivity]
      .sort((left, right) => parseTimestamp(right.date) - parseTimestamp(left.date))
      .slice(0, RECENT_ACTIVITY_LIMIT)
  })

  const formatChartDate = (value: unknown): string => formatUtcDateShort(value, { locale: locale.value })

  async function refreshDashboard() {
    await Promise.all([
      usersFetch.refresh(),
      rolesFetch.refresh(),
      auditLogsFetch.refresh(),
    ])
  }

  return {
    users,
    roles,
    auditLogs,
    pendingAny,
    stats,
    auditActionCounts,
    auditByDay,
    auditCumulative,
    auditCumulativeCategories,
    resourceDistribution,
    resourceCategories,
    auditCountCategories,
    recentActivity,
    formatChartDate,
    refreshDashboard,
  }
}

export type AdminDashboard = Awaited<ReturnType<typeof useAdminDashboard>>
