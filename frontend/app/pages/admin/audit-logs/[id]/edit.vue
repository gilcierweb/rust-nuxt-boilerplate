<template>
  <section class="space-y-6">
    <AdminBreadcrumb :items="breadcrumbItems" />
    <div class="card shadow-base-300/10 shadow-md"><div class="card-body"><div class="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between"><div><div class="mb-3 inline-flex items-center gap-2 rounded-field bg-primary/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.22em] text-primary"><span class="icon-[tabler--history] size-4"></span><span>{{ $t('admin.auditLogs.title') }}</span></div><h1 class="card-title text-3xl font-semibold tracking-tight text-base-content">{{ $t('admin.auditLogs.editTitle') }}</h1><p class="mt-2 max-w-3xl text-sm leading-relaxed text-base-content/60">{{ $t('admin.auditLogs.editDescription') }}</p></div><div class="card-actions flex flex-wrap gap-2"><NuxtLink to="/admin/audit-logs" class="btn btn-ghost"><span class="icon-[tabler--arrow-left] size-4.5"></span>{{ $t('admin.common.back') }}</NuxtLink></div></div></div></div>
    <div v-if="pending" class="card shadow-base-300/10 shadow-md"><div class="card-body p-12"><div class="flex flex-col items-center justify-center gap-4 text-base-content/55"><span class="icon-[tabler--loader-2] size-10 animate-spin"></span><p>{{ $t('admin.common.loadingData') }}</p></div></div></div>
    <div v-else-if="error" class="card border-error/20 bg-error/10 shadow-md"><div class="card-body"><div class="flex items-center gap-3 text-error"><span class="icon-[tabler--alert-circle] size-6"></span><div><p class="font-semibold">{{ $t('admin.common.errorLoadingData') }}</p><p class="text-sm">{{ requestError }}</p></div></div><button class="btn btn-soft mt-4" @click="refresh()">{{ $t('admin.common.tryAgain') }}</button></div></div>
    <div v-else-if="item" class="card shadow-base-300/10 shadow-md"><div class="card-body"><AuditLogsForm mode="edit" :initial-values="item" :saving="saving" @submit="handleSubmit" /></div></div>
  </section>
</template>

<script setup lang="ts">
import AuditLogsForm from '~/components/admin/audit-logs/AuditLogsForm.vue'
import { extractErrorMessage, stringifyJson } from '~/utils/admin-ui'

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

definePageMeta({ layout: 'admin' })

const { t } = useI18n()

const breadcrumbItems = computed(() => [
  { label: t('admin.common.dashboard'), to: '/admin/dashboard' },
  { label: t('admin.auditLogs.title'), to: '/admin/audit-logs' },
  { label: t('admin.common.edit') },
])

const api = useApi()
const toast = useToast()
const route = useRoute()
const itemId = computed(() => route.params.id as string)
const { data, pending, error, refresh } = await useApiFetch<AuditLogItem | { data: AuditLogItem }>(() => `/admin/audit-logs/${itemId.value}`, { key: `admin-audit-logs-edit-${itemId.value}`, server: true, default: () => null })
const requestError = computed(() => (error.value ? extractErrorMessage(error.value) : ''))
const item = computed(() => {
  if (!data.value) return null
  const log = 'data' in data.value ? data.value.data : data.value
  return { ...log, company_id: log.company_id || '', actor_user_id: log.actor_user_id || '', actor_role_snapshot: log.actor_role_snapshot || '', resource_id: log.resource_id || '', target_customer_id: log.target_customer_id || '', ip_address: log.ip_address || '', user_agent: log.user_agent || '', request_id: log.request_id || '', changes: stringifyJson(log.changes), metadata: stringifyJson(log.metadata) }
})

const saving = ref(false)

async function handleSubmit(values: any) {
  saving.value = true
  try {
    await api.patch(`/admin/audit-logs/${itemId.value}`, {
      body: {
        ...(values.company_id ? { company_id: values.company_id } : {}),
        ...(values.actor_user_id ? { actor_user_id: values.actor_user_id } : {}),
        ...(values.actor_role_snapshot ? { actor_role_snapshot: values.actor_role_snapshot } : {}),
        action: values.action,
        resource_type: values.resource_type,
        ...(values.resource_id ? { resource_id: values.resource_id } : {}),
        ...(values.target_customer_id ? { target_customer_id: values.target_customer_id } : {}),
        ...(values.ip_address ? { ip_address: values.ip_address } : {}),
        ...(values.user_agent ? { user_agent: values.user_agent } : {}),
        ...(values.request_id ? { request_id: values.request_id } : {}),
        changes: JSON.parse(values.changes || '{}'),
        metadata: JSON.parse(values.metadata || '{}'),
      },
    })
    toast.success(t('admin.auditLogs.messages.updateSuccess'))
    await refresh()
  } catch (err: any) {
    toast.error(err?.message || t('admin.auditLogs.messages.updateError'))
  } finally {
    saving.value = false
  }
}
</script>
