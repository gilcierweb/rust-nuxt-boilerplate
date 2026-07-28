<template>
  <section class="space-y-6">
    <AdminBreadcrumb :items="breadcrumbItems" />

    <div class="rounded-box border border-base-content/10 bg-base-100 p-6 shadow-md shadow-base-content/5">
      <div class="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <div class="mb-3 inline-flex items-center gap-2 rounded-field bg-primary/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.22em] text-primary">
            <span class="icon-[tabler--shield] size-4"></span>
            <span>{{ $t('admin.roles.title') }}</span>
          </div>
          <h1 class="text-3xl font-semibold tracking-tight text-base-content">{{ $t('admin.roles.title') }}</h1>
          <p class="mt-2 max-w-3xl text-sm leading-relaxed text-base-content/60">{{ $t('admin.roles.description') }}</p>
        </div>

        <div class="flex flex-wrap gap-2">
          <NuxtLink :to="localePath('/admin/roles/new')" class="btn btn-primary">
            <span class="icon-[tabler--plus] size-4.5"></span>
            {{ $t('admin.roles.newTitle') }}
          </NuxtLink>
          <button type="button" class="btn btn-soft" :disabled="pending" @click="refresh()">
            <span class="icon-[tabler--refresh] size-4.5"></span>
            {{ $t('admin.common.refresh') }}
          </button>
        </div>
      </div>
    </div>

    <AppDataTable
      :data="sortedItems"
      :columns="columns"
      row-id-key="id"
      :search-placeholder="$t('admin.roles.searchPlaceholder')"
      :total-label="$t('admin.common.total')"
      :total="sortedItems.length"
      :loading="pending"
      :error="requestError"
      :empty-label="$t('admin.roles.empty')"
      :loading-label="$t('admin.common.loadingRecords')"
      :show-refresh="false"
      :height="540"
      :enable-sorting="true"
      :enable-global-filter="true"
      mode="client"
      @row-click="onRowClick"
    >
      <template #footer>
        <div class="flex flex-col gap-3 rounded-box border border-base-content/10 bg-base-200/40 px-4 py-3 text-xs text-base-content/60 lg:flex-row lg:items-center lg:justify-between">
          <div class="flex flex-wrap items-center gap-3">
            <span class="font-semibold">{{ $t('admin.common.total') }}: {{ sortedItems.length }}</span>
            <span class="badge badge-soft badge-sm">{{ $t('admin.common.endpoint') }}: /admin/roles</span>
          </div>
        </div>
      </template>
    </AppDataTable>
  </section>
</template>

<script setup lang="ts">
import { computed, h, resolveComponent } from 'vue'
import { normalizeResourceResponse } from '~/utils/admin-resources'
import { useAdminResource } from '~/utils/admin-resource-helpers'
import { extractErrorMessage, formatDateTime } from '~/utils/admin-ui'
import type { DataTableColumn } from '~/types/data-table'

definePageMeta({ layout: 'admin' })

const { t } = useI18n()
const localePath = useLocalePath()

interface RoleRow {
  id: string
  name: string
  resource_type?: string
  resource_id?: string
  updated_at?: string
  created_at?: string
}

const deletePendingId = ref<string | null>(null)

const breadcrumbItems = computed(() => [
  { label: t('admin.common.dashboard'), to: localePath('/admin/dashboard') },
  { label: t('admin.roles.title') },
])

const { data, pending, error, refresh } = await useApiFetch<any>(() => '/admin/roles', { key: 'admin-roles-index', server: true, default: () => [] })
const requestError = computed(() => (error.value ? extractErrorMessage(error.value) : ''))
const sortedItems = computed<RoleRow[]>(() => {
  const normalized = normalizeResourceResponse(data.value) as RoleRow[]
  return [...normalized].sort((left, right) =>
    new Date(right.updated_at || right.created_at || 0).getTime() -
    new Date(left.updated_at || left.created_at || 0).getTime(),
  )
})

const columns = computed<DataTableColumn<RoleRow>[]>(() => [
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
    cell: (info) => info.getValue() || '—',
    meta: { align: 'left' },
  },
  {
    id: 'resource_id',
    accessorKey: 'resource_id',
    header: () => t('admin.roles.form.resource_id'),
    cell: (info) => info.getValue() || '—',
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
      const row = info.row.original as RoleRow
      const NuxtLink = resolveComponent('NuxtLink')
      return h('div', { class: 'flex justify-end gap-1.5' }, [
        h(
          NuxtLink,
          {
            to: localePath(`/admin/roles/${row.id}`),
            class: 'btn btn-circle btn-text btn-sm',
            'aria-label': t('admin.common.view'),
            title: t('admin.common.view'),
          },
          { default: () => h('span', { class: 'icon-[tabler--eye] size-5' }) },
        ),
        h(
          NuxtLink,
          {
            to: localePath(`/admin/roles/${row.id}/edit`),
            class: 'btn btn-circle btn-text btn-sm',
            'aria-label': t('admin.common.edit'),
            title: t('admin.common.edit'),
          },
          { default: () => h('span', { class: 'icon-[tabler--pencil] size-5' }) },
        ),
        h(
          'button',
          {
            type: 'button',
            class: 'btn btn-circle btn-text btn-sm text-error',
            disabled: deletePendingId.value === row.id,
            onClick: () => removeRole(row),
          },
          {
            default: () =>
              deletePendingId.value === row.id
                ? h('span', { class: 'icon-[tabler--loader-2] size-5 animate-spin' })
                : h('span', { class: 'icon-[tabler--trash] size-5' }),
          },
        ),
      ])
    },
    meta: { align: 'right' },
  },
])

const { removeItem } = useAdminResource('roles')

async function removeRole(item: RoleRow) {
  deletePendingId.value = item.id
  try {
    const success = await removeItem(item, {
      confirmMessage: t('admin.roles.messages.confirmDelete'),
      successMessage: t('admin.roles.messages.deleteSuccess'),
      errorMessage: t('admin.roles.messages.deleteError'),
      deleteEndpoint: `/admin/roles/${item.id}`,
    })
    if (success) await refresh()
  } finally {
    deletePendingId.value = null
  }
}

function onRowClick(row: RoleRow) {
  navigateTo(localePath(`/admin/roles/${row.id}`))
}
</script>
