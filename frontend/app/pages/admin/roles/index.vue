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

    <div class="rounded-box bg-base-100 p-5 pb-2 shadow-md shadow-base-300/10">
      <div class="mb-5 flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <div class="rounded-box bg-base-200/70 px-4 py-3">
          <p class="text-xs font-semibold uppercase tracking-[0.18em] text-base-content/45">{{ $t('admin.common.total') }}</p>
          <p class="mt-2 text-2xl font-semibold text-base-content">{{ filteredItems.length }}</p>
        </div>

        <label class="flex min-w-72 items-center gap-3 rounded-box border border-base-content/10 bg-base-200/70 px-3 py-2.5">
          <span class="icon-[tabler--search] size-4 text-base-content/55"></span>
          <input v-model="search" type="search" class="w-full bg-transparent text-sm outline-none placeholder:text-base-content/45" :placeholder="$t('admin.roles.searchPlaceholder')" />
        </label>
      </div>

      <div v-if="requestError" class="mb-4 rounded-box border border-error/20 bg-error/10 px-4 py-3 text-sm text-error">
        {{ requestError }}
      </div>

      <div class="overflow-x-auto">
        <table class="table">
          <thead>
            <tr>
              <th>{{ $t('admin.roles.table.name') }}</th>
              <th>{{ $t('admin.roles.table.resourceType') }}</th>
              <th>{{ $t('admin.roles.form.resource_id') }}</th>
              <th>{{ $t('admin.roles.form.updated_at') }}</th>
              <th class="w-36 text-right">{{ $t('admin.common.actions') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-if="pending">
              <td colspan="5" class="py-10 text-center text-base-content/55">
                <span class="icon-[tabler--loader-2] mr-2 inline-block size-6 animate-spin"></span>
                {{ $t('admin.common.loadingRecords') }}
              </td>
            </tr>
            <tr v-else-if="!filteredItems.length">
              <td colspan="5" class="py-10 text-center text-base-content/55">{{ $t('admin.roles.empty') }}</td>
            </tr>
            <tr v-for="item in filteredItems" :key="item.id">
              <td>{{ item.name }}</td>
              <td>{{ item.resource_type || '—' }}</td>
              <td class="max-w-56 truncate">{{ item.resource_id || '—' }}</td>
              <td>{{ formatDateTime(item.updated_at) }}</td>
              <td>
                <div class="flex justify-end gap-1.5">
                  <NuxtLink :to="showPath(item.id)" class="btn btn-circle btn-text btn-sm" :aria-label="$t('admin.common.view')" :title="$t('admin.common.view')">
                    <span class="icon-[tabler--eye] size-5"></span>
                  </NuxtLink>
                  <NuxtLink :to="editPath(item.id)" class="btn btn-circle btn-text btn-sm" :aria-label="$t('admin.common.edit')" :title="$t('admin.common.edit')">
                    <span class="icon-[tabler--pencil] size-5"></span>
                  </NuxtLink>
                  <button type="button" class="btn btn-circle btn-text btn-sm text-error" :disabled="deletePendingId === item.id" @click="removeRole(item)">
                    <span v-if="deletePendingId === item.id" class="icon-[tabler--loader-2] size-5 animate-spin"></span>
                    <span v-else class="icon-[tabler--trash] size-5"></span>
                  </button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { normalizeResourceResponse } from '~/utils/admin-resources'
import { useAdminResource } from '~/utils/admin-resource-helpers'
import { extractErrorMessage, formatDateTime } from '~/utils/admin-ui'

definePageMeta({ layout: 'admin' })

const { t } = useI18n()
const localePath = useLocalePath()

type RoleRow = {
  id: string
  name: string
  resource_type?: string
  resource_id?: string
  updated_at?: string
  created_at?: string
}

const search = ref('')
const deletePendingId = ref<string | null>(null)
const basePath = '/admin/roles'

const breadcrumbItems = computed(() => [
  { label: t('admin.common.dashboard'), to: localePath('/admin/dashboard') },
  { label: t('admin.roles.title') },
])

const { data, pending, error, refresh } = await useApiFetch<any>(() => '/admin/roles', {
  key: 'admin-roles-index',
  server: true,
  default: () => [],
})

const requestError = computed(() => (error.value ? extractErrorMessage(error.value) : ''))
const items = computed<RoleRow[]>(() => {
  const normalized = normalizeResourceResponse(data.value) as RoleRow[]
  return [...normalized].sort((left, right) => new Date(right.updated_at || right.created_at || 0).getTime() - new Date(left.updated_at || left.created_at || 0).getTime())
})
const filteredItems = computed(() => {
  const query = search.value.trim().toLowerCase()
  if (!query) return items.value
  return items.value.filter((item) => JSON.stringify(item).toLowerCase().includes(query))
})

const { showPath, editPath, removeItem } = useAdminResource('roles')

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
</script>
