<template>
  <section class="space-y-6">
    <AdminBreadcrumb :items="breadcrumbItems" />
    <div class="card shadow-base-300/10 shadow-md"><div class="card-body"><div class="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between"><div><div class="mb-3 inline-flex items-center gap-2 rounded-field bg-primary/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.22em] text-primary"><span class="icon-[tabler--history] size-4"></span><span>{{ $t('admin.auditLogs.title') }}</span></div><h1 class="card-title text-3xl font-semibold tracking-tight text-base-content">{{ $t('admin.auditLogs.newTitle') }}</h1><p class="mt-2 max-w-3xl text-sm leading-relaxed text-base-content/60">{{ $t('admin.auditLogs.newDescription') }}</p></div><div class="card-actions flex flex-wrap gap-2"><NuxtLink to="/admin/audit-logs" class="btn btn-ghost"><span class="icon-[tabler--arrow-left] size-4.5"></span>{{ $t('admin.common.back') }}</NuxtLink></div></div></div></div>
    <div class="card shadow-base-300/10 shadow-md"><div class="card-body"><AuditLogsForm mode="create" :initial-values="initialValues" :saving="saving" @submit="handleSubmit" /></div></div>
  </section>
</template>

<script setup lang="ts">
import AuditLogsForm from '~/components/admin/audit-logs/AuditLogsForm.vue'

definePageMeta({ layout: 'admin' })

const { t } = useI18n()

const breadcrumbItems = computed(() => [
  { label: t('admin.common.dashboard'), to: '/admin/dashboard' },
  { label: t('admin.auditLogs.title'), to: '/admin/audit-logs' },
  { label: t('admin.common.new') },
])

const api = useApi()
const toast = useToast()

const initialValues = {
  company_id: '',
  actor_user_id: '',
  actor_role_snapshot: '',
  action: '',
  resource_type: '',
  resource_id: '',
  target_customer_id: '',
  ip_address: '',
  user_agent: '',
  request_id: '',
  changes: '{}',
  metadata: '{}',
}

const saving = ref(false)

async function handleSubmit(values: typeof initialValues) {
  saving.value = true
  try {
    await api.post('/admin/audit-logs', {
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
    toast.success(t('admin.auditLogs.messages.createSuccess'))
    await navigateTo('/admin/audit-logs')
  } catch (error: any) {
    toast.error(error?.message || t('admin.auditLogs.messages.createError'))
  } finally {
    saving.value = false
  }
}
</script>
