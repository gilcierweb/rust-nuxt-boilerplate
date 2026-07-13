/**
 * Generic pagination composable.
 * Works with API endpoints that return:
 * - Cursor-based: { data/posts/creators/etc, next_cursor, has_more }
 * - Offset-based: { data, pagination: { page, per_page, has_next } }
 */
export function usePagination<T>(
  fetcher: (cursor?: string, page?: number) => Promise<{
    data?: T[]
    posts?: T[]
    creators?: T[]
    subscribers?: T[]
    notifications?: T[]
    next_cursor?: string | null
    has_more?: boolean
    pagination?: {
      page: number
      per_page: number
      total: number
      has_next: boolean
    }
  }>,
  options: { mode?: 'cursor' | 'offset' } = {}
) {
  const mode = options.mode || 'offset' // Default to offset-based

  const items = ref<T[]>([]) as Ref<T[]>
  const isLoading = ref(false)
  const isLoadingMore = ref(false)
  const hasMore = ref(true)
  const cursor = ref<string | null>(null)
  const page = ref(1)
  const error = ref<string | null>(null)

  async function load(reset = false) {
    if (isLoading.value && !reset) return
    if (!hasMore.value && !reset) return

    if (reset) {
      isLoading.value = true
      cursor.value = null
      page.value = 1
      hasMore.value = true
    } else {
      isLoadingMore.value = true
    }

    error.value = null

    try {
      const response = mode === 'cursor'
        ? await fetcher(reset ? undefined : cursor.value ?? undefined, undefined)
        : await fetcher(undefined, reset ? 1 : page.value)

      // Support different response key names
      const newItems = (
        response.data ??
        response.posts ??
        response.creators ??
        response.subscribers ??
        response.notifications ??
        []
      ) as T[]

      if (reset) {
        items.value = newItems
      } else {
        items.value = [...items.value, ...newItems]
      }

      // Handle pagination
      if (mode === 'cursor') {
        cursor.value = response.next_cursor ?? null
        hasMore.value = response.has_more ?? false
      } else {
        // Offset mode
        if (response.pagination) {
          page.value = response.pagination.page + 1
          hasMore.value = response.pagination.has_next
        } else {
          // Fallback for endpoints that only return data
          hasMore.value = newItems.length > 0
          page.value++
        }
      }
    } catch (err: any) {
      error.value = err.statusMessage || 'Failed to load'
    } finally {
      isLoading.value = false
      isLoadingMore.value = false
    }
  }

  function reset() {
    return load(true)
  }

  function loadMore() {
    if (!hasMore.value || isLoadingMore.value) return
    return load(false)
  }

  return {
    items,
    isLoading,
    isLoadingMore,
    hasMore,
    cursor,
    page,
    error,
    load,
    reset,
    loadMore,
  }
}