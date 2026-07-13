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

    <div class="rounded-box bg-base-100 p-5 pb-2 shadow-md shadow-base-300/10">
      <div class="mb-5 flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <div class="rounded-box bg-base-200/70 px-4 py-3">
          <p class="text-xs font-semibold uppercase tracking-[0.18em] text-base-content/45">{{ $t('admin.common.total') }}</p>
          <p class="mt-2 text-2xl font-semibold text-base-content">{{ filteredItems.length }}</p>
        </div>

        <label class="flex min-w-72 items-center gap-3 rounded-box border border-base-content/10 bg-base-200/70 px-3 py-2.5">
          <span class="icon-[tabler--search] size-4 text-base-content/55"></span>
          <input v-model="search" type="search" class="w-full bg-transparent text-sm outline-none placeholder:text-base-content/45" :placeholder="$t('admin.users.searchPlaceholder')" />
        </label>
      </div>

      <div v-if="requestError" class="mb-4 rounded-box border border-error/20 bg-error/10 px-4 py-3 text-sm text-error">
        {{ requestError }}
      </div>

      <div class="overflow-x-auto">
        <table class="table">
          <thead>
            <tr>
              <th>{{ $t('admin.users.table.displayName') }}</th>
              <th>{{ $t('admin.users.table.email') }}</th>
              <th>{{ $t('admin.users.table.displayName') }}</th>
              <th>{{ $t('admin.users.table.email') }}</th>
              <th class="w-20 text-right">{{ $t('admin.common.actions') }}</th>
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
              <td colspan="5" class="py-10 text-center text-base-content/55">{{ $t('admin.users.empty') }}</td>
            </tr>
            <tr v-for="item in filteredItems" :key="item.id">
              <td>{{ item.display_name || '—' }}</td>
              <td>{{ item.email }}</td>
              <td>{{ item.first_name || '—' }}</td>
              <td>{{ item.last_name || '—' }}</td>
              <td class="text-right">
                <NuxtLink :to="`/admin/users/${item.id}`" class="btn btn-circle btn-text btn-sm" :aria-label="$t('admin.common.view')" :title="$t('admin.common.view')">
                  <span class="icon-[tabler--eye] size-5"></span>
                </NuxtLink>
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
import { extractErrorMessage } from '~/utils/admin-ui'

definePageMeta({
  layout: 'admin'
})

const { t } = useI18n()

const breadcrumbItems = computed(() => [
  { label: t('admin.common.dashboard'), to: '/admin/dashboard' },
  { label: t('admin.users.title') },
])

interface UserRow {
  id: string
  display_name?: string
  email: string
  first_name?: string
  last_name?: string
}

const search = ref('')
const { data, pending, error, refresh } = await useApiFetch<any>(() => '/admin/users', {
  key: 'admin-users-index',
  server: true,
  default: () => [],
})

const requestError = computed(() => (error.value ? extractErrorMessage(error.value) : ''))
const items = computed<UserRow[]>(() => normalizeResourceResponse(data.value) as UserRow[])
const filteredItems = computed(() => {
  const query = search.value.trim().toLowerCase()
  if (!query) return items.value
  return items.value.filter((item) => JSON.stringify(item).toLowerCase().includes(query))
})
</script>
