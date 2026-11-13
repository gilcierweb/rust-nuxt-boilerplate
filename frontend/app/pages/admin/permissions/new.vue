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
              {{ $t('admin.permissions.newTitle') }}
            </h1>
            <p class="mt-2 max-w-3xl text-sm leading-relaxed text-base-content/60">
              {{ $t('admin.permissions.newDescription') }}
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
  </section>
</template>

<script setup lang="ts">
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

// The form needs two things beside each other: the catalog that feeds the
// selects, and the codes already in use, so it can warn about a collision
// before the submit instead of letting the backend refuse the row.
const { data, pending, error, refresh } = await useApiAsyncData(
  'admin-permissions-new',
  async () => {
    const [catalogResponse, listResponse] = await Promise.all([
      permissions.catalog(),
      permissions.list({ per_page: 100 }),
    ])
    return {
      catalog: catalogResponse,
      codes: normalizeResourceResponse(listResponse).map((item: Permission) => item.code),
    }
  },
)

const catalog = computed<PermissionCatalog>(
  () => data.value?.catalog ?? { resources: [], actions: [] },
)
const existingCodes = computed<string[]>(() => data.value?.codes ?? [])
const loadError = computed(() => (error.value ? mapAdminError(error.value, t) : ''))

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
