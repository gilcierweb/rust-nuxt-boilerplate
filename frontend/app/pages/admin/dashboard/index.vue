<template>
  <section class="space-y-6">
    <AdminBreadcrumb :items="breadcrumbItems" />

    <div class="rounded-box border border-base-content/10 bg-base-100 p-6 shadow-md shadow-base-content/5">
      <div class="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <div class="mb-3 inline-flex items-center gap-2 rounded-field bg-primary/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.22em] text-primary">
            <span class="icon-[tabler--layout-dashboard] size-4"></span>
            <span>{{ $t('admin.section') }}</span>
          </div>
          <h1 class="text-3xl font-semibold tracking-tight text-base-content">{{ $t('admin.title') }}</h1>
          <p class="mt-2 max-w-3xl text-sm leading-relaxed text-base-content/60">
            Monitoramento rápido de usuários, cargos e logs de auditoria.
          </p>
        </div>

        <div class="flex flex-wrap gap-2">
          <button type="button" class="btn btn-soft" :disabled="pendingAny" @click="refreshDashboard">
            <span class="icon-[tabler--refresh] size-4.5"></span>
            Atualizar
          </button>
        </div>
      </div>
    </div>

    <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
      <div class="rounded-box border border-base-content/10 bg-base-100 p-4 shadow-md shadow-base-content/5">
        <p class="text-xs font-semibold uppercase tracking-[0.18em] text-base-content/45">{{ $t('admin.dashboard.totalUsers') }}</p>
        <p class="mt-2 text-2xl font-semibold text-base-content">{{ stats.totalUsers }}</p>
      </div>
      <div class="rounded-box border border-base-content/10 bg-base-100 p-4 shadow-md shadow-base-content/5">
        <p class="text-xs font-semibold uppercase tracking-[0.18em] text-base-content/45">{{ $t('admin.dashboard.totalRoles') }}</p>
        <p class="mt-2 text-2xl font-semibold text-base-content">{{ stats.totalRoles }}</p>
      </div>
      <div class="rounded-box border border-base-content/10 bg-base-100 p-4 shadow-md shadow-base-content/5">
        <p class="text-xs font-semibold uppercase tracking-[0.18em] text-base-content/45">{{ $t('admin.dashboard.totalAuditLogs') }}</p>
        <p class="mt-2 text-2xl font-semibold text-base-content">{{ stats.totalAuditLogs }}</p>
      </div>
    </div>

    <div class="grid gap-6 xl:grid-cols-3">
      <div class="rounded-box border border-base-content/10 bg-base-100 p-5 shadow-md shadow-base-content/5 xl:col-span-2">
        <div class="mb-4 flex items-center justify-between gap-3">
          <h2 class="text-lg font-semibold text-base-content">{{ $t('admin.dashboard.recentActivity') }}</h2>
        </div>
        <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          <div class="rounded-box bg-base-200/70 p-4">
            <p class="text-xs font-semibold uppercase tracking-[0.18em] text-base-content/45">{{ $t('admin.dashboard.totalUsers') }}</p>
            <p class="mt-2 text-2xl font-semibold text-base-content">{{ stats.totalUsers }}</p>
          </div>
          <div class="rounded-box bg-base-200/70 p-4">
            <p class="text-xs font-semibold uppercase tracking-[0.18em] text-base-content/45">{{ $t('admin.dashboard.totalRoles') }}</p>
            <p class="mt-2 text-2xl font-semibold text-base-content">{{ stats.totalRoles }}</p>
          </div>
          <div class="rounded-box bg-base-200/70 p-4">
            <p class="text-xs font-semibold uppercase tracking-[0.18em] text-base-content/45">{{ $t('admin.dashboard.totalAuditLogs') }}</p>
            <p class="mt-2 text-2xl font-semibold text-base-content">{{ stats.totalAuditLogs }}</p>
          </div>
        </div>
      </div>

      <div class="rounded-box border border-base-content/10 bg-base-100 p-5 shadow-md shadow-base-content/5">
        <h2 class="text-lg font-semibold text-base-content">{{ $t('admin.dashboard.quickLinks') }}</h2>
        <div class="mt-4 space-y-3">
          <NuxtLink to="/admin/users" class="btn btn-soft w-full justify-start gap-2">
            <span class="icon-[tabler--users] size-4.5"></span>
            {{ $t('admin.sidebar.users') }}
          </NuxtLink>
          <NuxtLink to="/admin/roles" class="btn btn-soft w-full justify-start gap-2">
            <span class="icon-[tabler--shield] size-4.5"></span>
            {{ $t('admin.sidebar.roles') }}
          </NuxtLink>
          <NuxtLink to="/admin/audit-logs" class="btn btn-soft w-full justify-start gap-2">
            <span class="icon-[tabler--history] size-4.5"></span>
            {{ $t('admin.sidebar.auditLogs') }}
          </NuxtLink>
        </div>
      </div>
    </div>

    <div class="rounded-box border border-base-content/10 bg-base-100 p-5 shadow-md shadow-base-content/5">
      <div class="mb-4 flex items-center justify-between">
        <h2 class="text-lg font-semibold text-base-content">{{ $t('admin.dashboard.recentActivity') }}</h2>
      </div>

      <ul v-if="recentActivity.length > 0" class="space-y-3">
        <li
          v-for="item in recentActivity"
          :key="item.key"
          class="rounded-box border border-base-content/10 bg-base-200/40 px-4 py-3"
        >
          <div class="flex flex-col gap-1 sm:flex-row sm:items-center sm:justify-between">
            <div>
              <p class="font-medium text-base-content">{{ item.title }}</p>
              <p class="text-sm text-base-content/65">{{ item.description }}</p>
            </div>
            <span class="text-sm text-base-content/60">{{ formatDateTime(item.date) }}</span>
          </div>
        </li>
      </ul>
      <p v-else class="text-sm text-base-content/60">{{ $t('admin.dashboard.noRecentActivity') }}</p>
    </div>
  </section>
</template>

<script setup lang="ts">
import { formatDateTime } from '~/utils/admin-ui'
import { normalizeResourceResponse } from '~/utils/admin-resources'

const { t } = useI18n()

definePageMeta({
  layout: 'admin',
})

const breadcrumbItems = computed(() => [{ label: t('admin.common.dashboard') }])

const usersFetch = await useApiFetch<any>('/admin/users', { key: 'admin-dashboard-users', server: true, default: () => [] })
const rolesFetch = await useApiFetch<any>('/admin/roles', { key: 'admin-dashboard-roles', server: true, default: () => [] })
const auditLogsFetch = await useApiFetch<any>('/admin/audit-logs', { key: 'admin-dashboard-audit-logs', server: true, default: () => [] })

const users = computed<any[]>(() => normalizeResourceResponse(usersFetch.data.value))
const roles = computed<any[]>(() => normalizeResourceResponse(rolesFetch.data.value))
const auditLogs = computed<any[]>(() => normalizeResourceResponse(auditLogsFetch.data.value))

const pendingAny = computed(() =>
  usersFetch.pending.value
  || rolesFetch.pending.value
  || auditLogsFetch.pending.value,
)

function parseDate(value?: string): number {
  if (!value) return 0
  const parsed = new Date(value).getTime()
  return Number.isNaN(parsed) ? 0 : parsed
}

const stats = computed(() => {
  const totalUsers = users.value.length
  const totalRoles = roles.value.length
  const totalAuditLogs = auditLogs.value.length

  return {
    totalUsers,
    totalRoles,
    totalAuditLogs,
  }
})

const recentActivity = computed(() => {
  const userActivity = users.value.slice(0, 3).map((item) => ({
    key: `user-${item.id}`,
    date: item.updated_at || item.created_at,
    title: 'Usuário',
    description: `${item.first_name || ''} ${item.last_name || ''}`.trim() || item.email || item.id,
  }))

  const roleActivity = roles.value.slice(0, 3).map((item) => ({
    key: `role-${item.id}`,
    date: item.updated_at || item.created_at,
    title: 'Cargo',
    description: item.name || item.id,
  }))

  const auditActivity = auditLogs.value.slice(0, 3).map((item) => ({
    key: `audit-${item.id}`,
    date: item.created_at,
    title: 'Auditoria',
    description: `${item.action} - ${item.resource_type || ''}`.trim(),
  }))

  return [...userActivity, ...roleActivity, ...auditActivity]
    .sort((left, right) => parseDate(right.date) - parseDate(left.date))
    .slice(0, 8)
})

async function refreshDashboard() {
  await Promise.all([
    usersFetch.refresh(),
    rolesFetch.refresh(),
    auditLogsFetch.refresh(),
  ])
}
</script>