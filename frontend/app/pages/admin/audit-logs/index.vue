<template>
  <section class="space-y-6">
    <AdminBreadcrumb :items="breadcrumbItems" />

    <div class="rounded-box border border-base-content/10 bg-base-100 p-6 shadow-md shadow-base-content/5">
      <div class="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <div class="mb-3 inline-flex items-center gap-2 rounded-field bg-primary/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.22em] text-primary">
            <span class="icon-[tabler--history] size-4"></span>
            <span>{{ $t('admin.auditLogs.title') }}</span>
          </div>
          <h1 class="text-3xl font-semibold tracking-tight text-base-content">{{ $t('admin.auditLogs.title') }}</h1>
          <p class="mt-2 max-w-3xl text-sm leading-relaxed text-base-content/60">{{ $t('admin.auditLogs.description') }}</p>
        </div>

        <div class="flex flex-wrap gap-2">
          <NuxtLink :to="localePath('/admin/audit-logs/new')" class="btn btn-primary">
            <span class="icon-[tabler--plus] size-4.5"></span>
            {{ $t('admin.auditLogs.newTitle') }}
          </NuxtLink>
          <button type="button" class="btn btn-soft" :disabled="pending" @click="refresh()">
            <span class="icon-[tabler--refresh] size-4.5"></span>
            {{ $t('admin.common.refresh') }}
          </button>
        </div>
      </div>
    </div>

    <AppDataTable
      v-model:row-selection="rowSelection"
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
      enable-selection
      mode="server"
      :sorting="sorting"
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

      <template #bulk-actions>
        <button
          type="button"
          class="btn btn-error btn-sm"
          :disabled="bulkRemovalPending"
          data-testid="bulk-delete"
          @click="requestBulkAuditRemoval"
        >
          <span class="icon-[tabler--trash] size-4"></span>
          {{ $t('admin.common.bulkActions.deleteSelected') }}
        </button>
      </template>
    </AppDataTable>

    <ConfirmDialog
      v-if="removal"
      :open="true"
      :pending="removalPending"
      :title="removal.title"
      :message="removal.message"
      :confirm-label="removal.confirmLabel"
      :cancel-label="removal.cancelLabel"
      @confirm="onRemovalConfirmed"
      @cancel="cancelRemoval"
    />

    <ConfirmDialog
      v-if="bulkRemoval"
      :open="true"
      :pending="bulkRemovalPending"
      :title="bulkRemoval.title"
      :message="bulkRemoval.message"
      :confirm-label="bulkRemoval.confirmLabel"
      :cancel-label="bulkRemoval.cancelLabel"
      @confirm="onBulkRemovalConfirmed"
      @cancel="cancelBulkRemoval"
    />
  </section>
</template>

<script setup lang="ts">

// Unique component name: NuxtPage collects keepalive page names globally, so
// two pages sharing the inferred filename name break KeepAlive routing.
defineOptions({ name: 'AdminAuditLogsIndexPage' })

import { computed, h, ref, resolveComponent } from 'vue'
import type { RowSelectionState } from '@tanstack/vue-table'
import { useAdminResource } from '~/utils/admin-resource-helpers'
import { adminDeleteButton, formatDateTime } from '~/utils/admin-ui'
import ConfirmDialog from '~/components/admin/ConfirmDialog.vue'
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
} = useTablePagination<AuditLogRow>(() => ({
  key: 'admin-audit-logs-index',
  url: '/admin/audit-logs',
}))

// The selection is page scoped: the bar above the table acts on the rows the
// screen shows, so the ids of the current page are the ones it deletes.
const rowSelection = ref<RowSelectionState>({})

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
      return h('div', { class: 'flex justify-end gap-1' }, [
        h(
          NuxtLink,
          {
            to: localePath(`/admin/audit-logs/${rowId}`),
            class: 'btn btn-circle btn-text btn-sm',
            'aria-label': t('admin.common.view'),
            title: t('admin.common.view'),
            prefetch: false,
          },
          { default: () => h('span', { class: 'icon-[tabler--eye] size-5' }) },
        ),
        adminDeleteButton({
          pending: removalPending.value && removal.value?.item?.id === rowId,
          onClick: () => requestAuditRemoval(row),
        }),
      ])
    },
    meta: { align: 'right' },
  },
])

const {
  requestRemoval,
  confirmRemoval,
  cancelRemoval,
  removal,
  removalPending,
  requestBulkRemoval,
  confirmBulkRemoval,
  cancelBulkRemoval,
  bulkRemoval,
  bulkRemovalPending,
} = useAdminResource('audit-logs')

const selectedAuditLogs = computed(() =>
  items.value.filter((item) => rowSelection.value[String(item.id)] === true),
)

function requestAuditRemoval(item: AuditLogRow) {
  if (!item.id) return
  requestRemoval(item, {
    title: t('admin.auditLogs.dialog.deleteTitle'),
    message: t('admin.auditLogs.messages.confirmDelete'),
    successMessage: t('admin.auditLogs.messages.deleteSuccess'),
    errorKey: 'admin.auditLogs.messages.deleteError',
    deleteEndpoint: `/admin/audit-logs/${item.id}`,
  })
}

async function onRemovalConfirmed() {
  const removed = await confirmRemoval()
  if (removed) await refresh()
}

function requestBulkAuditRemoval() {
  const selected = selectedAuditLogs.value
  if (selected.length === 0) return
  requestBulkRemoval(selected, {
    title: t('admin.auditLogs.dialog.deleteTitle'),
    message: t('admin.auditLogs.messages.confirmBulkDelete', { count: selected.length }),
    successKey: 'admin.auditLogs.messages.bulkDeleteSuccess',
    partialKey: 'admin.auditLogs.messages.bulkDeletePartial',
    errorKey: 'admin.auditLogs.messages.deleteError',
  })
}

async function onBulkRemovalConfirmed() {
  const result = await confirmBulkRemoval()
  if (result.deleted > 0) await refresh()
}

function onRowClick(row: AuditLogRow) {
  if (!row?.id) return
  navigateTo(localePath(`/admin/audit-logs/${row.id}`))
}
</script>