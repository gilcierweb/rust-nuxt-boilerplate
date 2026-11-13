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
          <h1 class="text-3xl font-semibold tracking-tight text-base-content">
            {{ role ? role.name : $t('admin.roles.showTitle') }}
          </h1>
          <p class="mt-2 max-w-3xl text-sm leading-relaxed text-base-content/60">
            {{ $t('admin.roles.showDescription') }}
          </p>
        </div>

        <div class="flex flex-wrap gap-2">
          <NuxtLink :to="localePath('/admin/roles')" class="btn btn-ghost" :prefetch="false">
            <span class="icon-[tabler--arrow-left] size-4.5"></span>
            {{ $t('admin.common.back') }}
          </NuxtLink>
          <NuxtLink v-if="role" :to="localePath(`/admin/roles/${role.id}/edit`)" class="btn btn-primary" :prefetch="false">
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
          <p class="text-sm">{{ error }}</p>
        </div>
      </div>
      <button class="btn btn-soft mt-4" @click="refresh()">{{ $t('admin.common.tryAgain') }}</button>
    </div>

    <div v-else-if="!role" class="rounded-box border border-base-content/10 bg-base-100 p-12 shadow-md">
      <AppEmptyState
        :title="$t('admin.roles.not_found.title')"
        :description="$t('admin.roles.not_found.description')"
      >
        <template #action>
          <NuxtLink :to="localePath('/admin/roles')" class="btn btn-primary">
            {{ $t('admin.common.back') }}
          </NuxtLink>
        </template>
      </AppEmptyState>
    </div>

    <template v-else>
      <div class="rounded-box border border-base-content/10 bg-base-100 p-6 shadow-md shadow-base-content/5">
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
            <p class="mt-1 text-base-content">{{ role.resource_type || $t('admin.roles.form.class_level') }}</p>
          </div>
          <div>
            <p class="text-sm font-semibold text-base-content/70">{{ $t('admin.roles.form.resource_id') }}</p>
            <p class="mt-1 text-base-content">{{ role.resource_id || $t('admin.roles.form.class_level') }}</p>
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

      <div class="rounded-box border border-base-content/10 bg-base-100 p-6 shadow-md shadow-base-content/5">
        <h2 class="text-lg font-semibold text-base-content">
          {{ $t('admin.roles.permissions.card_title') }}
        </h2>
        <p class="mt-1 text-sm text-base-content/60">
          {{ $t('admin.roles.permissions.card_description') }}
        </p>

        <div v-if="permissionsPending" class="flex items-center justify-center gap-3 py-12 text-base-content/55">
          <span class="icon-[tabler--loader-2] size-8 animate-spin"></span>
          <p>{{ $t('admin.common.loadingData') }}</p>
        </div>

        <div v-else-if="permissionsError" class="py-6">
          <div class="flex items-center gap-3 text-error">
            <span class="icon-[tabler--alert-circle] size-6"></span>
            <p class="font-semibold">{{ $t('admin.roles.permissions.loadError') }}</p>
          </div>
        </div>

        <RolePermissionsEditor
          v-else
          class="mt-4"
          :permissions="catalog"
          :assignments="assignments"
          :saving="saving"
          @save="handleSave"
        />
      </div>

      <div class="rounded-box border border-base-content/10 bg-base-100 p-6 shadow-md shadow-base-content/5">
        <h2 class="text-lg font-semibold text-base-content">
          {{ $t('admin.roles.members.card_title') }}
        </h2>

        <div v-if="membersPending" class="flex items-center justify-center gap-3 py-12 text-base-content/55">
          <span class="icon-[tabler--loader-2] size-8 animate-spin"></span>
          <p>{{ $t('admin.common.loadingData') }}</p>
        </div>

        <AppEmptyState
          v-else-if="members.length === 0"
          class="mt-4"
          :title="$t('admin.roles.members.empty')"
        />

        <ul v-else class="mt-4 divide-y divide-base-content/10">
          <li v-for="member in members" :key="member.id" class="flex items-center justify-between gap-4 py-3">
            <div>
              <NuxtLink :to="localePath(`/admin/users/${member.id}`)" class="text-sm font-medium text-primary hover:underline">
                {{ member.display_name || member.email }}
              </NuxtLink>
              <p class="text-xs text-base-content/55">{{ member.email }}</p>
            </div>
            <NuxtLink :to="localePath(`/admin/users/${member.id}`)" class="btn btn-ghost btn-sm">
              {{ $t('admin.common.view') }}
            </NuxtLink>
          </li>
        </ul>
      </div>
    </template>
  </section>
</template>

<script setup lang="ts">
import { useAdminRbac } from '~/composables/useAdminRbac'
import { normalizeResourceResponse } from '~/utils/admin-resources'
import { formatDateTime } from '~/utils/admin-ui'
import { mapAdminError } from '~/utils/admin-errors'
import AppEmptyState from '~/components/admin/AppEmptyState.vue'
import RolePermissionsEditor from '~/components/admin/roles/RolePermissionsEditor.vue'
import type { Permission, PermissionAssignment } from '~/types/rbac'

definePageMeta({
  layout: 'admin'
})

const { t } = useI18n()
const localePath = useLocalePath()
const toast = useToast()
const { permissions, rolePermissions, roleMembers } = useAdminRbac()

const breadcrumbItems = computed(() => [
  { label: t('admin.common.dashboard'), to: localePath('/admin/dashboard') },
  { label: t('admin.roles.title'), to: localePath('/admin/roles') },
  { label: t('admin.common.details') },
])

const { itemId: roleId, item: role, pending, error, refresh } = useAdminResourceItem<{
  id: string
  name: string
  resource_type: string | null
  resource_id: string | null
  created_at: string
  updated_at: string
}>('roles', { keyPrefix: 'admin-roles-show' })

// The catalog, the links of this role and its members load beside the role:
// the editor needs the whole catalog to offer every permission, and the
// members card comes from the same role.
const { data, pending: permissionsPending, error: permissionsError, refresh: refreshRoleData } =
  await useApiAsyncData(
    () => `admin-roles-permissions-${roleId.value ?? 'none'}`,
    async () => {
      if (!roleId.value) return { catalog: [], assignments: [], members: [] }
      const [catalogResponse, linksResponse, membersResponse] = await Promise.all([
        permissions.list({ per_page: 100 }),
        rolePermissions.get(roleId.value),
        roleMembers.list(roleId.value),
      ])
      return {
        catalog: normalizeResourceResponse(catalogResponse) as Permission[],
        assignments: linksResponse,
        members: membersResponse,
      }
    },
  )

const catalog = computed<Permission[]>(() => data.value?.catalog ?? [])
const assignments = computed<PermissionAssignment[]>(() => data.value?.assignments ?? [])
const members = computed(() => data.value?.members ?? [])
const membersPending = computed(() => permissionsPending.value)

const saving = ref(false)

async function handleSave(next: PermissionAssignment[]) {
  if (!roleId.value) return
  saving.value = true
  try {
    await rolePermissions.set(roleId.value, next)
    toast.success(t('admin.roles.permissions.updated'))
    await Promise.all([refreshRoleData(), refresh()])
  } catch (saveError: any) {
    toast.error(mapAdminError(saveError, t, 'admin.roles.permissions.updateError'))
  } finally {
    saving.value = false
  }
}
</script>
