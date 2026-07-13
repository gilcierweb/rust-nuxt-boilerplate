import { normalizeResourceResponse } from '~/utils/admin-resources'

type LookupEntity =
  | 'company'
  | 'site'
  | 'device'
  | 'reservoir'
  | 'pump'
  | 'sensor'
  | 'telemetryEvent'
  | 'alert'
  | 'report'

type EntityItem = {
  id: string
  code?: string
  name?: string
  title?: string
  legal_name?: string
  trade_name?: string
  event_key?: string
}

const ENDPOINTS: Record<LookupEntity, string> = {
  company: '/admin/companies',
  site: '/admin/sites',
  device: '/admin/devices',
  reservoir: '/admin/reservoirs',
  pump: '/admin/pumps',
  sensor: '/admin/sensors',
  telemetryEvent: '/admin/telemetry-events',
  alert: '/admin/alerts',
  report: '/admin/reports',
}

function getLabel(entity: LookupEntity, id: string, item?: EntityItem) {
  if (!item) return id

  switch (entity) {
    case 'company':
      return item.legal_name || item.trade_name || id
    case 'site':
    case 'device':
    case 'reservoir':
    case 'pump':
    case 'sensor':
      return item.code ? `${item.code} - ${item.name || id}` : (item.name || id)
    case 'telemetryEvent':
      return item.event_key || id
    case 'alert':
    case 'report':
      return item.title || id
    default:
      return id
  }
}

export function useAdminEntityLookup() {
  const api = useApi()

  const itemsByEntity = useState<Partial<Record<LookupEntity, EntityItem[]>>>(
    'admin-entity-lookup-items',
    () => ({}),
  )
  const loadingByEntity = useState<Partial<Record<LookupEntity, boolean>>>(
    'admin-entity-lookup-loading',
    () => ({}),
  )
  const loadedByEntity = useState<Partial<Record<LookupEntity, boolean>>>(
    'admin-entity-lookup-loaded',
    () => ({}),
  )

  async function ensureLoaded(entity: LookupEntity) {
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

  function resolve(entity: LookupEntity, id?: string | null) {
    const rawId = id ?? null
    if (!rawId) return '-'
    const item = (itemsByEntity.value[entity] || []).find((entry) => entry.id === rawId)
    return getLabel(entity, rawId, item)
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
    ensureLoaded,
    resolve,
    invalidate,
  }
}
