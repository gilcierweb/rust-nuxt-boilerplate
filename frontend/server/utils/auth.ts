import { getRequestHeaders, type H3Event } from 'h3'

/**
 * Extracts a Bearer token from an Authorization header.
 * Exported for unit testing.
 */
export function parseBearerToken(authorizationHeader?: string): string | null {
  if (!authorizationHeader) {
    return null
  }

  const value = authorizationHeader.trim()
  if (!value.toLowerCase().startsWith('bearer ')) {
    return null
  }

  const token = value.slice(7).trim()
  return token.length > 0 ? token : null
}

/**
 * Classifies a caught $fetch error for the session proxy:
 * - 401 / 404  → "no session" — return null (expected auth failures)
 * - 5xx / network → re-throw (unexpected; must not silently degrade auth)
 *
 * Exported for unit testing.
 */
export function classifySessionFetchError(err: unknown): null | never {
  const status: number | undefined = (
    err as { response?: { status?: number } }
  )?.response?.status

  if (status === 401 || status === 404) {
    return null
  }

  throw err
}

/// Paths that never require an access token (public auth endpoints).
/// Resolving a token for these would waste time calling /auth/session
/// with the old refresh_token cookie before the actual request is forwarded.
const PUBLIC_AUTH_PATHS = [
  '/auth/login',
  '/auth/register',
  '/auth/recover',
  '/auth/forgot-password',
  '/auth/reset',
  '/auth/logout',
]

export async function resolveAccessTokenForProxy(
  event: H3Event,
): Promise<string | null> {
  const incomingHeaders = getRequestHeaders(event)
  const tokenFromAuthorization = parseBearerToken(incomingHeaders.authorization)
  if (tokenFromAuthorization) {
    return tokenFromAuthorization
  }

  const path = event.context.params?.path || ''
  if (PUBLIC_AUTH_PATHS.some((p) => path.startsWith(p))) {
    return null
  }

  if (!incomingHeaders.cookie?.includes('refresh_token')) {
    return null
  }

  try {
    const sessionResponse = await $fetch<{ access_token?: string }>(
      '/api/v1/auth/session',
      {
        headers: {
          cookie: incomingHeaders.cookie,
          accept: 'application/json',
        },
      },
    )
    return sessionResponse?.access_token || null
  } catch (err: unknown) {
    // `$fetch` throws a `FetchError` for non-2xx responses.
    // The error object carries a `response` property with the HTTP status.
    //
    // 401 Unauthorized — session expired or refresh token invalid: expected,
    //   return null so the proxy forwards the request unauthenticated.
    // 404 Not Found    — session endpoint missing on this backend replica:
    //   also safe to treat as "no token".
    //
    // 5xx / network errors — these are UNEXPECTED and must NOT be swallowed:
    //   silently returning null would forward the request as if the user is
    //   unauthenticated, which is a silent auth downgrade. Re-throw so the
    //   Nitro error handler returns 502/503 to the browser instead.
    return classifySessionFetchError(err)
  }
}
