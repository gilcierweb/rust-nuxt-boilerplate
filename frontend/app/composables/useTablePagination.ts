import type { Ref, ComputedRef } from 'vue'
import type { SortingState } from '@tanstack/vue-table'
import {
  extractPaginationMeta,
  normalizeResourceResponse,
  type ResourcePaginationMeta,
} from '~/utils/admin-resources'
import { extractErrorMessage } from '~/utils/admin-ui'

export interface UseTablePaginationOptions {
  defaultPageSize?: number
  pageSizes?: number[]
}

export interface TablePaginationState {
  page: number
  perPage: number
}

export interface UseTablePaginationReturn<T extends Record<string, any>> {
  pagination: Ref<TablePaginationState>
  paginationMeta: Ref<ResourcePaginationMeta>
  pageSizes: number[]
  items: ComputedRef<T[]>
  pending: ComputedRef<boolean>
  error: Ref<string>
  refresh: () => Promise<void>
  setPage: (page: number) => void
  setPageSize: (size: number) => void
  sorting: Ref<SortingState>
  setSorting: (state: SortingState) => void
  search: Ref<string>
  setSearch: (term: string) => void
}

export interface TablePaginationFetcherArgs {
  /** The list endpoint, relative to the api base. */
  url: string
  /**
   * The key of the fetch, one per screen: two lists of the same endpoint read
   * different data, so they cannot share a key.
   */
  key?: string
  options?: Record<string, any>
}

const DEFAULT_PAGE_SIZES: number[] = [10, 20, 30, 50]

/// The quiet time before a typed term reaches the server. A keystroke is
/// faster than a request, so the term is read once the typing pauses instead
/// of once per character.
const SEARCH_DEBOUNCE_MS = 300

export function useTablePagination<T extends Record<string, any>>(
  fetcher: TablePaginationFetcherArgs | (() => TablePaginationFetcherArgs),
  options: UseTablePaginationOptions = {},
): UseTablePaginationReturn<T> {
  const pageSizes: number[] = options.pageSizes ?? [...DEFAULT_PAGE_SIZES]
  const initialSize = options.defaultPageSize ?? pageSizes[0] ?? 10

  const pagination = ref<TablePaginationState>({ page: 1, perPage: initialSize })
  const sorting = ref<SortingState>([])
  // The search runs server side: the list endpoint resolves `q` against its
  // own columns, so the term travels as a query parameter.
  const search = ref('')

  const resolvedFetcher = typeof fetcher === 'function' ? fetcher() : fetcher
  const baseOptions = resolvedFetcher.options ?? {}

  const pageQuery = computed(() => pagination.value.page)
  const perPageQuery = computed(() => pagination.value.perPage)
  const sortByQuery = computed(() => sorting.value[0]?.id ?? null)
  const sortDirQuery = computed(() =>
    sorting.value[0] ? (sorting.value[0].desc ? 'desc' : 'asc') : null,
  )

  const fetchData = async () => {
    const { $api } = useNuxtApp()
    // The same base the rest of the app reads, so a configured direct API base
    // routes the tables the same way it routes the imperative client.
    const config = useRuntimeConfig()
    const directBase = config.public.apiDirectBase?.replace(/\/+$/, '')
    const apiBase = directBase || config.public.apiBase || '/api/v1'
    const url = resolvedFetcher.url.startsWith('/api/')
      ? resolvedFetcher.url
      : `${apiBase}${resolvedFetcher.url.startsWith('/') ? '' : '/'}${resolvedFetcher.url}`
      ? resolvedFetcher.url 
      : `${apiBase}${resolvedFetcher.url.startsWith('/') ? '' : '/'}${resolvedFetcher.url}`

    const response = await $api<any>(url, {
      method: 'GET',
      ...baseOptions,
      query: {
        page: pageQuery.value,
        per_page: perPageQuery.value,
        sort_by: sortByQuery.value,
        sort_dir: sortDirQuery.value,
        ...(search.value ? { q: search.value } : {}),
      },
    })
    return response
  }

  const { data, pending, error, refresh, execute } = useAsyncData(
    () => resolvedFetcher.key ?? resolvedFetcher.url,
    fetchData,
    {
      ...baseOptions,
      server: true,
      default: () => ({ data: [], pagination: null }),
      immediate: true,
    }
  )

  const items = computed<T[]>(() => normalizeResourceResponse(data.value) as T[])
  const paginationMeta = computed<ResourcePaginationMeta>(() => extractPaginationMeta(data.value))
  const requestError = ref<string>('')

  watch(
    () => error.value,
    (err) => {
      requestError.value = err ? extractErrorMessage(err) : ''
    },
    { immediate: true },
  )

  function setPage(page: number) {
    const next = Math.max(1, Math.floor(page))
    if (next === pagination.value.page) return
    pagination.value = { ...pagination.value, page: next }
    execute()
  }

  function setPageSize(size: number) {
    if (size === pagination.value.perPage) return
    pagination.value = { page: 1, perPage: size }
    execute()
  }

  function setSorting(state: SortingState) {
    if (state === sorting.value) return
    pagination.value = { ...pagination.value, page: 1 }
    sorting.value = state
    execute()
  }

  // The search travels as a query parameter, so every keystroke would fire a
  // request: the term is debounced and the request runs on the quiet term.
  let searchTimer: ReturnType<typeof setTimeout> | null = null

  function setSearch(term: string) {
    if (term === search.value) return
    if (searchTimer) clearTimeout(searchTimer)
    searchTimer = setTimeout(() => {
      searchTimer = null
      pagination.value = { ...pagination.value, page: 1 }
      search.value = term
      execute()
    }, SEARCH_DEBOUNCE_MS)
  }

  return {
    pagination: pagination as Ref<TablePaginationState>,
    paginationMeta: paginationMeta as Ref<ResourcePaginationMeta>,
    pageSizes,
    items: items as ComputedRef<T[]>,
    pending: computed(() => pending.value),
    error: requestError,
    refresh: execute,
    setPage,
    setPageSize,
    sorting: sorting as Ref<SortingState>,
    setSorting,
    search: search as Ref<string>,
    setSearch,
  }
}
