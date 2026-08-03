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
          <NuxtLink :to="localePath('/admin/users')" class="btn btn-ghost" :prefetch="false">
            <span class="icon-[tabler--arrow-left] size-4.5"></span>
            {{ $t('admin.common.back') }}
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
          <p class="text-sm">{{ error }}</p>
        </div>
      </div>
      <button class="btn btn-soft mt-4" @click="refresh()">{{ $t('admin.common.tryAgain') }}</button>
    </div>

    <div v-else-if="!item" class="rounded-box border border-warning/20 bg-warning/10 p-6 text-warning">
      {{ $t('admin.users.notFound') }}
    </div>

    <div v-else class="rounded-box border border-base-content/10 bg-base-100 p-6 shadow-md shadow-base-content/5">
      <div class="grid gap-6 md:grid-cols-2">
        <div>
          <p class="text-sm font-semibold text-base-content/70">ID</p>
          <p class="mt-1 break-all text-base-content">{{ item.id }}</p>
        </div>
        <div>
          <p class="text-sm font-semibold text-base-content/70">{{ $t('admin.users.fields.email') }}</p>
          <p class="mt-1 text-base-content">{{ item.email || '—' }}</p>
        </div>
        <div>
          <p class="text-sm font-semibold text-base-content/70">{{ $t('admin.users.fields.displayName') }}</p>
          <p class="mt-1 text-base-content">{{ item.display_name || '—' }}</p>
        </div>
        <div>
          <p class="text-sm font-semibold text-base-content/70">{{ $t('admin.users.fields.fullName') }}</p>
          <p class="mt-1 text-base-content">{{ item.full_name || '—' }}</p>
        </div>
        <div>
          <p class="text-sm font-semibold text-base-content/70">{{ $t('admin.users.fields.firstName') }}</p>
          <p class="mt-1 text-base-content">{{ item.first_name || '—' }}</p>
        </div>
        <div>
          <p class="text-sm font-semibold text-base-content/70">{{ $t('admin.users.fields.lastName') }}</p>
          <p class="mt-1 text-base-content">{{ item.last_name || '—' }}</p>
        </div>
        <div>
          <p class="text-sm font-semibold text-base-content/70">{{ $t('admin.users.fields.nickname') }}</p>
          <p class="mt-1 text-base-content">{{ item.nickname || '—' }}</p>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
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

const { itemId, item, pending, error, refresh } = useAdminResourceItem<UserRow>('users', {
  keyPrefix: 'admin-users-show',
})
</script>
