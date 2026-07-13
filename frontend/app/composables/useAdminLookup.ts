import { normalizeResourceResponse } from '~/utils/admin-resources'

export type LookupEntity =
  | 'companies'
  | 'customers'
  | 'debt_categories'
  | 'debts'
  | 'documents'
  | 'invoice_requests'
  | 'payment_transactions'
  | 'storage_objects'
  | 'roles'
  | 'users'
  | 'company_domains'
  | 'company_settings'
  | 'customer_users'
  | 'issued_invoices'

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
  companies: '/admin/companies',
  customers: '/admin/customers',
  debt_categories: '/admin/debt-categories',
  debts: '/admin/debts',
  documents: '/admin/documents',
  invoice_requests: '/admin/invoice-requests',
  payment_transactions: '/admin/payment-transactions',
  storage_objects: '/admin/storage-objects',
  roles: '/admin/roles',
  users: '/admin/users',
  company_domains: '/admin/company-domains',
  company_settings: '/admin/company-settings',
  customer_users: '/admin/customer-users',
  issued_invoices: '/admin/issued-invoices',
}

function getLabel(entity: LookupEntity, item?: EntityItem): string {
  if (!item) return '-'

  switch (entity) {
    case 'companies':
      return item.legal_name || item.trade_name || item.name || item.id
    case 'customers':
      return item.customer_code || `${item.first_name || ''} ${item.last_name || ''}`.trim() || item.email || item.id
    case 'debt_categories':
      return item.code ? `${item.code} - ${item.name || ''}`.trim() : (item.name || item.id)
    case 'debts':
      return item.title || item.code || item.description || item.id
    case 'documents':
      return item.title || item.description || item.id
    case 'invoice_requests':
      return item.fiscal_provider_reference || item.title || item.description || item.code || item.id
    case 'payment_transactions':
      return item.provider_reference || item.id
    case 'storage_objects':
      return item.original_file_name || item.object_key || item.id
    case 'roles':
      return item.name || item.id
    case 'users':
      return item.display_name || `${item.first_name || ''} ${item.last_name || ''}`.trim() || item.email || item.id
    case 'company_domains':
      return item.host || item.id
    case 'company_settings':
      return item.company_id || item.id
    case 'customer_users':
      return item.customer_id || item.id
    case 'issued_invoices':
      return item.invoice_number || item.provider_reference || item.id
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
