/**
 * CSRF header extraction (single source of truth).
 *
 * Used by:
 * - `app/plugins/api.ts` (client `$api`: attaches the `csrf-token` header
 *   the Actix CSRF middleware requires on cookie-authenticated state
 *   changes)
 * - `app/stores/auth.ts` (SSR refresh: the server-side branch builds its
 *   headers manually and must attach the same header)
 *
 * The backend CSRF cookie is intentionally NOT httpOnly — the SPA must be
 * able to read it and echo it in the `csrf-token` header (double-submit +
 * HMAC). Lives in `shared/` (Nuxt universal code) so both runtimes parse
 * cookies with identical semantics.
 */

const CSRF_COOKIE_PREFIX = 'csrf_token='

export function csrfTokenFromCookieHeader(
  cookieHeader: string | null | undefined,
): string | null {
  if (!cookieHeader) {
    return null
  }

  for (const row of cookieHeader.split(';')) {
    const trimmed = row.trim()
    if (trimmed.startsWith(CSRF_COOKIE_PREFIX)) {
      // Slice on the prefix keeps any "=" characters inside the value —
      // the token format is `timestamp.nonce.hmac`, but the parser must
      // not truncate values that contain one.
      return trimmed.slice(CSRF_COOKIE_PREFIX.length) || null
    }
  }

  return null
}
