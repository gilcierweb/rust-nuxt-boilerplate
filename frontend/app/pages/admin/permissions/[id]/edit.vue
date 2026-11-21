<template>
  <AdminPageShell
    icon="key"
    title-key="admin.permissions.title"
    description-key="admin.permissions.editDescription"
    header="card"
    :breadcrumb-items="breadcrumbItems"
  >
    <template #header-actions>
      <NuxtLink :to="localePath('/admin/permissions')" class="btn btn-ghost">
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

    <div v-else-if="loadError" class="card border-error/20 bg-error/10 shadow-md">
      <div class="card-body">
        <div class="flex items-center gap-3 text-error">
          <span class="icon-[tabler--alert-circle] size-6"></span>
          <div>
            <p class="font-semibold">{{ $t('admin.common.errorLoadingData') }}</p>
            <p class="text-sm">{{ loadError }}</p>
          </div>
        </div>
        <button class="btn btn-soft mt-4" @click="refresh()">{{ $t('admin.common.tryAgain') }}</button>
      </div>
    </div>

    <div v-else class="card shadow-base-300/10 shadow-md">
      <div class="card-body">
        <PermissionsForm
          mode="edit"
          :catalog="catalog"
          :initial-values="initialValues"
          :existing-codes="existingCodes"
          :saving="saving"
          @submit="handleSubmit"
        />
      </div>
    </div>
  </AdminPageShell>
</template>

<script setup lang="ts">

// Unique component name: NuxtPage collects keepalive page names globally, so
// two pages sharing the inferred filename name break KeepAlive routing.
defineOptions({ name: 'AdminPermissionEditPage' })

import { useAdminRbac } from '~/composables/useAdminRbac'
import { useAdminResourceItem } from '~/composables/useAdminResourceItem'
import { usePermissionFormData } from '~/composables/usePermissionFormData'
import { mapAdminError } from '~/utils/admin-errors'
import PermissionsForm from '~/components/admin/permissions/PermissionsForm.vue'
import type { Permission } from '~/types/rbac'

type PermissionFormValues = {
  name: string
  resource: string
  action: string
  description: string | null
}

definePageMeta({ layout: 'admin' })

const { t } = useI18n()
const localePath = useLocalePath()
const toast = useToast()
const { permissions } = useAdminRbac()

const breadcrumbItems = computed(() => [
  { label: t('admin.common.dashboard'), to: localePath('/admin/dashboard') },
  { label: t('admin.permissions.title'), to: localePath('/admin/permissions') },
  { label: t('admin.common.edit') },
])

const saving = ref(false)
const { itemId: permissionId, item, pending, error, refresh } = useAdminResourceItem<Permission>(
  'permissions',
  { keyPrefix: 'admin-permissions-edit' },
)

const initialValues = computed(() => {
  const current = item.value
  if (!current) return { name: '', resource: '', action: '', description: null }
  return {
    name: current.name,
    resource: current.resource,
    action: current.action,
    description: current.description,
  }
})

// The catalog and the collision list of the form, as one shared request pair:
// `usePermissionFormData` owns the wiring (see permissions/new.vue). The key
// follows the route id, so a navigation between two rows refetches beside the
// item the page displays.
const { catalog, codes: formCodes } = await usePermissionFormData(
  () => `admin-permissions-edit-form-${permissionId.value ?? 'none'}`,
)

// The item's own code is excluded reactively from the form's own codes: the
// two requests may answer in any order, and the row being edited is never a
// collision of itself.
const existingCodes = computed<string[]>(() => {
  const currentCode = item.value?.code
  return formCodes.value.filter((code: string) => code !== currentCode)
})
const loadError = computed(() => (error.value ? mapAdminError(error.value, t) : ''))

async function handleSubmit(values: PermissionFormValues) {
  if (!permissionId.value) return
  saving.value = true
  try {
    await permissions.update(permissionId.value, {
      name: values.name,
      description: values.description,
    })
    toast.success(t('admin.permissions.messages.updateSuccess'))
    await refresh()
  } catch (submitError: any) {
    toast.error(mapAdminError(submitError, t, 'admin.permissions.messages.updateError'))
  } finally {
    saving.value = false
  }
}
</script>
