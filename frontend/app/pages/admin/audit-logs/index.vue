<template>
  <AdminPageShell
    icon="history"
    title-key="admin.auditLogs.title"
    description-key="admin.auditLogs.description"
    :breadcrumb-items="breadcrumbItems"
  >
    <template #header-actions>
      <NuxtLink :to="localePath('/admin/audit-logs/new')" class="btn btn-primary">
        <span class="icon-[tabler--plus] size-4.5"></span>
        {{ $t('admin.auditLogs.newTitle') }}
      </NuxtLink>
      <button type="button" class="btn btn-soft" :disabled="pending" @click="refresh()">
        <span class="icon-[tabler--refresh] size-4.5"></span>
        {{ $t('admin.common.refresh') }}
      </button>
    </template>

    <AppDataTable
      :data="items"
      :columns="columns"
      :row-id-key="'id'"
      :search-placeholder="$t('admin.auditLogs.searchPlaceholder')"
      :search-label="$t('admin.common.search')"
      :total-label="$t('admin.common.total')"
      :total="paginationMeta.total"
      :page="pagination.page"
      :page-size="pagination.perPage"
      :page-count="paginationMeta.totalPages"
      :page-sizes="pageSizes"
      :loading="pending"
      :error="requestError"
      :empty-label="$t('admin.auditLogs.empty')"
      :loading-label="$t('admin.common.loadingRecords')"
      :show-refresh="false"
      :height="540"
      :enable-sorting="true"
      :enable-global-filter="true"
      mode="server"
      :sorting="sorting"
      @update:search="setSearch"
      @row-click="onRowClick"
      @update:page="setPage"
      @update:page-size="setPageSize"
      @update:sorting="setSorting"
    >
      <template #footer>
        <div class="flex flex-col gap-3 rounded-box border border-base-content/10 bg-base-200/40 px-4 py-3 text-xs text-base-content/60 lg:flex-row lg:items-center lg:justify-between">
          <div class="flex flex-wrap items-center gap-3">
            <span class="font-semibold">{{ $t('admin.common.total') }}: {{ paginationMeta.total }}</span>
            <span class="badge badge-soft badge-sm">{{ $t('admin.common.endpoint') }}: /admin/audit-logs</span>
          </div>
        </div>
      </template>
    </AppDataTable>
  </AdminPageShell>
</template>

<script setup lang="ts">

// Unique component name: NuxtPage collects keepalive page names globally, so
// two pages sharing the inferred filename name break KeepAlive routing.
defineOptions({ name: 'AdminAuditLogsIndexPage' })

import { computed, h, resolveComponent } from 'vue'
import { formatDateTime } from '~/utils/format-date'
import type { DataTableColumn } from '~/types/data-table'

definePageMeta({ layout: 'admin', keepalive: true })

const { t } = useI18n()
const localePath = useLocalePath()

const breadcrumbItems = computed(() => [
  { label: t('admin.common.dashboard'), to: localePath('/admin/dashboard') },
  { label: t('admin.auditLogs.title') },
])

const lookup = useAdminLookup()
await lookup.load('users')

interface AuditLogRow {
  id: string
  action: string
  resource_type: string
  actor_user_id?: string
  created_at?: string
}

const {
  pagination,
  paginationMeta,
  pageSizes,
  items,
  pending,
  requestError,
  refresh,
  setPage,
  setPageSize,
  sorting,
  setSorting,
  setSearch,
} = useTablePagination<AuditLogRow>(() => ({
  key: 'admin-audit-logs-index',
  url: '/admin/audit-logs',
}))

const columns = computed<DataTableColumn<AuditLogRow>[]>(() => [
  {
    id: 'action',
    accessorKey: 'action',
    header: () => t('admin.auditLogs.table.action'),
    cell: (info) => info.getValue(),
    meta: { align: 'left' },
  },
  {
    id: 'resource_type',
    accessorKey: 'resource_type',
    header: () => t('admin.auditLogs.table.resource'),
    cell: (info) => info.getValue(),
    meta: { align: 'left' },
  },
  {
    id: 'actor_user_id',
    accessorKey: 'actor_user_id',
    header: () => t('admin.auditLogs.table.actor'),
    cell: (info) => {
      const row = info.row.original as AuditLogRow
      return lookup.resolveLabel('users', row.actor_user_id)
    },
    meta: { align: 'left' },
  },
  {
    id: 'created_at',
    accessorKey: 'created_at',
    header: () => t('admin.auditLogs.table.createdAt'),
    cell: (info) => formatDateTime(info.getValue() as string | undefined),
    meta: { align: 'right' },
  },
  {
    id: 'actions',
    header: () => t('admin.common.actions'),
    enableSorting: false,
    cell: (info) => {
      const row = info.row.original as AuditLogRow
      const rowId = row?.id
      const NuxtLink = resolveComponent('NuxtLink')
      if (!rowId) return null
      return h(
        NuxtLink,
        {
          to: localePath(`/admin/audit-logs/${rowId}`),
          class: 'btn btn-circle btn-text btn-sm',
          'aria-label': t('admin.common.view'),
          title: t('admin.common.view'),
          prefetch: false,
        },
        { default: () => h('span', { class: 'icon-[tabler--eye] size-5' }) },
      )
    },
    meta: { align: 'right' },
  },
])

// The audit trail is append only and hash chained: the API exposes no
// deletion, and erasing an entry would break the chain of the rows that
// follow it. This screen therefore reads only, and the row action just
// opens the entry.

function onRowClick(row: AuditLogRow) {
  if (!row?.id) return
  navigateTo(localePath(`/admin/audit-logs/${row.id}`))
}
</script>