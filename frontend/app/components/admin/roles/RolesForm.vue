<template>
  <form class="needs-validation peer" novalidate @submit.prevent="submit">
    <AppAlert
      v-if="formAlertMessage"
      class="mb-6"
      tone="error"
      variant="soft"
      :title="$t('admin.roles.form.error_title', 'Erro')"
      :message="formAlertMessage"
    />
    <div class="space-y-6">
      <div v-if="mode === 'edit' && initialValues.id" class="md:w-1/2">
        <label class="label-text font-semibold" for="roleId">{{ $t('admin.roles.form.id', 'ID') }}</label>
        <input id="roleId" :value="initialValues.id" type="text" class="input input-disabled w-full bg-base-200" disabled />
      </div>

      <div class="grid gap-6 md:grid-cols-2">
        <div>
          <label class="label-text font-semibold" for="roleName">
            {{ $t('admin.roles.form.name', 'Nome') }} <span class="text-error">*</span>
          </label>
          <input id="roleName" v-model="name" type="text" placeholder="admin" class="input w-full" :class="{ 'is-invalid': errors.name }" />
          <span v-if="errors.name" class="text-error text-xs mt-1 block">{{ errors.name }}</span>
        </div>

        <div>
          <label class="label-text font-semibold" for="roleResourceType">{{ $t('admin.roles.form.resource_type', 'Tipo de recurso') }}</label>
          <input id="roleResourceType" v-model="resourceType" type="text" placeholder="company" class="input w-full" :class="{ 'is-invalid': errors.resource_type }" />
          <span v-if="errors.resource_type" class="text-error text-xs mt-1 block">{{ errors.resource_type }}</span>
        </div>

        <div>
          <label class="label-text font-semibold" for="roleResourceId">{{ $t('admin.roles.form.resource_id', 'ID do recurso') }}</label>
          <input id="roleResourceId" v-model="resourceId" type="text" class="input w-full" :class="{ 'is-invalid': errors.resource_id }" />
          <span v-if="errors.resource_id" class="text-error text-xs mt-1 block">{{ errors.resource_id }}</span>
        </div>
      </div>

      <div v-if="mode === 'edit' && initialValues.created_at" class="grid gap-6 md:grid-cols-2 pt-4 border-t border-base-content/10">
        <div>
          <label class="label-text font-semibold" for="roleCreatedAt">{{ $t('admin.roles.form.created_at', 'Criado em') }}</label>
          <input id="roleCreatedAt" :value="formatDateTime(initialValues.created_at)" type="text" class="input input-disabled w-full bg-base-200" disabled />
        </div>
        <div>
          <label class="label-text font-semibold" for="roleUpdatedAt">{{ $t('admin.roles.form.updated_at', 'Atualizado em') }}</label>
          <input id="roleUpdatedAt" :value="formatDateTime(initialValues.updated_at)" type="text" class="input input-disabled w-full bg-base-200" disabled />
        </div>
      </div>

      <div class="flex items-center justify-end gap-3 pt-4 border-t border-base-content/10">
        <NuxtLink to="/admin/roles" class="btn btn-ghost">{{ $t('admin.roles.form.cancel', 'Cancelar') }}</NuxtLink>
        <button type="submit" class="btn btn-primary" :disabled="saving">
          <span v-if="saving" class="icon-[tabler--loader-2] size-5 animate-spin"></span>
          <span v-else class="icon-[tabler--check] size-5"></span>
          {{ submitLabel }}
        </button>
      </div>
    </div>
  </form>
</template>

<script setup lang="ts">
import * as v from 'valibot'
import { toTypedSchema } from '@vee-validate/valibot'
import { formatDateTime } from '~/utils/admin-ui'
import AppAlert from '~/components/AppAlert.vue'

const { t } = useI18n()
const { formAlertMessage } = useFormAlert()

const props = defineProps<{
  mode: 'create' | 'edit'
  initialValues: {
    id?: string
    name: string
    resource_type?: string
    resource_id?: string
    created_at?: string
    updated_at?: string
  }
  saving?: boolean
}>()

const emit = defineEmits<{
  submit: [values: typeof props.initialValues]
}>()

const schema = computed(() => toTypedSchema(
  v.object({
    id: v.optional(v.string()),
    name: v.pipe(v.string(), v.nonEmpty(t('admin.roles.validation.name_required', 'O nome é obrigatório'))),
    resource_type: v.optional(v.string()),
    resource_id: v.optional(v.string()),
  })
))

const { handleSubmit, errors, resetForm, defineField } = useForm({
  validationSchema: schema,
  initialValues: props.initialValues,
})

const [name] = defineField('name')
const [resourceType] = defineField('resource_type')
const [resourceId] = defineField('resource_id')

watch(
  () => props.initialValues,
  (values) => {
    resetForm({ values })
  },
  { deep: true }
)

const submitLabel = computed(() => {
  if (props.saving) return t('admin.roles.form.saving', 'Salvando...')
  return props.mode === 'create' ? t('admin.roles.form.create_role', 'Criar cargo') : t('admin.roles.form.save_changes', 'Salvar alterações')
})

const submit = handleSubmit((values) => {
  emit('submit', { ...props.initialValues, ...values } as typeof props.initialValues)
})
</script>
