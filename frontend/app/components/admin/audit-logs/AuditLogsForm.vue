<template>
  <form class="needs-validation peer" novalidate @submit.prevent="submit">
    <AppAlert
      v-if="formAlertMessage"
      class="mb-6"
      tone="error"
      variant="soft"
      :title="$t('common.error')"
      :message="formAlertMessage"
    />
    <div class="space-y-6">
      <div v-if="mode === 'edit' && initialValues.id" class="md:w-1/2">
        <label class="label-text font-semibold" for="auditLogId">ID</label>
        <input id="auditLogId" :value="initialValues.id" type="text" class="input input-disabled w-full bg-base-200" disabled />
      </div>

      <div class="grid gap-6 md:grid-cols-2">
        <div>
          <label class="label-text font-semibold" for="auditLogActorUserId">
            {{ $t('admin.auditLogs.form.fields.actorUserId.label') }}
          </label>
          <select id="auditLogActorUserId" v-model="actorUserId" class="select w-full" :class="{ 'is-invalid': errors.actor_user_id }" :disabled="lookup.isLoading('users')">
            <option value="">—</option>
            <option v-for="user in users" :key="user.id" :value="user.id">
              {{ user.display_name || [user.first_name, user.last_name].filter(Boolean).join(' ') || user.email || user.id }}
            </option>
          </select>
          <span v-if="errors.actor_user_id" class="text-error text-xs mt-1 block">{{ errors.actor_user_id }}</span>
        </div>

        <div>
          <label class="label-text font-semibold" for="auditLogActorRoleSnapshot">
            {{ $t('admin.auditLogs.form.fields.actorRoleSnapshot.label') }}
          </label>
          <input id="auditLogActorRoleSnapshot" v-model="actorRoleSnapshot" type="text" class="input w-full" :class="{ 'is-invalid': errors.actor_role_snapshot }" />
          <span v-if="errors.actor_role_snapshot" class="text-error text-xs mt-1 block">{{ errors.actor_role_snapshot }}</span>
        </div>

        <div>
          <label class="label-text font-semibold" for="auditLogAction">
            {{ $t('admin.auditLogs.form.fields.action.label') }} <span class="text-error">*</span>
          </label>
          <input id="auditLogAction" v-model="action" type="text" placeholder="payment.approved" class="input w-full" :class="{ 'is-invalid': errors.action }" />
          <span v-if="errors.action" class="text-error text-xs mt-1 block">{{ errors.action }}</span>
        </div>

        <div>
          <label class="label-text font-semibold" for="auditLogResourceType">
            {{ $t('admin.auditLogs.form.fields.resourceType.label') }} <span class="text-error">*</span>
          </label>
          <input id="auditLogResourceType" v-model="resourceType" type="text" placeholder="payment" class="input w-full" :class="{ 'is-invalid': errors.resource_type }" />
          <span v-if="errors.resource_type" class="text-error text-xs mt-1 block">{{ errors.resource_type }}</span>
        </div>

        <div>
          <label class="label-text font-semibold" for="auditLogResourceId">
            {{ $t('admin.auditLogs.form.fields.resourceId.label') }}
          </label>
          <input id="auditLogResourceId" v-model="resourceId" type="text" class="input w-full" :class="{ 'is-invalid': errors.resource_id }" />
          <span v-if="errors.resource_id" class="text-error text-xs mt-1 block">{{ errors.resource_id }}</span>
        </div>

        <div>
          <label class="label-text font-semibold" for="auditLogRequestId">
            {{ $t('admin.auditLogs.form.fields.requestId.label') }}
          </label>
          <input id="auditLogRequestId" v-model="requestId" type="text" class="input w-full" :class="{ 'is-invalid': errors.request_id }" />
          <span v-if="errors.request_id" class="text-error text-xs mt-1 block">{{ errors.request_id }}</span>
        </div>

        <div>
          <label class="label-text font-semibold" for="auditLogIpAddress">
            {{ $t('admin.auditLogs.form.fields.ipAddress.label') }}
          </label>
          <input id="auditLogIpAddress" v-model="ipAddress" type="text" class="input w-full" :class="{ 'is-invalid': errors.ip_address }" />
          <span v-if="errors.ip_address" class="text-error text-xs mt-1 block">{{ errors.ip_address }}</span>
        </div>

        <div>
          <label class="label-text font-semibold" for="auditLogUserAgent">
            {{ $t('admin.auditLogs.form.fields.userAgent.label') }}
          </label>
          <input id="auditLogUserAgent" v-model="userAgent" type="text" class="input w-full" :class="{ 'is-invalid': errors.user_agent }" />
          <span v-if="errors.user_agent" class="text-error text-xs mt-1 block">{{ errors.user_agent }}</span>
        </div>
      </div>

      <div>
        <label class="label-text font-semibold" for="auditLogChanges">
          {{ $t('admin.auditLogs.form.fields.changesJson.label') }}
        </label>
        <textarea id="auditLogChanges" v-model="changes" class="textarea w-full font-mono text-sm" rows="6" :placeholder="$t('admin.auditLogs.form.fields.changesJson.placeholder')" :class="{ 'is-invalid': errors.changes }"></textarea>
        <span v-if="errors.changes" class="text-error text-xs mt-1 block">{{ errors.changes }}</span>
      </div>

      <div>
        <label class="label-text font-semibold" for="auditLogMetadata">
          {{ $t('admin.auditLogs.form.fields.metadataJson.label') }}
        </label>
        <textarea id="auditLogMetadata" v-model="metadata" class="textarea w-full font-mono text-sm" rows="6" :placeholder="$t('admin.auditLogs.form.fields.metadataJson.placeholder')" :class="{ 'is-invalid': errors.metadata }"></textarea>
        <span v-if="errors.metadata" class="text-error text-xs mt-1 block">{{ errors.metadata }}</span>
      </div>

      <div v-if="mode === 'edit' && initialValues.created_at" class="grid gap-6 border-t border-base-content/10 pt-4 md:grid-cols-1">
        <div>
          <label class="label-text font-semibold" for="auditLogCreatedAt">
            {{ $t('admin.auditLogs.form.fields.createdAt.label') }}
          </label>
          <input id="auditLogCreatedAt" :value="formatDateTime(initialValues.created_at)" type="text" class="input input-disabled w-full bg-base-200" disabled />
        </div>
      </div>

      <div class="flex items-center justify-end gap-3 border-t border-base-content/10 pt-4">
        <NuxtLink :to="localePath('/admin/audit-logs')" class="btn btn-ghost">
          {{ $t('common.actions.cancel') }}
        </NuxtLink>
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
const lookup = useAdminLookup()

onMounted(() => {
  lookup.load('users')
})

const users = computed(() => lookup.getItems('users'))

const { formAlertMessage } = useFormAlert()
const localePath = useLocalePath()

const props = defineProps<{
  mode: 'create' | 'edit'
  initialValues: {
    id?: string
    actor_user_id?: string
    actor_role_snapshot?: string
    action: string
    resource_type: string
    resource_id?: string
    ip_address?: string
    user_agent?: string
    request_id?: string
    changes: string
    metadata: string
    created_at?: string
  }
  saving?: boolean
}>()

const emit = defineEmits<{
  submit: [values: typeof props.initialValues]
}>()

const UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i
const IP_NET_PATTERN = /^(\d{1,3}\.){3}\d{1,3}(\/\d{1,2})?$|^([0-9a-fA-F:]+)(\/\d{1,3})?$/

const optionalUuid = (message: string) =>
  v.union([
    v.literal(''),
    v.pipe(v.string(), v.regex(UUID_PATTERN, message)),
  ])

const optionalIpNet = (message: string) =>
  v.union([
    v.literal(''),
    v.pipe(v.string(), v.regex(IP_NET_PATTERN, message)),
  ])

const schema = computed(() =>
  toTypedSchema(
    v.object({
      id: v.optional(v.string()),
      actor_user_id: optionalUuid(t('admin.auditLogs.form.validation.actor_user_id_uuid')),
      actor_role_snapshot: v.pipe(
        v.string(),
        v.maxLength(255, t('admin.auditLogs.form.validation.actor_role_snapshot_max')),
      ),
      action: v.pipe(
        v.string(),
        v.nonEmpty(t('admin.auditLogs.form.validation.action_required')),
        v.maxLength(255, t('admin.auditLogs.form.validation.action_max')),
      ),
      resource_type: v.pipe(
        v.string(),
        v.nonEmpty(t('admin.auditLogs.form.validation.resource_type_required')),
        v.maxLength(255, t('admin.auditLogs.form.validation.resource_type_max')),
      ),
      resource_id: optionalUuid(t('admin.auditLogs.form.validation.resource_id_uuid')),
      ip_address: optionalIpNet(t('admin.auditLogs.form.validation.ip_address_invalid')),
      user_agent: v.pipe(
        v.string(),
        v.maxLength(500, t('admin.auditLogs.form.validation.user_agent_max')),
      ),
      request_id: optionalUuid(t('admin.auditLogs.form.validation.request_id_uuid')),
      changes: v.pipe(
        v.string(),
        v.check((value) => {
          if (!value.trim()) return true
          try { JSON.parse(value); return true } catch { return false }
        }, t('admin.auditLogs.form.validation.changes_json')),
      ),
      metadata: v.pipe(
        v.string(),
        v.check((value) => {
          if (!value.trim()) return true
          try { JSON.parse(value); return true } catch { return false }
        }, t('admin.auditLogs.form.validation.metadata_json')),
      ),
    }),
  ),
)

const { handleSubmit, errors, resetForm, defineField } = useForm({
  validationSchema: schema,
  initialValues: props.initialValues,
})

const [actorUserId] = defineField('actor_user_id')
const [actorRoleSnapshot] = defineField('actor_role_snapshot')
const [action] = defineField('action')
const [resourceType] = defineField('resource_type')
const [resourceId] = defineField('resource_id')
const [ipAddress] = defineField('ip_address')
const [userAgent] = defineField('user_agent')
const [requestId] = defineField('request_id')
const [changes] = defineField('changes')
const [metadata] = defineField('metadata')

watch(
  () => props.initialValues,
  (values) => {
    resetForm({ values })
  },
  { deep: true },
)

const submitLabel = computed(() => {
  if (props.saving) return t('admin.auditLogs.form.actions.saving')
  return props.mode === 'create'
    ? t('admin.auditLogs.new.actions.submit')
    : t('admin.auditLogs.edit.actions.submit')
})

const submit = handleSubmit((values) => {
  emit('submit', { ...props.initialValues, ...values } as typeof props.initialValues)
})
</script>
