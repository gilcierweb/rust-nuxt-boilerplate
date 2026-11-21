/**
 * The API base URL each execution context must use.
 *
 * The server always renders through the Nitro `/api/v1` proxy: the proxy
 * forwards the cookies of the browser request and attaches the access token it
 * resolves from the refresh cookie, so a server render is authenticated. A
 * direct call from the SSR process drops both, whatever base is configured.
 *
 * The client may call the backend directly when `NUXT_PUBLIC_API_BASE` is
 * configured, which skips the proxy round-trip for client-initiated requests.
 */
export function resolveApiBase(
  config: unknown,
  isServer: boolean,
): string {
  if (isServer) {
    return '/api/v1'
  }

  // The Nuxt runtime config types `public` as an index signature, so the
  // values are read defensively instead of through a brittle cast.
  const publicConfig =
    config && typeof config === 'object'
      ? (config as { public?: unknown }).public
      : undefined
  const configured =
    publicConfig && typeof publicConfig === 'object'
      ? (publicConfig as Record<string, unknown>)
      : {}

  const directBase =
    typeof configured.apiDirectBase === 'string'
      ? configured.apiDirectBase.replace(/\/+$/, '')
      : ''
  if (directBase) {
    return directBase
  }

  return typeof configured.apiBase === 'string' && configured.apiBase
    ? configured.apiBase
    : '/api/v1'
}