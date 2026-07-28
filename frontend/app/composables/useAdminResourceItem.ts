import type { ComputedRef, Ref } from 'vue'
import { extractErrorMessage } from '~/utils/admin-ui'

export interface UseAdminResourceItemReturn<T> {
  itemId: ComputedRef<string | null>
  item: ComputedRef<T | null>
  raw: ComputedRef<any>
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
 * DRY replacement for the repeated `route.params.id as string` + `useApiFetch`
 * pattern that was producing `/admin/<resource>/undefined` requests.
 */
export function useAdminResourceItem<T extends Record<string, any>>(
  resource: string,
  options: { keyPrefix?: string } = {},
): UseAdminResourceItemReturn<T> {
  const { keyPrefix } = options
  const itemId = useRouteResourceId()
  const api = useApi()

  const data = ref<T | null>(null) as Ref<T | null>
  const pending = ref(false)
  const error = ref<unknown>(null)
  let lastFetchedId: string | null = null

  async function fetchItem(id: string) {
    pending.value = true
    error.value = null
    lastFetchedId = id
    try {
      const response = await api.request<any>(`/admin/${resource}/${id}`, { method: 'GET' })
      if (lastFetchedId !== id) return
      data.value = unwrapItem<T>(response)
    } catch (err) {
      if (lastFetchedId !== id) return
      error.value = err
      data.value = null
    } finally {
      if (lastFetchedId === id) pending.value = false
    }
  }

  watch(
    itemId,
    (id) => {
      if (!id) {
        pending.value = false
        error.value = null
        data.value = null
        return
      }
      fetchItem(id)
    },
    { immediate: true },
  )

  const item = computed<T | null>(() => data.value)
  const requestError = computed(() => (error.value ? extractErrorMessage(error.value) : ''))

  return {
    itemId,
    item,
    raw: computed(() => data.value),
    pending: computed(() => pending.value),
    error: requestError as Ref<string>,
    refresh: async () => {
      if (itemId.value) await fetchItem(itemId.value)
    },
  }
}
