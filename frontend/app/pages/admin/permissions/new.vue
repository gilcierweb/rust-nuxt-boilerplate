<template>
  <AdminPageShell
    icon="key"
    title-key="admin.permissions.title"
    description-key="admin.permissions.newDescription"
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
            <p class="font-semibold">{{ $t('admin.permissions.form.loadError') }}</p>
            <p class="text-sm">{{ loadError }}</p>
          </div>
        </div>
        <button class="btn btn-soft mt-4" @click="refresh()">{{ $t('admin.common.tryAgain') }}</button>
      </div>
    </div>

    <div v-else class="card shadow-base-300/10 shadow-md">
      <div class="card-body">
        <PermissionsForm
          mode="create"
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
defineOptions({ name: 'AdminPermissionNewPage' })

import { useAdminRbac } from '~/composables/useAdminRbac'
import { normalizeResourceResponse } from '~/utils/admin-resources'
import { mapAdminError } from '~/utils/admin-errors'
import PermissionsForm from '~/components/admin/permissions/PermissionsForm.vue'
import type { Permission, PermissionCatalog } from '~/types/rbac'

definePageMeta({ layout: 'admin' })

const { t } = useI18n()
const localePath = useLocalePath()
const toast = useToast()
const { permissions } = useAdminRbac()

const breadcrumbItems = computed(() => [
  { label: t('admin.common.dashboard'), to: localePath('/admin/dashboard') },
  { label: t('admin.permissions.title'), to: localePath('/admin/permissions') },
  { label: t('admin.common.new') },
])

const initialValues = { name: '', resource: '', action: '', description: null }
const saving = ref(false)

// The catalog and the collision list of the form, as one shared request pair:
// `usePermissionFormData` owns the wiring so the edit page cannot drift.
const { catalog, codes: existingCodes, pending, error: loadError, refresh } = await usePermissionFormData(
  'admin-permissions-new',
)

async function handleSubmit(values: typeof initialValues) {
  saving.value = true
  try {
    await permissions.create(values)
    toast.success(t('admin.permissions.messages.createSuccess'))
    await navigateTo(localePath('/admin/permissions'))
  } catch (submitError: any) {
    toast.error(mapAdminError(submitError, t, 'admin.permissions.messages.createError'))
  } finally {
    saving.value = false
  }
}
</script>
