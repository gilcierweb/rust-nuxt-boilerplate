<template>
  <section class="space-y-6">
    <AdminBreadcrumb :items="breadcrumbItems" />

    <div class="rounded-box border border-base-content/10 bg-base-100 p-6 shadow-md shadow-base-content/5"><div class="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between"><div><div class="mb-3 inline-flex items-center gap-2 rounded-field bg-primary/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.22em] text-primary"><span class="icon-[tabler--history] size-4"></span><span>{{ $t('admin.auditLogs.title') }}</span></div><h1 class="text-3xl font-semibold tracking-tight text-base-content">{{ $t('admin.auditLogs.title') }}</h1><p class="mt-2 max-w-3xl text-sm leading-relaxed text-base-content/60">{{ $t('admin.auditLogs.description') }}</p></div><div class="flex flex-wrap gap-2"><NuxtLink :to="localePath('/admin/audit-logs/new')" class="btn btn-primary"><span class="icon-[tabler--plus] size-4.5"></span>{{ $t('admin.auditLogs.newTitle') }}</NuxtLink><button type="button" class="btn btn-soft" :disabled="pending" @click="refresh()"><span class="icon-[tabler--refresh] size-4.5"></span>{{ $t('admin.common.refresh') }}</button></div></div></div>
    <div class="rounded-box bg-base-100 p-5 pb-2 shadow-md shadow-base-300/10"><div class="mb-5 flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between"><div class="rounded-box bg-base-200/70 px-4 py-3"><p class="text-xs font-semibold uppercase tracking-[0.18em] text-base-content/45">{{ $t('admin.common.total') }}</p><p class="mt-2 text-2xl font-semibold text-base-content">{{ filteredItems.length }}</p></div><label class="flex min-w-72 items-center gap-3 rounded-box border border-base-content/10 bg-base-200/70 px-3 py-2.5"><span class="icon-[tabler--search] size-4 text-base-content/55"></span><input v-model="search" type="search" class="w-full bg-transparent text-sm outline-none placeholder:text-base-content/45" :placeholder="$t('admin.auditLogs.searchPlaceholder')" /></label></div><div v-if="requestError" class="mb-4 rounded-box border border-error/20 bg-error/10 px-4 py-3 text-sm text-error">{{ requestError }}</div><div class="overflow-x-auto"><table class="table"><thead><tr><th>{{ $t('admin.auditLogs.table.action') }}</th><th>{{ $t('admin.auditLogs.table.resource') }}</th><th>{{ $t('admin.auditLogs.table.actor') }}</th><th>{{ $t('admin.auditLogs.table.customer') }}</th><th>{{ $t('admin.auditLogs.table.createdAt') }}</th><th class="w-36 text-right">{{ $t('admin.common.actions') }}</th></tr></thead><tbody><tr v-if="pending"><td colspan="6" class="py-10 text-center text-base-content/55"><span class="icon-[tabler--loader-2] mr-2 inline-block size-6 animate-spin"></span>{{ $t('admin.common.loadingRecords') }}</td></tr><tr v-else-if="!filteredItems.length"><td colspan="6" class="py-10 text-center text-base-content/55">{{ $t('admin.auditLogs.empty') }}</td></tr><tr v-for="item in filteredItems" :key="item.id"><td>{{ item.action }}</td><td>{{ item.resource_type }}</td><td>{{ lookup.resolveLabel('users', item.actor_user_id) }}</td><td>{{ lookup.resolveLabel('customers', item.target_customer_id) }}</td><td>{{ formatDateTime(item.created_at) }}</td><td><div class="flex justify-end gap-1.5"><NuxtLink :to="showPath(item.id)" class="btn btn-circle btn-text btn-sm"><span class="icon-[tabler--eye] size-5"></span></NuxtLink><NuxtLink :to="editPath(item.id)" class="btn btn-circle btn-text btn-sm"><span class="icon-[tabler--pencil] size-5"></span></NuxtLink><button type="button" class="btn btn-circle btn-text btn-sm text-error" :disabled="deletePendingId === item.id" @click="removeEntity(item)"><span v-if="deletePendingId === item.id" class="icon-[tabler--loader-2] size-5 animate-spin"></span><span v-else class="icon-[tabler--trash] size-5"></span></button></div></td></tr></tbody></table></div></div>
  </section>
</template>

<script setup lang="ts">
import { normalizeResourceResponse } from '~/utils/admin-resources'
import { useAdminResource } from '~/utils/admin-resource-helpers'
import { extractErrorMessage, formatDateTime } from '~/utils/admin-ui'

definePageMeta({ layout: 'admin' })

const { t } = useI18n()
const localePath = useLocalePath()

const breadcrumbItems = computed(() => [
  { label: t('admin.common.dashboard'), to: localePath('/admin/dashboard') },
  { label: t('admin.auditLogs.title') },
])

const lookup = useAdminLookup()
await Promise.all([lookup.load('users'), lookup.load('customers')])

type Row = { id: string; action: string; resource_type: string; actor_user_id?: string; target_customer_id?: string; created_at?: string }

const search = ref('')
const deletePendingId = ref<string | null>(null)
const { data, pending, error, refresh } = await useApiFetch<any>(() => '/admin/audit-logs', { key: 'admin-audit-logs-index', server: true, default: () => [] })
const requestError = computed(() => (error.value ? extractErrorMessage(error.value) : ''))
const items = computed<Row[]>(() => normalizeResourceResponse(data.value) as Row[])
const filteredItems = computed(() => {
  const query = search.value.trim().toLowerCase()
  if (!query) return items.value
  return items.value.filter((item) => JSON.stringify(item).toLowerCase().includes(query))
})

const { showPath, editPath, removeItem } = useAdminResource('audit-logs')

async function removeEntity(item: Row) {
  deletePendingId.value = item.id
  try {
    const success = await removeItem(item, {
      confirmMessage: t('admin.auditLogs.messages.confirmDelete'),
      successMessage: t('admin.auditLogs.messages.deleteSuccess'),
      errorMessage: t('admin.auditLogs.messages.deleteError'),
      deleteEndpoint: `/admin/audit-logs/${item.id}`,
    })
    if (success) await refresh()
  } finally {
    deletePendingId.value = null
  }
}
</script>
