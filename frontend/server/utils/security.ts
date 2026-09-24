/**
 * Secure-default resolution for `NUXT_PUBLIC_API_BASE`.
 *
 * When set, the backend URL is baked into the client bundle and browsers
 * can hit the backend directly, bypassing the Nitro `/api/v1` reverse
 * proxy (cookie/session mapping, auth header injection, request-id
 * correlation). The secure default in production-like environments is
 * therefore the proxy (empty value), unless the operator explicitly
 * opts out for a deliberate setup (e.g. CDN-fronted backend).
 *
 * Pure helper (unit-tested) — the Nitro `security-check` plugin applies it.
 */

export interface DirectApiResolution {
  /** Effective value the app must use (`''` forces the Nitro proxy). */
  value: string
  /** True when the secure default overrode an explicitly configured value. */
  enforced: boolean
}

export function resolveApiDirectBase(options: {
  apiDirectBase?: unknown
  isProdLike: boolean
  allowDirectApi?: unknown
}): DirectApiResolution {
  const configured =
    typeof options.apiDirectBase === 'string' ? options.apiDirectBase : ''
  if (!configured || !options.isProdLike) {
    return { value: configured, enforced: false }
  }
  const explicitlyAllowed =
    options.allowDirectApi === true || options.allowDirectApi === 'true'
  if (explicitlyAllowed) {
    return { value: configured, enforced: false }
  }
  return { value: '', enforced: true }
}
