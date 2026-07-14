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
  options?: {
    /** Filter cookies to only forward backend-relevant cookies */
    filterCookies?: boolean
    /** List of cookie name prefixes to forward (e.g., ['refresh_token', 'csrf_']) */
    allowedCookiePrefixes?: string[]
  },
): Headers {
  const incomingHeaders = getRequestHeaders(event)
  const headers = new Headers()
  headers.set('x-api-key', backendApiKey)

  for (const headerName of headerNames) {
    const value = incomingHeaders[headerName]
    
    if (headerName === 'cookie') {
      // Special handling for cookies to filter sensitive client-side cookies
      let cookieValue: string | undefined

      if (typeof value === 'string' && value.length > 0) {
        cookieValue = options?.filterCookies !== false 
          ? filterCookies(value, options?.allowedCookiePrefixes)
          : value
      } else if (Array.isArray(value) && value.length > 0) {
        const joinedValue = value.join('; ')
        cookieValue = options?.filterCookies !== false
          ? filterCookies(joinedValue, options?.allowedCookiePrefixes)
          : joinedValue
      }

      if (cookieValue && cookieValue.length > 0) {
        headers.set('cookie', cookieValue)
      }
    } else {
      // Handle other headers normally
      if (typeof value === 'string' && value.length > 0) {
        headers.set(headerName, value)
      } else if (Array.isArray(value) && value.length > 0) {
        headers.set(headerName, value.join(', '))
      }
    }
  }

  if (accessToken) {
    headers.set('authorization', `Bearer ${accessToken}`)
  }

  return headers
}

/**
 * Filter cookies to only forward backend-relevant cookies.
 * By default, only forwards cookies that are likely backend session/auth cookies.
 * 
 * @param cookieString - The full cookie header string
 * @param allowedPrefixes - Cookie name prefixes to allow (defaults to auth-related cookies)
 * @returns Filtered cookie string containing only allowed cookies
 */
export function filterCookies(
  cookieString: string,
  allowedPrefixes?: string[],
): string {
  // Default allowed prefixes for backend cookies
  const prefixes = allowedPrefixes ?? [
    'refresh_token',
    'access_token',
    'session',
    'csrf',
    'auth',
  ]

  const cookies = cookieString.split(';').map((c) => c.trim())
  const filteredCookies: string[] = []

  for (const cookie of cookies) {
    if (!cookie) continue

    const cookieName = cookie.split('=')[0]?.trim()
    if (!cookieName) continue

    // Check if cookie name matches any allowed prefix (case-insensitive)
    const isAllowed = prefixes.some((prefix) =>
      cookieName.toLowerCase().startsWith(prefix.toLowerCase()),
    )

    if (isAllowed) {
      filteredCookies.push(cookie)
    }
  }

  return filteredCookies.join('; ')
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

// -- Tests

if (import.meta.vitest) {
  const { describe, it, expect } = import.meta.vitest

  describe('filterCookies', () => {
    it('filters to only backend-relevant cookies by default', () => {
      const cookies = 'refresh_token=abc123; analytics_id=xyz; csrf_token=def456; user_pref=dark'
      const filtered = filterCookies(cookies)
      
      expect(filtered).toContain('refresh_token=abc123')
      expect(filtered).toContain('csrf_token=def456')
      expect(filtered).not.toContain('analytics_id=xyz')
      expect(filtered).not.toContain('user_pref=dark')
    })

    it('filters with custom allowed prefixes', () => {
      const cookies = 'session_id=abc; tracking=xyz; auth_token=def'
      const filtered = filterCookies(cookies, ['session', 'auth'])
      
      expect(filtered).toContain('session_id=abc')
      expect(filtered).toContain('auth_token=def')
      expect(filtered).not.toContain('tracking=xyz')
    })

    it('handles case-insensitive prefix matching', () => {
      const cookies = 'Refresh_Token=abc; CSRF_Token=def; Other=xyz'
      const filtered = filterCookies(cookies, ['refresh', 'csrf'])
      
      expect(filtered).toContain('Refresh_Token=abc')
      expect(filtered).toContain('CSRF_Token=def')
      expect(filtered).not.toContain('Other=xyz')
    })

    it('returns empty string when no cookies match', () => {
      const cookies = 'analytics=abc; tracking=def; preferences=ghi'
      const filtered = filterCookies(cookies, ['session', 'auth'])
      
      expect(filtered).toBe('')
    })

    it('handles empty cookie string', () => {
      expect(filterCookies('')).toBe('')
    })

    it('preserves cookie values exactly', () => {
      const cookies = 'refresh_token=abc%20def; csrf_token=xyz123'
      const filtered = filterCookies(cookies)
      
      expect(filtered).toBe('refresh_token=abc%20def; csrf_token=xyz123')
    })
  })

  describe('normalizeBackendApiBase', () => {
    it('removes trailing slashes', () => {
      expect(normalizeBackendApiBase('http://localhost:8080/')).toBe('http://localhost:8080/api/v1')
    })

    it('preserves existing api/v1 paths', () => {
      expect(normalizeBackendApiBase('http://localhost:8080/api/v1')).toBe('http://localhost:8080/api/v1')
    })

    it('handles empty input', () => {
      expect(normalizeBackendApiBase('')).toBe('')
      expect(normalizeBackendApiBase(undefined as any)).toBe('')
    })
  })
}
