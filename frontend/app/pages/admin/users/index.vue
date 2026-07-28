<template>
  <section class="space-y-6">
    <AdminBreadcrumb :items="breadcrumbItems" />

    <div class="rounded-box border border-base-content/10 bg-base-100 p-6 shadow-md shadow-base-content/5">
      <div class="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <div class="mb-3 inline-flex items-center gap-2 rounded-field bg-primary/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.22em] text-primary">
            <span class="icon-[tabler--user] size-4"></span>
            <span>{{ $t('admin.users.title') }}</span>
          </div>
          <h1 class="text-3xl font-semibold tracking-tight text-base-content">{{ $t('admin.users.title') }}</h1>
          <p class="mt-2 max-w-3xl text-sm leading-relaxed text-base-content/60">{{ $t('admin.users.description') }}</p>
        </div>

        <div class="flex flex-wrap gap-2">
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
      :row-id-key="'id'"
      :search-placeholder="$t('admin.users.searchPlaceholder')"
      :total-label="$t('admin.common.total')"
      :total="paginationMeta.total"
      :page="pagination.page"
      :page-size="pagination.perPage"
      :page-count="paginationMeta.totalPages"
      :page-sizes="pageSizes"
      :loading="pending"
      :error="requestError"
      :empty-label="$t('admin.users.empty')"
      :loading-label="$t('admin.common.loadingRecords')"
      :show-refresh="false"
      :height="540"
      :enable-sorting="true"
      :enable-global-filter="true"
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
            <span class="badge badge-soft badge-sm">{{ $t('admin.common.endpoint') }}: /admin/users</span>
          </div>
        </div>
      </template>
    </AppDataTable>
  </section>
</template>

<script setup lang="ts">
import { computed, h, resolveComponent } from 'vue'
import type { DataTableColumn } from '~/types/data-table'

definePageMeta({ layout: 'admin', keepalive: true })

const { t } = useI18n()
const localePath = useLocalePath()

const breadcrumbItems = computed(() => [
  { label: t('admin.common.dashboard'), to: localePath('/admin/dashboard') },
  { label: t('admin.users.title') },
])

interface UserRow {
  id: string
  display_name?: string
  email: string
  first_name?: string
  last_name?: string
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
} = useTablePagination<UserRow>(() => ({
  key: 'admin-users-index',
  url: '/admin/users',
}))

const columns = computed<DataTableColumn<UserRow>[]>(() => [
  {
    id: 'display_name',
    accessorKey: 'display_name',
    header: () => t('admin.users.table.displayName'),
    cell: (info) => info.getValue() || '—',
    meta: { align: 'left' },
  },
  {
    id: 'email',
    accessorKey: 'email',
    header: () => t('admin.users.table.email'),
    cell: (info) => info.getValue(),
    meta: { align: 'left' },
  },
  {
    id: 'first_name',
    accessorKey: 'first_name',
    header: () => t('admin.users.table.firstName'),
    cell: (info) => info.getValue() || '—',
    meta: { align: 'left' },
  },
  {
    id: 'last_name',
    accessorKey: 'last_name',
    header: () => t('admin.users.table.lastName'),
    cell: (info) => info.getValue() || '—',
    meta: { align: 'left' },
  },
  {
    id: 'actions',
    header: () => t('admin.common.actions'),
    enableSorting: false,
    cell: (info) => {
      const row = info.row.original as UserRow
      const rowId = row?.id
      const NuxtLink = resolveComponent('NuxtLink')
      if (!rowId) return null
      return h('div', { class: 'flex justify-end gap-1' }, [
        h(
          NuxtLink,
          {
            to: localePath(`/admin/users/${rowId}`),
            class: 'btn btn-circle btn-text btn-sm',
            'aria-label': t('admin.common.view'),
            title: t('admin.common.view'),
            prefetch: false,
          },
          { default: () => h('span', { class: 'icon-[tabler--eye] size-5' }) },
        ),
      ])
    },
    meta: { align: 'right' },
  },
])

function onRowClick(row: UserRow) {
  if (!row?.id) return
  navigateTo(localePath(`/admin/users/${row.id}`))
}
</script>