/**
 * One-time token handling for query-string based auth flows.
 *
 * Password reset, magic link and email confirmation links carry their token in
 * the query string: `/auth/reset-password?token=...`. The SPA has to read it,
 * then must get rid of it, because a URL that keeps the token is exposed to
 * anything that records the address bar (browser history, session restore,
 * screen sharing, and the `Referer` header of any resource loaded afterwards).
 *
 * The backend is unaffected either way (the token is single-use and the route
 * that consumes it is a POST/GET that does not log it), so this is purely about
 * not leaving a live credential sitting in the address bar.
 */

/** Query parameter that carries the token. */
export const AUTH_TOKEN_QUERY_PARAM = 'token'

/**
 * Read the token out of a route query object.
 *
 * Returns `null` for a missing or non-string value (an array is rejected: a
 * repeated parameter is a malformed link, not a token).
 */
export function authTokenFromQuery(
  query: Record<string, unknown> | undefined,
  param: string = AUTH_TOKEN_QUERY_PARAM,
): string | null {
  const value = query?.[param]
  if (typeof value !== 'string' || value.length === 0) {
    return null
  }
  return value
}

/**
 * The path with the token parameter removed.
 *
 * Other query parameters are preserved so a link that combines the token with
 * something else (a locale or a return path, for instance) keeps working.
 */
export function pathWithoutAuthToken(
  path: string,
  query: Record<string, unknown> | undefined,
  param: string = AUTH_TOKEN_QUERY_PARAM,
): string {
  const search = new URLSearchParams()
  for (const [key, value] of Object.entries(query ?? {})) {
    if (key === param || value === undefined || value === null) {
      continue
    }
    if (Array.isArray(value)) {
      for (const item of value) {
        search.append(key, String(item))
      }
    } else {
      search.append(key, String(value))
    }
  }

  const searchString = search.toString()
  return searchString ? `${path}?${searchString}` : path
}
