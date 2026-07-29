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
        <input id="auditLogId" v-model="form.id" type="text" class="input input-disabled w-full bg-base-200" disabled />
      </div>

      <div class="grid gap-6 md:grid-cols-2">
        <div>
          <label class="label-text font-semibold" for="auditLogActorUserId">
            {{ $t('admin.auditLogs.form.fields.actorUserId.label') }}
          </label>
          <select id="auditLogActorUserId" v-model="form.actor_user_id" class="select w-full" :disabled="lookup.isLoading('users')">
            <option value="">—</option>
            <option v-for="user in users" :key="user.id" :value="user.id">
              {{ user.display_name || [user.first_name, user.last_name].filter(Boolean).join(' ') || user.email || user.id }}
            </option>
          </select>
        </div>

        <div>
          <label class="label-text font-semibold" for="auditLogActorRoleSnapshot">
            {{ $t('admin.auditLogs.form.fields.actorRoleSnapshot.label') }}
          </label>
          <input id="auditLogActorRoleSnapshot" v-model="form.actor_role_snapshot" type="text" class="input w-full" />
        </div>

        <div>
          <label class="label-text font-semibold" for="auditLogAction">
            {{ $t('admin.auditLogs.form.fields.action.label') }} <span class="text-error">*</span>
          </label>
          <input id="auditLogAction" v-model="form.action" type="text" placeholder="payment.approved" class="input w-full" required />
        </div>

        <div>
          <label class="label-text font-semibold" for="auditLogResourceType">
            {{ $t('admin.auditLogs.form.fields.resourceType.label') }} <span class="text-error">*</span>
          </label>
          <input id="auditLogResourceType" v-model="form.resource_type" type="text" placeholder="payment" class="input w-full" required />
        </div>

        <div>
          <label class="label-text font-semibold" for="auditLogResourceId">
            {{ $t('admin.auditLogs.form.fields.resourceId.label') }}
          </label>
          <input id="auditLogResourceId" v-model="form.resource_id" type="text" class="input w-full" />
        </div>

        <div>
          <label class="label-text font-semibold" for="auditLogRequestId">
            {{ $t('admin.auditLogs.form.fields.requestId.label') }}
          </label>
          <input id="auditLogRequestId" v-model="form.request_id" type="text" class="input w-full" />
        </div>

        <div>
          <label class="label-text font-semibold" for="auditLogIpAddress">
            {{ $t('admin.auditLogs.form.fields.ipAddress.label') }}
          </label>
          <input id="auditLogIpAddress" v-model="form.ip_address" type="text" class="input w-full" />
        </div>

        <div>
          <label class="label-text font-semibold" for="auditLogUserAgent">
            {{ $t('admin.auditLogs.form.fields.userAgent.label') }}
          </label>
          <input id="auditLogUserAgent" v-model="form.user_agent" type="text" class="input w-full" />
        </div>
      </div>

      <div>
        <label class="label-text font-semibold" for="auditLogChanges">
          {{ $t('admin.auditLogs.form.fields.changesJson.label') }}
        </label>
        <textarea id="auditLogChanges" v-model="form.changes" class="textarea w-full font-mono text-sm" rows="6" :placeholder="$t('admin.auditLogs.form.fields.changesJson.placeholder')"></textarea>
      </div>

      <div>
        <label class="label-text font-semibold" for="auditLogMetadata">
          {{ $t('admin.auditLogs.form.fields.metadataJson.label') }}
        </label>
        <textarea id="auditLogMetadata" v-model="form.metadata" class="textarea w-full font-mono text-sm" rows="6" :placeholder="$t('admin.auditLogs.form.fields.metadataJson.placeholder')"></textarea>
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

const form = reactive({ ...props.initialValues })

watch(
  () => props.initialValues,
  (values) => Object.assign(form, values),
  { deep: true },
)

const submitLabel = computed(() => {
  if (props.saving) return t('admin.auditLogs.form.actions.saving')
  return props.mode === 'create'
    ? t('admin.auditLogs.new.actions.submit')
    : t('admin.auditLogs.edit.actions.submit')
})

function submit() {
  emit('submit', { ...form })
}
</script>
