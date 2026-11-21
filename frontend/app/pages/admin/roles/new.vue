<template>
  <AdminPageShell
    icon="shield"
    title-key="admin.roles.title"
    description-key="admin.roles.newDescription"
    header="card"
    :breadcrumb-items="breadcrumbItems"
  >
    <template #header-actions>
      <NuxtLink :to="localePath('/admin/roles')" class="btn btn-ghost">
        <span class="icon-[tabler--arrow-left] size-4.5"></span>
        {{ $t('admin.common.back') }}
      </NuxtLink>
    </template>

    <div class="card shadow-base-300/10 shadow-md">
      <div class="card-body">
        <RolesForm mode="create" :initial-values="initialValues" :saving="saving" :submit-disabled="!can('roles:create')" @submit="handleSubmit" />
      </div>
    </div>
  </AdminPageShell>
</template>

<script setup lang="ts">

// Unique component name: NuxtPage collects keepalive page names globally, so
// two pages sharing the inferred filename name break KeepAlive routing.
defineOptions({ name: 'AdminRoleNewPage' })

import RolesForm from '~/components/admin/roles/RolesForm.vue'
import { mapAdminError } from '~/utils/admin-errors'
import { useRbac } from '~/composables/useRbac'

definePageMeta({ layout: 'admin' })

const { t } = useI18n()
const localePath = useLocalePath()

const breadcrumbItems = computed(() => [
  { label: t('admin.common.dashboard'), to: localePath('/admin/dashboard') },
  { label: t('admin.roles.title'), to: localePath('/admin/roles') },
  { label: t('admin.common.new') },
])

const api = useApi()
const toast = useToast()

const initialValues = {
  name: '',
  resource_type: '',
  resource_id: '',
}

const saving = ref(false)
const { can } = useRbac()

async function handleSubmit(values: typeof initialValues) {
  saving.value = true
  try {
    await api.post('/admin/roles', {
      body: {
        name: values.name,
        ...(values.resource_type ? { resource_type: values.resource_type } : {}),
        ...(values.resource_id ? { resource_id: values.resource_id } : {}),
      },
    })
    toast.success(t('admin.roles.messages.createSuccess'))
    await navigateTo(localePath('/admin/roles'))
  } catch (error: any) {
    toast.error(mapAdminError(error, t, 'admin.roles.messages.createError'))
  } finally {
    saving.value = false
  }
}
</script>
