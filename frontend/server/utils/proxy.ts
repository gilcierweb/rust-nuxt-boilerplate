import {
  appendResponseHeader,
  createError,
  getRequestHeaders,
  setResponseHeader,
  setResponseStatus,
  type H3Event,
} from 'h3'

type RawFetchResponse<T = any> = {
  status: number
  statusText: string
  headers: Headers
  _data: T
}

export function normalizeBackendApiBase(value: string): string {
  const trimmed = String(value || '').replace(/\/+$/, '')
  if (!trimmed) return ''
  if (/\/api\/v\d+$/i.test(trimmed)) {
    return trimmed
  }
  return `${trimmed}/api/v1`
}

export function candidateBackendApiBases(
  value: string,
  includeRawBase = false,
): string[] {
  const trimmed = String(value || '').replace(/\/+$/, '')
  const normalized = normalizeBackendApiBase(trimmed)
  const list = includeRawBase ? [normalized, trimmed] : [normalized]
  return Array.from(new Set(list.filter(Boolean)))
}

export function requireBackendProxyConfig(
  event: H3Event,
  includeRawBase = false,
): { backendApiBases: string[]; backendApiKey: string } {
  const config = useRuntimeConfig(event)
  const backendApiBases = candidateBackendApiBases(
    String(config.backendApiBase || ''),
    includeRawBase,
  )
  const backendApiKey = String(config.backendApiKey || '')

  if (backendApiBases.length === 0 || !backendApiKey) {
    throw createError({
      statusCode: 500,
      statusMessage: 'Backend proxy is not configured',
    })
  }

  return { backendApiBases, backendApiKey }
}

export function extractSetCookies(headers: Headers): string[] {
  const getSetCookie = (headers as Headers & { getSetCookie?: () => string[] })
    .getSetCookie
  if (typeof getSetCookie === 'function') {
    return getSetCookie.call(headers)
  }

  const combined = headers.get('set-cookie')
  if (!combined) {
    return []
  }

  return combined
    .split(/,(?=\s*[^;,\s]+=)/)
    .map((cookie) => cookie.trim())
    .filter(Boolean)
}

export function createForwardHeaders(
  event: H3Event,
  backendApiKey: string,
  headerNames: string[],
  accessToken?: string | null,
): Headers {
  const incomingHeaders = getRequestHeaders(event)
  const headers = new Headers()
  headers.set('x-api-key', backendApiKey)

  for (const headerName of headerNames) {
    const value = incomingHeaders[headerName]
    if (typeof value === 'string' && value.length > 0) {
      headers.set(headerName, value)
    } else if (Array.isArray(value) && value.length > 0) {
      const joinedValue =
        headerName === 'cookie' ? value.join('; ') : value.join(', ')
      headers.set(headerName, joinedValue)
    }
  }

  if (accessToken) {
    headers.set('authorization', `Bearer ${accessToken}`)
  }

  return headers
}

export function applyProxyResponse<T = any>(
  event: H3Event,
  response: RawFetchResponse<T>,
): T {
  setResponseStatus(event, response.status, response.statusText)

  const contentType = response.headers.get('content-type')
  if (contentType) {
    setResponseHeader(event, 'content-type', contentType)
  }

  for (const cookie of extractSetCookies(response.headers)) {
    appendResponseHeader(event, 'set-cookie', cookie)
  }

  return response._data
}
