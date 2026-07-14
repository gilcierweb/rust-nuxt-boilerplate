import { normalizeResourceResponse } from '~/utils/admin-resources'

export type LookupEntity =
  | 'roles'
  | 'users'
  | 'audit_logs'

type EntityItem = {
  id: string
  code?: string
  name?: string
  display_name?: string
  legal_name?: string
  trade_name?: string
  first_name?: string
  last_name?: string
  email?: string
  title?: string
  description?: string
  original_file_name?: string
  customer_code?: string
  provider_reference?: string
  fiscal_provider_reference?: string
  host?: string
  company_id?: string
  customer_id?: string
  invoice_number?: string
  object_key?: string
}

const ENDPOINTS: Record<LookupEntity, string> = {
  roles: '/admin/roles',
  users: '/admin/users',
  audit_logs: '/admin/audit-logs',
}

function getLabel(entity: LookupEntity, item?: EntityItem): string {
  if (!item) return '-'

  switch (entity) {
    case 'roles':
      return item.name || item.id
    case 'users':
      return item.display_name || `${item.first_name || ''} ${item.last_name || ''}`.trim() || item.email || item.id
    case 'audit_logs':
      return item.action || item.resource_type || item.id
    default:
      return item.name || item.id
  }
}

export function useAdminLookup() {
  const api = useApi()

  const itemsByEntity = useState<Partial<Record<LookupEntity, EntityItem[]>>>(
    'admin-lookup-items',
    () => ({}),
  )
  const loadingByEntity = useState<Partial<Record<LookupEntity, boolean>>>(
    'admin-lookup-loading',
    () => ({}),
  )
  const loadedByEntity = useState<Partial<Record<LookupEntity, boolean>>>(
    'admin-lookup-loaded',
    () => ({}),
  )

  async function load(entity: LookupEntity) {
    if (loadedByEntity.value[entity] || loadingByEntity.value[entity]) return

    loadingByEntity.value = { ...loadingByEntity.value, [entity]: true }
    try {
      const payload = await api.get<any>(ENDPOINTS[entity])
      itemsByEntity.value = {
        ...itemsByEntity.value,
        [entity]: normalizeResourceResponse(payload) as EntityItem[],
      }
      loadedByEntity.value = { ...loadedByEntity.value, [entity]: true }
    } catch {
      itemsByEntity.value = { ...itemsByEntity.value, [entity]: [] }
    } finally {
      loadingByEntity.value = { ...loadingByEntity.value, [entity]: false }
    }
  }

  function getItems(entity: LookupEntity): EntityItem[] {
    return itemsByEntity.value[entity] || []
  }

  function isLoading(entity: LookupEntity): boolean {
    return loadingByEntity.value[entity] || false
  }

  function resolveLabel(entity: LookupEntity, id?: string | null): string {
    if (!id) return '-'
    const item = getItems(entity).find((entry) => entry.id === id)
    return getLabel(entity, item) || id
  }

  function invalidate(entity?: LookupEntity) {
    if (!entity) {
      itemsByEntity.value = {}
      loadingByEntity.value = {}
      loadedByEntity.value = {}
      return
    }

    itemsByEntity.value = { ...itemsByEntity.value, [entity]: [] }
    loadingByEntity.value = { ...loadingByEntity.value, [entity]: false }
    loadedByEntity.value = { ...loadedByEntity.value, [entity]: false }
  }

  return {
    load,
    getItems,
    isLoading,
    resolveLabel,
    invalidate,
  }
}
