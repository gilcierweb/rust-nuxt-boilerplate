import type { LookupEntity } from '~/composables/useAdminLookup'

export type AdminFieldOption = {
  label: string
  value: string | number
}

export type AdminFieldType =
  | 'text'
  | 'number'
  | 'textarea'
  | 'select'
  | 'email'
  | 'date'
  | 'datetime-local'
  | 'json'
  | 'checkbox'
  | 'time'
  | 'decimal'

export type AdminFieldFormat =
  | 'text'
  | 'datetime'
  | 'date'
  | 'boolean'
  | 'json'
  | 'numeric'
  | 'currency'

export type AdminFormField = {
  key: string
  label: string
  type?: AdminFieldType
  placeholder?: string
  required?: boolean
  options?: AdminFieldOption[]
  defaultValue?: any
  help?: string
  lookupEntity?: LookupEntity
  rows?: number
  step?: string
}

export type AdminTableColumn = {
  key: string
  label: string
  format?: AdminFieldFormat
  lookupEntity?: LookupEntity
}

export type AdminResourceGroup = 'core' | 'management'

export type AdminResourceConfig = {
  slug: string
  label: string
  singularLabel: string
  description: string
  group: AdminResourceGroup
  icon: string
  endpoint: string
  fields: AdminFormField[]
  columns: AdminTableColumn[]
  titleKey?: string
  canCreate?: boolean
  canEdit?: boolean
  canDelete?: boolean
  canShow?: boolean
}

function idField(
  key: string,
  label: string,
  options: Partial<AdminFormField> = {},
): AdminFormField {
  return {
    key,
    label,
    type: 'text',
    placeholder: 'UUID',
    ...options,
  }
}

function lookupField(
  key: string,
  label: string,
  lookupEntity: LookupEntity,
  options: Partial<AdminFormField> = {},
): AdminFormField {
  return {
    key,
    label,
    type: 'select',
    lookupEntity,
    ...options,
  }
}

function jsonField(
  key: string,
  label: string,
  options: Partial<AdminFormField> = {},
): AdminFormField {
  return {
    key,
    label,
    type: 'json',
    defaultValue: '{}',
    placeholder: '{\n  "key": "value"\n}',
    ...options,
  }
}

export const ADMIN_RESOURCES: AdminResourceConfig[] = [
  {
    slug: 'users',
    label: 'Usuários',
    singularLabel: 'Usuário',
    description: 'Consulta operacional dos usuários e seus dados públicos de cadastro.',
    group: 'management',
    icon: 'user',
    endpoint: '/admin/users',
    titleKey: 'email',
    canCreate: false,
    canEdit: false,
    canDelete: false,
    canShow: true,
    fields: [],
    columns: [
      { key: 'display_name', label: 'Nome de exibição' },
      { key: 'email', label: 'Email' },
      { key: 'first_name', label: 'Nome' },
      { key: 'last_name', label: 'Sobrenome' },
    ],
  },
  {
    slug: 'roles',
    label: 'Cargos',
    singularLabel: 'Cargo',
    description: 'Papéis de acesso associados a usuários e escopos específicos.',
    group: 'management',
    icon: 'shield',
    endpoint: '/admin/roles',
    titleKey: 'name',
    canCreate: true,
    canEdit: true,
    canDelete: true,
    canShow: true,
    fields: [
      { key: 'name', label: 'Nome', required: true, placeholder: 'admin' },
      { key: 'resource_type', label: 'Tipo de recurso', placeholder: 'company' },
      idField('resource_id', 'ID do recurso'),
    ],
    columns: [
      { key: 'name', label: 'Nome' },
      { key: 'resource_type', label: 'Tipo de recurso' },
      { key: 'resource_id', label: 'ID do recurso' },
      { key: 'updated_at', label: 'Atualizado em', format: 'datetime' },
    ],
  },
  {
    slug: 'audit-logs',
    label: 'Auditoria',
    singularLabel: 'Log',
    description: 'Trilha de auditoria para ações críticas e eventos relevantes.',
    group: 'management',
    icon: 'history',
    endpoint: '/admin/audit-logs',
    titleKey: 'action',
    canCreate: true,
    canEdit: true,
    canDelete: true,
    canShow: true,
    fields: [
      lookupField('actor_user_id', 'Usuário executor', 'users'),
      { key: 'actor_role_snapshot', label: 'Perfil capturado' },
      { key: 'action', label: 'Ação', required: true, placeholder: 'user.created' },
      { key: 'resource_type', label: 'Tipo de recurso', required: true, placeholder: 'user' },
      idField('resource_id', 'ID do recurso'),
      { key: 'ip_address', label: 'IP' },
      { key: 'user_agent', label: 'User agent', type: 'textarea' },
      idField('request_id', 'Request ID'),
      jsonField('changes', 'Changes'),
      jsonField('metadata', 'Metadata'),
    ],
    columns: [
      { key: 'action', label: 'Ação' },
      { key: 'resource_type', label: 'Recurso' },
      { key: 'actor_user_id', label: 'Executor', lookupEntity: 'users' },
      { key: 'created_at', label: 'Criado em', format: 'datetime' },
    ],
  },
]

export const ADMIN_RESOURCE_GROUP_LABELS: Record<AdminResourceGroup, string> = {
  core: 'Estrutura Organizacional',
  management: 'Gestão',
}

function formatDateTimeLocalValue(value: unknown) {
  if (!value) return ''

  const date = new Date(String(value))
  if (Number.isNaN(date.getTime())) return ''

  const year = date.getFullYear()
  const month = `${date.getMonth() + 1}`.padStart(2, '0')
  const day = `${date.getDate()}`.padStart(2, '0')
  const hours = `${date.getHours()}`.padStart(2, '0')
  const minutes = `${date.getMinutes()}`.padStart(2, '0')

  return `${year}-${month}-${day}T${hours}:${minutes}`
}

function formatDateValue(value: unknown) {
  if (!value) return ''

  const date = new Date(String(value))
  if (Number.isNaN(date.getTime())) return ''

  const year = date.getFullYear()
  const month = `${date.getMonth() + 1}`.padStart(2, '0')
  const day = `${date.getDate()}`.padStart(2, '0')

  return `${year}-${month}-${day}`
}

export function getAdminResourceConfig(slug: string) {
  return ADMIN_RESOURCES.find((resource) => resource.slug === slug)
}

export function createAdminModel(
  config: AdminResourceConfig,
  source: Record<string, any> | null = null,
) {
  const model: Record<string, any> = {}

  for (const field of config.fields) {
    const sourceValue = source?.[field.key]

    if (sourceValue !== undefined && sourceValue !== null) {
      if (field.type === 'json') {
        model[field.key] = JSON.stringify(sourceValue, null, 2)
        continue
      }

      if (field.type === 'checkbox') {
        model[field.key] = Boolean(sourceValue)
        continue
      }

      if (field.type === 'datetime-local') {
        model[field.key] = formatDateTimeLocalValue(sourceValue)
        continue
      }

      if (field.type === 'date') {
        model[field.key] = formatDateValue(sourceValue)
        continue
      }

      model[field.key] = sourceValue
      continue
    }

    if (field.defaultValue !== undefined) {
      model[field.key] = field.defaultValue
      continue
    }

    model[field.key] = field.type === 'checkbox' ? false : ''
  }

  return model
}

export function buildAdminPayload(config: AdminResourceConfig, model: Record<string, any>) {
  const payload: Record<string, any> = {}

  for (const field of config.fields) {
    const raw = model[field.key]

    if (field.type === 'checkbox') {
      payload[field.key] = Boolean(raw)
      continue
    }

    if (raw === '' || raw === undefined || raw === null) {
      continue
    }

    if (field.type === 'number') {
      const parsed = Number(raw)
      payload[field.key] = Number.isNaN(parsed) ? raw : parsed
      continue
    }

    if (field.type === 'decimal') {
      payload[field.key] = String(raw).replace(',', '.')
      continue
    }

    if (field.type === 'json') {
      try {
        payload[field.key] = typeof raw === 'string' ? JSON.parse(raw) : raw
      } catch {
        throw new Error(`Campo JSON invalido: ${field.label}`)
      }
      continue
    }

    if (field.type === 'datetime-local') {
      const parsedDate = new Date(String(raw))
      payload[field.key] = Number.isNaN(parsedDate.getTime())
        ? raw
        : parsedDate.toISOString()
      continue
    }

    payload[field.key] = raw
  }

  return payload
}

export function normalizeResourceResponse(payload: any): Record<string, any>[] {
  if (Array.isArray(payload)) return payload
  if (payload && Array.isArray(payload.data)) return payload.data
  return []
}

export function extractResourceItem(payload: any): Record<string, any> | null {
  if (!payload) return null
  if (Array.isArray(payload)) return payload[0] ?? null
  if (payload.data && !Array.isArray(payload.data)) return payload.data
  return payload
}

export function extractResourceId(payload: any) {
  const item = extractResourceItem(payload)
  return item?.id ? String(item.id) : null
}