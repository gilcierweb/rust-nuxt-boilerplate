<template>
  <AdminPageShell
    icon="shield"
    title-key="admin.roles.title"
    description-key="admin.roles.editDescription"
    header="card"
    :breadcrumb-items="breadcrumbItems"
  >
    <template #header-actions>
      <NuxtLink :to="localePath('/admin/roles')" class="btn btn-ghost">
        <span class="icon-[tabler--arrow-left] size-4.5"></span>
        {{ $t('admin.common.back') }}
      </NuxtLink>
    </template>

    <div v-if="pending" class="card shadow-base-300/10 shadow-md">
      <div class="card-body p-12">
        <div class="flex flex-col items-center justify-center gap-4 text-base-content/55">
          <span class="icon-[tabler--loader-2] size-10 animate-spin"></span>
          <p>{{ $t('admin.common.loadingData') }}</p>
        </div>
      </div>
    </div>

    <div v-else-if="error" class="card border-error/20 bg-error/10 shadow-md">
      <div class="card-body">
        <div class="flex items-center gap-3 text-error">
          <span class="icon-[tabler--alert-circle] size-6"></span>
          <div>
            <p class="font-semibold">{{ $t('admin.common.errorLoadingData') }}</p>
            <p class="text-sm">{{ error }}</p>
          </div>
        </div>
        <button class="btn btn-soft mt-4" @click="refresh()">{{ $t('admin.common.tryAgain') }}</button>
      </div>
    </div>

    <div v-else-if="role" class="card shadow-base-300/10 shadow-md">
      <div class="card-body">
        <RolesForm mode="edit" :initial-values="role" :saving="saving" :submit-disabled="!can('roles:update')" @submit="handleSubmit" />
      </div>
    </div>
  </AdminPageShell>
</template>

<script setup lang="ts">

// Unique component name: NuxtPage collects keepalive page names globally, so
// two pages sharing the inferred filename name break KeepAlive routing.
defineOptions({ name: 'AdminRoleEditPage' })

import RolesForm from '~/components/admin/roles/RolesForm.vue'
import type { Role } from '~/types/rbac'
import { mapAdminError } from '~/utils/admin-errors'
import { useRbac } from '~/composables/useRbac'

definePageMeta({ layout: 'admin' })

const { can } = useRbac()
const { t } = useI18n()
const localePath = useLocalePath()

const breadcrumbItems = computed(() => [
  { label: t('admin.common.dashboard'), to: localePath('/admin/dashboard') },
  { label: t('admin.roles.title'), to: localePath('/admin/roles') },
  { label: t('admin.common.edit') },
])

const api = useApi()
const toast = useToast()
const { itemId: roleId, item: rawRole, pending, error, refresh } = useAdminResourceItem<Role>('roles', { keyPrefix: 'admin-roles-edit' })

const role = computed(() => {
  const item = rawRole.value
  if (!item) return null
  return {
    ...item,
    resource_type: item.resource_type || '',
    resource_id: item.resource_id || '',
  }
})

const saving = ref(false)

async function handleSubmit(values: Role) {
  if (!roleId.value) return
  saving.value = true
  try {
    await api.patch(`/admin/roles/${roleId.value}`, {
      body: {
        name: values.name,
        ...(values.resource_type ? { resource_type: values.resource_type } : {}),
        ...(values.resource_id ? { resource_id: values.resource_id } : {}),
      },
    })
    toast.success(t('admin.roles.messages.updateSuccess'))
    await refresh()
  } catch (err: any) {
    toast.error(mapAdminError(err, t, 'admin.roles.messages.updateError'))
  } finally {
    saving.value = false
  }
}
</script>
