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
      <div class="flex items-start gap-3 text-base-content">
        <span class="icon-[tabler--alert-circle] mt-0.5 size-6 shrink-0 text-error"></span>
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
          <p class="mt-1 text-base-content">{{ item.email || $t('admin.common.emptyValue') }}</p>
        </div>
        <div>
          <p class="text-sm font-semibold text-base-content/70">{{ $t('admin.users.fields.displayName') }}</p>
          <p class="mt-1 text-base-content">{{ item.display_name || $t('admin.common.emptyValue') }}</p>
        </div>
        <div>
          <p class="text-sm font-semibold text-base-content/70">{{ $t('admin.users.fields.fullName') }}</p>
          <p class="mt-1 text-base-content">{{ item.full_name || $t('admin.common.emptyValue') }}</p>
        </div>
        <div>
          <p class="text-sm font-semibold text-base-content/70">{{ $t('admin.users.fields.firstName') }}</p>
          <p class="mt-1 text-base-content">{{ item.first_name || $t('admin.common.emptyValue') }}</p>
        </div>
        <div>
          <p class="text-sm font-semibold text-base-content/70">{{ $t('admin.users.fields.lastName') }}</p>
          <p class="mt-1 text-base-content">{{ item.last_name || $t('admin.common.emptyValue') }}</p>
        </div>
        <div>
          <p class="text-sm font-semibold text-base-content/70">{{ $t('admin.users.fields.nickname') }}</p>
          <p class="mt-1 text-base-content">{{ item.nickname || $t('admin.common.emptyValue') }}</p>
        </div>
      </div>

      <div class="rounded-box border border-base-content/10 bg-base-100 p-6 shadow-md shadow-base-content/5">
        <h2 class="text-lg font-semibold text-base-content">
          {{ $t('admin.users.roles.card_title') }}
        </h2>
        <p class="mt-1 text-sm text-base-content/60">
          {{ $t('admin.users.roles.card_description') }}
        </p>

        <div v-if="rolesPending" class="flex items-center justify-center gap-3 py-12 text-base-content/55">
          <span class="icon-[tabler--loader-2] size-8 animate-spin"></span>
          <p>{{ $t('admin.common.loadingData') }}</p>
        </div>

        <AdminRoleSelector
          v-else
          class="mt-4"
          :roles="availableRoles"
          :selected-ids="assignedIds"
          :error-message="rolesError"
          :saving="saving"
          @save="handleSave"
        />
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { useAdminRbac } from '~/composables/useAdminRbac'
import { normalizeResourceResponse } from '~/utils/admin-resources'
import { mapAdminError } from '~/utils/admin-errors'
import AdminRoleSelector from '~/components/admin/AdminRoleSelector.vue'
import type { Role } from '~/types/rbac'

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
const toast = useToast()
const { userRoles, roles } = useAdminRbac()

const breadcrumbItems = computed(() => [
  { label: t('admin.common.dashboard'), to: localePath('/admin/dashboard') },
  { label: t('admin.users.title'), to: localePath('/admin/users') },
  { label: t('admin.common.details') },
])

const { itemId, item, pending, error, refresh } = useAdminResourceItem<UserRow>('users', {
  keyPrefix: 'admin-users-show',
})

// The roles of the subject and the catalog the selector offers load beside the
// user: the assignment card cannot render without either of them.
const { data, pending: rolesPending, refresh: refreshRoles } = await useApiAsyncData(
  () => `admin-users-roles-${itemId.value ?? 'none'}`,
  async () => {
    if (!itemId.value) return { assigned: [], catalog: [] as Role[] }
    const [assignedResponse, catalogResponse] = await Promise.all([
      userRoles.get(itemId.value),
      roles.list({ per_page: 100 }),
    ])
    return {
      assigned: assignedResponse,
      catalog: normalizeResourceResponse(catalogResponse) as Role[],
    }
  },
)

const assignedIds = computed(() => (data.value?.assigned ?? []).map((role) => role.id))
const availableRoles = computed<Role[]>(() => data.value?.catalog ?? [])
const rolesError = ref('')
const saving = ref(false)

async function handleSave(roleIds: string[]) {
  if (!itemId.value) return
  saving.value = true
  rolesError.value = ''
  try {
    await userRoles.set(itemId.value, roleIds)
    toast.success(t('admin.users.roles.updated'))
    await Promise.all([refreshRoles(), refresh()])
  } catch (saveError: any) {
    // A refusal (escalation, lockout, unknown role) answers 422 with the
    // reason: the message travels to the selector instead of a toast alone.
    rolesError.value = mapAdminError(saveError, t, 'admin.users.roles.updateError')
  } finally {
    saving.value = false
  }
}
</script>
