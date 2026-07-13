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
          <h1 class="text-3xl font-semibold tracking-tight text-base-content">{{ $t('admin.roles.showTitle') }}</h1>
          <p class="mt-2 max-w-3xl text-sm leading-relaxed text-base-content/60">{{ $t('admin.roles.showDescription') }}</p>
        </div>

        <div class="flex flex-wrap gap-2">
          <NuxtLink to="/admin/roles" class="btn btn-ghost">
            <span class="icon-[tabler--arrow-left] size-4.5"></span>
            {{ $t('admin.common.back') }}
          </NuxtLink>
          <NuxtLink :to="`/admin/roles/${roleId}/edit`" class="btn btn-primary">
            <span class="icon-[tabler--edit] size-4.5"></span>
            {{ $t('admin.common.edit') }}
          </NuxtLink>
        </div>
      </div>
    </div>

    <div v-if="pending" class="rounded-box border border-base-content/10 bg-base-100 p-12 shadow-md">
      <div class="flex flex-col items-center justify-center gap-4 text-base-content/55">
        <span class="icon-[tabler--loader-2] size-10 animate-spin"></span>
        <p>{{ $t('admin.common.loadingData') }}</p>
      </div>
    </div>

    <div v-else-if="error" class="rounded-box border border-error/20 bg-error/10 p-6">
      <div class="flex items-center gap-3 text-error">
        <span class="icon-[tabler--alert-circle] size-6"></span>
        <div>
          <p class="font-semibold">{{ $t('admin.common.errorLoadingData') }}</p>
          <p class="text-sm">{{ requestError }}</p>
        </div>
      </div>
      <button class="btn btn-soft mt-4" @click="refresh()">{{ $t('admin.common.tryAgain') }}</button>
    </div>

    <div v-else-if="role" class="rounded-box border border-base-content/10 bg-base-100 p-6 shadow-md shadow-base-content/5">
      <div class="grid gap-6 md:grid-cols-2">
        <div>
          <p class="text-sm font-semibold text-base-content/70">ID</p>
          <p class="mt-1 text-base-content">{{ role.id }}</p>
        </div>
        <div>
          <p class="text-sm font-semibold text-base-content/70">{{ $t('admin.roles.form.name') }}</p>
          <p class="mt-1 text-base-content">{{ role.name }}</p>
        </div>
        <div>
          <p class="text-sm font-semibold text-base-content/70">{{ $t('admin.roles.form.resource_type') }}</p>
          <p class="mt-1 text-base-content">{{ role.resource_type || '—' }}</p>
        </div>
        <div>
          <p class="text-sm font-semibold text-base-content/70">{{ $t('admin.roles.form.resource_id') }}</p>
          <p class="mt-1 text-base-content">{{ role.resource_id || '—' }}</p>
        </div>
      </div>

      <div class="grid gap-6 border-t border-base-content/10 pt-4 md:grid-cols-2">
        <div>
          <p class="text-sm font-semibold text-base-content/70">{{ $t('admin.roles.form.created_at') }}</p>
          <p class="mt-1 text-base-content">{{ formatDateTime(role.created_at) }}</p>
        </div>
        <div>
          <p class="text-sm font-semibold text-base-content/70">{{ $t('admin.roles.form.updated_at') }}</p>
          <p class="mt-1 text-base-content">{{ formatDateTime(role.updated_at) }}</p>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { extractErrorMessage, formatDateTime } from '~/utils/admin-ui'

interface Role {
  id: string
  name: string
  resource_type?: string
  resource_id?: string
  created_at?: string
  updated_at?: string
}

definePageMeta({
  layout: 'admin'
})

const { t } = useI18n()

const breadcrumbItems = computed(() => [
  { label: t('admin.common.dashboard'), to: '/admin/dashboard' },
  { label: t('admin.roles.title'), to: '/admin/roles' },
  { label: t('admin.common.details') },
])

const route = useRoute()
const roleId = computed(() => route.params.id as string)

const { data, pending, error, refresh } = await useApiFetch<Role | { data: Role }>(
  () => `/admin/roles/${roleId.value}`,
  {
    key: `admin-roles-show-${roleId.value}`,
    server: true,
    default: () => null,
  },
)

const requestError = computed(() => (error.value ? extractErrorMessage(error.value) : ''))
const role = computed(() => {
  if (!data.value) return null
  return 'data' in data.value ? data.value.data : data.value
})
</script>
