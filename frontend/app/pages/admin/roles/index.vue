<template>
  <AdminPageShell
    icon="shield"
    title-key="admin.roles.title"
    description-key="admin.roles.description"
    :breadcrumb-items="breadcrumbItems"
  >
    <template #header-actions>
      <NuxtLink :to="localePath('/admin/roles/new')" class="btn btn-primary">
        <span class="icon-[tabler--plus] size-4.5"></span>
        {{ $t('admin.roles.newTitle') }}
      </NuxtLink>
      <button type="button" class="btn btn-soft" :disabled="pending" @click="refresh()">
        <span class="icon-[tabler--refresh] size-4.5"></span>
        {{ $t('admin.common.refresh') }}
      </button>
    </template>

    <AppDataTable
      v-model:row-selection="rowSelection"
      :data="items"
      :columns="columns"
      row-id-key="id"
      :search-placeholder="$t('admin.roles.searchPlaceholder')"
      :search-label="$t('admin.common.search')"
      :total-label="$t('admin.common.total')"
      :total="paginationMeta.total"
      :page="pagination.page"
      :page-size="pagination.perPage"
      :page-count="paginationMeta.totalPages"
      :page-sizes="pageSizes"
      :loading="pending"
      :error="requestError"
      :empty-label="$t('admin.roles.empty')"
      :loading-label="$t('admin.common.loadingRecords')"
      :show-refresh="false"
      :height="540"
      :enable-sorting="true"
      :enable-global-filter="true"
      enable-selection
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
            <span class="badge badge-soft badge-sm">{{ $t('admin.common.endpoint') }}: /admin/roles</span>
          </div>
        </div>
      </template>

      <template #bulk-actions>
        <button
          type="button"
          class="btn btn-error btn-sm"
          :disabled="bulkRemovalPending"
          data-testid="bulk-delete"
          @click="requestBulkRoleRemoval"
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
  </AdminPageShell>
</template>

<script setup lang="ts">

// Unique component name: NuxtPage collects keepalive page names globally, so
// two pages sharing the inferred filename name break KeepAlive routing.
defineOptions({ name: 'AdminRolesIndexPage' })

import { computed, h, ref, resolveComponent } from 'vue'
import type { RowSelectionState } from '@tanstack/vue-table'
import { useAdminResource } from '~/utils/admin-resource-helpers'
import { formatDateTime } from '~/utils/format-date'
import { adminDeleteButton } from '~/utils/admin-ui'
import ConfirmDialog from '~/components/admin/ConfirmDialog.vue'
import type { DataTableColumn } from '~/types/data-table'
import type { Role } from '~/types/rbac'

definePageMeta({ layout: 'admin', keepalive: true })

const { t } = useI18n()
const localePath = useLocalePath()

const breadcrumbItems = computed(() => [
  { label: t('admin.common.dashboard'), to: localePath('/admin/dashboard') },
  { label: t('admin.roles.title') },
])

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
} = useTablePagination<Role>(() => ({
  key: 'admin-roles-index',
  url: '/admin/roles',
}))

const columns = computed<DataTableColumn<Role>[]>(() => [
  {
    id: 'name',
    accessorKey: 'name',
    header: () => t('admin.roles.table.name'),
    cell: (info) => info.getValue(),
    meta: { align: 'left' },
  },
  {
    id: 'resource_type',
    accessorKey: 'resource_type',
    header: () => t('admin.roles.table.resourceType'),
    cell: (info) => info.getValue() || t('admin.common.emptyValue'),
    meta: { align: 'left' },
  },
  {
    id: 'resource_id',
    accessorKey: 'resource_id',
    header: () => t('admin.roles.form.resource_id'),
    cell: (info) => info.getValue() || t('admin.common.emptyValue'),
    meta: { align: 'left', truncate: true },
  },
  {
    id: 'updated_at',
    accessorKey: 'updated_at',
    header: () => t('admin.roles.form.updated_at'),
    cell: (info) => formatDateTime(info.getValue() as string | undefined),
    meta: { align: 'right' },
  },
  {
    id: 'actions',
    header: () => t('admin.common.actions'),
    enableSorting: false,
    cell: (info) => {
      const row = info.row.original as Role
      const rowId = row?.id
      const NuxtLink = resolveComponent('NuxtLink')
      if (!rowId) return null
      return h('div', { class: 'flex justify-end gap-1.5' }, [
        h(
          NuxtLink,
          {
            to: localePath(`/admin/roles/${rowId}`),
            class: 'btn btn-circle btn-text btn-sm',
            'aria-label': t('admin.common.view'),
            title: t('admin.common.view'),
            prefetch: false,
          },
          { default: () => h('span', { class: 'icon-[tabler--eye] size-5' }) },
        ),
        h(
          NuxtLink,
          {
            to: localePath(`/admin/roles/${rowId}/edit`),
            class: 'btn btn-circle btn-text btn-sm',
            'aria-label': t('admin.common.edit'),
            title: t('admin.common.edit'),
            prefetch: false,
          },
          { default: () => h('span', { class: 'icon-[tabler--pencil] size-5' }) },
        ),
        adminDeleteButton({
          pending: removalPending.value && removal.value?.item?.id === rowId,
          onClick: () => removeRole(row),
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
} = useAdminResource('roles')

// The selection is page scoped: the bar above the table acts on the rows the
// screen shows, so the ids of the current page are the ones it deletes.
const rowSelection = ref<RowSelectionState>({})
const selectedRoles = computed(() =>
  items.value.filter((role) => rowSelection.value[String(role.id)] === true),
)

function requestBulkRoleRemoval() {
  const selected = selectedRoles.value
  if (selected.length === 0) return
  requestBulkRemoval(selected, {
    title: t('admin.roles.dialog.deleteTitle'),
    message: t('admin.roles.messages.confirmBulkDelete', { count: selected.length }),
    successKey: 'admin.roles.messages.bulkDeleteSuccess',
    partialKey: 'admin.roles.messages.bulkDeletePartial',
    errorKey: 'admin.roles.messages.deleteError',
  })
}

async function onBulkRemovalConfirmed() {
  const result = await confirmBulkRemoval()
  if (result.deleted > 0) await refresh()
}

function removeRole(item: Role) {
  if (!item.id) return
  requestRemoval(item, {
    title: t('admin.roles.dialog.deleteTitle'),
    message: t('admin.roles.messages.confirmDelete'),
    successMessage: t('admin.roles.messages.deleteSuccess'),
    errorKey: 'admin.roles.messages.deleteError',
    deleteEndpoint: `/admin/roles/${item.id}`,
  })
}

async function onRemovalConfirmed() {
  const removed = await confirmRemoval()
  if (removed) await refresh()
}

function onRowClick(row: Role) {
  if (!row?.id) return
  navigateTo(localePath(`/admin/roles/${row.id}`))
}
</script>
