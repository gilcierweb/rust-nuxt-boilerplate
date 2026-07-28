import type { Ref, ComputedRef } from 'vue'
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
  setPage: (page: number) => Promise<void>
  setPageSize: (size: number) => Promise<void>
}

export interface TablePaginationFetcherArgs {
  url: string
  options?: Record<string, any>
}

const DEFAULT_PAGE_SIZES = [10, 20, 30, 50] as const

export async function useTablePagination<T extends Record<string, any>>(
  fetcher: () => TablePaginationFetcherArgs | Promise<TablePaginationFetcherArgs>,
  options: UseTablePaginationOptions = {},
): Promise<UseTablePaginationReturn<T>> {
  const pageSizes = options.pageSizes ?? [...DEFAULT_PAGE_SIZES]
  const initialSize = options.defaultPageSize ?? pageSizes[0]

  const pagination = ref<TablePaginationState>({ page: 1, perPage: initialSize })

  const resolvedFetcher = await Promise.resolve(fetcher())

  const { data, pending, error, refresh } = await useApiFetch<any>(
    () => resolvedFetcher.url,
    () => ({
      ...(resolvedFetcher.options ?? {}),
      server: true,
      default: () => ({ data: [], pagination: null }),
      query: { page: pagination.value.page, per_page: pagination.value.perPage },
      watch: [pagination],
    }),
  )

  const items = computed<T[]>(() => normalizeResourceResponse(data.value) as T[])
  const paginationMeta = computed<ResourcePaginationMeta>(() => extractPaginationMeta(data.value))
  const requestError = computed(() => (error.value ? extractErrorMessage(error.value) : ''))

  async function setPage(page: number) {
    const next = Math.max(1, Math.floor(page))
    if (next === pagination.value.page) return
    pagination.value = { ...pagination.value, page: next }
  }

  async function setPageSize(size: number) {
    if (size === pagination.value.perPage) return
    pagination.value = { page: 1, perPage: size }
  }

  return {
    pagination: pagination as Ref<TablePaginationState>,
    paginationMeta: paginationMeta as Ref<ResourcePaginationMeta>,
    pageSizes,
    items: items as ComputedRef<T[]>,
    pending: computed(() => pending.value),
    error: requestError as Ref<string>,
    refresh,
    setPage,
    setPageSize,
  }
}
