<template>
  <section class="space-y-6">
    <AdminBreadcrumb :items="breadcrumbItems" />

    <div class="card shadow-base-300/10 shadow-md">
      <div class="card-body">
        <div class="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
          <div>
            <div class="mb-3 inline-flex items-center gap-2 rounded-field bg-primary/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.22em] text-primary">
              <span class="icon-[tabler--key] size-4"></span>
              <span>{{ $t('admin.permissions.title') }}</span>
            </div>
            <h1 class="card-title text-3xl font-semibold tracking-tight text-base-content">
              {{ $t('admin.permissions.editTitle') }}
            </h1>
            <p class="mt-2 max-w-3xl text-sm leading-relaxed text-base-content/60">
              {{ $t('admin.permissions.editDescription') }}
            </p>
          </div>

          <div class="card-actions flex flex-wrap gap-2">
            <NuxtLink :to="localePath('/admin/permissions')" class="btn btn-ghost">
              <span class="icon-[tabler--arrow-left] size-4.5"></span>
              {{ $t('admin.common.back') }}
            </NuxtLink>
          </div>
        </div>
      </div>
    </div>

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
  </section>
</template>

<script setup lang="ts">
import { useAdminRbac } from '~/composables/useAdminRbac'
import { useAdminResourceItem } from '~/composables/useAdminResourceItem'
import { normalizeResourceResponse } from '~/utils/admin-resources'
import { mapAdminError } from '~/utils/admin-errors'
import PermissionsForm from '~/components/admin/permissions/PermissionsForm.vue'
import type { Permission, PermissionCatalog } from '~/types/rbac'

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

// The code of a permission never changes: the edit form fixes the two selects,
// so the collision list only matters for the other rows.
const { data } = await useApiAsyncData('admin-permissions-edit', async () => {
  const [catalogResponse, listResponse] = await Promise.all([
    permissions.catalog(),
    permissions.list({ per_page: 100 }),
  ])
  const currentCode = item.value?.code
  return {
    catalog: catalogResponse,
    codes: normalizeResourceResponse(listResponse)
      .map((row: Permission) => row.code)
      .filter((code: string) => code !== currentCode),
  }
})

const catalog = computed<PermissionCatalog>(
  () => data.value?.catalog ?? { resources: [], actions: [] },
)
const existingCodes = computed<string[]>(() => data.value?.codes ?? [])
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
    toast.error(submitError?.message || t('admin.permissions.messages.updateError'))
  } finally {
    saving.value = false
  }
}
</script>
