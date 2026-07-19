<template>
  <section class="space-y-6">
    <AdminBreadcrumb :items="breadcrumbItems" />

    <div class="rounded-box border border-base-content/10 bg-base-100 p-6 shadow-md shadow-base-content/5"><div class="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between"><div><div class="mb-3 inline-flex items-center gap-2 rounded-field bg-primary/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.22em] text-primary"><span class="icon-[tabler--history] size-4"></span><span>{{ $t('admin.auditLogs.title') }}</span></div><h1 class="text-3xl font-semibold tracking-tight text-base-content">{{ $t('admin.auditLogs.showTitle') }}</h1><p class="mt-2 max-w-3xl text-sm leading-relaxed text-base-content/60">{{ $t('admin.auditLogs.showDescription') }}</p></div><div class="flex flex-wrap gap-2"><NuxtLink :to="localePath('/admin/audit-logs')" class="btn btn-ghost"><span class="icon-[tabler--arrow-left] size-4.5"></span>{{ $t('admin.common.back') }}</NuxtLink><NuxtLink :to="localePath(`/admin/audit-logs/${itemId}/edit`)" class="btn btn-primary"><span class="icon-[tabler--edit] size-4.5"></span>{{ $t('admin.common.edit') }}</NuxtLink></div></div></div>
    <div v-if="pending" class="rounded-box border border-base-content/10 bg-base-100 p-12 shadow-md"><div class="flex flex-col items-center justify-center gap-4 text-base-content/55"><span class="icon-[tabler--loader-2] size-10 animate-spin"></span><p>{{ $t('admin.common.loadingData') }}</p></div></div>
    <div v-else-if="error" class="rounded-box border border-error/20 bg-error/10 p-6"><div class="flex items-center gap-3 text-error"><span class="icon-[tabler--alert-circle] size-6"></span><div><p class="font-semibold">{{ $t('admin.common.errorLoadingData') }}</p><p class="text-sm">{{ requestError }}</p></div></div><button class="btn btn-soft mt-4" @click="refresh()">{{ $t('admin.common.tryAgain') }}</button></div>
    <div v-else-if="item" class="rounded-box border border-base-content/10 bg-base-100 p-6 shadow-md shadow-base-content/5"><div class="grid gap-6 md:grid-cols-2"><div><p class="text-sm font-semibold text-base-content/70">{{ $t('admin.auditLogs.fields.company') }}</p><p class="mt-1 text-base-content">{{ lookup.resolveLabel('companies', item.company_id) }}</p></div><div><p class="text-sm font-semibold text-base-content/70">{{ $t('admin.auditLogs.fields.actorUser') }}</p><p class="mt-1 text-base-content">{{ lookup.resolveLabel('users', item.actor_user_id) }}</p></div><div><p class="text-sm font-semibold text-base-content/70">{{ $t('admin.auditLogs.fields.targetCustomer') }}</p><p class="mt-1 text-base-content">{{ lookup.resolveLabel('customers', item.target_customer_id) }}</p></div><div><p class="text-sm font-semibold text-base-content/70">{{ $t('admin.auditLogs.fields.actorRoleSnapshot') }}</p><p class="mt-1 text-base-content">{{ item.actor_role_snapshot || '—' }}</p></div><div><p class="text-sm font-semibold text-base-content/70">{{ $t('admin.auditLogs.fields.action') }}</p><p class="mt-1 text-base-content">{{ item.action }}</p></div><div><p class="text-sm font-semibold text-base-content/70">{{ $t('admin.auditLogs.fields.resourceType') }}</p><p class="mt-1 text-base-content">{{ item.resource_type }}</p></div><div><p class="text-sm font-semibold text-base-content/70">{{ $t('admin.auditLogs.fields.resourceId') }}</p><p class="mt-1 text-base-content">{{ item.resource_id || '—' }}</p></div><div><p class="text-sm font-semibold text-base-content/70">{{ $t('admin.auditLogs.fields.requestId') }}</p><p class="mt-1 text-base-content">{{ item.request_id || '—' }}</p></div><div><p class="text-sm font-semibold text-base-content/70">{{ $t('admin.auditLogs.fields.ipAddress') }}</p><p class="mt-1 text-base-content">{{ item.ip_address || '—' }}</p></div><div><p class="text-sm font-semibold text-base-content/70">{{ $t('admin.auditLogs.fields.userAgent') }}</p><p class="mt-1 text-base-content">{{ item.user_agent || '—' }}</p></div><div class="md:col-span-2"><p class="text-sm font-semibold text-base-content/70">{{ $t('admin.auditLogs.fields.changes') }}</p><pre class="mt-1 overflow-x-auto rounded-box bg-base-200 p-4 text-sm">{{ stringifyJson(item.changes) }}</pre></div><div class="md:col-span-2"><p class="text-sm font-semibold text-base-content/70">{{ $t('admin.auditLogs.fields.metadata') }}</p><pre class="mt-1 overflow-x-auto rounded-box bg-base-200 p-4 text-sm">{{ stringifyJson(item.metadata) }}</pre></div></div><div class="grid gap-6 border-t border-base-content/10 pt-4 md:grid-cols-1"><div><p class="text-sm font-semibold text-base-content/70">{{ $t('admin.auditLogs.fields.createdAt') }}</p><p class="mt-1 text-base-content">{{ formatDateTime(item.created_at) }}</p></div></div></div>
  </section>
</template>

<script setup lang="ts">
import { extractErrorMessage, formatDateTime, stringifyJson } from '~/utils/admin-ui'

interface AuditLogItem {
  id: string
  company_id?: string
  actor_user_id?: string
  actor_role_snapshot?: string
  action: string
  resource_type: string
  resource_id?: string
  target_customer_id?: string
  ip_address?: string
  user_agent?: string
  request_id?: string
  changes?: unknown
  metadata?: unknown
  created_at?: string
}

definePageMeta({
  layout: 'admin'
})

const { t } = useI18n()
const localePath = useLocalePath()

const breadcrumbItems = computed(() => [
  { label: t('admin.common.dashboard'), to: localePath('/admin/dashboard') },
  { label: t('admin.auditLogs.title'), to: localePath('/admin/audit-logs') },
  { label: t('admin.common.details') },
])

const lookup = useAdminLookup()
await Promise.all([lookup.load('companies'), lookup.load('users'), lookup.load('customers')])
const route = useRoute()
const itemId = computed(() => route.params.id as string)
const { data, pending, error, refresh } = await useApiFetch<AuditLogItem | { data: AuditLogItem }>(() => `/admin/audit-logs/${itemId.value}`, { key: `admin-audit-logs-show-${itemId.value}`, server: true, default: () => null })
const requestError = computed(() => (error.value ? extractErrorMessage(error.value) : ''))
const item = computed(() => (!data.value ? null : ('data' in data.value ? data.value.data : data.value)))
</script>
