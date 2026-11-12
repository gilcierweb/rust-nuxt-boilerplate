<template>
  <section class="space-y-6">
    <AdminBreadcrumb :items="breadcrumbItems" />

    <div class="rounded-box border border-base-content/10 bg-base-100 p-6 shadow-md shadow-base-content/5">
      <div class="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <div class="mb-3 inline-flex items-center gap-2 rounded-field bg-primary/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.22em] text-primary">
            <span class="icon-[tabler--key] size-4"></span>
            <span>{{ $t('admin.permissions.title') }}</span>
          </div>
          <h1 class="text-3xl font-semibold tracking-tight text-base-content">
            {{ $t('admin.permissions.title') }}
          </h1>
          <p class="mt-2 max-w-3xl text-sm leading-relaxed text-base-content/60">
            {{ $t('admin.permissions.subtitle') }}
          </p>
        </div>

        <div class="flex flex-wrap gap-2">
          <NuxtLink :to="localePath('/admin/permissions/new')" class="btn btn-primary">
            <span class="icon-[tabler--plus] size-4.5"></span>
            {{ $t('admin.permissions.newTitle') }}
          </NuxtLink>
          <button type="button" class="btn btn-soft" :disabled="pending" @click="refresh()">
            <span class="icon-[tabler--refresh] size-4.5"></span>
            {{ $t('admin.common.refresh') }}
          </button>
        </div>
      </div>
    </div>

    <AppDataTable
      :data="items"
      :columns="columns"
      row-id-key="id"
      :search-placeholder="$t('admin.permissions.searchPlaceholder')"
      :total-label="$t('admin.common.total')"
      :total="paginationMeta.total"
      :page="pagination.page"
      :page-size="pagination.perPage"
      :page-count="paginationMeta.totalPages"
      :page-sizes="pageSizes"
      :loading="pending"
      :error="requestError"
      :empty-label="$t('admin.permissions.empty.title')"
      :loading-label="$t('admin.common.loadingRecords')"
      :show-refresh="false"
      :height="540"
      :enable-sorting="true"
      :enable-global-filter="true"
      mode="server"
      :sorting="sorting"
      @update:search="setSearch"
      @update:page="setPage"
      @update:page-size="setPageSize"
      @update:sorting="setSorting"
    >
      <template #footer>
        <div class="flex flex-col gap-3 rounded-box border border-base-content/10 bg-base-200/40 px-4 py-3 text-xs text-base-content/60 lg:flex-row lg:items-center lg:justify-between">
          <div class="flex flex-wrap items-center gap-3">
            <span class="font-semibold">{{ $t('admin.common.total') }}: {{ paginationMeta.total }}</span>
            <span class="badge badge-soft badge-sm">{{ $t('admin.common.endpoint') }}: /admin/permissions</span>
          </div>
        </div>
      </template>
    </AppDataTable>

    <div class="rounded-box border border-base-content/10 bg-base-100 p-6 shadow-md shadow-base-content/5">
      <div class="mb-4">
        <h2 class="text-lg font-semibold text-base-content">
          {{ $t('admin.permissions.matrix.title') }}
        </h2>
        <p class="mt-1 text-sm text-base-content/60">
          {{ $t('admin.permissions.matrix.description') }}
        </p>
      </div>

      <AppEmptyState
        v-if="items.length === 0"
        :title="$t('admin.permissions.empty.title')"
        :description="$t('admin.permissions.empty.description')"
      />
      <AdminPermissionMatrix v-else :roles="matrixRoles" :permissions="items" />
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, h } from 'vue'
import { useTablePagination } from '~/composables/useTablePagination'
import { useApiFetch } from '~/composables/useApi'
import { normalizeResourceResponse } from '~/utils/admin-resources'
import { formatDateTime } from '~/utils/admin-ui'
import { splitCode } from '~/utils/rbac'
import { toPermissionTableRow } from '~/utils/permissions-map'
import type { DataTableColumn } from '~/types/data-table'
import type { Permission, Role } from '~/types/rbac'

definePageMeta({ layout: 'admin', keepalive: true })

const { t } = useI18n()
const localePath = useLocalePath()

const breadcrumbItems = computed(() => [
  { label: t('admin.common.dashboard'), to: localePath('/admin/dashboard') },
  { label: t('admin.permissions.title') },
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
} = useTablePagination<Permission>(() => ({
  key: 'admin-permissions-index',
  url: '/admin/permissions',
}))

// The matrix columns are the roles of the catalog, loaded once beside the page.
const rolesFetch = await useApiFetch<{ data: Role[] }>('/admin/roles', {
  key: 'admin-permissions-roles',
  server: true,
  default: () => ({ data: [], pagination: null }),
})
const matrixRoles = computed(() =>
  normalizeResourceResponse(rolesFetch.data.value).map((role) => String(role.name ?? '')),
)

const columns = computed<DataTableColumn<Permission>[]>(() => [
  {
    id: 'name',
    accessorKey: 'name',
    header: () => t('admin.permissions.table.name'),
    cell: (info) => info.getValue(),
    meta: { align: 'left' },
  },
  {
    id: 'code',
    accessorKey: 'code',
    header: () => t('admin.permissions.table.code'),
    cell: (info) => {
      const code = String(info.getValue() ?? '')
      const parts = splitCode(code)
      if (!parts) return code
      return h('div', { class: 'flex flex-wrap gap-1' }, [
        h('span', { class: 'badge badge-soft badge-sm' }, parts.resource),
        h('span', { class: 'badge badge-outline badge-sm' }, parts.action),
      ])
    },
    meta: { align: 'left' },
  },
  {
    id: 'description',
    accessorKey: 'description',
    header: () => t('admin.permissions.table.description'),
    cell: (info) => info.getValue() || t('admin.permissions.table.noDescription'),
    meta: { align: 'left', truncate: true },
  },
  {
    id: 'roles',
    header: () => t('admin.permissions.table.roles'),
    enableSorting: false,
    cell: (info) => {
      const row = toPermissionTableRow(info.row.original as Permission)
      return row.roles.length > 0 ? row.roles.join(', ') : t('admin.permissions.table.noRoles')
    },
    meta: { align: 'left', truncate: true },
  },
  {
    id: 'updated_at',
    accessorKey: 'updated_at',
    header: () => t('admin.permissions.table.updated'),
    cell: (info) => formatDateTime(info.getValue() as string | undefined),
    meta: { align: 'right' },
  },
])
</script>
