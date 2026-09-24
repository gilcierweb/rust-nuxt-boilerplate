/**
 * Shared auth-endpoint classification (single source of truth).
 *
 * Used by:
 * - `app/plugins/api.ts` (client `$api`: skip the silent refresh→retry
 *   loop for auth endpoints)
 * - `server/utils/auth.ts` (Nitro proxy: skip the `/auth/session`
 *   round-trip for public auth endpoints)
 *
 * Lives in `shared/` (Nuxt universal code) so both runtimes import the
 * same lists and the same matching semantics. Previously each side kept
 * its own copy and they drifted: the client used substring matching
 * (`includes('/auth/')`), while the proxy compared `auth/login`
 * (no leading slash) against `/auth/login` (leading slash) and never
 * matched — silently disabling the optimization.
 *
 * Matching follows the repo convention (`app/utils/auth-routes.ts`):
 * exact match or segment-boundary prefix, so `/auth-history`
 * never matches `/auth`.
 */

const API_VERSION_PREFIX = '/api/v1'

const AUTH_PREFIX = '/auth'

/**
 * Auth endpoints that never require an access token (login, register, ...).
 * The Nitro proxy forwards these without resolving a token first.
 */
export const PUBLIC_AUTH_PATHS = [
  '/auth/login',
  '/auth/register',
  '/auth/recover',
  '/auth/forgot-password',
  '/auth/reset',
  '/auth/logout',
]

function toPathString(input: unknown): string {
  if (typeof input === 'string') return input
  // Preserve the old defensive semantics: callers may pass a Request-like.
  if (typeof input === 'object' && input !== null && 'url' in input) {
    const url = (input as { url?: unknown }).url
    if (typeof url === 'string') return url
  }
  return ''
}

/**
 * Normalize anything the callers may pass — a proxy-relative path
 * (`auth/login`), an app-relative path (`/auth/login`), a versioned path
 * (`/api/v1/auth/login`) or a full URL (`https://host/api/v1/auth/login?x=1`)
 * — to a comparable lowercase `/auth/...` path.
 */
export function normalizeAuthPath(input: unknown): string {
  let path = toPathString(input).split('?')[0].split('#')[0].trim()
  if (!path) return ''
  const schemeIndex = path.indexOf('://')
  if (schemeIndex >= 0) {
    const slashIndex = path.indexOf('/', schemeIndex + 3)
    path = slashIndex >= 0 ? path.slice(slashIndex) : '/'
  }
  if (!path.startsWith('/')) path = `/${path}`
  if (path.toLowerCase().startsWith(API_VERSION_PREFIX)) {
    path = path.slice(API_VERSION_PREFIX.length) || '/'
  }
  if (path.length > 1) path = path.replace(/\/+$/, '')
  return path.toLowerCase()
}

function matchesPath(path: string, candidate: string): boolean {
  return path === candidate || path.startsWith(`${candidate}/`)
}

/**
 * True for any `/auth/**` endpoint (login, refresh, session, logout, ...).
 * The client must not attempt a silent token refresh when the failing
 * request itself targets the auth surface — especially `/auth/refresh`,
 * where retrying would loop forever.
 */
export function isAuthPath(input: unknown): boolean {
  const path = normalizeAuthPath(input)
  if (!path) return false
  return matchesPath(path, AUTH_PREFIX)
}

/**
 * True for the public subset of `/auth/**` that never needs an access
 * token. The Nitro proxy skips the `/auth/session` exchange for these.
 */
export function isPublicAuthPath(input: unknown): boolean {
  const path = normalizeAuthPath(input)
  if (!path) return false
  return PUBLIC_AUTH_PATHS.some((candidate) => matchesPath(path, candidate))
}
