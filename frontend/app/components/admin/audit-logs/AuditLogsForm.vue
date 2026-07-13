<template>
  <form class="needs-validation peer" novalidate @submit.prevent="submit">
    <AppAlert
      v-if="formAlertMessage"
      class="mb-6"
      tone="error"
      variant="soft"
      title="Erro"
      :message="formAlertMessage"
    />
    <div class="space-y-6">
      <div v-if="mode === 'edit' && initialValues.id" class="md:w-1/2">
        <label class="label-text font-semibold" for="auditLogId">ID</label>
        <input id="auditLogId" v-model="form.id" type="text" class="input input-disabled w-full bg-base-200" disabled />
      </div>

      <div class="grid gap-6 md:grid-cols-2">
        <div>
          <label class="label-text font-semibold" for="auditLogCompanyId">Empresa</label>
          <select id="auditLogCompanyId" v-model="form.company_id" class="select w-full" :disabled="lookup.isLoading('companies')">
            <option value="">Selecione uma empresa</option>
            <option v-for="company in companies" :key="company.id" :value="company.id">
              {{ company.legal_name || company.trade_name || company.name }}
            </option>
          </select>
        </div>

        <div>
          <label class="label-text font-semibold" for="auditLogActorUserId">Usuário executor</label>
          <select id="auditLogActorUserId" v-model="form.actor_user_id" class="select w-full" :disabled="lookup.isLoading('users')">
            <option value="">Selecione um usuário</option>
            <option v-for="user in users" :key="user.id" :value="user.id">
              {{ user.display_name || [user.first_name, user.last_name].filter(Boolean).join(' ') || user.email || user.id }}
            </option>
          </select>
        </div>

        <div>
          <label class="label-text font-semibold" for="auditLogActorRoleSnapshot">Perfil capturado</label>
          <input id="auditLogActorRoleSnapshot" v-model="form.actor_role_snapshot" type="text" class="input w-full" />
        </div>

        <div>
          <label class="label-text font-semibold" for="auditLogTargetCustomerId">Cliente alvo</label>
          <select id="auditLogTargetCustomerId" v-model="form.target_customer_id" class="select w-full" :disabled="lookup.isLoading('customers')">
            <option value="">Selecione um cliente</option>
            <option v-for="customer in customers" :key="customer.id" :value="customer.id">
              {{ customer.customer_code || customer.display_name || customer.email || customer.id }}
            </option>
          </select>
        </div>

        <div>
          <label class="label-text font-semibold" for="auditLogAction">
            Ação <span class="text-error">*</span>
          </label>
          <input id="auditLogAction" v-model="form.action" type="text" placeholder="payment.approved" class="input w-full" required />
        </div>

        <div>
          <label class="label-text font-semibold" for="auditLogResourceType">
            Tipo de recurso <span class="text-error">*</span>
          </label>
          <input id="auditLogResourceType" v-model="form.resource_type" type="text" placeholder="payment" class="input w-full" required />
        </div>

        <div>
          <label class="label-text font-semibold" for="auditLogResourceId">ID do recurso</label>
          <input id="auditLogResourceId" v-model="form.resource_id" type="text" class="input w-full" />
        </div>

        <div>
          <label class="label-text font-semibold" for="auditLogRequestId">Request ID</label>
          <input id="auditLogRequestId" v-model="form.request_id" type="text" class="input w-full" />
        </div>

        <div>
          <label class="label-text font-semibold" for="auditLogIpAddress">IP</label>
          <input id="auditLogIpAddress" v-model="form.ip_address" type="text" class="input w-full" />
        </div>

        <div>
          <label class="label-text font-semibold" for="auditLogUserAgent">User agent</label>
          <input id="auditLogUserAgent" v-model="form.user_agent" type="text" class="input w-full" />
        </div>
      </div>

      <div>
        <label class="label-text font-semibold" for="auditLogChanges">Changes</label>
        <textarea id="auditLogChanges" v-model="form.changes" class="textarea w-full font-mono text-sm" rows="6"></textarea>
      </div>

      <div>
        <label class="label-text font-semibold" for="auditLogMetadata">Metadata</label>
        <textarea id="auditLogMetadata" v-model="form.metadata" class="textarea w-full font-mono text-sm" rows="6"></textarea>
      </div>

      <div v-if="mode === 'edit' && initialValues.created_at" class="grid gap-6 border-t border-base-content/10 pt-4 md:grid-cols-1">
        <div>
          <label class="label-text font-semibold" for="auditLogCreatedAt">Criado em</label>
          <input id="auditLogCreatedAt" :value="formatDateTime(initialValues.created_at)" type="text" class="input input-disabled w-full bg-base-200" disabled />
        </div>
      </div>

      <div class="flex items-center justify-end gap-3 border-t border-base-content/10 pt-4">
        <NuxtLink to="/admin/audit-logs" class="btn btn-ghost">Cancelar</NuxtLink>
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

const lookup = useAdminLookup()

onMounted(() => {
  lookup.load('companies')
  lookup.load('users')
  lookup.load('customers')
})

const companies = computed(() => lookup.getItems('companies'))
const users = computed(() => lookup.getItems('users'))
const customers = computed(() => lookup.getItems('customers'))

const { formAlertMessage } = useFormAlert()

const props = defineProps<{
  mode: 'create' | 'edit'
  initialValues: {
    id?: string
    company_id?: string
    actor_user_id?: string
    actor_role_snapshot?: string
    action: string
    resource_type: string
    resource_id?: string
    target_customer_id?: string
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
  { deep: true }
)

const submitLabel = computed(() => {
  if (props.saving) return 'Salvando...'
  return props.mode === 'create' ? 'Criar log' : 'Salvar alterações'
})

function submit() {
  emit('submit', { ...form })
}
</script>
