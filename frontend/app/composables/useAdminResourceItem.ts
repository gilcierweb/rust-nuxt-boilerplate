import type { ComputedRef, Ref } from 'vue'
import { mapAdminError } from '~/utils/admin-errors'
import { useApiAsyncData } from '~/composables/useApi'

export interface UseAdminResourceItemReturn<T> {
  itemId: ComputedRef<string | null>
  item: ComputedRef<T | null>
  raw: ComputedRef<T | null>
  pending: ComputedRef<boolean>
  error: Ref<string>
  refresh: () => Promise<void>
}

/**
 * Resolve the id route param safely. Returns `null` when the param is missing,
 * empty, or the literal string "undefined" — preventing `/admin/<resource>/undefined`
 * requests from ever being fired.
 */
function useRouteResourceId(): ComputedRef<string | null> {
  const route = useRoute()
  return computed(() => {
    const raw = route.params.id
    if (raw === undefined || raw === null) return null
    const value = String(raw)
    return value === '' || value === 'undefined' ? null : value
  })
}

/**
 * Unwrap a normalized resource item from either a bare object or an
 * `{ data: T }` envelope.
 */
function unwrapItem<T>(payload: any): T | null {
  if (!payload) return null
  if (typeof payload !== 'object') return payload as T
  if ('data' in payload && !Array.isArray(payload.data) && payload.data !== null) {
    return payload.data as T
  }
  return payload as T
}

/**
 * Composable for admin `[id]` detail/edit pages. Extracts the route id safely,
 * short-circuits the fetch when id is invalid, and normalizes the response shape.
 *
 * The item travels through `useAsyncData`, so the server render fills the
 * payload and the client hydration reads the item from it: the first client
 * render carries the item, and direct loads no longer flash the empty state
 * before the standalone ref fetch answers.
 */
export function useAdminResourceItem<T extends Record<string, any>>(
  resource: string,
  options: { keyPrefix?: string } = {},
): UseAdminResourceItemReturn<T> {
  const { keyPrefix } = options
  const itemId = useRouteResourceId()
  const api = useApi()

  const keyBase = keyPrefix ?? `admin-${resource}-item`
  const cacheKey = computed(() => `${keyBase}-${itemId.value ?? 'none'}`)

  const { data, pending, error: rawError, refresh: refreshData } = useApiAsyncData<T | null>(
    cacheKey,
    async () => {
      if (!itemId.value) return null
      const response = await api.request<any>(`/admin/${resource}/${itemId.value}`, { method: 'GET' })
      return unwrapItem<T>(response)
    },
  )

  const { t } = useI18n()
  const item = computed<T | null>(() => data.value ?? null)
  const error = computed(() => (rawError.value ? mapAdminError(rawError.value, t) : ''))

  return {
    itemId,
    item,
    raw: item,
    pending,
    error: error as Ref<string>,
    refresh: async () => {
      if (itemId.value) await refreshData()
    },
  }
}