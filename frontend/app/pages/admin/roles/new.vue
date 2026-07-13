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
            <h1 class="card-title text-3xl font-semibold tracking-tight text-base-content">{{ $t('admin.roles.newTitle') }}</h1>
            <p class="mt-2 max-w-3xl text-sm leading-relaxed text-base-content/60">{{ $t('admin.roles.newDescription') }}</p>
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

    <div class="card shadow-base-300/10 shadow-md">
      <div class="card-body">
        <RolesForm mode="create" :initial-values="initialValues" :saving="saving" @submit="handleSubmit" />
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import RolesForm from '~/components/admin/roles/RolesForm.vue'

definePageMeta({ layout: 'admin' })

const { t } = useI18n()

const breadcrumbItems = computed(() => [
  { label: t('admin.common.dashboard'), to: '/admin/dashboard' },
  { label: t('admin.roles.title'), to: '/admin/roles' },
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
    await navigateTo('/admin/roles')
  } catch (error: any) {
    toast.error(error?.message || t('admin.roles.messages.createError'))
  } finally {
    saving.value = false
  }
}
</script>
