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
          <input id="roleName" v-model="name" type="text" placeholder="admin" class="input w-full" :class="{ 'is-invalid': errors.name }" :aria-required="true" :aria-invalid="errors.name ? 'true' : 'false'" :aria-describedby="errors.name ? 'roleNameError' : 'roleNameHint'" />
          <span v-if="errors.name" id="roleNameError" role="alert" class="text-error text-xs mt-1 block">{{ errors.name }}</span>
          <span v-else id="roleNameHint" class="text-base-content/60 text-xs mt-1 block">{{ $t('admin.roles.validation.name_hint') }}</span>
        </div>

        <div>
          <label class="label-text font-semibold" for="roleResourceType">{{ $t('admin.roles.form.resource_type', 'Tipo de recurso') }}</label>
          <input id="roleResourceType" v-model="resourceType" type="text" placeholder="company" class="input w-full" :class="{ 'is-invalid': errors.resource_type }" :aria-invalid="errors.resource_type ? 'true' : 'false'" :aria-describedby="errors.resource_type ? 'roleResourceTypeError' : 'roleResourceTypeHint'" />
          <span v-if="errors.resource_type" id="roleResourceTypeError" role="alert" class="text-error text-xs mt-1 block">{{ errors.resource_type }}</span>
          <span v-else id="roleResourceTypeHint" class="text-base-content/60 text-xs mt-1 block">{{ $t('admin.roles.form.scope_hint') }}</span>
        </div>

        <div>
          <label class="label-text font-semibold" for="roleResourceId">{{ $t('admin.roles.form.resource_id', 'ID do recurso') }}</label>
          <input id="roleResourceId" v-model="resourceId" type="text" class="input w-full" :class="{ 'is-invalid': errors.resource_id }" :aria-invalid="errors.resource_id ? 'true' : 'false'" :aria-describedby="errors.resource_id ? 'roleResourceIdError' : 'roleResourceIdHint'" />
          <span v-if="errors.resource_id" id="roleResourceIdError" role="alert" class="text-error text-xs mt-1 block">{{ errors.resource_id }}</span>
          <span v-else id="roleResourceIdHint" class="text-base-content/60 text-xs mt-1 block">{{ $t('admin.roles.form.scope_hint') }}</span>
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
        <NuxtLink :to="localePath('/admin/roles')" class="btn btn-ghost">{{ $t('admin.roles.form.cancel', 'Cancelar') }}</NuxtLink>
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
const localePath = useLocalePath()

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

// The rules mirror the backend validator: name from 1 to 50 characters,
// resource_type up to 255, resource_id a UUID, and the two scope fields
// paired, because a role is either class level or scoped to one record.
const uuidPattern = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i

const schema = computed(() =>
  toTypedSchema(
    v.pipe(
      v.object({
        id: v.optional(v.string()),
        name: v.pipe(
          v.string(),
          v.nonEmpty(t('admin.roles.validation.name_required', 'O nome é obrigatório')),
          v.maxLength(50, t('admin.roles.validation.name_invalid')),
        ),
        resource_type: v.optional(
          v.pipe(
            v.string(),
            v.maxLength(255, t('admin.roles.validation.resource_type_invalid')),
          ),
        ),
        resource_id: v.optional(
          v.pipe(
            v.string(),
            v.check(
              (value) => value.trim() === '' || uuidPattern.test(value.trim()),
              t('admin.roles.validation.resource_id_invalid'),
            ),
          ),
        ),
      }),
      v.forward(
        v.partialCheck(
          [['resource_type'], ['resource_id']],
          (input) =>
            Boolean(input.resource_type?.trim()) === Boolean(input.resource_id?.trim()),
          t('admin.roles.validation.scope_invalid'),
        ),
        ['resource_type', 'resource_id'],
      ),
    ),
  ),
)

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
