<template>
  <section class="space-y-6">
    <AdminBreadcrumb :items="breadcrumbItems" />

    <div class="card shadow-base-300/10 shadow-md">
      <div class="card-body">
        <div class="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
          <div>
            <div class="mb-3 inline-flex items-center gap-2 rounded-field bg-primary/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.22em] text-primary">
              <span class="icon-[tabler--shield] size-4"></span>
              <span>{{ $t('admin.roles.title') }}</span>
            </div>
            <h1 class="card-title text-3xl font-semibold tracking-tight text-base-content">{{ $t('admin.roles.editTitle') }}</h1>
            <p class="mt-2 max-w-3xl text-sm leading-relaxed text-base-content/60">{{ $t('admin.roles.editDescription') }}</p>
          </div>

          <div class="card-actions flex flex-wrap gap-2">
            <NuxtLink to="/admin/roles" class="btn btn-ghost">
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

    <div v-else-if="error" class="card border-error/20 bg-error/10 shadow-md">
      <div class="card-body">
        <div class="flex items-center gap-3 text-error">
          <span class="icon-[tabler--alert-circle] size-6"></span>
          <div>
            <p class="font-semibold">{{ $t('admin.common.errorLoadingData') }}</p>
            <p class="text-sm">{{ requestError }}</p>
          </div>
        </div>
        <button class="btn btn-soft mt-4" @click="refresh()">{{ $t('admin.common.tryAgain') }}</button>
      </div>
    </div>

    <div v-else-if="role" class="card shadow-base-300/10 shadow-md">
      <div class="card-body">
        <RolesForm mode="edit" :initial-values="role" :saving="saving" @submit="handleSubmit" />
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import RolesForm from '~/components/admin/roles/RolesForm.vue'
import { extractErrorMessage } from '~/utils/admin-ui'

interface Role {
  id: string
  name: string
  resource_type?: string
  resource_id?: string
  created_at?: string
  updated_at?: string
}

definePageMeta({ layout: 'admin' })

const { t } = useI18n()

const breadcrumbItems = computed(() => [
  { label: t('admin.common.dashboard'), to: '/admin/dashboard' },
  { label: t('admin.roles.title'), to: '/admin/roles' },
  { label: t('admin.common.edit') },
])

const api = useApi()
const toast = useToast()
const route = useRoute()
const roleId = computed(() => route.params.id as string)

const { data, pending, error, refresh } = await useApiFetch<Role | { data: Role }>(
  () => `/admin/roles/${roleId.value}`,
  {
    key: `admin-roles-edit-${roleId.value}`,
    server: true,
    default: () => null,
  },
)

const requestError = computed(() => (error.value ? extractErrorMessage(error.value) : ''))
const role = computed(() => {
  if (!data.value) return null
  const item = 'data' in data.value ? data.value.data : data.value
  return {
    ...item,
    resource_type: item.resource_type || '',
    resource_id: item.resource_id || '',
  }
})

const saving = ref(false)

async function handleSubmit(values: Role) {
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
    toast.error(err?.message || t('admin.roles.messages.updateError'))
  } finally {
    saving.value = false
  }
}
</script>
