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
          <h1 class="text-3xl font-semibold tracking-tight text-base-content">{{ $t('admin.users.showTitle') }}</h1>
          <p class="mt-2 max-w-3xl text-sm leading-relaxed text-base-content/60">{{ $t('admin.users.showDescription') }}</p>
        </div>

        <div class="flex flex-wrap gap-2">
          <NuxtLink :to="localePath('/admin/users')" class="btn btn-ghost">
            <span class="icon-[tabler--arrow-left] size-4.5"></span>
            Voltar
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

    <div v-else-if="requestError" class="rounded-box border border-error/20 bg-error/10 p-6">
      <div class="flex items-center gap-3 text-error">
        <span class="icon-[tabler--alert-circle] size-6"></span>
        <div>
          <p class="font-semibold">{{ $t('admin.common.errorLoadingData') }}</p>
          <p class="text-sm">{{ requestError }}</p>
        </div>
      </div>
      <button class="btn btn-soft mt-4" @click="refresh()">{{ $t('admin.common.tryAgain') }}</button>
    </div>

    <div v-else-if="!user" class="rounded-box border border-warning/20 bg-warning/10 p-6 text-warning">
      Usuário não encontrado.
    </div>

    <div v-else class="rounded-box border border-base-content/10 bg-base-100 p-6 shadow-md shadow-base-content/5">
      <div class="grid gap-6 md:grid-cols-2">
        <div>
          <p class="text-sm font-semibold text-base-content/70">ID</p>
          <p class="mt-1 break-all text-base-content">{{ user.id }}</p>
        </div>
        <div>
          <p class="text-sm font-semibold text-base-content/70">{{ $t('admin.account.fields.email') }}</p>
          <p class="mt-1 text-base-content">{{ user.email || '—' }}</p>
        </div>
        <div>
          <p class="text-sm font-semibold text-base-content/70">{{ $t('admin.users.table.displayName') }}</p>
          <p class="mt-1 text-base-content">{{ user.display_name || '—' }}</p>
        </div>
        <div>
          <p class="text-sm font-semibold text-base-content/70">{{ $t('admin.users.table.displayName') }}</p>
          <p class="mt-1 text-base-content">{{ user.full_name || '—' }}</p>
        </div>
        <div>
          <p class="text-sm font-semibold text-base-content/70">{{ $t('admin.users.table.displayName') }}</p>
          <p class="mt-1 text-base-content">{{ user.first_name || '—' }}</p>
        </div>
        <div>
          <p class="text-sm font-semibold text-base-content/70">{{ $t('admin.users.table.displayName') }}</p>
          <p class="mt-1 text-base-content">{{ user.last_name || '—' }}</p>
        </div>
        <div>
          <p class="text-sm font-semibold text-base-content/70">{{ $t('admin.users.table.displayName') }}</p>
          <p class="mt-1 text-base-content">{{ user.nickname || '—' }}</p>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { extractErrorMessage } from '~/utils/admin-ui'
import { normalizeResourceResponse } from '~/utils/admin-resources'

definePageMeta({
  layout: 'admin',
})

type UserRow = {
  id: string
  email?: string
  display_name?: string
  first_name?: string
  last_name?: string
  full_name?: string
  nickname?: string
}

const { t } = useI18n()
const localePath = useLocalePath()

const breadcrumbItems = computed(() => [
  { label: t('admin.common.dashboard'), to: localePath('/admin/dashboard') },
  { label: t('admin.users.title'), to: localePath('/admin/users') },
  { label: t('admin.common.details') },
])

const route = useRoute()
const userId = computed(() => String(route.params.id || ''))

const { data, pending, error, refresh } = await useApiFetch<any>(() => '/admin/users', {
  key: `admin-users-show-${userId.value}`,
  server: true,
  default: () => [],
})

const requestError = computed(() => (error.value ? extractErrorMessage(error.value) : ''))
const items = computed<UserRow[]>(() => normalizeResourceResponse(data.value) as UserRow[])
const user = computed(() => items.value.find((item) => item.id === userId.value) || null)
</script>
