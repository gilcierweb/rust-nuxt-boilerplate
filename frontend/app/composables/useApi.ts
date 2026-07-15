import type { FetchError } from 'ofetch'

type ApiMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE'

type ApiRequestOptions = {
  method?: ApiMethod
  body?: any
  query?: Record<string, any>
  params?: Record<string, any>
  headers?: HeadersInit
}

/**
 * Determines the API base URL based on the execution context.
 * - SSR: Always use the proxy ('/api/v1') for proper hydration
 * - CSR: Use direct backend URL if configured (NUXT_PUBLIC_API_BASE), otherwise fall back to proxy
 */
function getApiBase(config: any): string {
  // SSR always uses proxy for proper hydration
  if (import.meta.server) {
    return '/api/v1'
  }
  
  // CSR: Check if direct API base is configured
  const directBase = config.public.apiDirectBase?.replace(/\/+$/, '')
  if (directBase) {
    return directBase
  }
  
  // Fallback to proxy
  return config.public.apiBase || '/api/v1'
}

export const useApi = () => {
  const { $api } = useNuxtApp()
  const { clearFormAlertMessage, setFormAlertMessage } = useFormAlert()

  // Imperative client for event-driven work such as form submissions and mutations.
  const request = async <T>(endpoint: string, options: ApiRequestOptions = {}) => {
    try {
      const response = await $api<T>(endpoint, options)
      const maybeError = (response as any)?.error

      if (maybeError?.message) {
        setFormAlertMessage(String(maybeError.message))
        const apiError: any = new Error(String(maybeError.message))
        apiError.statusCode = 400
        apiError.code = 'BAD_REQUEST'
        apiError.data = response
        throw apiError
      }

      clearFormAlertMessage()
      return response
    } catch (error: any) {
      const apiMessage =
        error?.data?.error?.message ||
        error?.response?._data?.error?.message

      if (apiMessage) {
        setFormAlertMessage(String(apiMessage))
        const normalizedError: any = new Error(String(apiMessage))
        normalizedError.statusCode = error?.statusCode || error?.response?.status || 400
        normalizedError.data = error?.data || error?.response?._data
        normalizedError.response = error?.response
        normalizedError.cause = error
        throw normalizedError
      }

      throw error
    }
  }

  return {
    request,
    get: <T>(endpoint: string, options: Omit<ApiRequestOptions, 'method' | 'body'> = {}) =>
      request<T>(endpoint, { ...options, method: 'GET' }),
    post: <T>(endpoint: string, options: Omit<ApiRequestOptions, 'method'> = {}) =>
      request<T>(endpoint, { ...options, method: 'POST' }),
    put: <T>(endpoint: string, options: Omit<ApiRequestOptions, 'method'> = {}) =>
      request<T>(endpoint, { ...options, method: 'PUT' }),
    patch: <T>(endpoint: string, options: Omit<ApiRequestOptions, 'method'> = {}) =>
      request<T>(endpoint, { ...options, method: 'PATCH' }),
    delete: <T>(endpoint: string, options: Omit<ApiRequestOptions, 'method' | 'body'> = {}) =>
      request<T>(endpoint, { ...options, method: 'DELETE' }),
  }
}

function isAbsoluteUrl(value: string) {
  return /^https?:\/\//i.test(value)
}

function withApiBase(value: string, apiBase: string) {
  if (!value.startsWith('/')) {
    return `${apiBase}/${value}`
  }

  return `${apiBase}${value}`
}

function normalizeApiUrl(value: string, apiBase: string) {
  if (isAbsoluteUrl(value) || value.startsWith(apiBase)) {
    return value
  }

  if (value.startsWith('/api/')) {
    return value
  }

  return withApiBase(value, apiBase)
}

function withRouteFetchDefaults(options: Record<string, any>) {
  // Default behavior: use SSR (server: true) unless explicitly disabled
  // This ensures data is fetched on server and hydrated to client for better UX
  return options
}

export function useApiFetch<T>(url: string | (() => string), options: Record<string, any> = {}) {
  const { $api } = useNuxtApp()
  const config = useRuntimeConfig()
  const apiBase = getApiBase(config)
  const fetchOptions = withRouteFetchDefaults(options)
  const route = useRoute()
  const resolvedUrl =
    typeof url === 'function'
      ? () => normalizeApiUrl(url(), apiBase)
      : normalizeApiUrl(url, apiBase)

  return useFetch<T, FetchError>(resolvedUrl, {
    ...fetchOptions,
    $fetch: $api as typeof $fetch,
  })
}

export function useApiLazyFetch<T>(
  url: string | (() => string),
  options: Record<string, any> = {},
) {
  const { $api } = useNuxtApp()
  const config = useRuntimeConfig()
  const apiBase = getApiBase(config)
  const fetchOptions = withRouteFetchDefaults(options)
  const resolvedUrl =
    typeof url === 'function'
      ? () => normalizeApiUrl(url(), apiBase)
      : normalizeApiUrl(url, apiBase)

  return useLazyFetch<T, FetchError>(resolvedUrl, {
    ...fetchOptions,
    $fetch: $api as typeof $fetch,
  })
}

export function useApiAsyncData<T>(
  key: string,
  handler: () => Promise<T>,
  options?: Record<string, any>,
) {
  return useAsyncData(key, handler, options)
}
